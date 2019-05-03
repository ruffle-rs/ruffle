use crate::display_object::{
    DisplayObject, DisplayObjectBase, DisplayObjectImpl, DisplayObjectUpdate,
};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use bacon_rajan_cc::{Cc, Trace, Tracer};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Button {
    base: DisplayObjectBase,
    children: Vec<(Depth, bool, bool, bool, bool, Cc<RefCell<DisplayObject>>)>,
    state: ButtonState,
    release_actions: Vec<u8>,
}

impl Button {
    pub fn new(button: &swf::Button, library: &crate::library::Library) -> Button {
        use swf::ButtonState;
        let mut children = vec![];
        for record in &button.records {
            let mut child = library.instantiate_display_object(record.id).unwrap();
            child.set_matrix(&record.matrix.clone().into());
            let child_ptr = Cc::new(RefCell::new(DisplayObject::new(Box::new(child))));
            children.push((
                record.depth,
                record.states.contains(&ButtonState::Up),
                record.states.contains(&ButtonState::Over),
                record.states.contains(&ButtonState::Down),
                record.states.contains(&ButtonState::HitTest),
                child_ptr,
            ));
        }

        let mut release_actions = vec![];
        for actions in &button.actions {
            if actions
                .conditions
                .contains(&swf::ButtonActionCondition::OverDownToOverUp)
            {
                release_actions = actions.action_data.clone();
            }
        }

        Button {
            base: Default::default(),
            children,
            state: self::ButtonState::Up,
            release_actions,
        }
    }
}

impl_display_object!(Button, base);

impl DisplayObjectUpdate for Button {
    fn run_frame(&mut self, context: &mut UpdateContext) {
        if self.state == ButtonState::Down {
            let mut action_context = crate::avm1::ActionContext {
                global_time: context.global_time,
                active_clip: &mut crate::movie_clip::MovieClip::new(),
                audio: context.audio,
            };
            context
                .avm1
                .do_action(&mut action_context, &self.release_actions[..]);
            self.state = ButtonState::Up;
        } else if self.state == ButtonState::Up {
            let dx = self.get_matrix().tx - context.mouse_pos.0;
            let dy = self.get_matrix().ty - context.mouse_pos.1;
            let len = f32::sqrt(dx * dx + dy * dy);
            self.state = if len > 20.0 {
                ButtonState::Up
            } else {
                ButtonState::Over
            };
        }
        for (_, _, _, _, _, child) in &mut self.children {
            child.borrow_mut().run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext) {
        for (_, _, _, _, _, child) in &mut self.children {
            child.borrow_mut().run_post_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        for (_, up, over, down, _, child) in &self.children {
            if (*up && self.state == ButtonState::Up)
                || (*over && self.state == ButtonState::Over)
                || (*down && self.state == ButtonState::Down)
            {
                child.borrow().render(context);
            }
        }
        context.transform_stack.pop();
    }

    fn handle_click(&mut self, _pos: (f32, f32)) {
        if self.state == ButtonState::Over {
            self.state = ButtonState::Down;
        }
    }
}

impl Trace for Button {
    fn trace(&mut self, tracer: &mut Tracer) {
        for (_, _, _, _, _, child) in &mut self.children {
            child.borrow_mut().trace(tracer);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ButtonState {
    Up,
    Over,
    Down,
}
