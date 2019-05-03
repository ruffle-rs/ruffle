use crate::backend::render::ShapeHandle;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObjectImpl, DisplayObjectUpdate};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};
use bacon_rajan_cc::{Trace, Tracer};
use swf::DefineMorphShape;

pub struct MorphShape {
    base: DisplayObjectBase,
    shape_handle: ShapeHandle,
    x_min: f32,
    y_min: f32,
}

impl MorphShape {
    pub fn new(swf_tag: &DefineMorphShape) -> MorphShape {
        Graphic {
            base: Default::default(),
            shape_handle,
            x_min,
            y_min,
        }
    }
}

impl_display_object!(MorphShape, base);

impl DisplayObjectUpdate for MorphShape {
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

impl Trace for MorphShape {
    fn trace(&mut self, _tracer: &mut Tracer) {
        // Noop
    }
}
