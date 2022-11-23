use crate::avm1::{Object, StageObject, Value};
use crate::backend::ui::MouseCursor;
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{
    dispatch_added_event, dispatch_removed_event, ChildContainer,
};
use crate::display_object::interactive::{
    InteractiveObject, InteractiveObjectBase, TInteractiveObject,
};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::vminterface::Instantiator;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::collections::BTreeMap;
use std::sync::Arc;
use swf::ButtonActionCondition;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Avm1Button<'gc>(GcCell<'gc, Avm1ButtonData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Avm1ButtonData<'gc> {
    base: InteractiveObjectBase<'gc>,
    static_data: GcCell<'gc, ButtonStatic>,
    state: ButtonState,
    hit_area: BTreeMap<Depth, DisplayObject<'gc>>,
    container: ChildContainer<'gc>,
    tracking: ButtonTracking,
    object: Option<Object<'gc>>,
    initialized: bool,
    has_focus: bool,
    enabled: bool,
    use_hand_cursor: bool,
}

impl<'gc> Avm1Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        source_movie: &SwfSlice,
        gc_context: gc_arena::MutationContext<'gc, '_>,
    ) -> Self {
        let mut actions = vec![];
        for action in &button.actions {
            let action_data = source_movie.to_unbounded_subslice(action.action_data);
            let bits = action.conditions.bits();
            let mut bit = 1u16;
            while bits & !(bit - 1) != 0 {
                if bits & bit != 0 {
                    actions.push(ButtonAction {
                        action_data: action_data.clone(),
                        condition: ButtonActionCondition::from_bits_truncate(bit),
                        key_code: action.key_code.and_then(ButtonKeyCode::from_u8),
                    });
                }
                bit <<= 1;
            }
        }

        let static_data = ButtonStatic {
            swf: source_movie.movie.clone(),
            id: button.id,
            records: button.records.clone(),
            actions,
            up_to_over_sound: None,
            over_to_down_sound: None,
            down_to_over_sound: None,
            over_to_up_sound: None,
        };

        Avm1Button(GcCell::allocate(
            gc_context,
            Avm1ButtonData {
                base: Default::default(),
                static_data: GcCell::allocate(gc_context, static_data),
                container: ChildContainer::new(),
                hit_area: BTreeMap::new(),
                state: self::ButtonState::Up,
                initialized: false,
                object: None,
                tracking: if button.is_track_as_menu {
                    ButtonTracking::Menu
                } else {
                    ButtonTracking::Push
                },
                has_focus: false,
                enabled: true,
                use_hand_cursor: true,
            },
        ))
    }

    pub fn set_sounds(self, gc_context: MutationContext<'gc, '_>, sounds: swf::ButtonSounds) {
        let button = self.0.write(gc_context);
        let mut static_data = button.static_data.write(gc_context);
        static_data.up_to_over_sound = sounds.up_to_over_sound;
        static_data.over_to_down_sound = sounds.over_to_down_sound;
        static_data.down_to_over_sound = sounds.down_to_over_sound;
        static_data.over_to_up_sound = sounds.over_to_up_sound;
    }

    /// Handles the ancient DefineButtonCxform SWF tag.
    /// Set the color transform for all children of each state.
    pub fn set_colors(
        self,
        gc_context: MutationContext<'gc, '_>,
        color_transforms: &[swf::ColorTransform],
    ) {
        let button = self.0.write(gc_context);
        let mut static_data = button.static_data.write(gc_context);

        // This tag isn't documented well in SWF19. It is only used in very old SWF<=2 content.
        // It applies color transforms to every character in a button, in sequence(?).
        for (record, color_transform) in static_data.records.iter_mut().zip(color_transforms.iter())
        {
            record.color_transform = color_transform.clone();
        }
    }

    /// Set the state of a button, creating or destroying children as needed.
    ///
    /// This function instantiates children and thus must not be called whilst
    /// the caller is holding a write lock on the button data.
    fn set_state(
        mut self,
        context: &mut crate::context::UpdateContext<'_, 'gc, '_>,
        state: ButtonState,
    ) {
        let mut removed_depths: fnv::FnvHashSet<_> =
            self.iter_render_list().map(|o| o.depth()).collect();

        let movie = self.movie().unwrap();
        let mut write = self.0.write(context.gc_context);
        write.state = state;

        // Create any new children that exist in this state, and remove children
        // that only exist in the previous state.
        // Children that exist in both states should persist and not be recreated.
        // TODO: This behavior probably differs in AVM2 (I suspect they always get recreated).
        let mut children = Vec::new();

        for record in &write.static_data.read().records {
            if record.states.contains(state.into()) {
                // State contains this depth, so we don't have to remove it.
                removed_depths.remove(&record.depth.into());

                let child = match write.container.get_depth(record.depth.into()) {
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
                            child.set_parent(context.gc_context, Some(self.into()));
                            child.set_depth(context.gc_context, record.depth.into());

                            children.push((child, record.depth));
                            child
                        } else {
                            continue;
                        }
                    }
                };

                // Set transform of child (and modify previous child if it already existed)
                child.set_matrix(context.gc_context, record.matrix.into());
                child
                    .set_color_transform(context.gc_context, record.color_transform.clone().into());
            }
        }
        drop(write);

        // Kill children that no longer exist in this state.
        for depth in removed_depths {
            if let Some(child) = self.child_by_depth(depth) {
                self.remove_child(context, child, Lists::all());
            }
        }

        for (child, depth) in children {
            // Initialize new child.
            child.post_instantiation(context, None, Instantiator::Movie, false);
            child.run_frame(context);
            let removed_child = self.replace_at_depth(context, child, depth.into());
            dispatch_added_event(self.into(), child, false, context);
            if let Some(removed_child) = removed_child {
                dispatch_removed_event(removed_child, context);
            }
        }
    }

    pub fn enabled(self) -> bool {
        self.0.read().enabled
    }

    pub fn set_enabled(self, context: &mut UpdateContext<'_, 'gc, '_>, enabled: bool) {
        self.0.write(context.gc_context).enabled = enabled;
        if !enabled {
            self.set_state(context, ButtonState::Up);
        }
    }

    pub fn use_hand_cursor(self) -> bool {
        self.0.read().use_hand_cursor
    }

    pub fn set_use_hand_cursor(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        use_hand_cursor: bool,
    ) {
        self.0.write(context.gc_context).use_hand_cursor = use_hand_cursor;
    }
}

