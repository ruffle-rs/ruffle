use crate::avm1::{Activation, ActivationIdentifier, NativeObject, Object, Value};
use crate::avm2::StageObject as Avm2StageObject;
use crate::backend::audio::AudioManager;
use crate::backend::ui::MouseCursor;
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{
    dispatch_added_event, dispatch_removed_event, ChildContainer,
};
use crate::display_object::interactive::{
    Avm2MousePick, InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{Avm1TextFieldBinding, DisplayObjectBase};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::string::AvmString;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use ruffle_render::filters::Filter;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::sync::Arc;
use swf::ButtonActionCondition;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Avm1Button<'gc>(Gc<'gc, Avm1ButtonData<'gc>>);

impl fmt::Debug for Avm1Button<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Avm1Button")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Avm1ButtonData<'gc> {
    base: InteractiveObjectBase<'gc>,
    cell: RefLock<Avm1ButtonDataMut<'gc>>,
    shared: Gc<'gc, ButtonShared>,
    object: Lock<Option<Object<'gc>>>,
    state: Cell<ButtonState>,
    tracking: Cell<ButtonTracking>,
    initialized: Cell<bool>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
struct Avm1ButtonDataMut<'gc> {
    hit_area: BTreeMap<Depth, DisplayObject<'gc>>,
    #[collect(require_static)]
    hit_bounds: Rectangle<Twips>,
    container: ChildContainer<'gc>,
    text_field_bindings: Vec<Avm1TextFieldBinding<'gc>>,
}

impl<'gc> Avm1Button<'gc> {
    pub fn from_swf_tag(button: &swf::Button, source_movie: &SwfSlice, mc: &Mutation<'gc>) -> Self {
        let actions = button
            .actions
            .iter()
            .map(|action| ButtonAction {
                action_data: source_movie.to_unbounded_subslice(action.action_data),
                conditions: action.conditions,
            })
            .collect();

        Avm1Button(Gc::new(
            mc,
            Avm1ButtonData {
                base: Default::default(),
                cell: RefLock::new(Avm1ButtonDataMut {
                    container: ChildContainer::new(&source_movie.movie),
                    hit_area: BTreeMap::new(),
                    hit_bounds: Default::default(),
                    text_field_bindings: Vec::new(),
                }),
                shared: Gc::new(
                    mc,
                    ButtonShared {
                        swf: source_movie.movie.clone(),
                        id: button.id,
                        actions,
                        cell: RefCell::new(ButtonSharedMut {
                            records: button.records.clone(),
                            up_to_over_sound: None,
                            over_to_down_sound: None,
                            down_to_over_sound: None,
                            over_to_up_sound: None,
                        }),
                    },
                ),
                state: Cell::new(ButtonState::Up),
                initialized: Cell::new(false),
                object: Lock::new(None),
                tracking: Cell::new(if button.is_track_as_menu {
                    ButtonTracking::Menu
                } else {
                    ButtonTracking::Push
                }),
            },
        ))
    }

    pub fn set_sounds(self, sounds: swf::ButtonSounds) {
        let mut shared = self.0.shared.cell.borrow_mut();
        shared.up_to_over_sound = sounds.up_to_over_sound;
        shared.over_to_down_sound = sounds.over_to_down_sound;
        shared.down_to_over_sound = sounds.down_to_over_sound;
        shared.over_to_up_sound = sounds.over_to_up_sound;
    }

    /// Handles the ancient DefineButtonCxform SWF tag.
    /// Set the color transform for all children of each state.
    pub fn set_colors(self, color_transforms: &[swf::ColorTransform]) {
        let mut shared = self.0.shared.cell.borrow_mut();

        // This tag isn't documented well in SWF19. It is only used in very old SWF<=2 content.
        // It applies color transforms to every character in a button, in sequence(?).
        for (record, color_transform) in shared.records.iter_mut().zip(color_transforms.iter()) {
            record.color_transform = *color_transform;
        }
    }

