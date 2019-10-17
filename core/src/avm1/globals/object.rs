//! Object global impls
use crate::avm1::object::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};

pub fn has_own_property<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(Value::String(name)) => Ok(Value::Bool(this.read().has_own_property(name)).into()),
        _ => Ok(Value::Bool(false).into()),
    }
}

fn to_string<'gc>(
    _: &mut Avm1<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    _: GcCell<'gc, Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(ReturnValue::Immediate("[Object object]".into()))
}

/// Partially construct `Object.prototype`.
///
/// `__proto__` and other cross-linked properties of this object will *not*
/// be defined here. The caller of this function is responsible for linking
/// them in order to obtain a valid ECMAScript `Object` prototype.
///
/// Since Object and Function are so heavily intertwined, this function does
/// not allocate an object to store either proto. Instead, you must allocate
/// bare objects for both and let this function fill Object for you.
pub fn fill_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    object_proto: GcCell<'gc, Object<'gc>>,
    fn_proto: GcCell<'gc, Object<'gc>>,
) {
    let mut ob_proto_write = object_proto.write(gc_context);

    ob_proto_write.force_set_function(
        "hasOwnProperty",
        has_own_property,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    ob_proto_write.force_set_function(
        "toString",
        to_string,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
}
