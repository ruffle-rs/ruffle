//! flash.external.ExternalInterface object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::external::{Callback, Value as ExternalValue};
use crate::string::StringContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "available" => property(get_available; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "addCallback" => method(add_callback; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "call" => method(call; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn get_available<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.external_interface.available().into())
}

pub fn add_callback<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !activation.context.external_interface.available() || args.len() < 3 {
        return Ok(false.into());
    }

    let name = args.get(0).unwrap().coerce_to_string(activation)?;
    let this = args.get(1).unwrap().to_owned();
    let method = args.get(2).unwrap();

    if let Value::Object(method) = method {
        activation.context.external_interface.add_callback(
            name.to_string(),
            Callback::Avm1 {
                this,
                method: *method,
            },
        );
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !activation.context.external_interface.available() {
        return Ok(Value::Null);
    }

    let (name, external_args_len) = if let Some(method_name) = args.get(0) {
        (*method_name, args.len() - 1)
    } else {
        (Value::Undefined, 0)
    };
    let name = name.coerce_to_string(activation)?;

    if let Some(method) = activation
        .context
        .external_interface
        .get_method_for(&name.to_utf8_lossy())
    {
        let mut external_args = Vec::with_capacity(external_args_len);
        if external_args_len > 0 {
            for arg in &args[1..] {
                external_args.push(ExternalValue::from_avm1(activation, arg.to_owned())?);
            }
        }
        Ok(method
            .call(activation.context, &external_args)
            .into_avm1(activation))
    } else {
        Ok(Value::Null)
    }
}

pub fn create_external_interface_object<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(OBJECT_DECLS, context, object, fn_proto);
    object.into()
}

pub fn create_proto<'gc>(context: &mut StringContext<'gc>, proto: Object<'gc>) -> Object<'gc> {
    // It's a custom prototype but it's empty.
    ScriptObject::new(context.gc_context, Some(proto)).into()
}
