//! Object prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::display_object::TDisplayObject;
use crate::string::{AvmString, StringContext};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "addProperty" => method(add_property; DONT_ENUM | DONT_DELETE | VERSION_6);
    "hasOwnProperty" => method(has_own_property; DONT_ENUM | DONT_DELETE | VERSION_6);
    "isPropertyEnumerable" => method(is_property_enumerable; DONT_DELETE | DONT_ENUM | VERSION_6);
    "isPrototypeOf" => method(is_prototype_of; DONT_ENUM | DONT_DELETE | VERSION_6);
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE);
    "valueOf" => method(value_of; DONT_ENUM | DONT_DELETE);
    "watch" => method(watch; DONT_ENUM | DONT_DELETE | VERSION_6);
    "unwatch" => method(unwatch; DONT_ENUM | DONT_DELETE | VERSION_6);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "registerClass" => method(register_class; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// Implements `Object` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = match args.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => this,
        val => val.coerce_to_object(activation),
    };
    Ok(this.into())
}

/// Implements `Object` function
pub fn object_function<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = match args.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => {
            Object::from(ScriptObject::new(activation.context.gc_context, None))
        }
        val => val.coerce_to_object(activation),
    };
    Ok(obj.into())
}

/// Implements `Object.prototype.addProperty`
pub fn add_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .and_then(|v| v.coerce_to_string(activation).ok())
        .unwrap_or_else(|| "undefined".into());
    let getter = args.get(1).unwrap_or(&Value::Undefined);
    let setter = args.get(2).unwrap_or(&Value::Undefined);

    match getter {
        Value::Object(get) if !name.is_empty() => {
            if let Value::Object(set) = setter {
                this.add_property_with_case(
                    activation,
                    name,
                    get.to_owned(),
                    Some(set.to_owned()),
                    Attribute::empty(),
                );
            } else if let Value::Null = setter {
                this.add_property_with_case(
                    activation,
                    name,
                    get.to_owned(),
                    None,
                    Attribute::READ_ONLY,
                );
            } else {
                return Ok(false.into());
            }

            Ok(true.into())
        }
        _ => Ok(false.into()),
    }
}

/// Implements `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(value) = args.get(0) {
        let name = value.coerce_to_string(activation)?;
        Ok(this.has_own_property(activation, name).into())
    } else {
        Ok(false.into())
    }
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if this.as_executable().is_some() {
        Ok("[type Function]".into())
    } else {
        Ok("[object Object]".into())
    }
}

/// Implements `Object.prototype.isPropertyEnumerable`
fn is_property_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(name) => {
            let name = name.coerce_to_string(activation)?;
            Ok(this.is_property_enumerable(activation, name).into())
        }
        None => Ok(false.into()),
    }
}

/// Implements `Object.prototype.isPrototypeOf`
fn is_prototype_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(val) => {
            let ob = val.coerce_to_object(activation);
            Ok(this.is_prototype_of(activation, ob).into())
        }
        _ => Ok(false.into()),
    }
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

/// Implements `Object.registerClass`
pub fn register_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (class_name, constructor) = match args {
        [class_name, constructor, ..] => (class_name, constructor),
        _ => return Ok(false.into()),
    };

    let constructor = match constructor {
        Value::Null | Value::Undefined => None,
        Value::Object(Object::FunctionObject(func)) => Some(*func),
        _ => return Ok(false.into()),
    };

    let class_name = class_name.coerce_to_string(activation)?;

    activation.context.avm1.register_constructor(
        activation.base_clip().movie().version(),
        class_name,
        constructor,
    );
    Ok(true.into())
}

/// Implements `Object.prototype.watch`
fn watch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = if let Some(name) = args.get(0) {
        name.coerce_to_string(activation)?
    } else {
        return Ok(false.into());
    };
    let callback = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);
    if callback.as_executable().is_none() {
        return Ok(false.into());
    }
    let user_data = args.get(2).cloned().unwrap_or(Value::Undefined);

    this.watch(activation, name, callback, user_data);

    Ok(true.into())
}

/// Implements `Object.prototype.unmwatch`
fn unwatch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = if let Some(name) = args.get(0) {
        name.coerce_to_string(activation)?
    } else {
        return Ok(false.into());
    };

    let result = this.unwatch(activation, name);

    Ok(result.into())
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
    context: &mut StringContext<'gc>,
    object_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) {
    let object = object_proto.raw_script_object();
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
}

/// Implements `ASSetPropFlags`.
///
/// This is an undocumented function that allows ActionScript 2.0 classes to
/// declare the property flags of a given property. It's not part of
/// `Object.prototype`, and I suspect that's a deliberate omission.
pub fn as_set_prop_flags<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let object = if let Some(v) = args.get(0) {
        v.coerce_to_object(activation)
    } else {
        avm_warn!(
            activation,
            "ASSetPropFlags called without object to apply to!"
        );
        return Ok(Value::Undefined);
    };

    let set_flags = args.get(2).unwrap_or(&0.into()).coerce_to_f64(activation)? as u16;
    let set_attributes = Attribute::from_bits_retain(set_flags);

    let clear_flags = args.get(3).unwrap_or(&0.into()).coerce_to_f64(activation)? as u16;
    let clear_attributes = Attribute::from_bits_retain(clear_flags);

    if set_attributes.bits() != set_flags || clear_attributes.bits() != clear_flags {
        avm_warn!(
            activation,
            "ASSetPropFlags: Unimplemented support for flags > 7"
        );
    }

    match args.get(1) {
        Some(&Value::Null) => object.set_attributes(
            activation.context.gc_context,
            None,
            set_attributes,
            clear_attributes,
        ),
        Some(v) => {
            let props = v.coerce_to_string(activation)?;
            if props.contains(b',') {
                for prop_name in props.split(b',') {
                    object.set_attributes(
                        activation.context.gc_context,
                        Some(AvmString::new(activation.context.gc_context, prop_name)),
                        set_attributes,
                        clear_attributes,
                    )
                }
            } else {
                object.set_attributes(
                    activation.context.gc_context,
                    Some(props),
                    set_attributes,
                    clear_attributes,
                )
            }
        }
        None => {
            avm_warn!(activation, "ASSetPropFlags called without property list!");
        }
    }

    Ok(Value::Undefined)
}

pub fn create_object_object<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object_function = FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        Executable::Native(object_function),
        fn_proto,
        proto,
    );
    let object = object_function.raw_script_object();
    define_properties_on(OBJECT_DECLS, context, object, fn_proto);
    object_function
}
