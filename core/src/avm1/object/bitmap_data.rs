use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::context::UpdateContext;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::bitmap::bitmap_data::BitmapData;
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
    data: GcCell<'gc, BitmapData<'gc>>,
}

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BitmapData")
            .field("data", &this.data)
            .finish()
    }
}

impl<'gc> BitmapDataObject<'gc> {
    add_field_accessors!(
        [data, GcCell<'gc, BitmapData<'gc>>, set => set_bitmap_data, get => bitmap_data],
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        Self::with_bitmap_data(gc_context, proto, Default::default())
    }

    pub fn with_bitmap_data(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
        bitmap_data: BitmapData<'gc>,
    ) -> Self {
        Self(GcCell::allocate(
            gc_context,
            BitmapDataData {
                base: ScriptObject::new(gc_context, proto),
                data: GcCell::allocate(gc_context, bitmap_data),
            },
        ))
    }

    pub fn disposed(&self) -> bool {
        self.bitmap_data().read().disposed()
    }

    pub fn dispose(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.bitmap_data()
            .write(context.gc_context)
            .dispose(context.renderer);
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_bitmap_data_object -> BitmapDataObject::empty_object);
    });
}
