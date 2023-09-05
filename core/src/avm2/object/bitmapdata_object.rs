//! Object representation for BitmapData

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapDataWrapper;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates BitmapData objects.
pub fn bitmap_data_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(BitmapDataObject(GcCell::new(
        activation.context.gc_context,
        BitmapDataObjectData {
            base,
            // This always starts out as a dummy (invalid) BitmapDataWrapper, so
            // that custom subclasses see a disposed BitmapData before they call super().
            // The real BitmapDataWrapper is set by BitmapData.init()
            bitmap_data: Some(BitmapDataWrapper::dummy(activation.context.gc_context)),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct BitmapDataObject<'gc>(pub GcCell<'gc, BitmapDataObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct BitmapDataObjectWeak<'gc>(pub GcWeakCell<'gc, BitmapDataObjectData<'gc>>);

impl fmt::Debug for BitmapDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BitmapDataObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BitmapDataObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    bitmap_data: Option<BitmapDataWrapper<'gc>>,
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
    ) -> Result<Object<'gc>, Error<'gc>> {
        let instance: Object<'gc> = Self(GcCell::new(
            activation.context.gc_context,
            BitmapDataObjectData {
                base: ScriptObjectData::new(class),
                bitmap_data: Some(bitmap_data),
            },
        ))
        .into();

        bitmap_data.init_object2(activation.context.gc_context, instance);
        instance.install_instance_slots(activation.context.gc_context);

        // We call the custom BitmapData class with width and height...
        // but, it always seems to be 1 in Flash Player when constructed from timeline?
        // This will not actually cause us to create a BitmapData with dimensions (1, 1) -
        // when the custom class makes a super() call, the BitmapData constructor will
        // load in the real data from the linked SymbolClass.
        if class != activation.avm2().classes().bitmapdata {
            class.call_native_init(instance.into(), &[1.into(), 1.into()], activation)?;
        }

        Ok(instance)
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_bitmap_data(&self) -> Option<BitmapDataWrapper<'gc>> {
        self.0.read().bitmap_data
    }

    /// Initialize the bitmap data in this object, if it's capable of
    /// supporting said data
    fn init_bitmap_data(&self, mc: &Mutation<'gc>, new_bitmap: BitmapDataWrapper<'gc>) {
        self.0.write(mc).bitmap_data = Some(new_bitmap)
    }
}
