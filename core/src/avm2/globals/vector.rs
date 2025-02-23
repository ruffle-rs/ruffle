//! `Vector` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::{argument_error, type_error};
use crate::avm2::globals::array::{
    compare_numeric, compare_string_case_insensitive, compare_string_case_sensitive, ArrayIter,
    SortOptions,
};
use crate::avm2::object::{ClassObject, Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::string::AvmString;
use ruffle_macros::istr;
use std::cmp::{max, min, Ordering};

// Allocator for generic Vector, not specialized Vector
pub fn vector_allocator<'gc>(
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vector) = this.as_vector_storage_mut(activation.gc()) {
        let length = args.get_u32(activation, 0)? as usize;
        let is_fixed = args.get_bool(1);

        vector.resize(length, activation)?;
        vector.set_is_fixed(is_fixed);
    }

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
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

    let this_class = activation
        .bound_class()
        .expect("Method call without bound class?");

    let value_type = this_class
        .param()
        .expect("Cannot convert to unparametrized Vector"); // technically unreachable

    let arg = args.get_value(0);

    if arg.instance_class(activation) == this_class {
        return Ok(arg);
    }

    let length = arg
        .get_public_property("length", activation)?
        .coerce_to_i32(activation)?;

    let arg = arg.as_object().ok_or("Cannot convert to Vector")?;

    let mut new_storage = VectorStorage::new(0, false, value_type, activation);
    new_storage.reserve_exact(length as usize);

    let value_type_for_coercion = new_storage.value_type_for_coercion(activation);

    let mut iter = ArrayIter::new(activation, arg)?;

    while let Some((_, item)) = iter.next(activation)? {
        let coerced_item = item.coerce_to_type(activation, value_type_for_coercion)?;
        new_storage.push(coerced_item, activation)?;
    }

    Ok(VectorObject::from_vector(new_storage, activation)?.into())
}

/// `Vector.length` getter
pub fn get_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(vector) = this.as_vector_storage() {
        return Ok(vector.length().into());
    }

    Ok(Value::Undefined)
}

/// `Vector.length` setter
pub fn set_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vector) = this.as_vector_storage_mut(activation.gc()) {
        let new_length = args.get_u32(activation, 0)? as usize;

        vector.resize(new_length, activation)?;
    }

    Ok(Value::Undefined)
}

/// `Vector.fixed` getter
pub fn get_fixed<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(vector) = this.as_vector_storage() {
        return Ok(vector.is_fixed().into());
    }

    Ok(Value::Undefined)
}

/// `Vector.fixed` setter
pub fn set_fixed<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vector) = this.as_vector_storage_mut(activation.gc()) {
        let new_fixed = args.get_bool(0);

        vector.set_is_fixed(new_fixed);
    }

    Ok(Value::Undefined)
}

/// `Vector.concat` impl
pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let mut new_vector_storage = if let Some(vector) = this.as_vector_storage() {
        vector.clone()
    } else {
        return Err("Not a vector-structured object".into());
    };

    let original_length = new_vector_storage.length();

    let use_swf10_behavior = activation.caller_movie().is_some_and(|m| m.version() < 11);

    let val_class = new_vector_storage.value_type_for_coercion(activation);

    for arg in args {
        let arg = arg.null_check(activation, None)?;

        // this is Vector.<int/uint/Number/*>
        let my_base_vector_class = activation
            .bound_class()
            .expect("Method call without bound class?");

        if !arg.is_of_type(activation, my_base_vector_class) {
            let base_vector_name = my_base_vector_class
                .name()
                .to_qualified_name_err_message(activation.gc());

            let instance_of_class_name = arg.instance_of_class_name(activation);

            return Err(Error::AvmError(type_error(
                activation,
                &format!(
                    "Error #1034: Type Coercion failed: cannot convert {}@00000000000 to {}.",
                    instance_of_class_name, base_vector_name,
                ),
                1034,
            )?));
        }

        let old_vec: Vec<Value<'gc>> = if let Some(old_vec) = arg.as_object() {
            if let Some(old_vec) = old_vec.as_vector_storage() {
                old_vec.iter().collect()
            } else {
                continue;
            }
        } else {
            continue;
        };

        for (i, val) in old_vec.iter().enumerate() {
            let insertion_index = (original_length + i) as i32;
            let coerced_val = val.coerce_to_type(activation, val_class)?;

            if use_swf10_behavior {
                // See bugzilla 504525: In SWFv10, calling `concat` with multiple
                // arguments passed results in concatenating in the wrong order.
                new_vector_storage.insert(insertion_index, coerced_val, activation)?;
            } else {
                new_vector_storage.push(coerced_val, activation)?;
            }
        }
    }

    Ok(VectorObject::from_vector(new_vector_storage, activation)?.into())
}

