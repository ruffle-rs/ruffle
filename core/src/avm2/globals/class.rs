//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use gc_arena::MutationContext;

/// Implements `Class`
///
/// Notably, you cannot construct new classes this way, so this returns an
/// error.
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Classes cannot be constructed.".into())
}

/// Construct `Class.prototype`.
pub fn create_proto<'gc>(
    mc: MutationContext<'gc, '_>,
    super_proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) -> Object<'gc> {
    ScriptObject::object(mc, super_proto)
}
