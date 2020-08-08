//! Function builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::MutationContext;

/// Implements `Function`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Function.prototype.call`
fn call<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    func: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = args
        .get(0)
        .and_then(|v| v.coerce_to_object(activation).ok());
    let base_proto = this.and_then(|that| that.proto());

    if let Some(func) = func {
        if args.len() > 1 {
            Ok(func.call(this, &args[1..], activation, base_proto)?)
        } else {
            Ok(func.call(this, &[], activation, base_proto)?)
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
        QName::new(Namespace::as3_namespace(), "call"),
        0,
        FunctionObject::from_builtin(gc_context, call, function_proto),
    );

    function_proto
}
