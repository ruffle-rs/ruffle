//! `Vector` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::{argument_error, type_error};
use crate::avm2::globals::array::{
    compare_numeric, compare_string_case_insensitive, compare_string_case_sensitive, ArrayIter,
    SortOptions,
};
use crate::avm2::object::{ClassObject, Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Error, Multiname, QName};
use crate::string::{AvmString, WStr};
use ruffle_macros::istr;
use std::cmp::{max, min, Ordering};

// Allocator for generic Vector, not specialized Vector
pub fn vector_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    return Err(Error::avm_error(type_error(
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
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() != 1 {
        return Err(Error::avm_error(argument_error(
            activation,
            &format!(
                "Error #1112: Argument count mismatch on class coercion.  Expected 1, got {}.",
                args.len()
            ),
            1112,
        )?));
    }

    let this_class = this
        .as_object()
        .unwrap()
        .as_class_object()
        .unwrap()
        .inner_class_definition();

    let value_type = this_class
        .param()
        .expect("Cannot convert to unparametrized Vector"); // technically unreachable

    let arg = args.get_value(0);

    if arg.instance_class(activation) == this_class {
        return Ok(arg);
    }

    let length = arg
        .get_public_property(istr!("length"), activation)?
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

    let mut new_vector_storage = this
        .as_vector_storage()
        .expect("Receiver is of type Vector.<T>")
        .clone();

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

            return Err(Error::avm_error(type_error(
                activation,
                &format!(
                    "Error #1034: Type Coercion failed: cannot convert {instance_of_class_name}@00000000000 to {base_vector_name}.",
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

macro_rules! delegate_method_to_array {
    ($method:ident) => {
        pub fn $method<'gc>(
            activation: &mut Activation<'_, 'gc>,
            this: Value<'gc>,
            args: &[Value<'gc>],
        ) -> Result<Value<'gc>, Error<'gc>> {
            super::array::$method(activation, this, args)
        }
    };
}

delegate_method_to_array!(every);
delegate_method_to_array!(_some);
delegate_method_to_array!(for_each);

/// Implements `Vector.filter`
pub fn filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let value_type = this
        .instance_class()
        .param()
        .expect("Receiver is parametrized vector"); // technically unreachable

    let mut new_storage = VectorStorage::new(0, false, value_type, activation);

    let callback = match args.get_value(0) {
        Value::Null => return Ok(VectorObject::from_vector(new_storage, activation)?.into()),
        value => value,
    };
    let receiver = args.get_value(1);

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
        .expect("Receiver is parametrized vector"); // technically unreachable
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

/// Set up a builtin vector's Class. This will change its name, mark it as a
/// specialization of Vector, and set its class parameter to the passed
/// `param_class`. This function returns the vector Class.
fn setup_vector_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    old_name: &'static str,
    new_name: &'static str,
    param_class: Option<Class<'gc>>,
) -> Class<'gc> {
    let generic_vector_cls = activation.avm2().class_defs().generic_vector;

    let vector_ns = activation.avm2().namespaces.vector_internal;

    // First, lookup the class
    let old_name = activation
        .strings()
        .intern_static(WStr::from_units(old_name.as_bytes()));

    let vector_cls = activation
        .domain()
        .get_class(activation.context, &Multiname::new(vector_ns, old_name))
        .expect("Vector class should be defined");

    // Set its name to Vector.<T>
    let new_name = AvmString::new_utf8(activation.gc(), new_name);

    vector_cls.set_name(activation.gc(), QName::new(vector_ns, new_name));

    // Set its parameter to the given parameter and add it to the map of
    // applications on the generic vector Class
    vector_cls.set_param(activation.gc(), Some(param_class));
    generic_vector_cls.add_application(activation.gc(), param_class, vector_cls);

    vector_cls
}

pub fn init_vector_class_defs(activation: &mut Activation<'_, '_>) {
    // Mark Vector as a generic class
    let generic_vector = activation.avm2().class_defs().generic_vector;
    generic_vector.set_attributes(ClassAttributes::GENERIC | ClassAttributes::FINAL);

    // Setup the four builtin vector classes

    let number_cls = activation.avm2().class_defs().number;
    setup_vector_class(
        activation,
        "Vector$double",
        "Vector.<Number>",
        Some(number_cls),
    );

    let int_cls = activation.avm2().class_defs().int;
    setup_vector_class(activation, "Vector$int", "Vector.<int>", Some(int_cls));

    let uint_cls = activation.avm2().class_defs().uint;
    setup_vector_class(activation, "Vector$uint", "Vector.<uint>", Some(uint_cls));

    setup_vector_class(activation, "Vector$object", "Vector.<*>", None);
}

/// Set up a builtin vector's ClassObject. This marks it as a specialization of
/// Vector. This function returns the vector ClassObject.
fn setup_vector_class_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    vector_name: &'static str,
    param_class: Option<Class<'gc>>,
) -> ClassObject<'gc> {
    let generic_vector_cls = activation.avm2().classes().generic_vector;

    let vector_ns = activation.avm2().namespaces.vector_internal;

    // `vector_name` should be ASCII
    let class_name = activation
        .strings()
        .intern_static(WStr::from_units(vector_name.as_bytes()));

    let value = activation
        .domain()
        .get_defined_value(activation, QName::new(vector_ns, class_name))
        .expect("Vector class should be defined");

    let vector_cls = value.as_object().unwrap().as_class_object().unwrap();

    generic_vector_cls.add_application(activation.gc(), param_class, vector_cls);

    vector_cls
}

pub fn init_vector_class_objects(activation: &mut Activation<'_, '_>) {
    // Register Vector$int/uint/Number/Object as being applications of the Vector ClassObject
    let number_cls = activation.avm2().class_defs().number;
    setup_vector_class_object(activation, "Vector$double", Some(number_cls));

    let int_cls = activation.avm2().class_defs().int;
    setup_vector_class_object(activation, "Vector$int", Some(int_cls));

    let uint_cls = activation.avm2().class_defs().uint;
    setup_vector_class_object(activation, "Vector$uint", Some(uint_cls));

    let object_vector = setup_vector_class_object(activation, "Vector$object", None);

    // Manually set the object vector class since it's in an internal namespace
    // (`avm2_system_classes_playerglobal` only works for classes in public namespaces)
    activation
        .avm2()
        .system_classes
        .as_mut()
        .unwrap()
        .object_vector = object_vector;
}