    /// Set the state of a button, creating or destroying children as needed.
    ///
    /// This function instantiates children and thus must not be called whilst
    /// the caller is holding a write lock on the button data.
    pub fn set_state(mut self, context: &mut UpdateContext<'gc>, state: ButtonState) {
        let mut removed_depths: fnv::FnvHashSet<_> =
            self.iter_render_list().map(|o| o.depth()).collect();

        let movie = self.movie();
        self.0.state.set(state);

        // Create any new children that exist in this state, and remove children
        // that only exist in the previous state.
        // Children that exist in both states should persist and not be recreated.
        // TODO: This behavior probably differs in AVM2 (I suspect they always get recreated).
        let mut children = Vec::new();

        for record in &self.0.shared.cell.borrow().records {
            if record.states.contains(state.into()) {
                // State contains this depth, so we don't have to remove it.
                removed_depths.remove(&record.depth.into());

                let child = match self.child_by_depth(record.depth.into()) {
                    // Re-use existing child.
                    Some(child) if child.id() == record.id => child,

                    // Instantiate new child.
                    _ => {
                        if let Some(child) = context
                            .library
                            .library_for_movie_mut(movie.clone())
                            .instantiate_by_id(record.id, context.gc_context)
                        {
                            // New child that did not previously exist, create it.
                            child.set_parent(context, Some(self.into()));
                            child.set_depth(record.depth.into());

                            children.push((child, record.depth));
                            child
                        } else {
                            continue;
                        }
                    }
                };

                // Set transform of child (and modify previous child if it already existed)
                child.set_matrix(record.matrix.into());
                child.set_color_transform(record.color_transform);
                child.set_blend_mode(record.blend_mode.into());
                child.set_filters(record.filters.iter().map(Filter::from).collect());
            }
        }

        // Kill children that no longer exist in this state.
        for depth in removed_depths {
            if let Some(child) = self.child_by_depth(depth) {
                self.remove_child(context, child);
            }
        }

        for (child, depth) in children {
            // Initialize new child.
            child.post_instantiation(context, None, Instantiator::Movie, false);
            if let Some(clip) = child.as_movie_clip() {
                clip.run_frame_avm1(context);
            }
            let removed_child = self.replace_at_depth(context, child, depth.into());
            dispatch_added_event(self.into(), child, false, context);
            if let Some(removed_child) = removed_child {
                dispatch_removed_event(removed_child, context);
            }
        }

        self.invalidate_cached_bitmap();
    }

    pub fn state(self) -> Option<ButtonState> {
        Some(self.0.state.get())
    }

    fn get_boolean_property(
        self,
        name: AvmString<'gc>,
        default: bool,
        context: &mut UpdateContext<'gc>,
    ) -> bool {
        if let Some(object) = self.object1() {
            let mut activation = Activation::from_nothing(
                context,
                ActivationIdentifier::root("[AVM1 Boolean Property]"),
                self.avm1_root(),
            );
            if let Ok(value) = object.get(name, &mut activation) {
                match value {
                    Value::Undefined => default,
                    _ => value.as_bool(activation.swf_version()),
                }
            } else {
                default
            }
        } else {
            false
        }
    }

    fn enabled(self, context: &mut UpdateContext<'gc>) -> bool {
        self.get_boolean_property(istr!(context, "enabled"), true, context)
    }

    fn use_hand_cursor(self, context: &mut UpdateContext<'gc>) -> bool {
        self.get_boolean_property(istr!(context, "useHandCursor"), true, context)
    }
}

impl<'gc> TDisplayObject<'gc> for Avm1Button<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.raw_interactive())
    }

    fn instantiate(self, mc: &Mutation<'gc>) -> DisplayObject<'gc> {
        let data: &Avm1ButtonData = &self.0;
        Self(Gc::new(mc, data.clone())).into()
    }

    fn id(self) -> CharacterId {
        self.0.shared.id
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie()
    }

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if self.0.object.get().is_none() {
            let object = Object::new_with_native(
                &context.strings,
                Some(context.avm1.prototypes(self.swf_version()).button),
                NativeObject::Button(self),
            );
            let obj = unlock!(Gc::write(context.gc(), self.0), Avm1ButtonData, object);
            obj.set(Some(object));
        }

        if !self.0.initialized.get() {
            let mut new_children = Vec::new();

            self.set_state(context, ButtonState::Up);
            self.0.initialized.set(true);

            for record in &self.0.shared.cell.borrow().records {
                if record.states.contains(swf::ButtonState::HIT_TEST) {
                    match context
                        .library
                        .library_for_movie_mut(self.0.movie())
                        .instantiate_by_id(record.id, context.gc_context)
                    {
                        Some(child) => {
                            child.set_matrix(record.matrix.into());
                            child.set_parent(context, Some(self.into()));
                            child.set_depth(record.depth.into());
                            new_children.push((child, record.depth.into()));
                        }
                        None => {
                            tracing::error!(
                                "Button ID {}: could not instantiate child ID {}",
                                self.0.shared.id,
                                record.id,
                            );
                        }
                    }
                }
            }

            let write = unlock!(Gc::write(context.gc(), self.0), Avm1ButtonData, cell);
            let mut hit_bounds = Rectangle::INVALID;
            for (child, depth) in new_children {
                child.post_instantiation(context, None, Instantiator::Movie, false);
                write.borrow_mut().hit_area.insert(depth, child);
                hit_bounds = hit_bounds.union(&child.local_bounds());
            }
            write.borrow_mut().hit_bounds = hit_bounds;
        }
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        self.render_children(context);
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        // No inherent bounds; contains child DisplayObjects.
        Default::default()
    }

    fn hit_test_shape(
        self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        for child in self.iter_render_list() {
            if child.hit_test_shape(context, point, options) {
                return true;
            }
        }

        false
    }

    fn object1(self) -> Option<Object<'gc>> {
        self.0.object.get()
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        // AVM1 buttons don't have an associated AVM2 object
        None
    }

    fn allow_as_mask(self) -> bool {
        !self.is_empty()
    }

    fn avm1_unload(self, context: &mut UpdateContext<'gc>) {
        for child in self.iter_render_list() {
            child.avm1_unload(context);
        }

        self.drop_focus(context);

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc(), None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc(), None, true);
        }

        // Do *not* unregister text field bindings.

        self.set_avm1_removed(true);
    }

    fn avm1_text_field_bindings(&self) -> Option<Ref<'_, [Avm1TextFieldBinding<'gc>]>> {
        let read = Gc::as_ref(self.0).cell.borrow();
        Some(Ref::map(read, |r| &*r.text_field_bindings))
    }

    fn avm1_text_field_bindings_mut(
        &self,
        mc: &Mutation<'gc>,
    ) -> Option<RefMut<'_, Vec<Avm1TextFieldBinding<'gc>>>> {
        let write = unlock!(Gc::write(mc, self.0), Avm1ButtonData, cell).borrow_mut();
        Some(RefMut::map(write, |w| &mut w.text_field_bindings))
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for Avm1Button<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        Ref::map(self.0.cell.borrow(), |this| &this.container)
    }

    fn raw_container_mut(&self, mc: &Mutation<'gc>) -> RefMut<'_, ChildContainer<'gc>> {
        let data = unlock!(Gc::write(mc, self.0), Avm1ButtonData, cell);
        RefMut::map(data.borrow_mut(), |this| &mut this.container)
    }
}

