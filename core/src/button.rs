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
    children: HashMap<Depth, (bool, bool, bool, bool, Cc<RefCell<DisplayObject>>)>,
}

impl Button {
    pub fn new(button: &swf::Button, context: &UpdateContext) -> Button {
        use swf::ButtonState;
        let mut children = HashMap::new();;
        let library = &context.library;
        for record in &button.records {
            let mut child = library.instantiate_display_object(record.id).unwrap();
            child.set_matrix(&record.matrix.clone().into());
            let child_ptr = Cc::new(RefCell::new(DisplayObject::new(Box::new(child))));
            children.insert(
                record.depth,
                (
                    record.states.contains(&ButtonState::Up),
                    record.states.contains(&ButtonState::Over),
                    record.states.contains(&ButtonState::Down),
                    record.states.contains(&ButtonState::HitTest),
                    child_ptr,
                ),
            );
        }

        Button {
            base: Default::default(),
            children,
        }
    }
}

impl_display_object!(Button, base);

impl DisplayObjectUpdate for Button {
    fn run_frame(&mut self, context: &mut UpdateContext) {
        for (_, _, _, _, child) in self.children.values_mut() {
            child.borrow_mut().run_frame(context);
        }
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext) {
        for (_, _, _, _, child) in self.children.values_mut() {
            child.borrow_mut().run_post_frame(context);
        }
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        for (up, _, _, _, child) in self.children.values() {
            if *up {
                child.borrow().render(context);
            }
        }

        context.transform_stack.pop();
    }
}

impl Trace for Button {
    fn trace(&mut self, tracer: &mut Tracer) {
        for (_, _, _, _, child) in &mut self.children.values_mut() {
            child.borrow_mut().trace(tracer);
        }
    }
}
