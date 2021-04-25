use crate::avm1::Object as Avm1Object;
use crate::avm2::{
    Activation as Avm2Activation, Namespace as Avm2Namespace, Object as Avm2Object,
    QName as Avm2QName, StageObject as Avm2StageObject, TObject as Avm2TObject, Value as Avm2Value,
};
use crate::backend::ui::MouseCursor;
use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::container::{dispatch_added_event, dispatch_removed_event};
use crate::display_object::{DisplayObjectBase, MovieClip, TDisplayObject};
use crate::events::{ButtonKeyCode, ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::tag_utils::{SwfMovie, SwfSlice};
use crate::types::{Degrees, Percent};
use crate::vminterface::Instantiator;
use gc_arena::{Collect, GcCell, MutationContext};
use std::convert::TryFrom;
use std::sync::Arc;
use swf::ButtonActionCondition;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Avm2Button<'gc>(GcCell<'gc, Avm2ButtonData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Avm2ButtonData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: GcCell<'gc, ButtonStatic>,

    /// The current button state to render.
    state: ButtonState,

    /// The display object tree to render when the button is in the UP state.
    up_state: Option<DisplayObject<'gc>>,

    /// The display object tree to render when the button is in the OVER state.
    over_state: Option<DisplayObject<'gc>>,

    /// The display object tree to render when the button is in the DOWN state.
    down_state: Option<DisplayObject<'gc>>,

    /// The display object tree to use for mouse hit checks.
    hit_area: Option<DisplayObject<'gc>>,

    /// The current tracking mode of this button.
    tracking: ButtonTracking,

    /// The AVM2 representation of this button.
    object: Option<Avm2Object<'gc>>,
    has_focus: bool,
    enabled: bool,
    use_hand_cursor: bool,
}

