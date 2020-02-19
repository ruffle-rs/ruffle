//! Function builtin and prototype

use crate::avm2::function::FunctionObject;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::MutationContext;

/// Implements `Function`
pub fn constructor<'gc>(
    _avm: &mut Avm2<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

/// Implements `Function.prototype.toString`
fn to_string<'gc>(
    _: &mut Avm2<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    _: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(ReturnValue::Immediate("[type Function]".into()))
}

/// Partially construct `Function.prototype`.
///
/// `__proto__` and other cross-linked properties of this object will *not*
/// be defined here. The caller of this function is responsible for linking
/// them in order to obtain a valid ECMAScript `Function` prototype. The
/// returned object is also a bare object, which will need to be linked into
/// the prototype of `Object`.
pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    let mut function_proto = ScriptObject::object(gc_context, proto);

    function_proto.install_method(
        gc_context,
        QName::new(Namespace::public_namespace(), "toString"),
        FunctionObject::from_builtin(gc_context, to_string, function_proto),
    );

    function_proto
}
