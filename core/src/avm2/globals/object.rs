//! Object builtin and prototype

use crate::avm2::function::FunctionObject;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
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

/// `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    _avm: &mut Avm2<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.as_string()?;

    if let Some(ns) = this.resolve_any(&name) {
        let qname = QName::new(ns, &name);
        return Ok(this.has_own_property(&qname).into());
    }

    Ok(false.into())
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
    gc_context: MutationContext<'gc, '_>,
    mut object_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) {
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::public_namespace(), "hasOwnProperty"),
        0,
        FunctionObject::from_builtin(gc_context, has_own_property, fn_proto),
    );
}
