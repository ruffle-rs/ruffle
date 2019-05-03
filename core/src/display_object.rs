use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::transform::Transform;
use bacon_rajan_cc::{Trace, Tracer};

pub struct DisplayObjectBase {
    depth: Depth,
    transform: Transform,
}

impl Default for DisplayObjectBase {
    fn default() -> Self {
        Self {
            depth: Default::default(),
            transform: Default::default(),
        }
    }
}

impl DisplayObjectImpl for DisplayObjectBase {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn get_matrix(&self) -> &Matrix {
        &self.transform.matrix
    }
    fn set_matrix(&mut self, matrix: &Matrix) {
        self.transform.matrix = *matrix;
    }
    fn get_color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }
    fn set_color_transform(&mut self, color_transform: &ColorTransform) {
        self.transform.color_transform = *color_transform;
    }
}

impl DisplayObjectUpdate for DisplayObjectBase {}

impl Trace for DisplayObjectBase {
    fn trace(&mut self, _tracer: &mut Tracer) {}
}

pub trait DisplayObjectImpl: DisplayObjectUpdate {
    fn transform(&self) -> &Transform;
    fn get_matrix(&self) -> &Matrix;
    fn set_matrix(&mut self, matrix: &Matrix);
    fn get_color_transform(&self) -> &ColorTransform;
    fn set_color_transform(&mut self, color_transform: &ColorTransform);
}

pub trait DisplayObjectUpdate: Trace {
    fn run_frame(&mut self, _context: &mut UpdateContext) {}
    fn run_post_frame(&mut self, _context: &mut UpdateContext) {}
    fn render(&self, _context: &mut RenderContext) {}

    fn handle_click(&mut self, _pos: (f32, f32)) {}
}

macro_rules! impl_display_object {
    ($name:ident, $field:ident) => {
        impl crate::display_object::DisplayObjectImpl for $name {
            fn transform(&self) -> &crate::transform::Transform {
                self.$field.transform()
            }
            fn get_matrix(&self) -> &Matrix {
                self.$field.get_matrix()
            }
            fn set_matrix(&mut self, matrix: &Matrix) {
                self.$field.set_matrix(matrix)
            }
            fn get_color_transform(&self) -> &ColorTransform {
                self.$field.get_color_transform()
            }
            fn set_color_transform(&mut self, color_transform: &ColorTransform) {
                self.$field.set_color_transform(color_transform)
            }
        }
    };
}

// TODO(Herschel): We wrap in a box because using a trait object
// directly with Cc gets hairy.
// Extra heap allocation, though.
// Revisit this eventually, some possibilities:
// - Just use a dumb enum.
// - Some DST magic if we remove the Box below and mark this !Sized?
pub struct DisplayObject {
    inner: Box<DisplayObjectImpl>,
}

impl DisplayObject {
    pub fn new(inner: Box<DisplayObjectImpl>) -> DisplayObject {
        DisplayObject { inner }
    }
}

impl_display_object!(DisplayObject, inner);

impl DisplayObjectUpdate for DisplayObject {
    fn run_frame(&mut self, context: &mut UpdateContext) {
        self.inner.run_frame(context)
    }

    fn run_post_frame(&mut self, context: &mut UpdateContext) {
        self.inner.run_post_frame(context)
    }

    fn render(&self, context: &mut RenderContext) {
        self.inner.render(context)
    }

    fn handle_click(&mut self, pos: (f32, f32)) {
        self.inner.handle_click(pos)
    }
}

impl Trace for DisplayObject {
    fn trace(&mut self, tracer: &mut Tracer) {
        self.inner.trace(tracer)
    }
}
