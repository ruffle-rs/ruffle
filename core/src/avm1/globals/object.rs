//! Object prototype
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, TObject, UpdateContext, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

/// Implements `Object`
pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

/// Implements `Object.prototype.addProperty`
pub fn add_property<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let name = args.get(0).unwrap_or(&Value::Undefined);
    let getter = args.get(1).unwrap_or(&Value::Undefined);
    let setter = args.get(2).unwrap_or(&Value::Undefined);

    match (name, getter) {
        (Value::String(name), Value::Object(get)) if !name.is_empty() => {
            if let Some(get_func) = get.as_executable() {
                if let Value::Object(set) = setter {
                    if let Some(set_func) = set.as_executable() {
                        this.add_property(
                            context.gc_context,
                            name,
                            get_func,
                            Some(set_func),
                            EnumSet::empty(),
                        );
                    } else {
                        return Ok(false.into());
                    }
                } else if let Value::Null = setter {
                    this.add_property(context.gc_context, name, get_func, None, ReadOnly.into());
                } else {
                    return Ok(false.into());
                }
            }

            Ok(true.into())
        }
        _ => Ok(false.into()),
    }
}

/// Implements `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(Value::String(name)) => Ok(Value::Bool(this.has_own_property(name)).into()),
        _ => Ok(Value::Bool(false).into()),
    }
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    _: &mut Avm1<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    _: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(ReturnValue::Immediate("[object Object]".into()))
}

/// Implements `Object.prototype.isPropertyEnumerable`
fn is_property_enumerable<'gc>(
    _: &mut Avm1<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(Value::String(name)) => Ok(Value::Bool(this.is_property_enumerable(name)).into()),
        _ => Ok(Value::Bool(false).into()),
    }
}

/// Implements `Object.prototype.isPrototypeOf`
fn is_prototype_of<'gc>(
    _: &mut Avm1<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(val) => {
            let ob = match val.as_object() {
                Ok(ob) => ob,
                Err(_) => return Ok(Value::Bool(false).into()),
            };
            let mut proto = ob.proto();

            while let Some(proto_ob) = proto {
                if Object::ptr_eq(this, proto_ob) {
                    return Ok(Value::Bool(true).into());
                }

                proto = proto_ob.proto();
            }

            Ok(Value::Bool(false).into())
        }
        _ => Ok(Value::Bool(false).into()),
    }
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    _: &mut Avm1<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(ReturnValue::Immediate(this.into()))
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
    object_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) {
    object_proto.as_script_object().unwrap().force_set_function(
        "addProperty",
        add_property,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
    object_proto.as_script_object().unwrap().force_set_function(
        "hasOwnProperty",
        has_own_property,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
    object_proto.as_script_object().unwrap().force_set_function(
        "isPropertyEnumerable",
        is_property_enumerable,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
    object_proto.as_script_object().unwrap().force_set_function(
        "isPrototypeOf",
        is_prototype_of,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
    object_proto.as_script_object().unwrap().force_set_function(
        "toString",
        to_string,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
    object_proto.as_script_object().unwrap().force_set_function(
        "valueOf",
        value_of,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
}
