use crate::backend::render::{RenderBackend, ShapeHandle};
use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::player::{RenderContext, UpdateContext};

#[derive(Clone)]
pub struct Graphic {
    base: DisplayObjectBase,

    shape_handle: ShapeHandle,
}

impl Graphic {
    pub fn from_swf_tag(swf_shape: &swf::Shape, renderer: &mut dyn RenderBackend) -> Graphic {
        let shape_handle = renderer.register_shape(swf_shape);
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
