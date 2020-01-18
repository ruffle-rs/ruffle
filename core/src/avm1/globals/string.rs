//! `String` class impl

use crate::avm1::return_value::ReturnValue;
use crate::avm1::value_object::ValueObject;
use crate::avm1::{Avm1, Error, Object, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;

/// `String` constructor
pub fn string_constructor<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let arg = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_string(avm, ac)?;

    if let Some(mut vbox) = this.as_value_object() {
        vbox.replace_value(ac.gc_context, arg.into());
    }

    Ok(Value::Undefined.into())
}

/// `String.toString` / `String.valueOf` impl
pub fn to_string_value_of<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(vbox) = this.as_value_object() {
        return Ok(vbox.unbox().coerce_to_string(avm, ac)?.into());
    }

    //TODO: This normally falls back to `[object Object]` or `[type Function]`,
    //implying that `toString` and `valueOf` are inherent object properties and
    //not just methods.
    Ok(Value::Undefined.into())
}

/// `String.toUpperCase` impl
pub fn to_upper_case<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let sval = Value::Object(this).coerce_to_string(avm, ac)?;

    Ok(sval.to_uppercase().into())
}

/// `String.prototype` definition
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let string_proto = ValueObject::empty_box(gc_context, Some(proto));

    string_proto.as_script_object().unwrap().force_set_function(
        "toString",
        to_string_value_of,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    string_proto.as_script_object().unwrap().force_set_function(
        "valueOf",
        to_string_value_of,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    string_proto.as_script_object().unwrap().force_set_function(
        "toUpperCase",
        to_upper_case,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    string_proto
}
