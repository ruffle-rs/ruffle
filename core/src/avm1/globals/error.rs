//! Error object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute::*;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let message: Value<'gc> = args.get(0).cloned().unwrap_or(Value::Undefined);

    if message != Value::Undefined {
        this.set("message", message, activation)?;
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.define_value(gc_context, "message", "Error".into(), EnumSet::empty());
    object.define_value(gc_context, "name", "Error".into(), EnumSet::empty());

    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        Some(fn_proto),
    );

    object.into()
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let message = this.get("message", activation)?;
    Ok(AvmString::new(
        activation.context.gc_context,
        message.coerce_to_string(activation)?.to_string(),
    )
    .into())
}
