use crate::backend::render::common::ShapeHandle;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObjectImpl, DisplayObjectUpdate};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};

pub struct Graphic {
    base: DisplayObjectBase,
    shape_handle: ShapeHandle,
    x_min: f32,
    y_min: f32,
}

impl Graphic {
    pub fn new(shape_handle: ShapeHandle, x_min: f32, y_min: f32) -> Graphic {
        Graphic {
            base: Default::default(),
            shape_handle,
            x_min,
            y_min,
        }
    }
}

impl_display_object!(Graphic, base);

impl DisplayObjectUpdate for Graphic {
    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        context.matrix_stack.push(self.get_matrix());
        context
            .color_transform_stack
            .push(self.get_color_transform());

        let world_matrix = context.matrix_stack.matrix();
        let color_transform = context.color_transform_stack.color_transform();

        context
            .renderer
            .render_shape(self.shape_handle, &world_matrix);

        context.color_transform_stack.pop();
        context.matrix_stack.pop();
    }
}

impl Trace for Graphic {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop
    }
}
