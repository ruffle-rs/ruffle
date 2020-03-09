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
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

/// Implements `Function.prototype.call`
fn call<'gc>(
    avm: &mut Avm2<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    func: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this = args.get(0).and_then(|v| v.as_object().ok());
    let base_proto = this.and_then(|that| that.proto());

    if let Some(func) = func {
        if args.len() > 1 {
            func.call(this, &args[1..], avm, context, base_proto)
        } else {
            func.call(this, &[], avm, context, base_proto)
        }
    } else {
        Err("Not a callable function".into())
    }
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
        QName::new(Namespace::public_namespace(), "call"),
        0,
        FunctionObject::from_builtin(gc_context, call, function_proto),
    );

    function_proto
}
