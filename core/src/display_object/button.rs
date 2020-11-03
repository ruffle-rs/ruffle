use crate::avm1::{Object, StageObject, Value};
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::sync::Arc;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Button<'gc>(GcCell<'gc, ButtonData<'gc>>);

#[derive(Clone, Debug)]
pub struct ButtonData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: GcCell<'gc, ButtonStatic>,
    state: ButtonState,
    hit_area: BTreeMap<Depth, DisplayObject<'gc>>,
    children: BTreeMap<Depth, DisplayObject<'gc>>,
    tracking: ButtonTracking,
    object: Option<Object<'gc>>,
    initialized: bool,
    has_focus: bool,
}

impl<'gc> Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        source_movie: &SwfSlice,
        _library: &crate::library::Library<'gc>,
        gc_context: gc_arena::MutationContext<'gc, '_>,
    ) -> Self {
        let mut actions = vec![];
        for action in &button.actions {
            let action_data =
                source_movie.owned_subslice(action.action_data.clone(), &source_movie.movie);
            for condition in &action.conditions {
                let button_action = ButtonAction {
                    action_data: action_data.clone(),
                    condition: *condition,
                    key_code: action
                        .key_code
                        .and_then(|k| ButtonKeyCode::try_from(k).ok()),
                };
                actions.push(button_action);
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

        Button(GcCell::allocate(
            gc_context,
            ButtonData {
                base: Default::default(),
                static_data: GcCell::allocate(gc_context, static_data),
                children: BTreeMap::new(),
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
        &self,
        self_display_object: DisplayObject<'gc>,
        context: &mut crate::context::UpdateContext<'_, 'gc, '_>,
        state: ButtonState,
    ) {
        // Clear previous child execution list.
        for child in self.children() {
            child.set_next_sibling(context.gc_context, None);
            child.set_prev_sibling(context.gc_context, None);
        }
        self.set_first_child(context.gc_context, None);

        let movie = self.movie().unwrap();
        let mut write = self.0.write(context.gc_context);
        write.state = state;
        let swf_state = match state {
            ButtonState::Up => swf::ButtonState::Up,
            ButtonState::Over => swf::ButtonState::Over,
            ButtonState::Down => swf::ButtonState::Down,
        };
        write.children.clear();

        let mut new_children = Vec::new();
        for record in &write.static_data.read().records {
            if record.states.contains(&swf_state) {
                if let Ok(child) = context
                    .library
                    .library_for_movie_mut(movie.clone())
                    .instantiate_by_id(record.id, context.gc_context)
                {
                    child.set_parent(context.gc_context, Some(self_display_object));
                    child.set_matrix(context.gc_context, &record.matrix);
                    child.set_color_transform(
                        context.gc_context,
                        &record.color_transform.clone().into(),
                    );
                    child.set_depth(context.gc_context, record.depth.into());

                    new_children.push((child, record.depth.into()));
                }
            }
        }

        drop(write);

        let mut prev_child = None;
        for (child, depth) in new_children {
            // Wire up new execution list.
            if let Some(prev_child) = prev_child {
                child.set_prev_sibling(context.gc_context, Some(prev_child));
                prev_child.set_next_sibling(context.gc_context, Some(child));
            } else {
                self.set_first_child(context.gc_context, Some(child));
            }
            // Initialize child.
            child.post_instantiation(context, child, None, Instantiator::Movie, false);
            child.run_frame(context);
            self.0
                .write(context.gc_context)
                .children
                .insert(depth, child);
            prev_child = Some(child);
        }
    }
}

impl<'gc> TDisplayObject<'gc> for Button<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.read().id
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        Some(self.0.read().static_data.read().swf.clone())
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        _init_object: Option<Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        let mut mc = self.0.write(context.gc_context);
        if mc.object.is_none() {
            let object = StageObject::for_display_object(
                context.gc_context,
                display_object,
                Some(context.system_prototypes.button),
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

            self.set_state(self_display_object, context, ButtonState::Up);
            self.0.write(context.gc_context).initialized = true;

            let read = self.0.read();

            for record in &read.static_data.read().records {
                if record.states.contains(&swf::ButtonState::HitTest) {
                    match context
                        .library
                        .library_for_movie_mut(read.static_data.read().swf.clone())
                        .instantiate_by_id(record.id, context.gc_context)
                    {
                        Ok(child) => {
                            child.set_matrix(context.gc_context, &record.matrix);
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
                child.post_instantiation(context, child, None, Instantiator::Movie, false);
                self.0
                    .write(context.gc_context)
                    .hit_area
                    .insert(depth, child);
            }
        }

        for child in self.children() {
            child.run_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(&*self.transform());

        crate::display_object::render_children(context, &self.0.read().children);

        context.transform_stack.pop();
    }

    fn self_bounds(&self) -> BoundingBox {
        // No inherent bounds; contains child DisplayObjects.
        BoundingBox::default()
    }

    fn hit_test_shape(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        point: (Twips, Twips),
    ) -> bool {
        for child in self.children() {
            if child.hit_test_shape(context, point) {
                return true;
            }
        }

        false
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        self_node: DisplayObject<'gc>,
        point: (Twips, Twips),
    ) -> Option<DisplayObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() {
            for child in self.0.read().hit_area.values() {
                if child.hit_test_shape(context, point) {
                    return Some(self_node);
                }
            }
        }
        None
    }

    fn object(&self) -> Value<'gc> {
        self.0
            .read()
            .object
            .map(Value::from)
            .unwrap_or(Value::Undefined)
    }

    fn as_button(&self) -> Option<Self> {
        Some(*self)
    }

    fn allow_as_mask(&self) -> bool {
        !self.0.read().children.is_empty()
    }

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed by its parent, and
    /// so forth.
    fn handle_clip_event(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if event.propagates() {
            for child in self.children() {
                if child.handle_clip_event(context, event) == ClipEventResult::Handled {
                    return ClipEventResult::Handled;
                }
            }
        }

        let mut handled = ClipEventResult::NotHandled;
        let self_display_object = (*self).into();
        let mut write = self.0.write(context.gc_context);

        // Translate the clip event to a button event, based on how the button state changes.
        let cur_state = write.state;
        let new_state = match event {
            ClipEvent::RollOut => ButtonState::Up,
            ClipEvent::RollOver => ButtonState::Over,
            ClipEvent::Press => ButtonState::Down,
            ClipEvent::Release => ButtonState::Over,
            ClipEvent::KeyPress { key_code } => {
                handled = write.run_actions(
                    context,
                    swf::ButtonActionCondition::KeyPress,
                    Some(key_code),
                );
                cur_state
            }
            _ => return ClipEventResult::NotHandled,
        };

        match (cur_state, new_state) {
            (ButtonState::Up, ButtonState::Over) => {
                write.run_actions(context, swf::ButtonActionCondition::IdleToOverUp, None);
                write.play_sound(context, write.static_data.read().up_to_over_sound.as_ref());
            }
            (ButtonState::Over, ButtonState::Up) => {
                write.run_actions(context, swf::ButtonActionCondition::OverUpToIdle, None);
                write.play_sound(context, write.static_data.read().over_to_up_sound.as_ref());
            }
            (ButtonState::Over, ButtonState::Down) => {
                write.run_actions(context, swf::ButtonActionCondition::OverUpToOverDown, None);
                write.play_sound(
                    context,
                    write.static_data.read().over_to_down_sound.as_ref(),
                );
            }
            (ButtonState::Down, ButtonState::Over) => {
                write.run_actions(context, swf::ButtonActionCondition::OverDownToOverUp, None);
                write.play_sound(
                    context,
                    write.static_data.read().down_to_over_sound.as_ref(),
                );
            }
            _ => (),
        };

        // Queue ActionScript-defined event handlers after the SWF defined ones.
        // (e.g., clip.onRelease = foo).
        if context.swf.version() >= 6 {
            if let Some(name) = event.method_name() {
                context.action_queue.queue_actions(
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
            self.set_state(self_display_object, context, new_state);
        }

        handled
    }

    fn get_child_by_name(&self, name: &str, case_sensitive: bool) -> Option<DisplayObject<'gc>> {
        crate::display_object::get_child_by_name(&self.0.read().children, name, case_sensitive)
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn on_focus_changed(&self, context: MutationContext<'gc, '_>, focused: bool) {
        self.0.write(context).has_focus = focused;
    }

    fn unload(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let had_focus = self.0.read().has_focus;
        if had_focus {
            let tracker = context.focus_tracker;
            tracker.set(None, context);
        }
    }
}

impl<'gc> ButtonData<'gc> {
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
                let _ = context.audio.start_sound(sound_handle, sound_info);
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
        if let Some(parent) = self.base.parent {
            for action in &self.static_data.read().actions {
                if action.condition == condition
                    && (action.condition != swf::ButtonActionCondition::KeyPress
                        || action.key_code == key_code)
                {
                    // Note that AVM1 buttons run actions relative to their parent, not themselves.
                    handled = ClipEventResult::Handled;
                    context.action_queue.queue_actions(
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

unsafe impl<'gc> gc_arena::Collect for ButtonData<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for child in self.children.values() {
            child.trace(cc);
        }
        for child in self.hit_area.values() {
            child.trace(cc);
        }
        self.base.trace(cc);
        self.static_data.trace(cc);
        self.object.trace(cc);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum ButtonState {
    Up,
    Over,
    Down,
}

#[derive(Clone, Debug)]
struct ButtonAction {
    action_data: crate::tag_utils::SwfSlice,
    condition: swf::ButtonActionCondition,
    key_code: Option<ButtonKeyCode>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ButtonTracking {
    Push,
    Menu,
}

/// Static data shared between all instances of a button.
#[allow(dead_code)]
#[derive(Clone, Debug)]
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

unsafe impl gc_arena::Collect for ButtonStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
