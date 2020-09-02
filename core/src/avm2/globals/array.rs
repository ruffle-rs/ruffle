//! Array class

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{ArrayObject, Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};
use std::cmp::min;
use std::mem::swap;

/// Implements `Array`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut array) = this.as_array_storage_mut(activation.context.gc_context) {
            if args.len() == 1 {
                if let Some(expected_len) = args
                    .get(0)
                    .and_then(|v| v.as_number(activation.context.gc_context).ok())
                {
                    array.set_length(expected_len as usize);

                    return Ok(Value::Undefined);
                }
            }

            for (i, arg) in args.iter().enumerate() {
                array.set(i, arg.clone());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Array.length`
pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(array) = this.as_array_storage() {
            return Ok(array.length().into());
        }
    }

    Ok(Value::Undefined)
}

/// Bundle an already-constructed `ArrayStorage` in an `Object`.
pub fn build_array<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    array: ArrayStorage<'gc>,
) -> Result<Value<'gc>, Error> {
    Ok(ArrayObject::from_array(
        array,
        activation
            .context
            .avm2
            .system_prototypes
            .as_ref()
            .map(|sp| sp.array)
            .unwrap(),
        activation.context.gc_context,
    )
    .into())
}

/// Implements `Array.concat`
#[allow(clippy::map_clone)] //You can't clone `Option<Ref<T>>` without it
pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let mut base_array = this
        .and_then(|this| this.as_array_storage().map(|a| a.clone()))
        .unwrap_or_else(|| ArrayStorage::new(0));

    for arg in args {
        if let Some(other_array) = arg.coerce_to_object(activation)?.as_array_storage() {
            base_array.append(&other_array);
        } else {
            base_array.push(arg.clone());
        }
    }

    build_array(activation, base_array)
}

/// Resolves array holes.
fn resolve_array_hole<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    i: usize,
    item: Option<Value<'gc>>,
) -> Result<Value<'gc>, Error> {
    item.map(Ok).unwrap_or_else(|| {
        this.proto()
            .map(|mut p| {
                p.get_property(
                    p,
                    &QName::new(
                        Namespace::public_namespace(),
                        AvmString::new(activation.context.gc_context, format!("{}", i)),
                    ),
                    activation,
                )
            })
            .unwrap_or(Ok(Value::Undefined))
    })
}

pub fn join_inner<'gc, 'a, 'ctxt, C>(
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
        if let Some(array) = this.as_array_storage() {
            let string_separator = separator.coerce_to_string(activation)?;
            let mut accum = Vec::new();

            for (i, item) in array.iter().enumerate() {
                let item = resolve_array_hole(activation, this, i, item)?;

                accum.push(
                    conv(item, activation)?
                        .coerce_to_string(activation)?
                        .to_string(),
                );
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

/// Implements `Array.join`
pub fn join<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(activation, this, args, |v, _act| Ok(v))
}

/// Implements `Array.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(activation, this, &[",".into()], |v, _act| Ok(v))
}

/// Implements `Array.toLocaleString`
pub fn to_locale_string<'gc>(
    act: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(act, this, &[",".into()], |v, activation| {
        let mut o = v.coerce_to_object(activation)?;

        let tls = o.get_property(
            o,
            &QName::new(Namespace::public_namespace(), "toLocaleString"),
            activation,
        )?;

        tls.coerce_to_object(activation)?
            .call(Some(o), &[], activation, o.proto())
    })
}

/// Implements `Array.valueOf`
pub fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join_inner(activation, this, &[",".into()], |v, _act| Ok(v))
}

/// An iterator that allows iterating over the contents of an array whilst also
/// executing user code.
///
/// Note that this does not actually implement `Iterator` as this struct needs
/// to share access to the activation with you. We can't claim your activation
/// and give it back in `next`, so we instead ask for it in `next`, which is
/// incompatible with the trait.
///
/// This technically works with Array-shaped, non-Array objects, since we
/// access arrays in this iterator the same way user code would. If it is
/// necessary to only work with Arrays, you must first check for array storage
/// before creating this iterator.
///
/// The primary purpose of `ArrayIterator` is to maintain lock safety in the
/// presence of arbitrary user code. It is legal for, say, a method callback to
/// mutate the array under iteration. Normally, holding an `Iterator` on the
/// array while this happens would cause a panic; this code exists to prevent
/// that.
struct ArrayIterator<'gc> {
    array_object: Object<'gc>,
    index: u32,
    length: u32,
}

