//! `Vector` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::globals::array::{
    compare_numeric, compare_string_case_insensitive, compare_string_case_sensitive, ArrayIter,
    SortOptions,
};
use crate::avm2::globals::NS_VECTOR;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{vector_allocator, FunctionObject, Object, TObject, VectorObject};
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};
use std::cmp::{max, min, Ordering};

/// Implements `Vector`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if let Some(mut vector) = this.as_vector_storage_mut(activation.context.gc_context) {
            let length = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Unsigned(0))
                .coerce_to_u32(activation)? as usize;
            let is_fixed = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| false.into())
                .coerce_to_boolean();

            vector.resize(length, activation)?;
            vector.set_is_fixed(is_fixed);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector`'s class constructor.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut globals = this.get_scope().map(|s| s.read().globals()).unwrap();
        let mut domain = globals.as_application_domain().unwrap();

        //We have to grab Object's defining script instead of our own, because
        //at this point Vector hasn't actually been defined yet. It doesn't
        //matter because we only have one script for our globals.
        let (_, script) = domain
            .get_defining_script(&QName::new(Namespace::public(), "Object").into())?
            .unwrap();

        let int_class = activation.avm2().classes().int;
        let int_vector_class = this.apply(activation, &[int_class.into()])?;
        let int_vector_name = QName::new(Namespace::internal(NS_VECTOR), "Vector$int");
        int_vector_class
            .as_class()
            .unwrap()
            .write(activation.context.gc_context)
            .set_name(int_vector_name.clone());

        globals.install_const(
            activation.context.gc_context,
            int_vector_name.clone(),
            0,
            int_vector_class.into(),
            false,
        );
        domain.export_definition(int_vector_name, script, activation.context.gc_context)?;

        let uint_class = activation.avm2().classes().uint;
        let uint_vector_class = this.apply(activation, &[uint_class.into()])?;
        let uint_vector_name = QName::new(Namespace::internal(NS_VECTOR), "Vector$uint");
        uint_vector_class
            .as_class()
            .unwrap()
            .write(activation.context.gc_context)
            .set_name(uint_vector_name.clone());

        globals.install_const(
            activation.context.gc_context,
            uint_vector_name.clone(),
            0,
            uint_vector_class.into(),
            false,
        );
        domain.export_definition(uint_vector_name, script, activation.context.gc_context)?;

        let number_class = activation.avm2().classes().number;
        let number_vector_class = this.apply(activation, &[number_class.into()])?;
        let number_vector_name = QName::new(Namespace::internal(NS_VECTOR), "Vector$double");
        number_vector_class
            .as_class()
            .unwrap()
            .write(activation.context.gc_context)
            .set_name(number_vector_name.clone());

        globals.install_const(
            activation.context.gc_context,
            number_vector_name.clone(),
            0,
            number_vector_class.into(),
            false,
        );
        domain.export_definition(number_vector_name, script, activation.context.gc_context)?;

        let object_vector_class = this.apply(activation, &[Value::Null])?;
        let object_vector_name = QName::new(Namespace::internal(NS_VECTOR), "Vector$object");
        object_vector_class
            .as_class()
            .unwrap()
            .write(activation.context.gc_context)
            .set_name(object_vector_name.clone());

        globals.install_const(
            activation.context.gc_context,
            object_vector_name.clone(),
            0,
            object_vector_class.into(),
            false,
        );
        domain.export_definition(object_vector_name, script, activation.context.gc_context)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Vector`'s specialized-class constructor.
pub fn specialized_class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut proto = this
            .get_property(this, &QName::dynamic_name("prototype"), activation)?
            .coerce_to_object(activation)?;
        let scope = this.get_scope();

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
            proto.set_property(
                this,
                &QName::dynamic_name(*pubname),
                FunctionObject::from_function(
                    activation,
                    Method::from_builtin(*func, pubname, activation.context.gc_context),
                    scope,
                )?
                .into(),
                activation,
            )?;
        }
    }

    Ok(Value::Undefined)
}

/// `Vector.length` getter
pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(vector) = this.as_vector_storage() {
            return Ok(vector.length().into());
        }
    }

    Ok(Value::Undefined)
}

/// `Vector.length` setter
pub fn set_length<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vector) = this.as_vector_storage_mut(activation.context.gc_context) {
            let new_length = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Unsigned(0))
                .coerce_to_u32(activation)? as usize;

            vector.resize(new_length, activation)?;
        }
    }

    Ok(Value::Undefined)
}

/// `Vector.fixed` getter
pub fn fixed<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(vector) = this.as_vector_storage() {
            return Ok(vector.is_fixed().into());
        }
    }

    Ok(Value::Undefined)
}

/// `Vector.fixed` setter
pub fn set_fixed<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vector) = this.as_vector_storage_mut(activation.context.gc_context) {
            let new_fixed = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Bool(false))
                .coerce_to_boolean();

            vector.set_is_fixed(new_fixed);
        }
    }

    Ok(Value::Undefined)
}

