//! Accessibility class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, Declaration};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "isActive" => method(is_active; DONT_DELETE | READ_ONLY | VERSION_6);
    "sendEvent" => method(send_event; DONT_DELETE | READ_ONLY | VERSION_6);
    "updateProperties" => method(update_properties; DONT_DELETE | READ_ONLY | VERSION_6);
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let accessibility = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(accessibility, OBJECT_DECLS);
    accessibility
}

pub fn is_active<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Accessibility", "isActive");
    Ok(Value::Bool(false))
}

pub fn send_event<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Accessibility", "sendEvent");
    Ok(Value::Undefined)
}

pub fn update_properties<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Accessibility", "updateProperties");
    Ok(Value::Undefined)
}