impl<'gc> ArrayIterator<'gc> {
    /// Construct a new `ArrayIterator`.
    pub fn new(
        activation: &mut Activation<'_, 'gc, '_>,
        mut array_object: Object<'gc>,
    ) -> Result<Self, Error> {
        let length = array_object
            .get_property(
                array_object,
                &QName::new(Namespace::public_namespace(), "length"),
                activation,
            )?
            .coerce_to_u32(activation)?;

        Ok(Self {
            array_object,
            index: 0,
            length,
        })
    }

    /// Get the next item in the array.
    ///
    /// Since this isn't a real iterator, this comes pre-enumerated; it yields
    /// a pair of the index and then the value.
    fn next(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Result<(u32, Value<'gc>), Error>> {
        if self.index < self.length {
            let i = self.index;

            self.index += 1;

            Some(
                self.array_object
                    .get_property(
                        self.array_object,
                        &QName::new(
                            Namespace::public_namespace(),
                            AvmString::new(activation.context.gc_context, format!("{}", i)),
                        ),
                        activation,
                    )
                    .map(|val| (i, val)),
            )
        } else {
            None
        }
    }
}

/// Implements `Array.forEach`
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
        let reciever = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut iter = ArrayIterator::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            callback.call(
                reciever,
                &[item, i.into(), this.into()],
                activation,
                reciever.and_then(|r| r.proto()),
            )?;
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.map`
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
        let reciever = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut new_array = ArrayStorage::new(0);
        let mut iter = ArrayIterator::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;
            let new_item = callback.call(
                reciever,
                &[item, i.into(), this.into()],
                activation,
                reciever.and_then(|r| r.proto()),
            )?;

            new_array.push(new_item);
        }

        return build_array(activation, new_array);
    }

    Ok(Value::Undefined)
}

/// Implements `Array.filter`
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
        let reciever = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut new_array = ArrayStorage::new(0);
        let mut iter = ArrayIterator::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;
            let is_allowed = callback
                .call(
                    reciever,
                    &[item.clone(), i.into(), this.into()],
                    activation,
                    reciever.and_then(|r| r.proto()),
                )?
                .coerce_to_boolean();

            if is_allowed {
                new_array.push(item);
            }
        }

        return build_array(activation, new_array);
    }

    Ok(Value::Undefined)
}

/// Implements `Array.every`
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
        let reciever = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut is_every = true;
        let mut iter = ArrayIterator::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            is_every &= callback
                .call(
                    reciever,
                    &[item, i.into(), this.into()],
                    activation,
                    reciever.and_then(|r| r.proto()),
                )?
                .coerce_to_boolean();
        }

        return Ok(is_every.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Array.some`
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
        let reciever = args
            .get(1)
            .cloned()
            .unwrap_or(Value::Null)
            .coerce_to_object(activation)
            .ok();
        let mut is_some = false;
        let mut iter = ArrayIterator::new(activation, this)?;

        while let Some(r) = iter.next(activation) {
            let (i, item) = r?;

            is_some |= callback
                .call(
                    reciever,
                    &[item, i.into(), this.into()],
                    activation,
                    reciever.and_then(|r| r.proto()),
                )?
                .coerce_to_boolean();
        }

        return Ok(is_some.into());
    }

    Ok(Value::Undefined)
}

