//! `TextFormat` impl

use crate::avm1::error::Error;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, UpdateContext, Value};
use gc_arena::MutationContext;

fn map_defined_to_string<'gc>(
    name: &str,
    this: Object<'gc>,
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.coerce_to_string(avm, ac)?.into(),
    };

    this.set(name, val, avm, ac)?;

    Ok(())
}

fn map_defined_to_number<'gc>(
    name: &str,
    this: Object<'gc>,
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.coerce_to_f64(avm, ac)?.into(),
    };

    this.set(name, val, avm, ac)?;

    Ok(())
}

fn map_defined_to_bool<'gc>(
    name: &str,
    this: Object<'gc>,
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    val: Option<Value<'gc>>,
) -> Result<(), Error> {
    let val = match val {
        Some(Value::Undefined) => Value::Null,
        Some(Value::Null) => Value::Null,
        None => Value::Null,
        Some(v) => v.as_bool(avm.current_swf_version()).into(),
    };

    this.set(name, val, avm, ac)?;

    Ok(())
}

/// `TextFormat` constructor
pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    map_defined_to_string("font", this, avm, ac, args.get(0).cloned())?;
    map_defined_to_number("size", this, avm, ac, args.get(1).cloned())?;
    map_defined_to_number("color", this, avm, ac, args.get(2).cloned())?;
    map_defined_to_bool("bold", this, avm, ac, args.get(3).cloned())?;
    map_defined_to_bool("italic", this, avm, ac, args.get(4).cloned())?;
    map_defined_to_bool("underline", this, avm, ac, args.get(5).cloned())?;
    map_defined_to_string("url", this, avm, ac, args.get(6).cloned())?;
    map_defined_to_string("target", this, avm, ac, args.get(7).cloned())?;
    map_defined_to_string("align", this, avm, ac, args.get(8).cloned())?;
    map_defined_to_number("leftMargin", this, avm, ac, args.get(9).cloned())?;
    map_defined_to_number("rightMargin", this, avm, ac, args.get(10).cloned())?;
    map_defined_to_number("indent", this, avm, ac, args.get(11).cloned())?;
    map_defined_to_number("leading", this, avm, ac, args.get(12).cloned())?;

    Ok(Value::Undefined.into())
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
