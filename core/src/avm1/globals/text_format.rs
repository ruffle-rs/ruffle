//! `TextFormat` impl

use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, UpdateContext, Value};
use gc_arena::MutationContext;

/// `TextFormat` constructor
pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    this.set(
        "font",
        args.get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "size",
        args.get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_number(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "color",
        args.get(2)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_number(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "bold",
        args.get(3)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_bool(avm.current_swf_version())
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "italic",
        args.get(4)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_bool(avm.current_swf_version())
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "underline",
        args.get(5)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_bool(avm.current_swf_version())
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "url",
        args.get(6)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "target",
        args.get(7)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "align",
        args.get(8)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "leftMargin",
        args.get(9)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_number(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "rightMargin",
        args.get(10)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_number(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "indent",
        args.get(11)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_number(avm, ac)?
            .into(),
        avm,
        ac,
    )?;
    this.set(
        "leading",
        args.get(12)
            .cloned()
            .unwrap_or(Value::Undefined)
            .as_number(avm, ac)?
            .into(),
        avm,
        ac,
    )?;

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
