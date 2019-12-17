use crate::context::{ActionType, RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::events::ButtonEvent;
use crate::prelude::*;
use gc_arena::{Collect, GcCell};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Button<'gc>(GcCell<'gc, ButtonData<'gc>>);

#[derive(Clone, Debug)]
pub struct ButtonData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, ButtonStatic>,
    state: ButtonState,
    hit_area: BTreeMap<Depth, DisplayObject<'gc>>,
    children: BTreeMap<Depth, DisplayObject<'gc>>,
    tracking: ButtonTracking,
    initialized: bool,
}

impl<'gc> Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        _library: &crate::library::Library<'gc>,
        gc_context: gc_arena::MutationContext<'gc, '_>,
    ) -> Self {
        let mut actions = vec![];
        for action in &button.actions {
            let action_data = crate::tag_utils::SwfSlice {
                data: std::sync::Arc::new(action.action_data.clone()),
                start: 0,
                end: action.action_data.len(),
            };
            for condition in &action.conditions {
                let button_action = ButtonAction {
                    action_data: action_data.clone(),
                    condition: *condition,
                    key_code: action.key_code,
                };
                actions.push(button_action);
            }
        }

        let static_data = ButtonStatic {
            id: button.id,
            records: button.records.clone(),
            actions,
        };

        Button(GcCell::allocate(
            gc_context,
            ButtonData {
                base: Default::default(),
                static_data: gc_arena::Gc::allocate(gc_context, static_data),
                children: BTreeMap::new(),
                hit_area: BTreeMap::new(),
                state: self::ButtonState::Up,
                initialized: false,
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
}

impl<'gc> TDisplayObject<'gc> for Button<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
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

    fn as_button(&self) -> Option<Self> {
        Some(*self)
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
        for record in &self.static_data.records {
            if record.states.contains(&swf_state) {
                if let Ok(mut child) = context.library.instantiate_display_object(
                    record.id,
                    context.gc_context,
                    &context.system_prototypes,
                ) {
                    child.set_parent(context.gc_context, Some(self_display_object));
                    child.set_matrix(context.gc_context, &record.matrix.clone().into());
                    child.set_color_transform(
                        context.gc_context,
                        &record.color_transform.clone().into(),
                    );
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

            for record in &self.static_data.records {
                if record.states.contains(&swf::ButtonState::HitTest) {
                    match context.library.instantiate_display_object(
                        record.id,
                        context.gc_context,
                        &context.system_prototypes,
                    ) {
                        Ok(mut child) => {
                            {
                                child.set_matrix(context.gc_context, &record.matrix.clone().into());
                                child.set_parent(context.gc_context, Some(self_display_object));
                            }
                            self.hit_area.insert(record.depth.into(), child);
                        }
                        Err(error) => {
                            log::error!(
                                "Button ID {}: could not instantiate child ID {}: {}",
                                self.static_data.id,
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
            ButtonEvent::KeyPress(key) => {
                self.run_actions(context, swf::ButtonActionCondition::KeyPress, Some(key));
                cur_state
            }
        };

        match (cur_state, new_state) {
            (ButtonState::Up, ButtonState::Over) => {
                self.run_actions(context, swf::ButtonActionCondition::IdleToOverUp, None);
            }
            (ButtonState::Over, ButtonState::Up) => {
                self.run_actions(context, swf::ButtonActionCondition::OverUpToIdle, None);
            }
            (ButtonState::Over, ButtonState::Down) => {
                self.run_actions(context, swf::ButtonActionCondition::OverUpToOverDown, None);
            }
            (ButtonState::Down, ButtonState::Over) => {
                self.run_actions(context, swf::ButtonActionCondition::OverDownToOverUp, None);
            }
            _ => (),
        }

        self.set_state(self_display_object, context, new_state);
    }
    fn run_actions(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        condition: swf::ButtonActionCondition,
        key_code: Option<u8>,
    ) {
        if let Some(parent) = self.base.parent {
            for action in &self.static_data.actions {
                if action.condition == condition && action.key_code == key_code {
                    // Note that AVM1 buttons run actions relative to their parent, not themselves.
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
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum ButtonState {
    Up,
    Over,
    Down,
}

#[derive(Clone)]
struct ButtonAction {
    action_data: crate::tag_utils::SwfSlice,
    condition: swf::ButtonActionCondition,
    key_code: Option<u8>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ButtonTracking {
    Push,
    Menu,
}

/// Static data shared between all instances of a button.
#[allow(dead_code)]
#[derive(Clone)]
struct ButtonStatic {
    id: CharacterId,
    records: Vec<swf::ButtonRecord>,
    actions: Vec<ButtonAction>,
}

unsafe impl gc_arena::Collect for ButtonStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
