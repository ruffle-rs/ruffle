//! Object builtin and prototype

use crate::avm2::object::Object;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::MutationContext;

/// Implements `Object`
pub fn constructor<'gc>(
    _avm: &mut Avm2<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
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
    _gc_context: MutationContext<'gc, '_>,
    _object_proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) {
}