impl<'gc> TInteractiveObject<'gc> for Avm1Button<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        event: ClipEvent,
    ) -> ClipEventResult {
        // An invisible button can still run its `rollOut` or `releaseOutside` event.
        // A disabled button doesn't run its events (`KeyPress` being the exception) but
        // its state can still change. This is tested at "avm1/mouse_events_visible_enabled".
        if !self.visible() && self.0.state.get() == ButtonState::Up {
            return ClipEventResult::NotHandled;
        }

        // The `keyPress` event doesn't fire if the button is inside another button.
        if matches!(event, ClipEvent::KeyPress { .. })
            && self.parent().and_then(|p| p.as_avm1_button()).is_some()
        {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
    }

    fn event_dispatch(self, context: &mut UpdateContext<'gc>, event: ClipEvent) -> ClipEventResult {
        let self_display_object = self.into();
        let is_enabled = self.enabled(context);

        // Translate the clip event to a button event, based on how the button state changes.
        let shared = self.0.shared;
        let shared = shared.cell.borrow();
        let (new_state, condition, sound) = match event {
            ClipEvent::DragOut { .. } => (
                ButtonState::Over,
                Some(ButtonActionCondition::OVER_DOWN_TO_OUT_DOWN),
                None,
            ),
            ClipEvent::DragOver { .. } => (
                ButtonState::Down,
                Some(ButtonActionCondition::OUT_DOWN_TO_OVER_DOWN),
                None,
            ),
            ClipEvent::Press { .. } => (
                ButtonState::Down,
                Some(ButtonActionCondition::OVER_UP_TO_OVER_DOWN),
                shared.over_to_down_sound.as_ref(),
            ),
            ClipEvent::Release { .. } => (
                ButtonState::Over,
                Some(ButtonActionCondition::OVER_DOWN_TO_OVER_UP),
                shared.down_to_over_sound.as_ref(),
            ),
            ClipEvent::ReleaseOutside => (
                ButtonState::Up,
                Some(ButtonActionCondition::OUT_DOWN_TO_IDLE),
                shared.over_to_up_sound.as_ref(),
            ),
            ClipEvent::RollOut { .. } => (
                ButtonState::Up,
                Some(ButtonActionCondition::OVER_UP_TO_IDLE),
                shared.over_to_up_sound.as_ref(),
            ),
            ClipEvent::RollOver { .. } => (
                ButtonState::Over,
                Some(ButtonActionCondition::IDLE_TO_OVER_UP),
                shared.up_to_over_sound.as_ref(),
            ),
            ClipEvent::KeyPress { key_code } => {
                return self.0.run_actions(
                    context,
                    ButtonActionCondition::from_key_code(key_code.to_u8()),
                );
            }
            // KeyUp and KeyDown might fire some event handlers
            ClipEvent::KeyUp => (self.0.state.get(), None, None),
            ClipEvent::KeyDown => (self.0.state.get(), None, None),
            _ => return ClipEventResult::NotHandled,
        };

        let (update_state, new_state) = if is_enabled {
            if let Some(condition) = condition {
                self.0.run_actions(context, condition);
            }
            if let Some((id, sound_info)) = sound {
                AudioManager::perform_sound_event(self.into(), context, *id, sound_info);
            }

            // Queue ActionScript-defined event handlers after the SWF defined ones.
            // (e.g., clip.onRelease = foo).
            if self.should_fire_event_handlers(context, event) {
                if let Some(name) = event.method_name(&context.strings) {
                    context.action_queue.queue_action(
                        self_display_object,
                        ActionType::Method {
                            object: self.0.object.get().unwrap(),
                            name,
                            args: vec![],
                        },
                        false,
                    );
                }
            }

            (self.0.state.get() != new_state, new_state)
        } else {
            // Remove the current mouse hovered and mouse down objects.
            // This is required to make sure the button will fire its events if it gets enabled.
            if InteractiveObject::option_ptr_eq(Some(self.into()), context.mouse_data.hovered) {
                context.mouse_data.hovered = None;
            }
            if InteractiveObject::option_ptr_eq(Some(self.into()), context.mouse_data.pressed) {
                context.mouse_data.pressed = None;
            }

            (new_state != ButtonState::Over, ButtonState::Up)
        };

        if update_state {
            self.set_state(context, new_state);
        }

        ClipEventResult::NotHandled
    }

    fn mouse_pick_avm1(
        self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() && self.mouse_enabled() {
            for child in self.iter_render_list().rev() {
                let result = child
                    .as_interactive()
                    .and_then(|c| c.mouse_pick_avm1(context, point, require_button_mode));
                if result.is_some() {
                    return result;
                }
            }

            for child in self.0.cell.borrow().hit_area.values() {
                if child.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
                    return Some(self.into());
                }
            }
        }
        None
    }

    fn mouse_pick_avm2(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        Avm2MousePick::Miss
    }

    fn mouse_cursor(self, context: &mut UpdateContext<'gc>) -> MouseCursor {
        if self.use_hand_cursor(context) && self.enabled(context) {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
    }

    fn tab_enabled_default(self, _context: &mut UpdateContext<'gc>) -> bool {
        true
    }

    fn highlight_bounds(self) -> Rectangle<Twips> {
        // Buttons are always highlighted using their hit bounds.
        // I guess it does have some sense to it, because their bounds
        // usually change on hover (children are swapped out),
        // which would cause the automatic tab order to change during tabbing.
        // That could potentially create a loop in the tab ordering (soft locking the tab).
        self.local_to_global_matrix() * self.0.cell.borrow().hit_bounds
    }
}