impl<'gc> Avm2Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        source_movie: &SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Self {
        let mut actions = vec![];
        for action in &button.actions {
            let action_data = source_movie
                .to_unbounded_subslice(action.action_data)
                .unwrap();
            let bits = action.conditions.bits();
            let mut bit = 1u16;
            while bits & !(bit - 1) != 0 {
                if bits & bit != 0 {
                    actions.push(ButtonAction {
                        action_data: action_data.clone(),
                        condition: ButtonActionCondition::from_bits_truncate(bit),
                        key_code: action
                            .key_code
                            .and_then(|k| ButtonKeyCode::try_from(k).ok()),
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

        Avm2Button(GcCell::allocate(
            context.gc_context,
            Avm2ButtonData {
                base: Default::default(),
                static_data: GcCell::allocate(context.gc_context, static_data),
                state: self::ButtonState::Up,
                hit_area: None,
                up_state: None,
                over_state: None,
                down_state: None,
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

    /// Construct a given state of the button and return it's containing
    /// DisplayObject.
    ///
    /// In contrast to AVM1Button, the AVM2 variety constructs all of it's
    /// children once and stores them in four named slots, either on their own
    /// (if they are singular) or in `Sprite`s created specifically to store
    /// button children. This means that, for example, a child that exists in
    /// multiple states in the SWF will actually be instantiated multiple
    /// times.
    fn create_state(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        swf_state: swf::ButtonState,
    ) -> DisplayObject<'gc> {
        let movie = self
            .movie()
            .expect("All SWF-defined buttons should have movies");
        let mut children = Vec::new();
        let static_data = self.0.read().static_data;

        for record in static_data.read().records.iter() {
            if record.states.contains(swf_state) {
                match context
                    .library
                    .library_for_movie_mut(movie.clone())
                    .instantiate_by_id(record.id, context.gc_context)
                {
                    Ok(child) => {
                        child.set_matrix(context.gc_context, &record.matrix);
                        child.set_depth(context.gc_context, record.depth.into());

                        if swf_state != swf::ButtonState::HIT_TEST {
                            child.set_color_transform(
                                context.gc_context,
                                &record.color_transform.clone().into(),
                            );
                        }

                        children.push((child, record.depth));
                    }
                    Err(error) => {
                        log::error!(
                            "Button ID {}: could not instantiate child ID {}: {}",
                            static_data.read().id,
                            record.id,
                            error
                        );
                    }
                };
            }
        }

        if children.len() > 1 {
            let child = children.first().cloned().unwrap().0;

            child.set_parent(context.gc_context, Some(self.into()));
            child.post_instantiation(context, child, None, Instantiator::Movie, false);
            child.run_frame(context);

            child
        } else {
            let empty_slice = SwfSlice::empty(movie);
            let mut sprite_proto = context.avm2.prototypes().sprite;
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let sprite_constr = sprite_proto
                .get_property(
                    sprite_proto,
                    &Avm2QName::new(Avm2Namespace::public(), "constructor"),
                    &mut activation,
                )
                .unwrap()
                .coerce_to_object(&mut activation)
                .unwrap();

            drop(activation);

            let state_sprite = MovieClip::new(empty_slice, context.gc_context);

            state_sprite.set_avm2_constructor(context.gc_context, Some(sprite_constr));
            state_sprite.construct_frame(context);

            for (child, depth) in children {
                let removed_child = state_sprite.replace_at_depth(context, child, depth.into());

                child.set_parent(context.gc_context, Some(self.into()));
                child.post_instantiation(context, child, None, Instantiator::Movie, false);
                child.run_frame(context);

                dispatch_added_event(self.into(), child, false, context);
                if let Some(removed_child) = removed_child {
                    dispatch_removed_event(removed_child, context);
                }
            }

            state_sprite.into()
        }
    }

    /// Change the rendered state of the button.
    pub fn set_state(self, context: &mut UpdateContext<'_, 'gc, '_>, state: ButtonState) {
        self.0.write(context.gc_context).state = state;
    }

    /// Get the display object that represents a particular button state.
    pub fn get_state_child(self, state: swf::ButtonState) -> Option<DisplayObject<'gc>> {
        match state {
            swf::ButtonState::UP => self.0.read().up_state,
            swf::ButtonState::OVER => self.0.read().over_state,
            swf::ButtonState::DOWN => self.0.read().down_state,
            swf::ButtonState::HIT_TEST => self.0.read().hit_area,
            _ => None,
        }
    }

    /// Set the display object that represents a particular button state.
    pub fn set_state_child(
        self,
        gc_context: MutationContext<'gc, '_>,
        state: swf::ButtonState,
        child: Option<DisplayObject<'gc>>,
    ) {
        match state {
            swf::ButtonState::UP => self.0.write(gc_context).up_state = child,
            swf::ButtonState::OVER => self.0.write(gc_context).over_state = child,
            swf::ButtonState::DOWN => self.0.write(gc_context).down_state = child,
            swf::ButtonState::HIT_TEST => self.0.write(gc_context).hit_area = child,
            _ => (),
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

impl<'gc> TDisplayObject<'gc> for Avm2Button<'gc> {
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
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        self.set_default_instance_name(context);

        let up_state = self.create_state(context, swf::ButtonState::UP);
        let over_state = self.create_state(context, swf::ButtonState::OVER);
        let down_state = self.create_state(context, swf::ButtonState::DOWN);
        let hit_area = self.create_state(context, swf::ButtonState::HIT_TEST);

        let mut write = self.0.write(context.gc_context);
        write.up_state = Some(up_state);
        write.over_state = Some(over_state);
        write.down_state = Some(down_state);
        write.hit_area = Some(hit_area);

        if write.object.is_none() {
            let object = Avm2StageObject::for_display_object(
                context.gc_context,
                display_object,
                context.avm2.prototypes().simplebutton,
            );
            write.object = Some(object.into());

            drop(write);

            if run_frame {
                self.run_frame(context);
            }
        } else {
            drop(write);
        }

        self.set_state(context, ButtonState::Up);
    }

    fn run_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let up_state = self.0.read().up_state;
        if let Some(up_state) = up_state {
            up_state.run_frame(context);
        }

        let over_state = self.0.read().over_state;
        if let Some(over_state) = over_state {
            over_state.run_frame(context);
        }

        let down_state = self.0.read().up_state;
        if let Some(down_state) = down_state {
            down_state.run_frame(context);
        }

        let hit_area = self.0.read().hit_area;
        if let Some(hit_area) = hit_area {
            hit_area.run_frame(context);
        }
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc>) {
        let state = self.0.read().state;
        let current_state = self.get_state_child(state.into());

        if let Some(state) = current_state {
            state.render(context);
        }
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
        let hit_area = self.0.read().hit_area;

        if let Some(hit_area) = hit_area {
            hit_area.hit_test_shape(context, point, options)
        } else {
            false
        }
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        self_node: DisplayObject<'gc>,
        point: (Twips, Twips),
    ) -> Option<DisplayObject<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.visible() {
            let state = self.0.read().state;
            let state_child = self.get_state_child(state.into());

            if let Some(state_child) = state_child {
                let mouse_pick = state_child.mouse_pick(context, state_child, point);
                if mouse_pick.is_some() {
                    return mouse_pick;
                }
            }

            let hit_area = self.0.read().hit_area;
            if let Some(hit_area) = hit_area {
                if hit_area.hit_test_shape(context, point, HitTestOptions::MOUSE_PICK) {
                    return Some(self_node);
                }
            }
        }
        None
    }

    fn mouse_cursor(&self) -> MouseCursor {
        if self.use_hand_cursor() {
            MouseCursor::Hand
        } else {
            MouseCursor::Arrow
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .object
            .map(Avm2Value::from)
            .unwrap_or(Avm2Value::Undefined)
    }

    fn as_avm2_button(&self) -> Option<Self> {
        Some(*self)
    }

    fn allow_as_mask(&self) -> bool {
        let state = self.0.read().state;
        let current_state = self.get_state_child(state.into());

        if let Some(current_state) = current_state.and_then(|cs| cs.as_container()) {
            current_state.is_empty()
        } else {
            false
        }
    }

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed by its parent, and
    /// so forth.
    fn handle_clip_event(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ClipEvent,
    ) -> ClipEventResult {
        if !self.visible() {
            return ClipEventResult::NotHandled;
        }

        if !self.enabled() && !matches!(event, ClipEvent::KeyPress { .. }) {
            return ClipEventResult::NotHandled;
        }

        if event.propagates() {
            let state = self.0.read().state;
            let current_state = self.get_state_child(state.into());

            if let Some(current_state) = current_state {
                if current_state.handle_clip_event(context, event) == ClipEventResult::Handled {
                    return ClipEventResult::Handled;
                }
            }
        }

        let mut handled = ClipEventResult::NotHandled;
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
                    swf::ButtonActionCondition::KEY_PRESS,
                    Some(key_code),
                );
                cur_state
            }
            _ => return ClipEventResult::NotHandled,
        };

        match (cur_state, new_state) {
            (ButtonState::Up, ButtonState::Over) => {
                write.run_actions(context, swf::ButtonActionCondition::IDLE_TO_OVER_UP, None);
                write.play_sound(context, write.static_data.read().up_to_over_sound.as_ref());
            }
            (ButtonState::Over, ButtonState::Up) => {
                write.run_actions(context, swf::ButtonActionCondition::OVER_UP_TO_IDLE, None);
                write.play_sound(context, write.static_data.read().over_to_up_sound.as_ref());
            }
            (ButtonState::Over, ButtonState::Down) => {
                write.run_actions(
                    context,
                    swf::ButtonActionCondition::OVER_UP_TO_OVER_DOWN,
                    None,
                );
                write.play_sound(
                    context,
                    write.static_data.read().over_to_down_sound.as_ref(),
                );
            }
            (ButtonState::Down, ButtonState::Over) => {
                write.run_actions(
                    context,
                    swf::ButtonActionCondition::OVER_DOWN_TO_OVER_UP,
                    None,
                );
                write.play_sound(
                    context,
                    write.static_data.read().down_to_over_sound.as_ref(),
                );
            }
            _ => (),
        };

        // Queue ActionScript-defined event handlers after the SWF defined ones.
        // (e.g., clip.onRelease = foo).
        /*if context.swf.version() >= 6 {
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
        }*/

        if write.state != new_state {
            drop(write);
            self.set_state(context, new_state);
        }

        handled
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

impl<'gc> Avm2ButtonData<'gc> {
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
        if let Some(parent) = self.base.parent {
            for action in &self.static_data.read().actions {
                if action.condition == condition
                    && (action.condition != swf::ButtonActionCondition::KEY_PRESS
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
    action_data: crate::tag_utils::SwfSlice,
    condition: swf::ButtonActionCondition,
    key_code: Option<ButtonKeyCode>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
enum ButtonTracking {
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
