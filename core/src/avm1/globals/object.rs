//! Object prototype
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute::{self, *};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::avm_warn;
use crate::character::Character;
use crate::display_object::TDisplayObject;
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::borrow::Cow;

/// Implements `Object` constructor
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `Object` function
pub fn object_function<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(val) = args.get(0) {
        Ok(val.coerce_to_object(activation).into())
    } else {
        Ok(ScriptObject::object(activation.context.gc_context, None).into())
    }
}

/// Implements `Object.prototype.addProperty`
pub fn add_property<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
                    activation.context.gc_context,
                    &name,
                    get.to_owned(),
                    Some(set.to_owned()),
                    EnumSet::empty(),
                );
            } else if let Value::Null = setter {
                this.add_property_with_case(
                    activation,
                    activation.context.gc_context,
                    &name,
                    get.to_owned(),
                    None,
                    ReadOnly.into(),
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(value) = args.get(0) {
        let name = value.coerce_to_string(activation)?;
        Ok(Value::Bool(this.has_own_property(activation, &name)))
    } else {
        Ok(false.into())
    }
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok("[object Object]".into())
}

/// Implements `Object.prototype.isPropertyEnumerable`
fn is_property_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(Value::String(name)) => Ok(Value::Bool(this.is_property_enumerable(activation, name))),
        _ => Ok(Value::Bool(false)),
    }
}

/// Implements `Object.prototype.isPrototypeOf`
fn is_prototype_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(val) => {
            let ob = val.coerce_to_object(activation);
            Ok(Value::Bool(this.is_prototype_of(ob)))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

/// Implements `Object.registerClass`
pub fn register_class<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(class_name) = args.get(0).cloned() {
        let class_name = class_name.coerce_to_string(activation)?;
        if let Some(movie) = activation.base_clip().movie() {
            if let Some(Character::MovieClip(movie_clip)) = activation
                .context
                .library
                .library_for_movie_mut(movie)
                .get_character_by_export_name(&class_name)
            {
                if let Some(constructor) = args.get(1) {
                    movie_clip.set_avm1_constructor(
                        activation.context.gc_context,
                        Some(constructor.coerce_to_object(activation)),
                    );
                } else {
                    movie_clip.set_avm1_constructor(activation.context.gc_context, None);
                }
            } else {
                log::warn!(
                    "Tried to register_class on an unknown export {}",
                    class_name
                );
            }
        } else {
            log::warn!("Tried to register_class on an unknown movie");
        }
    } else {
        log::warn!("Tried to register_class with an unknown class");
    }
    Ok(Value::Undefined)
}

/// Implements `Object.prototype.watch`
fn watch<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

    this.set_watcher(
        activation,
        activation.context.gc_context,
        Cow::Borrowed(&name),
        callback,
        user_data,
    );

    Ok(true.into())
}

/// Implements `Object.prototype.unmwatch`
fn unwatch<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,

    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = if let Some(name) = args.get(0) {
        name.coerce_to_string(activation)?
    } else {
        return Ok(false.into());
    };

    let result = this.remove_watcher(
        activation,
        activation.context.gc_context,
        Cow::Borrowed(&name),
    );

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
    object_proto.as_script_object().unwrap().force_set_function(
        "watch",
        watch,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
    object_proto.as_script_object().unwrap().force_set_function(
        "unwatch",
        unwatch,
        gc_context,
        DontDelete | DontEnum,
        Some(fn_proto),
    );
}

/// Implements `ASSetPropFlags`.
///
/// This is an undocumented function that allows ActionScript 2.0 classes to
/// declare the property flags of a given property. It's not part of
/// `Object.prototype`, and I suspect that's a deliberate omission.
pub fn as_set_prop_flags<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut object = if let Some(object) = args.get(0).map(|v| v.coerce_to_object(activation)) {
        object
    } else {
        avm_warn!(
            activation,
            "ASSetPropFlags called without object to apply to!"
        );
        return Ok(Value::Undefined);
    };

    let properties = match args.get(1) {
        Some(Value::Object(ob)) => {
            //Convert to native array.
            //TODO: Can we make this an iterator?
            let mut array = vec![];
            let length = ob.get("length", activation)?.coerce_to_f64(activation)? as usize;
            for i in 0..length {
                array.push(
                    ob.get(&format!("{}", i), activation)?
                        .coerce_to_string(activation)?
                        .to_string(),
                )
            }

            Some(array)
        }
        Some(Value::String(s)) => Some(s.split(',').map(String::from).collect()),
        Some(_) => None,
        None => {
            avm_warn!(activation, "ASSetPropFlags called without object list!");
            return Ok(Value::Undefined);
        }
    };

    let set_attributes = EnumSet::<Attribute>::from_u128(
        args.get(2)
            .unwrap_or(&Value::Number(0.0))
            .coerce_to_f64(activation)? as u128,
    );

    let clear_attributes = EnumSet::<Attribute>::from_u128(
        args.get(3)
            .unwrap_or(&Value::Number(0.0))
            .coerce_to_f64(activation)? as u128,
    );

    match properties {
        Some(properties) => {
            for prop_name in properties {
                object.set_attributes(
                    activation.context.gc_context,
                    Some(&prop_name),
                    set_attributes,
                    clear_attributes,
                )
            }
        }
        None => object.set_attributes(
            activation.context.gc_context,
            None,
            set_attributes,
            clear_attributes,
        ),
    }

    Ok(Value::Undefined)
}

pub fn create_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object_function = FunctionObject::function_and_constructor(
        gc_context,
        Executable::Native(object_function),
        Executable::Native(constructor),
        Some(fn_proto),
        proto,
    );
    let mut object = object_function.as_script_object().unwrap();

    object.force_set_function(
        "registerClass",
        register_class,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        Some(fn_proto),
    );

    object_function
}
