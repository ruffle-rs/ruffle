//! Object representation for BitmapData

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapData;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates BitmapData objects.
pub fn bitmapdata_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(BitmapDataObject(GcCell::allocate(
        activation.context.gc_context,
        BitmapDataObjectData {
            base,
            bitmap_data: None,
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct BitmapDataObject<'gc>(GcCell<'gc, BitmapDataObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct BitmapDataObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    bitmap_data: Option<GcCell<'gc, BitmapData<'gc>>>,
}

impl<'gc> BitmapDataObject<'gc> {
    pub fn from_bitmap_data(
        activation: &mut Activation<'_, 'gc, '_>,
        bitmap_data: GcCell<'gc, BitmapData<'gc>>,
        class: ClassObject<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let mut instance = Self(GcCell::allocate(
            activation.context.gc_context,
            BitmapDataObjectData {
                base: ScriptObjectData::new(class),
                bitmap_data: Some(bitmap_data),
            },
        ));

        bitmap_data
            .write(activation.context.gc_context)
            .init_object2(instance.into());
        instance.install_instance_slots(activation);
        class.call_native_init(Some(instance.into()), &[], activation)?;

        Ok(instance.into())
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    /// Unwrap this object's bitmap data
    fn as_bitmap_data(&self) -> Option<GcCell<'gc, BitmapData<'gc>>> {
        self.0.read().bitmap_data
    }

    /// Initialize the bitmap data in this object, if it's capable of
    /// supporting said data
    fn init_bitmap_data(
        &self,
        mc: MutationContext<'gc, '_>,
        new_bitmap: GcCell<'gc, BitmapData<'gc>>,
    ) {
        self.0.write(mc).bitmap_data = Some(new_bitmap)
    }
}
