//! Object representation for BitmapData

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::bitmap::bitmap_data::BitmapData;
use crate::{
    impl_avm2_custom_object, impl_avm2_custom_object_instance, impl_avm2_custom_object_properties,
};
use gc_arena::{Collect, GcCell, MutationContext};

/// A class instance allocator that allocates BitmapData objects.
pub fn bitmapdata_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

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
        class: Object<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let proto = class
            .get_property(
                class,
                &QName::new(Namespace::public(), "prototype"),
                activation,
            )?
            .coerce_to_object(activation)?;

        let mut instance = Self(GcCell::allocate(
            activation.context.gc_context,
            BitmapDataObjectData {
                base: ScriptObjectData::base_new(Some(proto), Some(class)),
                bitmap_data: Some(bitmap_data),
            },
        ));

        bitmap_data
            .write(activation.context.gc_context)
            .init_object2(instance.into());
        instance.install_instance_traits(activation, class)?;
        class.call_native_init(Some(instance.into()), &[], activation, Some(class))?;

        Ok(instance.into())
    }
}

impl<'gc> TObject<'gc> for BitmapDataObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);
    impl_avm2_custom_object_instance!(base);

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(BitmapDataObject(GcCell::allocate(
            activation.context.gc_context,
            BitmapDataObjectData {
                base,
                bitmap_data: None,
            },
        ))
        .into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
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
