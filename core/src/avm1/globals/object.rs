//! Object prototype

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm_warn;
use crate::display_object::TDisplayObject;
use crate::string::AvmString;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "watch" => method(WATCH; DONT_ENUM | DONT_DELETE | VERSION_6);
    "unwatch" => method(UNWATCH; DONT_ENUM | DONT_DELETE | VERSION_6);
    "addProperty" => method(ADD_PROPERTY; DONT_ENUM | DONT_DELETE | VERSION_6);
    "valueOf" => method(VALUE_OF; DONT_ENUM | DONT_DELETE);
    "toString" => method(TO_STRING; DONT_ENUM | DONT_DELETE);
    "hasOwnProperty" => method(HAS_OWN_PROPERTY; DONT_ENUM | DONT_DELETE | VERSION_6);
    "isPrototypeOf" => method(IS_PROTOTYPE_OF; DONT_ENUM | DONT_DELETE | VERSION_6);
    "isPropertyEnumerable" => method(IS_PROPERTY_ENUMERABLE; DONT_DELETE | DONT_ENUM | VERSION_6);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "registerClass" => method(REGISTER_CLASS; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// Constructs the `Object` class.
///
/// Since Object and Function are so heavily intertwined, this function does
/// not allocate an object to store either proto. Instead, they must be provided
/// through the `DeclContext`.
pub fn create_class<'gc>(context: &mut DeclContext<'_, 'gc>) -> SystemClass<'gc> {
    let class = context.native_class_with_proto(
        table_constructor!(method),
        Some(function),
        context.object_proto,
    );
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

pub mod method {
    pub const WATCH: u16 = 0;
    pub const UNWATCH: u16 = 1;
    pub const ADD_PROPERTY: u16 = 2;
    pub const VALUE_OF: u16 = 3;
    pub const TO_STRING: u16 = 4;
    pub const HAS_OWN_PROPERTY: u16 = 5;
    pub const IS_PROTOTYPE_OF: u16 = 6;
    pub const IS_PROPERTY_ENUMERABLE: u16 = 7;
    pub const REGISTER_CLASS: u16 = 8;
    pub const CONSTRUCTOR: u16 = 9;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;

    match index {
        CONSTRUCTOR => constructor(activation, this, args),
        WATCH => watch(activation, this, args),
        UNWATCH => unwatch(activation, this, args),
        ADD_PROPERTY => add_property(activation, this, args),
        VALUE_OF => value_of(activation, this, args),
        TO_STRING => to_string(activation, this, args),
        HAS_OWN_PROPERTY => has_own_property(activation, this, args),
        IS_PROTOTYPE_OF => is_prototype_of(activation, this, args),
        IS_PROPERTY_ENUMERABLE => is_property_enumerable(activation, this, args),
        REGISTER_CLASS => register_class(activation, this, args),
        _ => Ok(Value::Undefined),
    }
}

/// Implements `Object` constructor
fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = match args.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => this,
        val => val.coerce_to_object_or_bare(activation)?,
    };
    Ok(this.into())
}

/// Implements `Object` function
fn function<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = match args.get(0).unwrap_or(&Value::Undefined) {
        Value::Undefined | Value::Null => Object::new_without_proto(activation.gc()),
        val => val.coerce_to_object_or_bare(activation)?,
    };
    Ok(obj.into())
}

/// Implements `Object.prototype.addProperty`
pub fn add_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(name) = args.get(0) {
        let name = name.coerce_to_string(activation)?;
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

                return Ok(true.into());
            }
            _ => return Ok(false.into()),
        }
    }

    Ok(false.into())
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if this.as_function().is_some() {
        Ok(AvmString::new_ascii_static(activation.gc(), b"[type Function]").into())
    } else {
        Ok(AvmString::new_ascii_static(activation.gc(), b"[object Object]").into())
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
            let ob = val.coerce_to_object_or_bare(activation)?;
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
        Value::Object(obj) if obj.as_function().is_some() => Some(*obj),
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
        .coerce_to_object_or_bare(activation)?;
    if callback.as_function().is_none() {
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
        v.coerce_to_object_or_bare(activation)?
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
        Some(&Value::Null) => {
            object.set_attributes(activation.gc(), None, set_attributes, clear_attributes)
        }
        Some(v) => {
            let props = v.coerce_to_string(activation)?;
            if props.contains(b',') {
                for prop_name in props.split(b',') {
                    object.set_attributes(
                        activation.gc(),
                        Some(AvmString::new(activation.gc(), prop_name)),
                        set_attributes,
                        clear_attributes,
                    )
                }
            } else {
                object.set_attributes(
                    activation.gc(),
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
