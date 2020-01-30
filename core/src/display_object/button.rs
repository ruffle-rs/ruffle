use crate::avm1::{Object, StageObject, Value};
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::events::{ButtonEvent, ButtonEventResult, ButtonKeyCode};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
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
            let action_data = source_movie.owned_subslice(action.action_data.clone());
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
            },
        ))
    }

    pub fn handle_button_event(
        &mut self,
        context: &mut crate::context::UpdateContext<'_, 'gc, '_>,
        event: ButtonEvent,
    ) {
        self.0
            .write(context.gc_context)
            .handle_button_event((*self).into(), context, event)
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
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
        proto: Object<'gc>,
    ) {
        let mut mc = self.0.write(gc_context);
        if mc.object.is_none() {
            let object = StageObject::for_display_object(gc_context, display_object, Some(proto));
            mc.object = Some(object.into());
        }
    }

    fn run_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.0
            .write(context.gc_context)
            .run_frame((*self).into(), context)
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(&*self.transform());

        crate::display_object::render_children(context, &self.0.read().children);

        context.transform_stack.pop();
    }

    fn hit_test(&self, point: (Twips, Twips)) -> bool {
        for child in self.0.read().hit_area.values().rev() {
            if child.world_bounds().contains(point) {
                return true;
            }
        }

        false
    }

    fn mouse_pick(
        &self,
        self_node: DisplayObject<'gc>,
        point: (Twips, Twips),
    ) -> Option<DisplayObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.hit_test(point) {
            Some(self_node)
        } else {
            None
        }
    }

    fn propagate_button_event(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ButtonEvent,
    ) -> ButtonEventResult {
        for child in self.children() {
            if child.propagate_button_event(context, event) == ButtonEventResult::Handled {
                return ButtonEventResult::Handled;
            }
        }
        match event {
            ButtonEvent::KeyPress { key_code } => self.0.write(context.gc_context).run_actions(
                context,
                swf::ButtonActionCondition::KeyPress,
                Some(key_code),
            ),
            _ => ButtonEventResult::NotHandled,
        }
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
}

impl<'gc> ButtonData<'gc> {
    fn set_state(
        &mut self,
        self_display_object: DisplayObject<'gc>,
        context: &mut crate::context::UpdateContext<'_, 'gc, '_>,
        state: ButtonState,
    ) {
        self.state = state;
        let swf_state = match state {
            ButtonState::Up => swf::ButtonState::Up,
            ButtonState::Over => swf::ButtonState::Over,
            ButtonState::Down => swf::ButtonState::Down,
        };
        self.children.clear();
        for record in &self.static_data.read().records {
            if record.states.contains(&swf_state) {
                if let Ok(mut child) = context
                    .library
                    .library_for_movie_mut(self.movie())
                    .instantiate_by_id(record.id, context.gc_context, &context.system_prototypes)
                {
                    child.set_parent(context.gc_context, Some(self_display_object));
                    child.set_matrix(context.gc_context, &record.matrix.clone().into());
                    child.set_color_transform(
                        context.gc_context,
                        &record.color_transform.clone().into(),
                    );
                    child.set_depth(context.gc_context, record.depth.into());
                    self.children.insert(record.depth.into(), child);
                }
            }
        }
    }

    fn run_frame(
        &mut self,
        self_display_object: DisplayObject<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) {
        // TODO: Move this to post_instantiation.
        if !self.initialized {
            self.initialized = true;
            self.set_state(self_display_object, context, ButtonState::Up);

            for record in &self.static_data.read().records {
                if record.states.contains(&swf::ButtonState::HitTest) {
                    match context
                        .library
                        .library_for_movie_mut(self.static_data.read().swf.clone())
                        .instantiate_by_id(
                            record.id,
                            context.gc_context,
                            &context.system_prototypes,
                        ) {
                        Ok(mut child) => {
                            {
                                child.set_matrix(context.gc_context, &record.matrix.clone().into());
                                child.set_parent(context.gc_context, Some(self_display_object));
                                child.set_depth(context.gc_context, record.depth.into());
                            }
                            self.hit_area.insert(record.depth.into(), child);
                        }
                        Err(error) => {
                            log::error!(
                                "Button ID {}: could not instantiate child ID {}: {}",
                                self.static_data.read().id,
                                record.id,
                                error
                            );
                        }
                    }
                }
            }
        }

        for child in self.children.values_mut() {
            child.run_frame(context);
        }
    }

    fn handle_button_event(
        &mut self,
        self_display_object: DisplayObject<'gc>,
        context: &mut crate::context::UpdateContext<'_, 'gc, '_>,
        event: ButtonEvent,
    ) {
        let cur_state = self.state;
        let new_state = match event {
            ButtonEvent::RollOut => ButtonState::Up,
            ButtonEvent::RollOver => ButtonState::Over,
            ButtonEvent::Press => ButtonState::Down,
            ButtonEvent::Release => ButtonState::Over,
            ButtonEvent::KeyPress { key_code } => {
                self.run_actions(
                    context,
                    swf::ButtonActionCondition::KeyPress,
                    Some(key_code),
                );
                cur_state
            }
        };

        match (cur_state, new_state) {
            (ButtonState::Up, ButtonState::Over) => {
                self.run_actions(context, swf::ButtonActionCondition::IdleToOverUp, None);
                self.play_sound(context, self.static_data.read().up_to_over_sound.as_ref());
            }
            (ButtonState::Over, ButtonState::Up) => {
                self.run_actions(context, swf::ButtonActionCondition::OverUpToIdle, None);
                self.play_sound(context, self.static_data.read().over_to_up_sound.as_ref());
            }
            (ButtonState::Over, ButtonState::Down) => {
                self.run_actions(context, swf::ButtonActionCondition::OverUpToOverDown, None);
                self.play_sound(context, self.static_data.read().over_to_down_sound.as_ref());
            }
            (ButtonState::Down, ButtonState::Over) => {
                self.run_actions(context, swf::ButtonActionCondition::OverDownToOverUp, None);
                self.play_sound(context, self.static_data.read().down_to_over_sound.as_ref());
            }
            _ => (),
        }

        self.set_state(self_display_object, context, new_state);
    }

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
                context.audio.start_sound(sound_handle, sound_info);
            }
        }
    }
    fn run_actions(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        condition: swf::ButtonActionCondition,
        key_code: Option<ButtonKeyCode>,
    ) -> ButtonEventResult {
        let mut handled = ButtonEventResult::NotHandled;
        if let Some(parent) = self.base.parent {
            for action in &self.static_data.read().actions {
                if action.condition == condition
                    && (action.condition != swf::ButtonActionCondition::KeyPress
                        || action.key_code == key_code)
                {
                    // Note that AVM1 buttons run actions relative to their parent, not themselves.
                    handled = ButtonEventResult::Handled;
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
