use crate::avm1::{Object, ScriptObject, TObject};
use crate::context::UpdateContext;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::bitmap::bitmap_data::{BitmapData, BitmapDataWrapper};
use std::fmt;

/// A BitmapData
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct BitmapDataObject<'gc>(GcCell<'gc, BitmapDataData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BitmapDataData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,
    data: BitmapDataWrapper<'gc>,
}

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BitmapData")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

impl<'gc> BitmapDataObject<'gc> {
    pub fn bitmap_data(&self) -> GcCell<'gc, BitmapData<'gc>> {
        self.0.read().data.sync()
    }

    pub fn bitmap_data_wrapper(&self) -> BitmapDataWrapper<'gc> {
        self.0.read().data
    }

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Self {
        Self::with_bitmap_data(gc_context, proto, Default::default())
    }

    pub fn with_bitmap_data(
        gc_context: MutationContext<'gc, '_>,
        proto: Object<'gc>,
        bitmap_data: BitmapData<'gc>,
    ) -> Self {
        Self(GcCell::allocate(
            gc_context,
            BitmapDataData {
                base: ScriptObject::new(gc_context, Some(proto)),
                data: BitmapDataWrapper::new(GcCell::allocate(gc_context, bitmap_data)),
            },
        ))
    }

    pub fn width(&self) -> u32 {
        self.0.read().data.width()
    }

    pub fn height(&self) -> u32 {
        self.0.read().data.height()
    }

    pub fn transparency(&self) -> bool {
        self.0.read().data.transparency()
    }

    pub fn disposed(&self) -> bool {
        self.0.read().data.disposed()
    }

    pub fn dispose(&self, context: &mut UpdateContext<'_, 'gc>) {
        self.bitmap_data_wrapper().dispose(context.gc_context);
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_bitmap_data_object -> BitmapDataObject::empty_object);
    });
}
