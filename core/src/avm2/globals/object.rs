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
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    _: &mut Avm2<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(this
        .map(|t| t.to_string())
        .unwrap_or(Ok(Value::Undefined))?
        .into())
}

/// Implements `Object.prototype.toLocaleString`
fn to_locale_string<'gc>(
    _: &mut Avm2<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(this
        .map(|t| t.to_string())
        .unwrap_or(Ok(Value::Undefined))?
        .into())
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    _: &mut Avm2<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(this
        .map(|t| t.value_of())
        .unwrap_or(Ok(Value::Undefined))?
        .into())
}

/// `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    _avm: &mut Avm2<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.as_string()?;

    if let Some(ns) = this.resolve_any(&name)? {
        if !ns.is_private() {
            let qname = QName::new(ns, &name);
            return Ok(this.has_own_property(&qname)?.into());
        }
    }

    Ok(false.into())
}

/// `Object.prototype.isPrototypeOf`
pub fn is_prototype_of<'gc>(
    _avm: &mut Avm2<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let search_proto: Result<Object<'gc>, Error> =
        this.ok_or_else(|| "No valid this parameter".into());
    let search_proto = search_proto?;
    let mut target_proto = args.get(0).cloned().unwrap_or(Value::Undefined);

    while let Value::Object(proto) = target_proto {
        if Object::ptr_eq(search_proto, proto) {
            return Ok(true.into());
        }

        target_proto = proto.proto().map(|o| o.into()).unwrap_or(Value::Undefined);
    }

    Ok(false.into())
}

/// `Object.prototype.propertyIsEnumerable`
pub fn property_is_enumerable<'gc>(
    _avm: &mut Avm2<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.as_string()?;

    if let Some(ns) = this.resolve_any(&name)? {
        if !ns.is_private() {
            let qname = QName::new(ns, &name);
            return Ok(this.property_is_enumerable(&qname).into());
        }
    }

    Ok(false.into())
}

/// `Object.prototype.setPropertyIsEnumerable`
pub fn set_property_is_enumerable<'gc>(
    _avm: &mut Avm2<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.as_string()?;
    let is_enum = args
        .get(1)
        .cloned()
        .unwrap_or(Value::Bool(true))
        .as_bool()?;

    if let Some(ns) = this.resolve_any(&name)? {
        if !ns.is_private() {
            let qname = QName::new(ns, &name);
            this.set_local_property_is_enumerable(context.gc_context, &qname, is_enum)?;
        }
    }

    Ok(Value::Undefined.into())
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
        QName::new(Namespace::public_namespace(), "toString"),
        0,
        FunctionObject::from_builtin(gc_context, to_string, fn_proto),
    );
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::public_namespace(), "toLocaleString"),
        0,
        FunctionObject::from_builtin(gc_context, to_locale_string, fn_proto),
    );
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::public_namespace(), "valueOf"),
        0,
        FunctionObject::from_builtin(gc_context, value_of, fn_proto),
    );
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::as3_namespace(), "hasOwnProperty"),
        0,
        FunctionObject::from_builtin(gc_context, has_own_property, fn_proto),
    );
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::as3_namespace(), "isPrototypeOf"),
        0,
        FunctionObject::from_builtin(gc_context, is_prototype_of, fn_proto),
    );
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::as3_namespace(), "propertyIsEnumerable"),
        0,
        FunctionObject::from_builtin(gc_context, property_is_enumerable, fn_proto),
    );
    object_proto.install_method(
        gc_context,
        QName::new(Namespace::public_namespace(), "setPropertyIsEnumerable"),
        0,
        FunctionObject::from_builtin(gc_context, set_property_is_enumerable, fn_proto),
    );
}
