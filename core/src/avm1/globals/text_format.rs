//! `TextFormat` impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, UpdateContext, Value};
use gc_arena::{Gc, MutationContext};

fn map_defined_to_string<'gc>(
    name: &str,
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error<'gc>> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => Gc::allocate(
            ac.gc_context,
            v.coerce_to_string(activation, ac)?.to_string(),
        )
        .into(),
    };

    this.set(name, val, activation, ac)?;

    Ok(())
}

fn map_defined_to_number<'gc>(
    name: &str,
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error<'gc>> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.coerce_to_f64(activation, ac)?.into(),
    };

    this.set(name, val, activation, ac)?;

    Ok(())
}

fn map_defined_to_bool<'gc>(
    name: &str,
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error<'gc>> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.as_bool(activation.current_swf_version()).into(),
    };

    this.set(name, val, activation, ac)?;

    Ok(())
}

/// `TextFormat` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    map_defined_to_string("font", this, activation, ac, args.get(0).cloned())?;
    map_defined_to_number("size", this, activation, ac, args.get(1).cloned())?;
    map_defined_to_number("color", this, activation, ac, args.get(2).cloned())?;
    map_defined_to_bool("bold", this, activation, ac, args.get(3).cloned())?;
    map_defined_to_bool("italic", this, activation, ac, args.get(4).cloned())?;
    map_defined_to_bool("underline", this, activation, ac, args.get(5).cloned())?;
    map_defined_to_string("url", this, activation, ac, args.get(6).cloned())?;
    map_defined_to_string("target", this, activation, ac, args.get(7).cloned())?;
    map_defined_to_string("align", this, activation, ac, args.get(8).cloned())?;
    map_defined_to_number("leftMargin", this, activation, ac, args.get(9).cloned())?;
    map_defined_to_number("rightMargin", this, activation, ac, args.get(10).cloned())?;
    map_defined_to_number("indent", this, activation, ac, args.get(11).cloned())?;
    map_defined_to_number("leading", this, activation, ac, args.get(12).cloned())?;

    Ok(Value::Undefined)
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
