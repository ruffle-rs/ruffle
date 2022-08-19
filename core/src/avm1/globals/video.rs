//! Video class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::Object;
use crate::avm1::value::Value;
use crate::avm1::ScriptObject;
use gc_arena::MutationContext;

/// Implements `Video`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(gc_context, Some(proto));
    object.into()
}
