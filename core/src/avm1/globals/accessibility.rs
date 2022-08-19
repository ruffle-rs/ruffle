//! Accessibility class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use gc_arena::MutationContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "isActive" => method(is_active; DONT_DELETE | READ_ONLY);
    "sendEvent" => method(send_event; DONT_DELETE | READ_ONLY);
    "updateProperties" => method(update_properties; DONT_DELETE | READ_ONLY);
};

pub fn is_active<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Accessibility.isActive: not yet implemented");
    Ok(Value::Bool(false))
}

pub fn send_event<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Accessibility.sendEvent: not yet implemented");
    Ok(Value::Undefined)
}

pub fn update_properties<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("Accessibility.updateProperties: not yet implemented");
    Ok(Value::Undefined)
}

pub fn create_accessibility_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let accessibility = ScriptObject::new(gc_context, proto);
    define_properties_on(OBJECT_DECLS, gc_context, accessibility, fn_proto);
    accessibility.into()
}