impl<'gc> TDisplayObject<'gc> for Avm1Button<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().static_data.read().id
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().movie())
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _init_object: Option<Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        if !context.is_action_script_3() {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        let mut mc = self.0.write(context.gc_context);
        if mc.object.is_none() {
            let object = StageObject::for_display_object(
                context.gc_context,
                (*self).into(),
                context.avm1.prototypes().button,
            );
            mc.object = Some(object.into());

            drop(mc);

            if run_frame {
                self.run_frame(context);
            }
        }
    }

    fn run_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let self_display_object = (*self).into();
        let initialized = self.0.read().initialized;

        // TODO: Move this to post_instantiation.
        if !initialized {
            let mut new_children = Vec::new();

            self.set_state(context, ButtonState::Up);
            self.0.write(context.gc_context).initialized = true;

            let read = self.0.read();

            for record in &read.static_data.read().records {
                if record.states.contains(swf::ButtonState::HIT_TEST) {
                    match context
                        .library
                        .library_for_movie_mut(read.movie())
                        .instantiate_by_id(record.id, context.gc_context)
                    {
                        Ok(child) => {
                            child.set_matrix(context.gc_context, record.matrix.into());
                            child.set_parent(context.gc_context, Some(self_display_object));
                            child.set_depth(context.gc_context, record.depth.into());
                            new_children.push((child, record.depth.into()));
                        }
                        Err(error) => {
                            log::error!(
                                "Button ID {}: could not instantiate child ID {}: {}",
                                read.static_data.read().id,
                                record.id,
                                error
                            );
                        }
                    }
                }
            }

            drop(read);

            for (child, depth) in new_children {
                child.post_instantiation(context, None, Instantiator::Movie, false);
                self.0
                    .write(context.gc_context)
                    .hit_area
                    .insert(depth, child);
            }
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        self.render_children(context);
    }

    fn self_bounds(&self) -> BoundingBox {
        // No inherent bounds; contains child DisplayObjects.
        BoundingBox::default()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
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
            .read()
            .object
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

    fn is_focusable(&self) -> bool {
        true
    }

    fn on_focus_changed(&self, gc_context: MutationContext<'gc, '_>, focused: bool) {
        self.0.write(gc_context).has_focus = focused;
    }

    fn unload(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let had_focus = self.0.read().has_focus;
        if had_focus {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }
        if let Some(node) = self.maskee() {
            node.set_masker(context.gc_context, None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc_context, None, true);
        }
        self.set_removed(context.gc_context, true);
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for Avm1Button<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        Ref::map(self.0.read(), |this| &this.container)
    }

    fn raw_container_mut(
        &self,
        gc_context: MutationContext<'gc, '_>,
    ) -> RefMut<'_, ChildContainer<'gc>> {
        RefMut::map(self.0.write(gc_context), |this| &mut this.container)
    }
}

