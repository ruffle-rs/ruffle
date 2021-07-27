use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::{
    impl_avm2_custom_object, impl_avm2_custom_object_instance, impl_avm2_custom_object_properties,
};
use gc_arena::{Collect, GcCell, MutationContext};

/// A class instance allocator that allocates Date objects.
pub fn date_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(DateObject(GcCell::allocate(
        activation.context.gc_context,
        DateObjectData { base },
    ))
    .into())
}
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct DateObject<'gc>(GcCell<'gc, DateObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct DateObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for DateObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);
    impl_avm2_custom_object_instance!(base);

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::DateObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(DateObject(GcCell::allocate(
            activation.context.gc_context,
            DateObjectData { base },
        ))
        .into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }
}
