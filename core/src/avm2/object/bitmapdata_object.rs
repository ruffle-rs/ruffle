//! Object representation for BitmapData

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
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
    pub fn from_bitmap_data_internal(
        activation: &mut Activation<'_, 'gc>,
        bitmap_data: BitmapDataWrapper<'gc>,
        class: ClassObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let instance = Self(Gc::new(
            activation.gc(),
            BitmapDataObjectData {
                base: ScriptObjectData::new(class),
                bitmap_data: Lock::new(bitmap_data),
            },
        ));

        bitmap_data.init_object2(activation.gc(), instance.into());

        // We call the custom BitmapData class with width and height...
        // but, it always seems to be 1 in Flash Player when constructed from timeline?
        // This will not actually cause us to create a BitmapData with dimensions (1, 1) -
        // when the custom class makes a super() call, the BitmapData constructor will
        // load in the real data from the linked SymbolClass.
        if class != activation.avm2().classes().bitmapdata {
            class.call_init(instance.into(), &[1.into(), 1.into()], activation)?;
        }

        Ok(instance)
    }

    pub fn get_bitmap_data(self) -> BitmapDataWrapper<'gc> {
        self.0.bitmap_data.get()
    }

    /// This should only be called to initialize the association between an AVM
    /// object and it's associated bitmap data. This association should not be
    /// reinitialized later.
    pub fn init_bitmap_data(self, mc: &Mutation<'gc>, new_bitmap: BitmapDataWrapper<'gc>) {
        unlock!(Gc::write(mc, self.0), BitmapDataObjectData, bitmap_data).set(new_bitmap);
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
