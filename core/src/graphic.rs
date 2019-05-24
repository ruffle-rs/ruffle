use crate::backend::render::ShapeHandle;
use crate::color_transform::ColorTransform;
use crate::display_object::{DisplayObjectBase, DisplayObject};
use crate::matrix::Matrix;
use crate::player::{RenderContext, UpdateContext};

#[derive(Clone)]
pub struct Graphic {
    base: DisplayObjectBase,

    shape_handle: ShapeHandle,
}

impl Graphic {
    pub fn from_swf_tag(swf_shape: &swf::Shape, context: &mut UpdateContext) -> Graphic {
        let shape_handle = context.renderer.register_shape(swf_shape);
        Graphic {
            base: Default::default(),
            shape_handle,
        }
    }
}

impl<'gc> DisplayObject<'gc> for Graphic {
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

unsafe impl<'gc> gc_arena::Collect for Graphic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
