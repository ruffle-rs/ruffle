//! Function prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::{Avm1String, Object, ScriptObject, TObject, UpdateContext, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

/// Implements `Function`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `Function.prototype.call`
pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    func: Object<'gc>,
    myargs: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = match myargs.get(0) {
        Some(Value::Object(this)) => *this,
        _ => activation.avm.globals,
    };
    let empty = [];
    let args = match myargs.len() {
        0 => &empty,
        1 => &empty,
        _ => &myargs[1..],
    };

    match func.as_executable() {
        Some(exec) => exec.exec(
            "[Anonymous]",
            activation,
            action_context,
            this,
            None,
            args,
            ExecutionReason::FunctionCall,
        ),
        _ => Ok(Value::Undefined),
    }
}

/// Implements `Function.prototype.apply`
pub fn apply<'gc>(
    activation: &mut Activation<'_, 'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    func: Object<'gc>,
    myargs: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = match myargs.get(0) {
        Some(Value::Object(this)) => *this,
        _ => activation.avm.globals,
    };
    let mut child_args = Vec::new();
    let args_object = myargs.get(1).cloned().unwrap_or(Value::Undefined);
    let length = match args_object {
        Value::Object(a) => a
            .get("length", activation, action_context)?
            .coerce_to_f64(activation, action_context)? as usize,
        _ => 0,
    };

    while child_args.len() < length {
        let args = args_object.coerce_to_object(activation, action_context);
        let next_arg = args.get(&format!("{}", child_args.len()), activation, action_context)?;

        child_args.push(next_arg);
    }

    match func.as_executable() {
        Some(exec) => exec.exec(
            "[Anonymous]",
            activation,
            action_context,
            this,
            None,
            &child_args,
            ExecutionReason::FunctionCall,
        ),
        _ => Ok(Value::Undefined),
    }
}

/// Implements `Function.prototype.toString`
fn to_string<'gc>(
    _: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Avm1String::from("[type Function]").into())
}

/// Partially construct `Function.prototype`.
///
/// `__proto__` and other cross-linked properties of this object will *not*
/// be defined here. The caller of this function is responsible for linking
/// them in order to obtain a valid ECMAScript `Function` prototype. The
/// returned object is also a bare object, which will need to be linked into
/// the prototype of `Object`.
pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    let function_proto = ScriptObject::object_cell(gc_context, Some(proto));
    let this = Some(function_proto);
    function_proto
        .as_script_object()
        .unwrap()
        .force_set_function("call", call, gc_context, EnumSet::empty(), this);
    function_proto
        .as_script_object()
        .unwrap()
        .force_set_function("apply", apply, gc_context, EnumSet::empty(), this);
    function_proto
        .as_script_object()
        .unwrap()
        .force_set_function("toString", to_string, gc_context, EnumSet::empty(), this);

    function_proto
}
