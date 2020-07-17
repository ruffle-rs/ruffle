//! `int` impl

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::MutationContext;

/// Implements `int`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("int constructor is a stub.".into())
}

/// Construct `int.prototype`.
pub fn create_proto<'gc>(
    mc: MutationContext<'gc, '_>,
    super_proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) -> Object<'gc> {
    ScriptObject::object(mc, super_proto)
}
