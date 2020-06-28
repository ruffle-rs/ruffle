//! Button/SimpleButton prototype

use crate::avm1::error::Error;
use crate::avm1::globals::display_object;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::stack_frame::StackFrame;
use crate::avm1::{Object, ScriptObject, UpdateContext, Value};
use gc_arena::MutationContext;

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));

    display_object::define_display_object_proto(gc_context, object, fn_proto);

    object.into()
}

/// Implements `Button` constructor.
pub fn constructor<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(Value::Undefined.into())
}