impl<'gc> TInteractiveObject<'gc> for Avm1Button<'gc> {
    fn raw_interactive(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn raw_interactive_mut(
        &self,
        mc: MutationContext<'gc, '_>,
    ) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(self, event: ClipEvent) -> ClipEventResult {
        if !self.visible() && !matches!(event, ClipEvent::ReleaseOutside) {
            return ClipEventResult::NotHandled;
        }

        if !self.enabled() && !matches!(event, ClipEvent::KeyPress { .. }) {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
    }

    fn event_dispatch(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent,
    ) -> ClipEventResult {
        let self_display_object = self.into();
        let mut write = self.0.write(context.gc_context);

        // Translate the clip event to a button event, based on how the button state changes.
        let static_data = write.static_data;
        let static_data = static_data.read();
        let (new_state, condition, sound) = match event {
            ClipEvent::DragOut { .. } => (
                ButtonState::Over,
                ButtonActionCondition::OVER_DOWN_TO_OUT_DOWN,
                None,
            ),
            ClipEvent::DragOver { .. } => (
                ButtonState::Down,
                ButtonActionCondition::OUT_DOWN_TO_OVER_DOWN,
                None,
            ),
            ClipEvent::Press => (
                ButtonState::Down,
                ButtonActionCondition::OVER_UP_TO_OVER_DOWN,
                static_data.over_to_down_sound.as_ref(),
            ),
            ClipEvent::Release => (
                ButtonState::Over,
                ButtonActionCondition::OVER_DOWN_TO_OVER_UP,
                static_data.down_to_over_sound.as_ref(),
            ),
            ClipEvent::ReleaseOutside => (
                ButtonState::Up,
                ButtonActionCondition::OUT_DOWN_TO_IDLE,
                static_data.over_to_up_sound.as_ref(),
            ),
            ClipEvent::RollOut { .. } => (
                ButtonState::Up,
                ButtonActionCondition::OVER_UP_TO_IDLE,
                static_data.over_to_up_sound.as_ref(),
            ),
            ClipEvent::RollOver { .. } => (
                ButtonState::Over,
                ButtonActionCondition::IDLE_TO_OVER_UP,
                static_data.up_to_over_sound.as_ref(),
            ),
            ClipEvent::KeyPress { key_code } => {
                return write.run_actions(
                    context,
                    swf::ButtonActionCondition::KEY_PRESS,
                    Some(key_code),
                );
            }
            _ => return ClipEventResult::NotHandled,
        };

        write.run_actions(context, condition, None);
        write.play_sound(context, sound);

        // Queue ActionScript-defined event handlers after the SWF defined ones.
        // (e.g., clip.onRelease = foo).
        if context.swf.version() >= 6 {
            if let Some(name) = event.method_name() {
                context.action_queue.queue_action(
                    self_display_object,
                    ActionType::Method {
                        object: write.object.unwrap(),
                        name,
                        args: vec![],
                    },
                    false,
                );
            }
        }

        if write.state != new_state {
            drop(write);
            self.set_state(context, new_state);
        }

        ClipEventResult::NotHandled
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() && self.mouse_enabled() {
            for child in self.iter_render_list().rev() {
                let result = child
                    .as_interactive()
                    .and_then(|c| c.mouse_pick(context, point, require_button_mode));
                if result.is_some() {
                    return result;
                }
            }

            for child in self.0.read().hit_area.values() {
                if child.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
                    return Some((*self).into());
                }
            }
        }
        None
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'_, 'gc, '_>) -> MouseCursor {
        if self.use_hand_cursor() && self.enabled() {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
    }
}

impl<'gc> Avm1ButtonData<'gc> {
    fn play_sound(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        sound: Option<&swf::ButtonSound>,
    ) {
        if let Some((id, sound_info)) = sound {
            if let Some(sound_handle) = context
                .library
                .library_for_movie_mut(self.movie())
                .get_sound(*id)
            {
                let _ = context.start_sound(sound_handle, sound_info, None, None);
            }
        }
    }
    fn run_actions(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        condition: swf::ButtonActionCondition,
        key_code: Option<ButtonKeyCode>,
    ) -> ClipEventResult {
        let mut handled = ClipEventResult::NotHandled;
        if let Some(parent) = self.base.base.parent {
            for action in &self.static_data.read().actions {
                if action.condition == condition
                    && (action.condition != swf::ButtonActionCondition::KEY_PRESS
                        || action.key_code == key_code)
                {
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
        self.static_data.read().swf.clone()
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
    condition: swf::ButtonActionCondition,
    key_code: Option<ButtonKeyCode>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum ButtonTracking {
    Push,
    Menu,
}

/// Static data shared between all instances of a button.
#[allow(dead_code)]
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct ButtonStatic {
    swf: Arc<SwfMovie>,
    id: CharacterId,
    records: Vec<swf::ButtonRecord>,
    actions: Vec<ButtonAction>,

    /// The sounds to play on state changes for this button.
    up_to_over_sound: Option<swf::ButtonSound>,
    over_to_down_sound: Option<swf::ButtonSound>,
    down_to_over_sound: Option<swf::ButtonSound>,
    over_to_up_sound: Option<swf::ButtonSound>,
}
