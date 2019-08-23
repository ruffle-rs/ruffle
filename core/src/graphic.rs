use crate::backend::render::ShapeHandle;
use crate::display_object::{DisplayObject, DisplayObjectBase};
use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;

#[derive(Clone)]
pub struct Graphic<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: gc_arena::Gc<'gc, GraphicStatic>,
}

impl<'gc> Graphic<'gc> {
    pub fn from_swf_tag(context: &mut UpdateContext<'_, 'gc, '_>, swf_shape: &swf::Shape) -> Self {
        let static_data = GraphicStatic {
            id: swf_shape.id,
            render_handle: context.renderer.register_shape(swf_shape),
            bounds: swf_shape.shape_bounds.clone().into(),
        };
        Graphic {
            base: Default::default(),
            static_data: gc_arena::Gc::allocate(context.gc_context, static_data),
        }
    }
}

impl<'gc> DisplayObject<'gc> for Graphic<'gc> {
    impl_display_object!(base);

    fn local_bounds(&self) -> BoundingBox {
        self.static_data.bounds.clone()
    }

    fn world_bounds(&self) -> BoundingBox {
        // TODO: Use dirty flags and cache this.
        let mut bounds = self.local_bounds().transform(self.matrix());
        let mut node = self.parent();
        while let Some(display_object) = node {
            let display_object = display_object.read();
            bounds = bounds.transform(display_object.matrix());
            node = display_object.parent();
        }
        bounds
    }

    fn run_frame(&mut self, _context: &mut UpdateContext) {
        // Noop
    }

    fn render(&self, context: &mut RenderContext) {
        if !self.world_bounds().intersects(&context.view_bounds) {
            // Off-screen; culled
            return;
        }

        context.transform_stack.push(self.transform());

        context.renderer.render_shape(
            self.static_data.render_handle,
            context.transform_stack.transform(),
        );

        context.transform_stack.pop();
    }
}

unsafe impl<'gc> gc_arena::Collect for Graphic<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
    }
}

/// Static data shared between all instances of a graphic.
#[allow(dead_code)]
struct GraphicStatic {
    id: CharacterId,
    render_handle: ShapeHandle,
    bounds: BoundingBox,
}

unsafe impl<'gc> gc_arena::Collect for GraphicStatic {
    #[inline]
    fn needs_trace() -> bool {
        false
    }
}
