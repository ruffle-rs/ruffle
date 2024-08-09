//! `Vector` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::{argument_error, type_error};
use crate::avm2::globals::array::{
    compare_numeric, compare_string_case_insensitive, compare_string_case_sensitive, ArrayIter,
    SortOptions,
};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{
    vector_allocator, ClassObject, FunctionObject, Object, TObject, VectorObject,
};
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::avm2::QName;
use crate::string::AvmString;
use std::cmp::{max, min, Ordering};

pub fn generic_vector_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    return Err(Error::AvmError(type_error(
        activation,
        "Error #1007: Instantiation attempted on a non-constructor.",
        1007,
    )?));
}

/// Implements `Vector`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, &[])?;

    if let Some(mut vector) = this.as_vector_storage_mut(activation.context.gc_context) {
        let length = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Integer(0))
            .coerce_to_u32(activation)? as usize;
        let is_fixed = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| false.into())
            .coerce_to_boolean();

        vector.resize(length, activation)?;
        vector.set_is_fixed(is_fixed);
    }

    Ok(Value::Undefined)
}

fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() != 1 {
        return Err(Error::AvmError(argument_error(
            activation,
            &format!(
                "Error #1112: Argument count mismatch on class coercion.  Expected 1, got {}.",
                args.len()
            ),
            1112,
        )?));
    }

    let this_class = activation.subclass_object().unwrap();
    let value_type = this_class
        .inner_class_definition()
        .param()
        .ok_or("Cannot convert to unparametrized Vector")?; // technically unreachable

    let arg = args.get(0).cloned().unwrap();
    let arg = arg.as_object().ok_or("Cannot convert to Vector")?;

    if arg.instance_class() == this_class.inner_class_definition() {
        return Ok(arg.into());
    }

    let length = arg
        .get_public_property("length", activation)?
        .coerce_to_i32(activation)?;

    let mut new_storage = VectorStorage::new(0, false, value_type, activation);
    new_storage.reserve_exact(length as usize);

    let value_type_for_coercion = new_storage.value_type_for_coercion(activation);

    let mut iter = ArrayIter::new(activation, arg)?;

    while let Some(r) = iter.next(activation) {
        let (_, item) = r?;
        let coerced_item = item.coerce_to_type(activation, value_type_for_coercion)?;
        new_storage.push(coerced_item, activation)?;
    }

    Ok(VectorObject::from_vector(new_storage, activation)?.into())
}

pub fn generic_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)
}

fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let proto = this
        .get_public_property("prototype", activation)?
        .as_object()
        .ok_or_else(|| {
            format!(
                "Specialization {} has a prototype of null or undefined",
                this.instance_of_class_name(activation.context.gc_context)
            )
        })?;
    let scope = activation.create_scopechain();

    const PUBLIC_PROTOTYPE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("concat", concat),
        ("join", join),
        ("toString", to_string),
        ("toLocaleString", to_locale_string),
        ("every", every),
        ("some", some),
        ("forEach", for_each),
        ("filter", filter),
        ("indexOf", index_of),
        ("lastIndexOf", last_index_of),
        ("map", map),
        ("pop", pop),
        ("push", push),
        ("shift", shift),
        ("unshift", unshift),
        ("reverse", reverse),
        ("slice", slice),
        ("sort", sort),
        ("splice", splice),
    ];
    for (pubname, func) in PUBLIC_PROTOTYPE_METHODS {
        proto.set_string_property_local(
            *pubname,
            FunctionObject::from_function(
                activation,
                Method::from_builtin(*func, pubname, activation.context.gc_context),
                scope,
            )?
            .into(),
            activation,
        )?;
        proto.set_local_property_is_enumerable(
            activation.context.gc_context,
            (*pubname).into(),
            false,
        );
    }

    Ok(Value::Undefined)
}

/// `Vector.length` getter
pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vector) = this.as_vector_storage() {
        return Ok(vector.length().into());
    }

    Ok(Value::Undefined)
}

