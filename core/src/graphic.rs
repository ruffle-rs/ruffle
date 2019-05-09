use crate::backend::render::ShapeHandle;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObjectImpl};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};

#[derive(Clone, Trace, Finalize)]
pub struct Graphic {
    base: DisplayObjectBase,

    #[unsafe_ignore_trace]
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

impl DisplayObjectImpl for Graphic {
    impl_display_object!(base);

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
