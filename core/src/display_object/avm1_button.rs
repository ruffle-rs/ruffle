use super::interactive::Avm2MousePick;
use crate::avm1::{Activation, ActivationIdentifier, Object, StageObject, TObject, Value};
use crate::backend::audio::AudioManager;
use crate::backend::ui::MouseCursor;
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{
    dispatch_added_event, dispatch_removed_event, ChildContainer,
};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
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

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Avm1ButtonData<'gc> {
    cell: RefLock<Avm1ButtonDataMut<'gc>>,
    static_data: Gc<'gc, ButtonStatic>,
    state: Cell<ButtonState>,
    tracking: Cell<ButtonTracking>,
    object: Lock<Option<Object<'gc>>>,
    initialized: Cell<bool>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
struct Avm1ButtonDataMut<'gc> {
    base: InteractiveObjectBase<'gc>,
    hit_area: BTreeMap<Depth, DisplayObject<'gc>>,
    #[collect(require_static)]
    hit_bounds: Rectangle<Twips>,
    container: ChildContainer<'gc>,
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
                cell: RefLock::new(Avm1ButtonDataMut {
                    base: Default::default(),
                    container: ChildContainer::new(source_movie.movie.clone()),
                    hit_area: BTreeMap::new(),
                    hit_bounds: Default::default(),
                }),
                static_data: Gc::new(
                    mc,
                    ButtonStatic {
                        swf: source_movie.movie.clone(),
                        id: button.id,
                        actions,
                        cell: RefCell::new(ButtonStaticMut {
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
        let mut static_data = self.0.static_data.cell.borrow_mut();
        static_data.up_to_over_sound = sounds.up_to_over_sound;
        static_data.over_to_down_sound = sounds.over_to_down_sound;
        static_data.down_to_over_sound = sounds.down_to_over_sound;
        static_data.over_to_up_sound = sounds.over_to_up_sound;
    }

    /// Handles the ancient DefineButtonCxform SWF tag.
    /// Set the color transform for all children of each state.
    pub fn set_colors(self, color_transforms: &[swf::ColorTransform]) {
        let mut static_data = self.0.static_data.cell.borrow_mut();

        // This tag isn't documented well in SWF19. It is only used in very old SWF<=2 content.
        // It applies color transforms to every character in a button, in sequence(?).
        for (record, color_transform) in static_data.records.iter_mut().zip(color_transforms.iter())
        {
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

        for record in &self.0.static_data.cell.borrow().records {
            if record.states.contains(state.into()) {
                // State contains this depth, so we don't have to remove it.
                removed_depths.remove(&record.depth.into());

                let child = match self.child_by_depth(record.depth.into()) {
                    // Re-use existing child.
                    Some(child) if child.id() == record.id => child,

                    // Instantiate new child.
                    _ => {
                        if let Ok(child) = context
                            .library
                            .library_for_movie_mut(movie.clone())
                            .instantiate_by_id(record.id, context.gc_context)
                        {
                            // New child that did not previously exist, create it.
                            child.set_parent(context, Some(self.into()));
                            child.set_depth(context.gc(), record.depth.into());

                            children.push((child, record.depth));
                            child
                        } else {
                            continue;
                        }
                    }
                };

                // Set transform of child (and modify previous child if it already existed)
                child.set_matrix(context.gc(), record.matrix.into());
                child.set_color_transform(context.gc(), record.color_transform);
                child.set_blend_mode(context.gc(), record.blend_mode.into());
                child.set_filters(
                    context.gc(),
                    record.filters.iter().map(Filter::from).collect(),
                );
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
            child.run_frame_avm1(context);
            let removed_child = self.replace_at_depth(context, child, depth.into());
            dispatch_added_event(self.into(), child, false, context);
            if let Some(removed_child) = removed_child {
                dispatch_removed_event(removed_child, context);
            }
        }

        self.invalidate_cached_bitmap(context.gc());
    }

    pub fn state(&self) -> Option<ButtonState> {
        Some(self.0.state.get())
    }

    fn get_boolean_property(
        self,
        context: &mut UpdateContext<'gc>,
        name: &'static str,
        default: bool,
    ) -> bool {
        if let Value::Object(object) = self.object() {
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
        self.get_boolean_property(context, "enabled", true)
    }

    fn use_hand_cursor(self, context: &mut UpdateContext<'gc>) -> bool {
        self.get_boolean_property(context, "useHandCursor", true)
    }
}

impl<'gc> TDisplayObject<'gc> for Avm1Button<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.cell.borrow(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: &Mutation<'gc>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        let data = unlock!(Gc::write(mc, self.0), Avm1ButtonData, cell);
        RefMut::map(data.borrow_mut(), |w| &mut w.base.base)
    }

    fn instantiate(&self, mc: &Mutation<'gc>) -> DisplayObject<'gc> {
        let data: &Avm1ButtonData = &self.0;
        Self(Gc::new(mc, data.clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        Gc::as_ptr(self.0) as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.static_data.id
    }

    fn movie(&self) -> Arc<SwfMovie> {
        self.0.movie()
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if !self.movie().is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        if self.0.object.get().is_none() {
            let object = StageObject::for_display_object(
                context.gc(),
                (*self).into(),
                context.avm1.prototypes().button,
            );
            let obj = unlock!(Gc::write(context.gc(), self.0), Avm1ButtonData, object);
            obj.set(Some(object.into()));

            if run_frame {
                self.run_frame_avm1(context);
            }
        }
    }

    fn run_frame_avm1(&self, context: &mut UpdateContext<'gc>) {
        let self_display_object = (*self).into();
        let initialized = self.0.initialized.get();

        // TODO: Move this to post_instantiation.
        if !initialized {
            let mut new_children = Vec::new();

            self.set_state(context, ButtonState::Up);
            self.0.initialized.set(true);

            for record in &self.0.static_data.cell.borrow().records {
                if record.states.contains(swf::ButtonState::HIT_TEST) {
                    match context
                        .library
                        .library_for_movie_mut(self.0.movie())
                        .instantiate_by_id(record.id, context.gc_context)
                    {
                        Ok(child) => {
                            child.set_matrix(context.gc(), record.matrix.into());
                            child.set_parent(context, Some(self_display_object));
                            child.set_depth(context.gc(), record.depth.into());
                            new_children.push((child, record.depth.into()));
                        }
                        Err(error) => {
                            tracing::error!(
                                "Button ID {}: could not instantiate child ID {}: {}",
                                self.0.static_data.id,
                                record.id,
                                error
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

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        self.render_children(context);
    }

    fn self_bounds(&self) -> Rectangle<Twips> {
        // No inherent bounds; contains child DisplayObjects.
        Default::default()
    }

    fn hit_test_shape(
        &self,
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

    fn object(&self) -> Value<'gc> {
        self.0
            .object
            .get()
            .map(Value::from)
            .unwrap_or(Value::Undefined)
    }

    fn as_avm1_button(&self) -> Option<Self> {
        Some(*self)
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }

    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        Some(self.into())
    }

    fn allow_as_mask(&self) -> bool {
        !self.is_empty()
    }

    fn avm1_unload(&self, context: &mut UpdateContext<'gc>) {
        self.drop_focus(context);
        if let Some(node) = self.maskee() {
            node.set_masker(context.gc(), None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc(), None, true);
        }
        context
            .audio_manager
            .stop_sounds_with_display_object(context.audio, (*self).into());
        self.set_avm1_removed(context.gc(), true);
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
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.cell.borrow(), |r| &r.base)
    }

    fn raw_interactive_mut(&self, mc: &Mutation<'gc>) -> RefMut<InteractiveObjectBase<'gc>> {
        let data = unlock!(Gc::write(mc, self.0), Avm1ButtonData, cell);
        RefMut::map(data.borrow_mut(), |w| &mut w.base)
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
            && self
                .base()
                .parent
                .and_then(|p| p.as_avm1_button())
                .is_some()
        {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
    }

    fn event_dispatch(self, context: &mut UpdateContext<'gc>, event: ClipEvent) -> ClipEventResult {
        let self_display_object = self.into();
        let is_enabled = self.enabled(context);

        // Translate the clip event to a button event, based on how the button state changes.
        let static_data = self.0.static_data;
        let static_data = static_data.cell.borrow();
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
                static_data.over_to_down_sound.as_ref(),
            ),
            ClipEvent::Release { .. } => (
                ButtonState::Over,
                Some(ButtonActionCondition::OVER_DOWN_TO_OVER_UP),
                static_data.down_to_over_sound.as_ref(),
            ),
            ClipEvent::ReleaseOutside => (
                ButtonState::Up,
                Some(ButtonActionCondition::OUT_DOWN_TO_IDLE),
                static_data.over_to_up_sound.as_ref(),
            ),
            ClipEvent::RollOut { .. } => (
                ButtonState::Up,
                Some(ButtonActionCondition::OVER_UP_TO_IDLE),
                static_data.over_to_up_sound.as_ref(),
            ),
            ClipEvent::RollOver { .. } => (
                ButtonState::Over,
                Some(ButtonActionCondition::IDLE_TO_OVER_UP),
                static_data.up_to_over_sound.as_ref(),
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
                if let Some(name) = event.method_name() {
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
            if InteractiveObject::option_ptr_eq(self.as_interactive(), context.mouse_data.hovered) {
                context.mouse_data.hovered = None;
            }
            if InteractiveObject::option_ptr_eq(self.as_interactive(), context.mouse_data.pressed) {
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
        &self,
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
                    return Some((*self).into());
                }
            }
        }
        None
    }

    fn mouse_pick_avm2(
        &self,
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

    fn tab_enabled_default(&self, _context: &mut UpdateContext<'gc>) -> bool {
        true
    }

    fn highlight_bounds(self) -> Rectangle<Twips> {
        // Buttons are always highlighted using their hit bounds.
        // I guess it does have some sense to it, because their bounds
        // usually change on hover (children are swapped out),
        // which would cause the automatic tab order to change during tabbing.
        // That could potentially create a loop in the tab ordering (soft locking the tab).
        self.local_to_global_matrix() * self.0.cell.borrow().hit_bounds.clone()
    }
}

impl<'gc> Avm1ButtonData<'gc> {
    fn run_actions(
        &self,
        context: &mut UpdateContext<'gc>,
        condition: ButtonActionCondition,
    ) -> ClipEventResult {
        let mut handled = ClipEventResult::NotHandled;
        if let Some(parent) = self.cell.borrow().base.base.parent {
            for action in &self.static_data.actions {
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
        self.static_data.swf.clone()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
#[allow(dead_code)]
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

/// Static data shared between all instances of a button.
#[derive(Collect, Debug)]
#[collect(require_static)]
struct ButtonStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    actions: Vec<ButtonAction>,
    cell: RefCell<ButtonStaticMut>,
}

#[derive(Debug)]
struct ButtonStaticMut {
    records: Vec<swf::ButtonRecord>,

    /// The sounds to play on state changes for this button.
    up_to_over_sound: Option<swf::ButtonSound>,
    over_to_down_sound: Option<swf::ButtonSound>,
    down_to_over_sound: Option<swf::ButtonSound>,
    over_to_up_sound: Option<swf::ButtonSound>,
}
