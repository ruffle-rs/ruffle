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
    hit_area: BTreeMap<Depth, DisplayNode<'gc>>,
    children: BTreeMap<Depth, DisplayNode<'gc>>,
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

        Button {
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
        }
    }

    fn set_state(
        &mut self,
        context: &mut crate::player::UpdateContext<'_, 'gc, '_>,
        state: ButtonState,
    ) {
        self.state = state;
        let swf_state = match self.state {
            ButtonState::Up => swf::ButtonState::Up,
            ButtonState::Over => swf::ButtonState::Over,
            ButtonState::Down => swf::ButtonState::Down,
        };
        self.children.clear();
        for record in &self.static_data.records {
            if record.states.contains(&swf_state) {
                if let Ok(child) = context
                    .library
                    .instantiate_display_object(record.id, context.gc_context)
                {
                    child
                        .write(context.gc_context)
                        .set_parent(Some(context.active_clip));
                    child
                        .write(context.gc_context)
                        .set_matrix(&record.matrix.clone().into());
                    child
                        .write(context.gc_context)
                        .set_color_transform(&record.color_transform.clone().into());
                    self.children.insert(record.depth, child);
                }
            }
        }
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

        self.set_state(context, new_state);
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
        // TODO: Move this to post_instantiation.
        if !self.initialized {
            self.initialized = true;
            self.set_state(context, ButtonState::Up);

            for record in &self.static_data.records {
                if record.states.contains(&swf::ButtonState::HitTest) {
                    match context
                        .library
                        .instantiate_display_object(record.id, context.gc_context)
                    {
                        Ok(child) => {
                            {
                                let mut child = child.write(context.gc_context);
                                child.set_matrix(&record.matrix.clone().into());
                                child.set_parent(Some(context.active_clip));
                            }
                            self.hit_area.insert(record.depth, child);
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
            context.active_clip = *child;
            child.write(context.gc_context).run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        for child in self.children.values_mut() {
            context.active_clip = *child;
            child.write(context.gc_context).run_post_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        context.transform_stack.push(self.transform());

        for child in self.children.values() {
            child.read().render(context);
        }

        context.transform_stack.pop();
    }

    fn hit_test(&self, point: (Twips, Twips)) -> bool {
        for child in self.hit_area.values().rev() {
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
