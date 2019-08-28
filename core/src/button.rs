use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::events::ButtonEvent;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Button<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, ButtonStatic>,
    state: ButtonState,
    children: [BTreeMap<Depth, DisplayNode<'gc>>; 4],
    tracking: ButtonTracking,
}

const UP_STATE: usize = 0;
const OVER_STATE: usize = 1;
const DOWN_STATE: usize = 2;
const HIT_STATE: usize = 3;

impl<'gc> Button<'gc> {
    pub fn from_swf_tag(
        button: &swf::Button,
        library: &crate::library::Library<'gc>,
        gc_context: gc_arena::MutationContext<'gc, '_>,
    ) -> Self {
        use swf::ButtonState;
        let mut children = [
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
        ];
        for record in &button.records {
            match library.instantiate_display_object(record.id, gc_context) {
                Ok(child) => {
                    child
                        .write(gc_context)
                        .set_matrix(&record.matrix.clone().into());
                    child
                        .write(gc_context)
                        .set_color_transform(&record.color_transform.clone().into());
                    for state in &record.states {
                        let i = match state {
                            ButtonState::Up => UP_STATE,
                            ButtonState::Over => OVER_STATE,
                            ButtonState::Down => DOWN_STATE,
                            ButtonState::HitTest => HIT_STATE,
                        };
                        children[i].insert(record.depth, child);
                    }
                }
                Err(error) => {
                    log::error!(
                        "Button ID {}: could not instantiate child ID {}: {}",
                        button.id,
                        record.id,
                        error
                    );
                }
            }
        }

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
            actions,
        };

        Button {
            base: Default::default(),
            static_data: gc_arena::Gc::allocate(gc_context, static_data),
            children,
            state: self::ButtonState::Up,
            tracking: if button.is_track_as_menu {
                ButtonTracking::Menu
            } else {
                ButtonTracking::Push
            },
        }
    }

    fn children_in_state(
        &self,
        state: ButtonState,
    ) -> impl std::iter::DoubleEndedIterator<Item = &DisplayNode<'gc>> {
        let i = match state {
            ButtonState::Up => UP_STATE,
            ButtonState::Over => OVER_STATE,
            ButtonState::Down => DOWN_STATE,
            ButtonState::Hit => HIT_STATE,
        };
        self.children[i].values()
    }

    fn children_in_state_mut(
        &mut self,
        state: ButtonState,
    ) -> impl std::iter::DoubleEndedIterator<Item = &mut DisplayNode<'gc>> {
        let i = match state {
            ButtonState::Up => UP_STATE,
            ButtonState::Over => OVER_STATE,
            ButtonState::Down => DOWN_STATE,
            ButtonState::Hit => HIT_STATE,
        };
        self.children[i].values_mut()
    }

    pub fn handle_button_event(
        &mut self,
        context: &mut crate::player::UpdateContext<'_, 'gc, '_>,
        event: ButtonEvent,
    ) {
        let new_state = match event {
            ButtonEvent::RollOut => ButtonState::Up,
            ButtonEvent::RollOver => ButtonState::Over,
            ButtonEvent::Press => ButtonState::Down,
            ButtonEvent::Release => ButtonState::Over,
            ButtonEvent::KeyPress(key) => {
                self.run_actions(context, swf::ButtonActionCondition::KeyPress, Some(key));
                self.state
            }
        };

        match (self.state, new_state) {
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

        self.state = new_state;
    }

    fn run_actions(
        &mut self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        condition: swf::ButtonActionCondition,
        key_code: Option<u8>,
    ) {
        if let Some(parent) = self.parent() {
            for action in &self.static_data.actions {
                if action.condition == condition && action.key_code == key_code {
                    // Note that AVM1 buttons run actions relative to their parent, not themselves.
                    context.actions.push((parent, action.action_data.clone()));
                }
            }
        }
    }
}

impl<'gc> DisplayObject<'gc> for Button<'gc> {
    impl_display_object!(base);

    fn run_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // TODO: Set parent for all children. Yuck... Do this on creation instead.
        for state in &mut self.children {
            for child in state.values_mut() {
                child
                    .write(context.gc_context)
                    .set_parent(Some(context.active_clip));
            }
        }

        for child in self.children_in_state_mut(self.state) {
            child
                .write(context.gc_context)
                .set_parent(Some(context.active_clip));
            context.active_clip = *child;
            child.write(context.gc_context).run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.children_in_state_mut(self.state) {
            context.active_clip = *child;
            child.write(context.gc_context).run_post_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(self.transform());

        for child in self.children_in_state(self.state) {
            child.read().render(context);
        }
        context.transform_stack.pop();
    }

    fn hit_test(&self, point: (Twips, Twips)) -> bool {
        // Use hit state to determine hit area; otherwise use current state.
        let hit_state = if !self.children[HIT_STATE].is_empty() {
            ButtonState::Hit
        } else {
            self.state
        };
        for child in self.children_in_state(hit_state).rev() {
            if child.read().world_bounds().contains(point) {
                return true;
            }
        }

        false
    }

    fn mouse_pick(
        &self,
        self_node: DisplayNode<'gc>,
        point: (Twips, Twips),
    ) -> Option<DisplayNode<'gc>> {
        // The button is hovered if the mouse is over any child nodes.
        if self.hit_test(point) {
            Some(self_node)
        } else {
            None
        }
    }

    fn as_button(&self) -> Option<&Self> {
        Some(self)
    }

    fn as_button_mut(&mut self) -> Option<&mut Self> {
        Some(self)
    }
}

unsafe impl<'gc> gc_arena::Collect for Button<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for state in &self.children {
            for child in state.values() {
                child.trace(cc);
            }
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
    Hit,
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
    actions: Vec<ButtonAction>,
}

unsafe impl<'gc> gc_arena::Collect for ButtonStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
