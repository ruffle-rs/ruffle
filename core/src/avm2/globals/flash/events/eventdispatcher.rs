//! `flash.events.EventDispatcher` builtin/prototype

use crate::avm2::object::Object;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::MutationContext;

/// Implements `flash.events.EventDispatcher`'s constructor.
pub fn constructor<'gc>(
    _avm: &mut Avm2<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

/// Construct `EventDispatcher.prototype`.
pub fn create_proto<'gc>(
    mc: MutationContext<'gc, '_>,
    super_proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) -> Object<'gc> {
    // TODO: Use `StageObject` here.
    ScriptObject::object(mc, super_proto)
}