/// `Vector.length` setter
pub fn set_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vector) = this.as_vector_storage_mut(activation.context.gc_context) {
        let new_length = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Integer(0))
            .coerce_to_u32(activation)? as usize;

        vector.resize(new_length, activation)?;
    }

    Ok(Value::Undefined)
}

/// `Vector.fixed` getter
pub fn fixed<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vector) = this.as_vector_storage() {
        return Ok(vector.is_fixed().into());
    }

    Ok(Value::Undefined)
}

/// `Vector.fixed` setter
pub fn set_fixed<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vector) = this.as_vector_storage_mut(activation.context.gc_context) {
        let new_fixed = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Bool(false))
            .coerce_to_boolean();

        vector.set_is_fixed(new_fixed);
    }

    Ok(Value::Undefined)
}

/// `Vector.concat` impl
pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut new_vector_storage = if let Some(vector) = this.as_vector_storage() {
        vector.clone()
    } else {
        return Err("Not a vector-structured object".into());
    };

    let val_class = new_vector_storage.value_type_for_coercion(activation);

    for arg in args {
        let arg_obj = arg
            .as_object()
            .ok_or("Cannot concat Vector with null or undefined")?;

        // this is Vector.<int/uint/Number/*>
        let my_base_vector_class = activation
            .subclass_object()
            .expect("Method call without bound class?")
            .inner_class_definition();
        if !arg.is_of_type(activation, my_base_vector_class) {
            let base_vector_name = my_base_vector_class
                .name()
                .to_qualified_name_err_message(activation.context.gc_context);

            return Err(Error::AvmError(type_error(
                activation,
                &format!(
                    "Error #1034: Type Coercion failed: cannot convert {}@00000000000 to {}.",
                    arg_obj.instance_of_class_name(activation.context.gc_context),
                    base_vector_name,
                ),
                1034,
            )?));
        }

        let old_vec = arg_obj.as_vector_storage();
        let old_vec: Vec<Value<'gc>> = if let Some(old_vec) = old_vec {
            old_vec.iter().collect()
        } else {
            continue;
        };

        for val in old_vec {
            if let Ok(val_obj) = val.coerce_to_object(activation) {
                if !val.is_of_type(activation, val_class) {
                    let other_val_class = val_obj.instance_class();
                    return Err(format!(
                        "TypeError: Cannot coerce Vector value of type {:?} to type {:?}",
                        other_val_class.name(),
                        val_class.name()
                    )
                    .into());
                }
            }

            let coerced_val = val.coerce_to_type(activation, val_class)?;
            new_vector_storage.push(coerced_val, activation)?;
        }
    }

    Ok(VectorObject::from_vector(new_vector_storage, activation)?.into())
}

fn join_inner<'gc, 'a, 'ctxt, C>(
    activation: &mut Activation<'a, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    mut conv: C,
) -> Result<Value<'gc>, Error<'gc>>
where
    C: for<'b> FnMut(Value<'gc>, &'b mut Activation<'a, 'gc>) -> Result<Value<'gc>, Error<'gc>>,
{
    let mut separator = args.get(0).cloned().unwrap_or(Value::Undefined);
    if separator == Value::Undefined {
        separator = ",".into();
    }

    if let Some(vector) = this.as_vector_storage() {
        let string_separator = separator.coerce_to_string(activation)?;
        let mut accum = Vec::with_capacity(vector.length());

        for item in vector.iter() {
            if matches!(item, Value::Undefined) || matches!(item, Value::Null) {
                accum.push("null".into());
            } else {
                accum.push(conv(item, activation)?.coerce_to_string(activation)?);
            }
        }

        return Ok(AvmString::new(
            activation.context.gc_context,
            crate::string::join(&accum, &string_separator),
        )
        .into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.join`
pub fn join<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    join_inner(activation, this, args, |v, _act| Ok(v))
}

/// Implements `Vector.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    join_inner(activation, this, &[",".into()], |v, _act| Ok(v))
}

/// Implements `Vector.toLocaleString`
pub fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    join_inner(activation, this, &[",".into()], |v, act| {
        if let Ok(o) = v.coerce_to_object(act) {
            o.call_public_property("toLocaleString", &[], act)
        } else {
            Ok(v)
        }
    })
}

/// Implements `Vector.every`
pub fn every<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_callable(activation, None, None, false)?;
    let receiver = args.get(1).cloned().unwrap_or(Value::Null);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some(r) = iter.next(activation) {
        let (i, item) = r?;

        let result = callback
            .call(receiver, &[item, i.into(), this.into()], activation)?
            .coerce_to_boolean();

        if !result {
            return Ok(false.into());
        }
    }

    Ok(true.into())
}