/// Implements `Vector.join`
pub fn join<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let separator = args.get_string(activation, 0)?;

    if let Some(vector) = this.as_vector_storage() {
        let mut accum = Vec::with_capacity(vector.length());

        for item in vector.iter() {
            if matches!(item, Value::Undefined) || matches!(item, Value::Null) {
                accum.push(istr!("null"));
            } else {
                accum.push(item.coerce_to_string(activation)?);
            }
        }

        return Ok(AvmString::new(activation.gc(), crate::string::join(&accum, &separator)).into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.every`
pub fn every<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let callback = args.get_value(0);
    let receiver = args.get_value(1);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some((i, item)) = iter.next(activation)? {
        let result = callback
            .call(activation, receiver, &[item, i.into(), this.into()])?
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let callback = args.get_value(0);
    let receiver = args.get_value(1);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some((i, item)) = iter.next(activation)? {
        let result = callback
            .call(activation, receiver, &[item, i.into(), this.into()])?
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let callback = args.get_value(0);
    let receiver = args.get_value(1);

    let value_type = this
        .instance_class()
        .param()
        .ok_or("Cannot filter unparameterized vector")?; // technically unreachable
    let mut new_storage = VectorStorage::new(0, false, value_type, activation);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some((i, item)) = iter.next(activation)? {
        let result = callback
            .call(activation, receiver, &[item, i.into(), this.into()])?
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let callback = args.get_value(0);
    let receiver = args.get_value(1);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some((i, item)) = iter.next(activation)? {
        callback.call(activation, receiver, &[item, i.into(), this.into()])?;
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.indexOf`
pub fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let search_for = args.get_value(0);
    let from_index = args.get_f64(activation, 1)?;

    let from_index = if from_index < 0.0 {
        let length = this.as_vector_storage().unwrap().length() as i32;

        max(length + from_index as i32, 0) as u32
    } else {
        from_index as u32
    };

    let mut iter = ArrayIter::with_bounds(activation, this, from_index, u32::MAX)?;

    while let Some((i, item)) = iter.next(activation)? {
        if item == search_for {
            return Ok(i.into());
        }
    }

    Ok((-1).into())
}

/// Implements `Vector.lastIndexOf`
pub fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let search_for = args.get_value(0);
    let from_index = args.get_f64(activation, 1)?;

    let from_index = if from_index < 0.0 {
        let length = this.as_vector_storage().unwrap().length() as i32;

        max(length + from_index as i32, 0) as u32
    } else {
        from_index as u32
    };

    let mut iter = ArrayIter::with_bounds(activation, this, 0, from_index)?;

    while let Some((i, item)) = iter.next_back(activation)? {
        if item == search_for {
            return Ok(i.into());
        }
    }

    Ok((-1).into())
}

/// Implements `Vector.map`
pub fn map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let callback = args.get_value(0);
    let receiver = args.get_value(1);

    let value_type = this
        .instance_class()
        .param()
        .ok_or("Cannot filter unparameterized vector")?; // technically unreachable
    let mut new_storage = VectorStorage::new(0, false, value_type, activation);
    let value_type_for_coercion = new_storage.value_type_for_coercion(activation);
    let mut iter = ArrayIter::new(activation, this)?;

    while let Some((i, item)) = iter.next(activation)? {
        let new_item = callback.call(activation, receiver, &[item, i.into(), this.into()])?;
        let coerced_item = new_item.coerce_to_type(activation, value_type_for_coercion)?;

        new_storage.push(coerced_item, activation)?;
    }

    Ok(VectorObject::from_vector(new_storage, activation)?.into())
}

/// Implements `Vector.pop`
pub fn pop<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
        return vs.pop(activation);
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.push`
pub fn push<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
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
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
        return vs.shift(activation);
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.unshift`
pub fn unshift<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
        let index = args.get_i32(activation, 0)?;

        let value_type = vs.value_type_for_coercion(activation);

        let value = args.get_value(1).coerce_to_type(activation, value_type)?;

        vs.insert(index, value, activation)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.removeAt`
pub fn remove_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
        let index = args.get_i32(activation, 0)?;

        return vs.remove(index, activation);
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.reverse`
pub fn reverse<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
        vs.reverse();

        return Ok(this.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.slice`
pub fn slice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(vs) = this.as_vector_storage_mut(activation.gc()) {
        let from = args.get_i32(activation, 0)?;
        let to = args.get_i32(activation, 1)?;
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
///
/// TODO: Consider sharing this code with `globals::array::sort`?
pub fn sort<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(vs) = this.as_vector_storage_mut(activation.gc()) {
        let fn_or_options = args.get_value(0);

        let (compare_fnc, options) = if let Some(callable) = fn_or_options
            .as_object()
            .filter(|o| o.as_class_object().is_some() || o.as_function_object().is_some())
        {
            (Some(Value::from(callable)), SortOptions::empty())
        } else {
            (
                None,
                SortOptions::from_bits_truncate(fn_or_options.coerce_to_u32(activation)? as u8),
            )
        };

        let compare = move |activation: &mut Activation<'_, 'gc>, a, b| {
            if let Some(compare_fnc) = compare_fnc {
                let order = compare_fnc
                    .call(activation, this.into(), &[a, b])?
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
        super::array::qsort(&mut values, &mut |a, b| {
            compare(activation, *a, *b).map(|cmp| {
                if cmp == Ordering::Equal {
                    unique_sort_satisfied = false;
                    Ordering::Equal
                } else if options.contains(SortOptions::DESCENDING) {
                    cmp.reverse()
                } else {
                    cmp
                }
            })
        })?;

        //NOTE: RETURNINDEXEDARRAY does NOT actually return anything useful.
        //The actual sorting still happens, but the results are discarded.
        if options.contains(SortOptions::RETURN_INDEXED_ARRAY) {
            return Ok(this.into());
        }

        if !options.contains(SortOptions::UNIQUE_SORT) || unique_sort_satisfied {
            let mut vs = this.as_vector_storage_mut(activation.gc()).unwrap();
            vs.replace_storage(values.into_iter().collect());
        }

        return Ok(this.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.splice`
pub fn splice<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(mut vs) = this.as_vector_storage_mut(activation.gc()) {
        let start_len = args.get_i32(activation, 0)?;
        let delete_len = args.get_i32(activation, 1)?;
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
