//! `Vector` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::globals::array::ArrayIter;
use crate::avm2::globals::NS_VECTOR;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{vector_allocator, Object, TObject, VectorObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};
use std::cmp::max;

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
                .unwrap_or(Value::Bool(false))
                .coerce_to_boolean();

            vector.resize(length)?;
            vector.set_is_fixed(is_fixed);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Vector`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

            vector.resize(new_length)?;
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
            .as_class_object()
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
            let old_vec: Vec<Option<Value<'gc>>> = if let Some(old_vec) = old_vec {
                old_vec.iter().collect()
            } else {
                continue;
            };

            for val in old_vec {
                if let Some(val) = val {
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
                    new_vector_storage.push(Some(coerced_val))?;
                } else {
                    new_vector_storage.push(None)?;
                }
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
                if matches!(item, Some(Value::Undefined))
                    || matches!(item, Some(Value::Null))
                    || item.is_none()
                {
                    accum.push("".into());
                } else {
                    accum.push(
                        conv(item.unwrap(), activation)?
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
            .as_class_object()
            .and_then(|c| c.as_class_params().and_then(|p| p.get(0).copied()))
            .ok_or("Cannot filter unparameterized vector")?;
        let mut new_storage = VectorStorage::new(0, false, value_type);
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
                new_storage.push(Some(item))?;
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
    if let Some(mut this) = this {
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
    if let Some(mut this) = this {
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
            .as_class_object()
            .and_then(|c| c.as_class_params().and_then(|p| p.get(0).copied()))
            .ok_or("Cannot filter unparameterized vector")?;
        let mut new_storage = VectorStorage::new(0, false, value_type);
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

            new_storage.push(Some(coerced_item))?;
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

                vs.push(Some(coerced_arg))?;
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

                vs.unshift(Some(coerced_arg))?;
            }

            return Ok(vs.length().into());
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
        Method::from_builtin(class_init, "<Vector instance initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::GENERIC | ClassAttributes::FINAL);
    write.set_instance_allocator(vector_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("length", Some(length), Some(set_length)),
        ("fixed", Some(fixed), Some(set_fixed)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("concat", concat),
        ("join", join),
        ("toString", to_string),
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
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
