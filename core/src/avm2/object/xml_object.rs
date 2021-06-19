//! Object representation for XML objects

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};

/// A class instance deriver that constructs XML objects.
pub fn xml_deriver<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(class));

    Ok(XmlObject(GcCell::allocate(
        activation.context.gc_context,
        XmlObjectData { base },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct XmlObject<'gc>(GcCell<'gc, XmlObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct XmlObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for XmlObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::XmlObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(XmlObject(GcCell::allocate(
            activation.context.gc_context,
            XmlObjectData { base },
        ))
        .into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }
}
