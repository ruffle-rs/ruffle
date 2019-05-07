use crate::backend::render::ShapeHandle;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObjectImpl, DisplayObjectUpdate};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};

#[derive(Clone)]
pub struct Graphic {
    base: DisplayObjectBase,
    shape_handle: ShapeHandle,
    x_min: f32,
    y_min: f32,
}

impl Graphic {
    pub fn from_swf_tag(swf_shape: &swf::Shape, context: &mut UpdateContext) -> Graphic {
        let shape_handle = context.renderer.register_shape(swf_shape);
        Graphic {
            base: Default::default(),
            shape_handle,
            x_min: swf_shape.shape_bounds.x_min,
            y_min: swf_shape.shape_bounds.y_min,
        }
    }
}

impl_display_object!(Graphic, base);

impl DisplayObjectUpdate for Graphic {
    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        context.transform_stack.push(self.transform());

        context
            .renderer
            .render_shape(self.shape_handle, context.transform_stack.transform());

        context.transform_stack.pop();
    }
}

impl Trace for Graphic {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop
    }
}
