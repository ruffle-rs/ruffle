//! `TextFormat` impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::string::AvmString;
use gc_arena::MutationContext;

fn map_defined_to_string<'gc>(
    name: AvmString<'gc>,
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error<'gc>> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => AvmString::new(
            activation.context.gc_context,
            v.coerce_to_string(activation)?.to_string(),
        )
        .into(),
    };

    this.set(name, val, activation)?;

    Ok(())
}

fn map_defined_to_number<'gc>(
    name: AvmString<'gc>,
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error<'gc>> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.coerce_to_f64(activation)?.into(),
    };

    this.set(name, val, activation)?;

    Ok(())
}

fn map_defined_to_bool<'gc>(
    name: AvmString<'gc>,
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error<'gc>> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.as_bool(activation.swf_version()).into(),
    };

    this.set(name, val, activation)?;

    Ok(())
}

/// `TextFormat` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    map_defined_to_string("font".into(), this, activation, args.get(0).cloned())?;
    map_defined_to_number("size".into(), this, activation, args.get(1).cloned())?;
    map_defined_to_number("color".into(), this, activation, args.get(2).cloned())?;
    map_defined_to_bool("bold".into(), this, activation, args.get(3).cloned())?;
    map_defined_to_bool("italic".into(), this, activation, args.get(4).cloned())?;
    map_defined_to_bool("underline".into(), this, activation, args.get(5).cloned())?;
    map_defined_to_string("url".into(), this, activation, args.get(6).cloned())?;
    map_defined_to_string("target".into(), this, activation, args.get(7).cloned())?;
    map_defined_to_string("align".into(), this, activation, args.get(8).cloned())?;
    map_defined_to_number("leftMargin".into(), this, activation, args.get(9).cloned())?;
    map_defined_to_number(
        "rightMargin".into(),
        this,
        activation,
        args.get(10).cloned(),
    )?;
    map_defined_to_number("indent".into(), this, activation, args.get(11).cloned())?;
    map_defined_to_number("leading".into(), this, activation, args.get(12).cloned())?;

    Ok(this.into())
}

/// `TextFormat.prototype` constructor
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) -> Object<'gc> {
    let tf_proto = ScriptObject::object(gc_context, Some(proto));

    tf_proto.into()
}
