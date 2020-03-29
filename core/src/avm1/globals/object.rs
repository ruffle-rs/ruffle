//! Object prototype
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute::{self, *};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, TObject, UpdateContext, Value};
use crate::character::Character;
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
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let name = args
        .get(0)
        .and_then(|v| v.to_owned().coerce_to_string(avm, context).ok())
        .unwrap_or_else(|| "undefined".to_string());
    let getter = args.get(1).unwrap_or(&Value::Undefined);
    let setter = args.get(2).unwrap_or(&Value::Undefined);

    match getter {
        Value::Object(get) if !name.is_empty() => {
            if let Some(get_func) = get.as_executable() {
                if let Value::Object(set) = setter {
                    if let Some(set_func) = set.as_executable() {
                        this.add_property_with_case(
                            avm,
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
                        avm,
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
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(Value::String(name)) => {
            Ok(Value::Bool(this.has_own_property(avm, context, name)).into())
        }
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
    avm: &mut Avm1<'gc>,
    _: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(Value::String(name)) => Ok(Value::Bool(this.is_property_enumerable(avm, name)).into()),
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

            Ok(Value::Bool(this.is_prototype_of(ob)).into())
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

/// Implements `Object.registerClass`
pub fn register_class<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(class_name) = args.get(0).cloned() {
        let class_name = class_name.coerce_to_string(avm, context)?;
        if let Some(Character::MovieClip(movie_clip)) = context
            .library
            .library_for_movie_mut(context.swf.clone())
            .get_character_by_export_name(&class_name)
        {
            if let Some(constructor) = args.get(1) {
                movie_clip.set_avm1_constructor(context.gc_context, Some(constructor.as_object()?));
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
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    _: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    //This exists because `.into` won't work inline
    let my_error: Result<ReturnValue<'gc>, Error> =
        Err("ASSetPropFlags called without object to apply to!".into());
    let my_error_2: Result<ReturnValue<'gc>, Error> =
        Err("ASSetPropFlags called without object list!".into());

    let mut object = args
        .get(0)
        .ok_or_else(|| my_error.unwrap_err())?
        .as_object()?;
    let properties = match args.get(1).ok_or_else(|| my_error_2.unwrap_err())? {
        Value::Object(ob) => {
            //Convert to native array.
            //TODO: Can we make this an iterator?
            let mut array = vec![];
            let length = ob
                .get("length", avm, ac)?
                .resolve(avm, ac)?
                .as_number(avm, ac)? as usize;
            for i in 0..length {
                array.push(
                    ob.get(&format!("{}", i), avm, ac)?
                        .resolve(avm, ac)?
                        .coerce_to_string(avm, ac)?,
                )
            }

            Some(array)
        }
        Value::String(s) => Some(s.split(',').map(String::from).collect()),
        _ => None,
    };

    let set_attributes = EnumSet::<Attribute>::from_bits(
        args.get(2)
            .unwrap_or(&Value::Number(0.0))
            .as_number(avm, ac)? as u128,
    );

    let clear_attributes = EnumSet::<Attribute>::from_bits(
        args.get(3)
            .unwrap_or(&Value::Number(0.0))
            .as_number(avm, ac)? as u128,
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