/// Implements `Vector.some`
pub fn some<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_callable(activation, None, None, false)?;
    let receiver = args.get(1).cloned().unwrap_or(Value::Null);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some(r) = iter.next(activation) {
        let (i, item) = r?;

        let result = callback
            .call(receiver, &[item, i.into(), this.into()], activation)?
            .coerce_to_boolean();

        if result {
            return Ok(true.into());
        }
    }

    Ok(false.into())
}

/// Implements `Vector.filter`
pub fn filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_callable(activation, None, None, false)?;
    let receiver = args.get(1).cloned().unwrap_or(Value::Null);

    let value_type = this
        .instance_class()
        .param()
        .ok_or("Cannot filter unparameterized vector")?; // technically unreachable
    let mut new_storage = VectorStorage::new(0, false, value_type, activation);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some(r) = iter.next(activation) {
        let (i, item) = r?;

        let result = callback
            .call(receiver, &[item, i.into(), this.into()], activation)?
            .coerce_to_boolean();

        if result {
            new_storage.push(item, activation)?;
        }
    }

    Ok(VectorObject::from_vector(new_storage, activation)?.into())
}

/// Implements `Vector.forEach`
pub fn for_each<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_callable(activation, None, None, false)?;
    let receiver = args.get(1).cloned().unwrap_or(Value::Null);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some(r) = iter.next(activation) {
        let (i, item) = r?;

        callback.call(receiver, &[item, i.into(), this.into()], activation)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.indexOf`
pub fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let search_for = args.get(0).cloned().unwrap_or(Value::Undefined);
    let from_index = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| 0.into())
        .coerce_to_i32(activation)?;

    let from_index = if from_index < 0 {
        let length = this
            .get_public_property("length", activation)?
            .coerce_to_i32(activation)?;
        max(length + from_index, 0) as u32
    } else {
        from_index as u32
    };

    let mut iter = ArrayIter::with_bounds(activation, this, from_index, u32::MAX)?;

    while let Some(r) = iter.next(activation) {
        let (i, item) = r?;

        if item == search_for {
            return Ok(i.into());
        }
    }

    Ok((-1).into())
}

/// Implements `Vector.lastIndexOf`
pub fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let search_for = args.get(0).cloned().unwrap_or(Value::Undefined);
    let from_index = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| i32::MAX.into())
        .coerce_to_i32(activation)?;

    let from_index = if from_index < 0 {
        let length = this
            .get_public_property("length", activation)?
            .coerce_to_i32(activation)?;
        max(length + from_index, 0) as u32
    } else {
        from_index as u32
    };

    let mut iter = ArrayIter::with_bounds(activation, this, 0, from_index)?;

    while let Some(r) = iter.next_back(activation) {
        let (i, item) = r?;

        if item == search_for {
            return Ok(i.into());
        }
    }

    Ok((-1).into())
}

/// Implements `Vector.map`
pub fn map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let callback = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .as_callable(activation, None, None, false)?;
    let receiver = args.get(1).cloned().unwrap_or(Value::Null);

    let value_type = this
        .instance_class()
        .param()
        .ok_or("Cannot filter unparameterized vector")?; // technically unreachable
    let mut new_storage = VectorStorage::new(0, false, value_type, activation);
    let value_type_for_coercion = new_storage.value_type_for_coercion(activation);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some(r) = iter.next(activation) {
        let (i, item) = r?;

        let new_item = callback.call(receiver, &[item, i.into(), this.into()], activation)?;
        let coerced_item = new_item.coerce_to_type(activation, value_type_for_coercion)?;

        new_storage.push(coerced_item, activation)?;
    }

    Ok(VectorObject::from_vector(new_storage, activation)?.into())
}

/// Implements `Vector.pop`
pub fn pop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        return vs.pop(activation);
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.push`
pub fn push<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let value_type = vs.value_type_for_coercion(activation);

        // Pushing nothing will still throw if the Vector is fixed.
        vs.check_fixed(activation)?;

        for arg in args {
            let coerced_arg = arg.coerce_to_type(activation, value_type)?;

            vs.push(coerced_arg, activation)?;
        }

        return Ok(vs.length().into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.shift`
pub fn shift<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        return vs.shift(activation);
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.unshift`
pub fn unshift<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let value_type = vs.value_type_for_coercion(activation);

        for arg in args.iter().rev() {
            let coerced_arg = arg.coerce_to_type(activation, value_type)?;

            vs.unshift(coerced_arg, activation)?;
        }

        return Ok(vs.length().into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.insertAt`
pub fn insert_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let index = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;

        let value_type = vs.value_type_for_coercion(activation);

        let value = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_type(activation, value_type)?;

        vs.insert(index, value, activation)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.removeAt`
pub fn remove_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let index = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;

        return vs.remove(index, activation);
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.reverse`
pub fn reverse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        vs.reverse();

        return Ok(this.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.slice`
pub fn slice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let from = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| 0.into())
            .coerce_to_i32(activation)?;
        let to = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| 16777215.into())
            .coerce_to_i32(activation)?;
        let value_type = vs.value_type();

        let from = vs.clamp_parameter_index(from);
        let to = vs.clamp_parameter_index(to);

        let mut new_vs = VectorStorage::new(0, false, value_type, activation);

        if to > from {
            for value in vs.iter().skip(from).take(to - from) {
                new_vs.push(value, activation)?;
            }
        }

        let new_vector = VectorObject::from_vector(new_vs, activation)?;

        return Ok(new_vector.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.sort`
pub fn sort<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let fn_or_options = args.get(0).cloned().unwrap_or(Value::Undefined);

        let (compare_fnc, options) = if fn_or_options
            .as_callable(activation, None, None, false)
            .is_ok()
        {
            (
                Some(fn_or_options.as_object().unwrap()),
                SortOptions::empty(),
            )
        } else {
            (
                None,
                SortOptions::from_bits_truncate(fn_or_options.coerce_to_u32(activation)? as u8),
            )
        };

        let compare = move |activation: &mut Activation<'_, 'gc>, a, b| {
            if let Some(compare_fnc) = compare_fnc {
                let order = compare_fnc
                    .call(this.into(), &[a, b], activation)?
                    .coerce_to_number(activation)?;

                if order > 0.0 {
                    Ok(Ordering::Greater)
                } else if order < 0.0 {
                    Ok(Ordering::Less)
                } else {
                    Ok(Ordering::Equal)
                }
            } else if options.contains(SortOptions::NUMERIC) {
                compare_numeric(activation, a, b)
            } else if options.contains(SortOptions::CASE_INSENSITIVE) {
                compare_string_case_insensitive(activation, a, b)
            } else {
                compare_string_case_sensitive(activation, a, b)
            }
        };

        let mut values: Vec<_> = vs.iter().collect();
        drop(vs);

        let mut unique_sort_satisfied = true;
        let mut error_signal = Ok(());
        values.sort_unstable_by(|a, b| match compare(activation, *a, *b) {
            Ok(Ordering::Equal) => {
                unique_sort_satisfied = false;
                Ordering::Equal
            }
            Ok(v) if options.contains(SortOptions::DESCENDING) => v.reverse(),
            Ok(v) => v,
            Err(e) => {
                error_signal = Err(e);
                Ordering::Less
            }
        });

        error_signal?;

        //NOTE: RETURNINDEXEDARRAY does NOT actually return anything useful.
        //The actual sorting still happens, but the results are discarded.
        if options.contains(SortOptions::RETURN_INDEXED_ARRAY) {
            return Ok(this.into());
        }

        if !options.contains(SortOptions::UNIQUE_SORT) || unique_sort_satisfied {
            let mut vs = this
                .as_vector_storage_mut(activation.context.gc_context)
                .unwrap();
            vs.replace_storage(values.into_iter().collect());
        }

        return Ok(this.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.splice`
pub fn splice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
        let start_len = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        let delete_len = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_i32(activation)?;
        let value_type = vs.value_type();
        let value_type_for_coercion = vs.value_type_for_coercion(activation);

        let start = vs.clamp_parameter_index(start_len);
        let end = max(
            start,
            min(
                if delete_len < 0 {
                    vs.clamp_parameter_index(delete_len)
                } else {
                    start + delete_len as usize
                },
                vs.length(),
            ),
        );
        let mut to_coerce = Vec::new();

        for value in args[2..].iter() {
            to_coerce.push(value.coerce_to_type(activation, value_type_for_coercion)?);
        }

        let new_vs =
            VectorStorage::from_values(vs.splice(start..end, to_coerce)?, false, value_type);
        let new_vector = VectorObject::from_vector(new_vs, activation)?;

        return Ok(new_vector.into());
    }

    Ok(Value::Undefined)
}

