//! Error object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use ruffle_macros::istr;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "message" => string("Error");
    "name" => string("Error");
    "toString" => method(to_string);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let message: Value<'gc> = args.get(0).cloned().unwrap_or(Value::Undefined);

    if message != Value::Undefined {
        this.set(istr!("message"), message, activation)?;
    }

    Ok(Value::Undefined)
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let message = this.get(istr!("message"), activation)?;
    Ok(message.coerce_to_string(activation)?.into())
}