/// `Vector.concat` impl
pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut new_vector_storage = if let Some(vector) = this.as_vector_storage() {
            vector.clone()
        } else {
            return Err("Not a vector-structured object".into());
        };

        let my_class = this
            .instance_of()
            .ok_or("TypeError: Tried to concat into a bare object")?;
        let val_class = new_vector_storage.value_type();

        for arg in args.iter().map(|a| a.clone()) {
            let arg_obj = arg.coerce_to_object(activation)?;
            let arg_class = arg_obj
                .as_class()
                .ok_or("TypeError: Tried to concat from a bare object")?;
            if !arg.is_of_type(activation, my_class)? {
                return Err(format!(
                    "TypeError: Cannot coerce argument of type {:?} to argument of type {:?}",
                    arg_class.read().name(),
                    my_class
                        .as_class()
                        .ok_or("TypeError: Tried to concat into a bare object")?
                        .read()
                        .name()
                )
                .into());
            }

            let old_vec = arg_obj.as_vector_storage();
            let old_vec: Vec<Value<'gc>> = if let Some(old_vec) = old_vec {
                old_vec.iter().collect()
            } else {
                continue;
            };

            for val in old_vec {
                if let Ok(val_obj) = val.coerce_to_object(activation) {
                    if !val.is_of_type(activation, val_class)? {
                        let other_val_class = val_obj
                            .as_class()
                            .ok_or("TypeError: Tried to concat a bare object into a Vector")?;
                        return Err(format!(
                            "TypeError: Cannot coerce Vector value of type {:?} to type {:?}",
                            other_val_class.read().name(),
                            val_class
                                .as_class()
                                .ok_or("TypeError: Tried to concat into a bare object")?
                                .read()
                                .name()
                        )
                        .into());
                    }
                }

                let coerced_val = val.coerce_to_type(activation, val_class)?;
                new_vector_storage.push(coerced_val)?;
            }
        }

        return Ok(VectorObject::from_vector(new_vector_storage, activation)?.into());
    }

    Ok(Value::Undefined)
}

fn join_inner<'gc, 'a, 'ctxt, C>(
    activation: &mut Activation<'a, 'gc, 'ctxt>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
    mut conv: C,
) -> Result<Value<'gc>, Error>
where
    C: for<'b> FnMut(Value<'gc>, &'b mut Activation<'a, 'gc, 'ctxt>) -> Result<Value<'gc>, Error>,
{
    let mut separator = args.get(0).cloned().unwrap_or(Value::Undefined);
    if separator == Value::Undefined {
        separator = ",".into();
    }

    if let Some(this) = this {
        if let Some(vector) = this.as_vector_storage() {
            let string_separator = separator.coerce_to_string(activation)?;
            let mut accum = Vec::with_capacity(vector.length());

            for (_, item) in vector.iter().enumerate() {
                if matches!(item, Value::Undefined) || matches!(item, Value::Null) {
                    accum.push("".into());
                } else {
                    accum.push(
                        conv(item, activation)?
                            .coerce_to_string(activation)?
                            .to_string(),
                    );
                }
            }

            return Ok(AvmString::new(
                activation.context.gc_context,
                accum.join(&string_separator),
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.join`
pub fn join<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(activation, this, args, |v, _act| Ok(v))
}

/// Implements `Vector.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(activation, this, &[",".into()], |v, _act| Ok(v))
}

/// Implements `Vector.toLocaleString`
pub fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(activation, this, &[",".into()], |v, act| {
        if let Ok(o) = v.coerce_to_object(act) {
            let ls = o.get_property(o, &QName::new(Namespace::public(), "toLocaleString"), act)?;

            ls.coerce_to_object(act)?.call(Some(o), &[], act, None)
        } else {
            Ok(v)
        }
    })
}

/// Implements `Vector.every`
pub fn every<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let callback = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let receiver = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut iter = ArrayIter::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            let result = callback
                .call(
                    receiver,
                    &[item, i.into(), this.into()],
                    activation,
                    receiver.and_then(|r| r.proto()),
                )?
                .coerce_to_boolean();

            if !result {
                return Ok(false.into());
            }
        }

        return Ok(true.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.some`
pub fn some<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let callback = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let receiver = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut iter = ArrayIter::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            let result = callback
                .call(
                    receiver,
                    &[item, i.into(), this.into()],
                    activation,
                    receiver.and_then(|r| r.proto()),
                )?
                .coerce_to_boolean();

            if result {
                return Ok(true.into());
            }
        }

        return Ok(false.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.filter`
pub fn filter<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let callback = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let receiver = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();

        let value_type = this
            .instance_of()
            .unwrap()
            .as_class_object_really()
            .unwrap()
            .as_class_params()
            .ok_or("Cannot filter unparameterized vector")?
            .unwrap_or(activation.avm2().classes().object);
        let mut new_storage = VectorStorage::new(0, false, value_type, activation);
        let mut iter = ArrayIter::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            let result = callback
                .call(
                    receiver,
                    &[item.clone(), i.into(), this.into()],
                    activation,
                    receiver.and_then(|r| r.proto()),
                )?
                .coerce_to_boolean();

            if result {
                new_storage.push(item)?;
            }
        }

        return Ok(VectorObject::from_vector(new_storage, activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.forEach`
pub fn for_each<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let callback = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let receiver = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut iter = ArrayIter::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            callback.call(
                receiver,
                &[item, i.into(), this.into()],
                activation,
                receiver.and_then(|r| r.proto()),
            )?;
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.indexOf`
pub fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let search_for = args.get(0).cloned().unwrap_or(Value::Undefined);
        let from_index = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| 0.into())
            .coerce_to_i32(activation)?;

        let from_index = if from_index < 0 {
            let length = this
                .get_property(this, &QName::new(Namespace::public(), "length"), activation)?
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
    }

    Ok((-1).into())
}

/// Implements `Vector.lastIndexOf`
pub fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let search_for = args.get(0).cloned().unwrap_or(Value::Undefined);
        let from_index = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| i32::MAX.into())
            .coerce_to_i32(activation)?;

        let from_index = if from_index < 0 {
            let length = this
                .get_property(this, &QName::new(Namespace::public(), "length"), activation)?
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
    }

    Ok((-1).into())
}

