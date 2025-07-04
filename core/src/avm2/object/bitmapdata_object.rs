//! Object representation for BitmapData

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::context::UpdateContext;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak, Mutation};

/// A class instance allocator that allocates BitmapData objects.
pub fn bitmap_data_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(BitmapDataObject(Gc::new(
        activation.gc(),
        BitmapDataObjectData {
            base,
            // This always starts out as a dummy (invalid) BitmapDataWrapper, so
            // that custom subclasses see a disposed BitmapData before they call super().
            // The real BitmapDataWrapper is set by BitmapData.init()
            bitmap_data: Lock::new(BitmapDataWrapper::dummy(activation.gc())),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct BitmapDataObject<'gc>(pub Gc<'gc, BitmapDataObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct BitmapDataObjectWeak<'gc>(pub GcWeak<'gc, BitmapDataObjectData<'gc>>);

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitmapDataObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct BitmapDataObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    bitmap_data: Lock<BitmapDataWrapper<'gc>>,
}

impl<'gc> BitmapDataObject<'gc> {
    // Constructs a BitmapData object from a BitmapDataWrapper.
    // This is *not* used when explicitly constructing a BitmapData
    // instance from ActionScript (e.g. `new BitmapData(100, 100)`,
    // or `new MyBitmapDataSubclass(100, 100)`).
    //
    // Instead, this is used when constructing a `Bitmap` object,
    // (from ActionScript or from the timeline), or when we need
    // to produce a new BitmapData object from a `BitmapData` method
    // like `clone()`
    pub fn from_bitmap_data_and_class(
        mc: &Mutation<'gc>,
        bitmap_data: BitmapDataWrapper<'gc>,
        class: ClassObject<'gc>,
    ) -> Self {
        let instance = Self(Gc::new(
            mc,
            BitmapDataObjectData {
                base: ScriptObjectData::new(class),
                bitmap_data: Lock::new(bitmap_data),
            },
        ));

        bitmap_data.init_object2(mc, instance.into());

        instance
    }

    /// Construct a BitmapData for a given BitmapDataWrapper. The resulting
    /// object will have the BitmapData class.
    pub fn from_bitmap_data(
        context: &mut UpdateContext<'gc>,
        bitmap_data: BitmapDataWrapper<'gc>,
    ) -> Self {
        let bitmapdata_class = context.avm2.classes().bitmapdata;

        Self::from_bitmap_data_and_class(context.gc(), bitmap_data, bitmapdata_class)
    }

    pub fn bitmap_data(&self) -> BitmapDataWrapper<'gc> {
        self.0.bitmap_data.get()
    }

    pub fn init_bitmap_data(&self, mc: &Mutation<'gc>, new_bitmap: BitmapDataWrapper<'gc>) {
        unlock!(Gc::write(mc, self.0), BitmapDataObjectData, bitmap_data).set(new_bitmap);
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_bitmap_data_object(&self) -> Option<BitmapDataObject<'gc>> {
        Some(*self)
    }

    fn as_bitmap_data(&self) -> Option<BitmapDataWrapper<'gc>> {
        Some(self.0.bitmap_data.get())
    }
}
