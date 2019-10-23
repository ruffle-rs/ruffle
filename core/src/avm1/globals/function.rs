//! Function prototype

use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};

/// Implements `Function`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

pub fn call<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

pub fn apply<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

/// Partially construct `Function.prototype`.
///
/// `__proto__` and other cross-linked properties of this object will *not*
/// be defined here. The caller of this function is responsible for linking
/// them in order to obtain a valid ECMAScript `Function` prototype. The
/// returned object is also a bare object, which will need to be linked into
/// the prototype of `Object`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: GcCell<'gc, Object<'gc>>,
) -> GcCell<'gc, Object<'gc>> {
    let function_proto = GcCell::allocate(gc_context, Object::object(gc_context, Some(proto)));

    function_proto.write(gc_context).force_set_function(
        "call",
        call,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );
    function_proto.write(gc_context).force_set_function(
        "apply",
        apply,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );

    function_proto
}