/// Implements `Vector.map`
pub fn map<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let callback = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let receiver = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();

        let value_type = this
            .instance_of()
            .unwrap()
            .as_class_object_really()
            .unwrap()
            .as_class_params()
            .ok_or("Cannot filter unparameterized vector")?
            .unwrap_or(activation.avm2().classes().object);
        let mut new_storage = VectorStorage::new(0, false, value_type, activation);
        let mut iter = ArrayIter::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            let new_item = callback.call(
                receiver,
                &[item.clone(), i.into(), this.into()],
                activation,
                receiver.and_then(|r| r.proto()),
            )?;
            let coerced_item = new_item.coerce_to_type(activation, value_type)?;

            new_storage.push(coerced_item)?;
        }

        return Ok(VectorObject::from_vector(new_storage, activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.pop`
pub fn pop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            return vs.pop(activation);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.push`
pub fn push<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            let value_type = vs.value_type();

            for arg in args {
                let coerced_arg = arg.coerce_to_type(activation, value_type)?;

                vs.push(coerced_arg)?;
            }

            return Ok(vs.length().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.shift`
pub fn shift<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            return vs.shift(activation);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.unshift`
pub fn unshift<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            let value_type = vs.value_type();

            for arg in args.iter().rev() {
                let coerced_arg = arg.coerce_to_type(activation, value_type)?;

                vs.unshift(coerced_arg)?;
            }

            return Ok(vs.length().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.insertAt`
pub fn insert_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            let index = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;
            let value_type = vs.value_type();
            let value = args
                .get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_type(activation, value_type)?;

            vs.insert(index, value)?;
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.removeAt`
pub fn remove_at<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            let index = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?;

            return vs.remove(index);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.reverse`
pub fn reverse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            vs.reverse();

            return Ok(this.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.slice`
pub fn slice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
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
                    new_vs.push(value)?;
                }
            }

            let new_vector = VectorObject::from_vector(new_vs, activation)?;

            return Ok(new_vector.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.sort`
pub fn sort<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(vs) = this.as_vector_storage_mut(activation.context.gc_context) {
            let fn_or_options = args.get(0).cloned().unwrap_or(Value::Undefined);

            let (compare_fnc, options) = if fn_or_options
                .coerce_to_object(activation)
                .ok()
                .and_then(|o| o.as_executable())
                .is_some()
            {
                (
                    Some(fn_or_options.coerce_to_object(activation)?),
                    SortOptions::empty(),
                )
            } else {
                (
                    None,
                    SortOptions::from_bits_truncate(fn_or_options.coerce_to_u32(activation)? as u8),
                )
            };

            let compare = move |activation: &mut Activation<'_, 'gc, '_>, a, b| {
                if let Some(compare_fnc) = compare_fnc {
                    let order = compare_fnc
                        .call(Some(this), &[a, b], activation, None)?
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
            values.sort_unstable_by(|a, b| match compare(activation, a.clone(), b.clone()) {
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
    }

    Ok(Value::Undefined)
}

/// Implements `Vector.splice`
pub fn splice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
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
                to_coerce.push(value.coerce_to_type(activation, value_type)?);
            }

            let new_vs =
                VectorStorage::from_values(vs.splice(start..end, to_coerce)?, false, value_type);
            let new_vector = VectorObject::from_vector(new_vs, activation)?;

            return Ok(new_vector.into());
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Vector`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package(NS_VECTOR), "Vector"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Vector instance initializer>", mc),
        Method::from_builtin(class_init, "<Vector class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::GENERIC | ClassAttributes::FINAL);
    write.set_instance_allocator(vector_allocator);
    write.set_specialized_init(Method::from_builtin(
        specialized_class_init,
        "<Vector specialized class initializer>",
        mc,
    ));

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("length", Some(length), Some(set_length)),
        ("fixed", Some(fixed), Some(set_fixed)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

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
    write.define_as3_builtin_instance_methods(mc, AS3_INSTANCE_METHODS);

    class
}
