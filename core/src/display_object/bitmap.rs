//! Bitmap display object

use crate::backend::render::BitmapHandle;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use gc_arena::{Collect, Gc, GcCell};

/// A Bitmap display object is a raw bitamp on the stage.
/// This can only be instanitated on the display list in SWFv9 AVM2 files.
/// In AVM1, this is only a library symbol that is referenced by `Graphic`.
/// Normally bitmaps are drawn in Flash as part of a Shape tag (`Graphic`),
/// but starting in AVM2, a raw `Bitmap` display object can be crated
/// with the `PlaceObject3` tag.
/// It can also be crated in ActionScript using the `Bitmap` class.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Bitmap<'gc>(GcCell<'gc, BitmapData<'gc>>);

#[derive(Clone, Debug)]
pub struct BitmapData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: Gc<'gc, BitmapStatic>,
}

impl<'gc> Bitmap<'gc> {
    pub fn new(
        context: &mut UpdateContext<'_, 'gc, '_>,
        id: CharacterId,
        bitmap_handle: BitmapHandle,
        width: u16,
        height: u16,
    ) -> Self {
        Bitmap(GcCell::allocate(
            context.gc_context,
            BitmapData {
                base: Default::default(),
                static_data: Gc::allocate(
                    context.gc_context,
                    BitmapStatic {
                        id,
                        bitmap_handle,
                        width,
                        height,
                    },
                ),
            },
        ))
    }

    #[allow(dead_code)]
    pub fn bitmap_handle(self) -> BitmapHandle {
        self.0.read().static_data.bitmap_handle
    }

    pub fn width(self) -> u16 {
        self.0.read().static_data.width
    }

    pub fn height(self) -> u16 {
        self.0.read().static_data.height
    }
}

impl<'gc> TDisplayObject<'gc> for Bitmap<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn local_bounds(&self) -> BoundingBox {
        BoundingBox {
            x_min: Twips::new(0),
            y_min: Twips::new(0),
            x_max: Twips::new(self.width()),
            y_max: Twips::new(self.height()),
            valid: true,
        }
    }

    fn world_bounds(&self) -> BoundingBox {
        // TODO: Use dirty flags and cache this.
        let mut bounds = self.local_bounds().transform(&*self.matrix());
        let mut node = self.parent();
        while let Some(display_object) = node {
            bounds = bounds.transform(&*display_object.matrix());
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

        context.transform_stack.push(&*self.transform());

        context.renderer.render_bitmap(
            self.0.read().static_data.bitmap_handle,
            context.transform_stack.transform(),
        );

        context.transform_stack.pop();
    }
}

unsafe impl<'gc> gc_arena::Collect for BitmapData<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.static_data.trace(cc);
    }
}

/// Static data shared between all instances of a bitmap.
#[derive(Clone)]
struct BitmapStatic {
    id: CharacterId,
    bitmap_handle: BitmapHandle,
    width: u16,
    height: u16,
}

unsafe impl<'gc> gc_arena::Collect for BitmapStatic {
    #[inline]
    fn needs_trace() -> bool {
        true
    }
}