impl<'gc> Avm1ButtonData<'gc> {
    fn run_actions(
        &self,
        context: &mut UpdateContext<'gc>,
        condition: ButtonActionCondition,
    ) -> ClipEventResult {
        let mut handled = ClipEventResult::NotHandled;
        if let Some(parent) = self.base.base.parent() {
            for action in &self.shared.actions {
                if action.conditions.matches(condition) {
                    // Note that AVM1 buttons run actions relative to their parent, not themselves.
                    handled = ClipEventResult::Handled;
                    context.action_queue.queue_action(
                        parent,
                        ActionType::Normal {
                            bytecode: action.action_data.clone(),
                        },
                        false,
                    );
                }
            }
        }
        handled
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.shared.swf.clone()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum ButtonState {
    Up,
    Over,
    Down,
}

impl From<ButtonState> for swf::ButtonState {
    fn from(bs: ButtonState) -> Self {
        match bs {
            ButtonState::Up => Self::UP,
            ButtonState::Over => Self::OVER,
            ButtonState::Down => Self::DOWN,
        }
    }
}

#[derive(Clone, Debug)]
struct ButtonAction {
    action_data: SwfSlice,
    conditions: ButtonActionCondition,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonTracking {
    Push,
    Menu,
}

/// Data shared between all instances of a button.
#[derive(Collect, Debug)]
#[collect(require_static)]
struct ButtonShared {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    actions: Vec<ButtonAction>,
    cell: RefCell<ButtonSharedMut>,
}

#[derive(Debug)]
struct ButtonSharedMut {
    records: Vec<swf::ButtonRecord>,

    /// The sounds to play on state changes for this button.
    up_to_over_sound: Option<swf::ButtonSound>,
    over_to_down_sound: Option<swf::ButtonSound>,
    down_to_over_sound: Option<swf::ButtonSound>,
    over_to_up_sound: Option<swf::ButtonSound>,
}
