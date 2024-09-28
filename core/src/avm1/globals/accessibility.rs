//! Accessibility class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::avm1_stub;
use crate::string::StringContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "isActive" => method(is_active; DONT_DELETE | READ_ONLY);
    "sendEvent" => method(send_event; DONT_DELETE | READ_ONLY);
    "updateProperties" => method(update_properties; DONT_DELETE | READ_ONLY);
};

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

pub fn create_accessibility_object<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let accessibility = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(OBJECT_DECLS, context, accessibility, fn_proto);
    accessibility.into()
}