/// Implements `Array.indexOf`
pub fn index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(array) = this.as_array_storage() {
            let search_val = args.get(0).cloned().unwrap_or(Value::Undefined);
            let from = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| 0.into())
                .coerce_to_u32(activation)?;

            for (i, val) in array.iter().enumerate() {
                let val = resolve_array_hole(activation, this, i, val)?;
                if i >= from as usize && val == search_val {
                    return Ok(i.into());
                }
            }

            return Ok((-1).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.lastIndexOf`
pub fn last_index_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(array) = this.as_array_storage() {
            let search_val = args.get(0).cloned().unwrap_or(Value::Undefined);
            let from = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| i32::MAX.into())
                .coerce_to_u32(activation)?;

            for (i, val) in array.iter().enumerate().rev() {
                let val = resolve_array_hole(activation, this, i, val)?;
                if i <= from as usize && val == search_val {
                    return Ok(i.into());
                }
            }

            return Ok((-1).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.pop`
pub fn pop<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut array) = this.as_array_storage_mut(activation.context.gc_context) {
            return Ok(array.pop());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.push`
pub fn push<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut array) = this.as_array_storage_mut(activation.context.gc_context) {
            for arg in args {
                array.push(arg.clone())
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn reverse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut array) = this.as_array_storage_mut(activation.context.gc_context) {
            let mut last_non_hole_index = None;
            for (i, val) in array.iter().enumerate() {
                if val.is_some() {
                    last_non_hole_index = Some(i + 1);
                }
            }

            let mut new_array = ArrayStorage::new(0);

            for i in
                (0..last_non_hole_index.unwrap_or_else(|| array.length().saturating_sub(1))).rev()
            {
                if let Some(value) = array.get(i) {
                    new_array.push(value)
                } else {
                    new_array.push_hole()
                }
            }

            new_array.set_length(array.length());

            swap(&mut *array, &mut new_array);

            return Ok(this.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.shift`
pub fn shift<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut array) = this.as_array_storage_mut(activation.context.gc_context) {
            return Ok(array.shift());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.unshift`
pub fn unshift<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut array) = this.as_array_storage_mut(activation.context.gc_context) {
            for arg in args.iter().rev() {
                array.unshift(arg.clone())
            }
        }
    }

    Ok(Value::Undefined)
}

/// Resolve a possibly-negative array index to something guaranteed to be positive.
pub fn resolve_index<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    index: Value<'gc>,
    length: usize,
) -> Result<usize, Error> {
    let index = index.coerce_to_i32(activation)?;

    Ok(if index < 0 {
        (length as isize).saturating_add(index as isize) as usize
    } else {
        index as usize
    })
}

/// Implements `Array.slice`
pub fn slice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let array_length = this.as_array_storage().map(|a| a.length());

        if let Some(array_length) = array_length {
            let actual_start = resolve_index(
                activation,
                args.get(0).cloned().unwrap_or_else(|| 0.into()),
                array_length,
            )?;
            let actual_end = resolve_index(
                activation,
                args.get(1).cloned().unwrap_or_else(|| 0xFFFFFF.into()),
                array_length,
            )?;
            let mut new_array = ArrayStorage::new(0);
            for i in actual_start..actual_end {
                if i >= array_length {
                    break;
                }

                new_array.push(resolve_array_hole(
                    activation,
                    this,
                    i,
                    this.as_array_storage().unwrap().get(i),
                )?);
            }

            return build_array(activation, new_array);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Array.splice`
pub fn splice<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let array_length = this.as_array_storage().map(|a| a.length());

        if let Some(array_length) = array_length {
            if let Some(start) = args.get(0).cloned() {
                let actual_start = resolve_index(activation, start, array_length)?;
                let delete_count = args
                    .get(1)
                    .cloned()
                    .unwrap_or_else(|| array_length.into())
                    .coerce_to_i32(activation)?;

                let mut removed_array = ArrayStorage::new(0);
                if delete_count > 0 {
                    let actual_end = min(array_length, actual_start + delete_count as usize);
                    let args_slice = if args.len() > 2 {
                        args[2..].iter().cloned()
                    } else {
                        [].iter().cloned()
                    };

                    let contents = this
                        .as_array_storage()
                        .map(|a| a.iter().collect::<Vec<Option<Value<'gc>>>>())
                        .unwrap();
                    let mut resolved = Vec::new();

                    for (i, v) in contents.iter().enumerate() {
                        resolved.push(resolve_array_hole(activation, this, i, v.clone())?);
                    }

                    let removed = resolved
                        .splice(actual_start..actual_end, args_slice)
                        .collect::<Vec<Value<'gc>>>();
                    removed_array = ArrayStorage::from_args(&removed[..]);

                    let mut resolved_array = ArrayStorage::from_args(&resolved[..]);

                    if let Some(mut array) =
                        this.as_array_storage_mut(activation.context.gc_context)
                    {
                        swap(&mut *array, &mut resolved_array)
                    }
                }

                return build_array(activation, removed_array);
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Array`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package(""), "Array"),
        Some(QName::new(Namespace::public_namespace(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    class.write(mc).define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public_namespace(), "length"),
        Method::from_builtin(length),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "concat"),
        Method::from_builtin(concat),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::as3_namespace(), "join"),
        Method::from_builtin(join),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "toString"),
        Method::from_builtin(to_string),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "toLocaleString"),
        Method::from_builtin(to_locale_string),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "valueOf"),
        Method::from_builtin(value_of),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "forEach"),
        Method::from_builtin(for_each),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "map"),
        Method::from_builtin(map),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "filter"),
        Method::from_builtin(filter),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "every"),
        Method::from_builtin(every),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "some"),
        Method::from_builtin(some),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "indexOf"),
        Method::from_builtin(index_of),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "lastIndexOf"),
        Method::from_builtin(last_index_of),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "pop"),
        Method::from_builtin(pop),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "push"),
        Method::from_builtin(push),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "reverse"),
        Method::from_builtin(reverse),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "shift"),
        Method::from_builtin(shift),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "unshift"),
        Method::from_builtin(unshift),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "slice"),
        Method::from_builtin(slice),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "splice"),
        Method::from_builtin(splice),
    ));

    class
}
