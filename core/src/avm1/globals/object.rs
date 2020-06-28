//! Object prototype
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute::{self, *};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::stack_frame::StackFrame;
use crate::avm1::{Object, TObject, UpdateContext, Value};
use crate::character::Character;
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::borrow::Cow;

/// Implements `Object`
pub fn constructor<'gc>(
    _activation: &mut StackFrame<'_, 'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(Value::Undefined.into())
}

/// Implements `Object.prototype.addProperty`
pub fn add_property<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .and_then(|v| v.coerce_to_string(activation, context).ok())
        .unwrap_or_else(|| Cow::Borrowed("undefined"));
    let getter = args.get(1).unwrap_or(&Value::Undefined);
    let setter = args.get(2).unwrap_or(&Value::Undefined);

    match getter {
        Value::Object(get) if !name.is_empty() => {
            if let Some(get_func) = get.as_executable() {
                if let Value::Object(set) = setter {
                    if let Some(set_func) = set.as_executable() {
                        this.add_property_with_case(
                            activation,
                            context.gc_context,
                            &name,
                            get_func.clone(),
                            Some(set_func.clone()),
                            EnumSet::empty(),
                        );
                    } else {
                        return Ok(false.into());
                    }
                } else if let Value::Null = setter {
                    this.add_property_with_case(
                        activation,
                        context.gc_context,
                        &name,
                        get_func.clone(),
                        None,
                        ReadOnly.into(),
                    );
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
    activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    if let Some(value) = args.get(0) {
        let name = value.coerce_to_string(activation, context)?;
        Ok(Value::Bool(this.has_own_property(activation, context, &name)).into())
    } else {
        Ok(false.into())
    }
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    _: &mut StackFrame<'_, 'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    _: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(ReturnValue::Immediate("[object Object]".into()))
}

/// Implements `Object.prototype.isPropertyEnumerable`
fn is_property_enumerable<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(Value::String(name)) => {
            Ok(Value::Bool(this.is_property_enumerable(activation, name)).into())
        }
        _ => Ok(Value::Bool(false).into()),
    }
}

/// Implements `Object.prototype.isPrototypeOf`
fn is_prototype_of<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    match args.get(0) {
        Some(val) => {
            let ob = val.coerce_to_object(activation, context);
            Ok(Value::Bool(this.is_prototype_of(ob)).into())
        }
        _ => Ok(Value::Bool(false).into()),
    }
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    _: &mut StackFrame<'_, 'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(ReturnValue::Immediate(this.into()))
}

/// Implements `Object.registerClass`
pub fn register_class<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    if let Some(class_name) = args.get(0).cloned() {
        let class_name = class_name.coerce_to_string(activation, context)?;
        if let Some(Character::MovieClip(movie_clip)) = context
            .library
            .library_for_movie_mut(context.swf.clone())
            .get_character_by_export_name(&class_name)
        {
            if let Some(constructor) = args.get(1) {
                movie_clip.set_avm1_constructor(
                    context.gc_context,
                    Some(constructor.coerce_to_object(activation, context)),
                );
            } else {
                movie_clip.set_avm1_constructor(context.gc_context, None);
            }
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

/// Implements `ASSetPropFlags`.
///
/// This is an undocumented function that allows ActionScript 2.0 classes to
/// declare the property flags of a given property. It's not part of
/// `Object.prototype`, and I suspect that's a deliberate omission.
pub fn as_set_prop_flags<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    _: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let mut object = if let Some(object) = args.get(0).map(|v| v.coerce_to_object(activation, ac)) {
        object
    } else {
        log::warn!("ASSetPropFlags called without object to apply to!");
        return Ok(Value::Undefined.into());
    };

    let properties = match args.get(1) {
        Some(Value::Object(ob)) => {
            //Convert to native array.
            //TODO: Can we make this an iterator?
            let mut array = vec![];
            let length = ob
                .get("length", activation, ac)?
                .coerce_to_f64(activation, ac)? as usize;
            for i in 0..length {
                array.push(
                    ob.get(&format!("{}", i), activation, ac)?
                        .coerce_to_string(activation, ac)?
                        .to_string(),
                )
            }

            Some(array)
        }
        Some(Value::String(s)) => Some(s.split(',').map(String::from).collect()),
        Some(_) => None,
        None => {
            log::warn!("ASSetPropFlags called without object list!");
            return Ok(Value::Undefined.into());
        }
    };

    let set_attributes = EnumSet::<Attribute>::from_u128(
        args.get(2)
            .unwrap_or(&Value::Number(0.0))
            .coerce_to_f64(activation, ac)? as u128,
    );

    let clear_attributes = EnumSet::<Attribute>::from_u128(
        args.get(3)
            .unwrap_or(&Value::Number(0.0))
            .coerce_to_f64(activation, ac)? as u128,
    );

    match properties {
        Some(properties) => {
            for prop_name in properties {
                object.set_attributes(
                    ac.gc_context,
                    Some(&prop_name),
                    set_attributes,
                    clear_attributes,
                )
            }
        }
        None => object.set_attributes(ac.gc_context, None, set_attributes, clear_attributes),
    }

    Ok(Value::Undefined.into())
}

pub fn create_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object_function = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        Some(fn_proto),
        Some(proto),
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