/// Construct `Vector`'s class.
pub fn create_generic_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().vector_public_namespace, "Vector"),
        Some(activation.avm2().classes().object.inner_class_definition()),
        Method::from_builtin(generic_init, "<Vector instance initializer>", mc),
        Method::from_builtin(generic_init, "<Vector class initializer>", mc),
        activation.avm2().classes().class.inner_class_definition(),
        mc,
    );

    class.set_attributes(mc, ClassAttributes::GENERIC | ClassAttributes::FINAL);
    class.set_instance_allocator(mc, generic_vector_allocator);

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}

/// Construct `Vector.<int/uint/Number/*>`'s class.
pub fn create_builtin_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    param: Option<Class<'gc>>,
) -> Class<'gc> {
    let mc = activation.context.gc_context;

    // FIXME - we should store a `Multiname` instead of a `QName`, and use the
    // `params` field. For now, this is good enough to get tests passing
    let name = if let Some(param) = param {
        let name = format!("Vector.<{}>", param.name().to_qualified_name(mc));
        QName::new(
            activation.avm2().vector_public_namespace,
            AvmString::new_utf8(mc, name),
        )
    } else {
        QName::new(activation.avm2().vector_public_namespace, "Vector.<*>")
    };

    let class = Class::new(
        name,
        Some(activation.avm2().classes().object.inner_class_definition()),
        Method::from_builtin(instance_init, "<Vector.<T> instance initializer>", mc),
        Method::from_builtin(class_init, "<Vector.<T> class initializer>", mc),
        activation.avm2().classes().class.inner_class_definition(),
        mc,
    );

    // TODO: Vector.<*> is also supposed to be final, but currently
    // that'd make it impossible for us to create derived Vector.<MyType>.
    if param.is_some() {
        class.set_attributes(mc, ClassAttributes::FINAL);
    }
    class.set_param(mc, Some(param));
    class.set_instance_allocator(mc, vector_allocator);
    class.set_call_handler(
        mc,
        Method::from_builtin(class_call, "<Vector.<T> call handler>", mc),
    );

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("length", Some(length), Some(set_length)),
        ("fixed", Some(fixed), Some(set_fixed)),
    ];
    class.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace_base_version,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("concat", concat),
        ("join", join),
        ("toString", to_string),
        ("toLocaleString", to_locale_string),
        ("every", every),
        ("some", some),
        ("forEach", for_each),
        ("filter", filter),
        ("indexOf", index_of),
        ("lastIndexOf", last_index_of),
        ("map", map),
        ("pop", pop),
        ("push", push),
        ("shift", shift),
        ("unshift", unshift),
        ("insertAt", insert_at),
        ("removeAt", remove_at),
        ("reverse", reverse),
        ("slice", slice),
        ("sort", sort),
        ("splice", splice),
    ];
    class.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}
