//! Array class

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::object::{ArrayObject, Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use enumset::{EnumSet, EnumSetType};
use gc_arena::{GcCell, MutationContext};
use std::cmp::{min, Ordering};
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
                    if expected_len < 0.0 || expected_len.is_nan() {
                        return Err("Length must be a positive integer".into());
                    }

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
                        AvmString::new(activation.context.gc_context, i.to_string()),
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
            let mut accum = Vec::with_capacity(array.length());

            for (i, item) in array.iter().enumerate() {
                let item = resolve_array_hole(activation, this, i, item)?;

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
/// The primary purpose of `ArrayIter` is to maintain lock safety in the
/// presence of arbitrary user code. It is legal for, say, a method callback to
/// mutate the array under iteration. Normally, holding an `Iterator` on the
/// array while this happens would cause a panic; this code exists to prevent
/// that.
struct ArrayIter<'gc> {
    array_object: Object<'gc>,
    index: u32,
    length: u32,
}

impl<'gc> ArrayIter<'gc> {
    /// Construct a new `ArrayIter`.
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
                            AvmString::new(activation.context.gc_context, i.to_string()),
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
        let mut iter = ArrayIter::new(activation, this)?;

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
        let mut iter = ArrayIter::new(activation, this)?;

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
        let mut iter = ArrayIter::new(activation, this)?;

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
        let mut iter = ArrayIter::new(activation, this)?;

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
        let mut iter = ArrayIter::new(activation, this)?;

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

/// The array options that a given sort operation may use.
///
/// These are provided as a number by the VM and converted into an enumset.
#[derive(EnumSetType)]
enum SortOptions {
    /// Request case-insensitive string value sort.
    CaseInsensitive,

    /// Reverse the order of sorting.
    Descending,

    /// Reject sorting on arrays with multiple equivalent values.
    UniqueSort,

    /// Yield a list of indicies rather than sorting the array in-place.
    ReturnIndexedArray,

    /// Request numeric value sort.
    Numeric,
}

/// Identity closure shim which exists purely to decorate closure types with
/// the HRTB necessary to accept an activation.
fn constrain<'a, 'gc, 'ctxt, F>(f: F) -> F
where
    F: FnMut(&mut Activation<'a, 'gc, 'ctxt>, Value<'gc>, Value<'gc>) -> Result<Ordering, Error>,
{
    f
}

/// Sort array storage.
///
/// This function expects it's values to have been pre-enumerated and
/// pre-resolved. They will be sorted in-place. It is the caller's
/// responsibility to place the resulting half of the sorted array wherever.
///
/// This function will reverse the sort order if `Descending` sort is requested.
///
/// This function will return `false` in the event that the `UniqueSort`
/// constraint has been violated (`sort_func` returned `Ordering::Equal`). In
/// this case, you should cancel the in-place sorting operation and return 0 to
/// the caller. In the event that this function yields a runtime error, the
/// contents of the `values` array will be sorted in a random order.
fn sort_inner<'a, 'gc, 'ctxt, C>(
    activation: &mut Activation<'a, 'gc, 'ctxt>,
    values: &mut [(usize, Value<'gc>)],
    options: EnumSet<SortOptions>,
    mut sort_func: C,
) -> Result<bool, Error>
where
    C: FnMut(&mut Activation<'a, 'gc, 'ctxt>, Value<'gc>, Value<'gc>) -> Result<Ordering, Error>,
{
    let mut unique_sort_satisfied = true;
    let mut error_signal = Ok(());

    values.sort_unstable_by(|(_a_index, a), (_b_index, b)| {
        let unresolved_a = a.clone();
        let unresolved_b = b.clone();

        if matches!(unresolved_a, Value::Undefined) && matches!(unresolved_b, Value::Undefined) {
            unique_sort_satisfied = false;
            return Ordering::Equal;
        } else if matches!(unresolved_a, Value::Undefined) {
            return Ordering::Greater;
        } else if matches!(unresolved_b, Value::Undefined) {
            return Ordering::Less;
        }

        match sort_func(activation, a.clone(), b.clone()) {
            Ok(Ordering::Equal) => {
                unique_sort_satisfied = false;
                Ordering::Equal
            }
            Ok(v) if options.contains(SortOptions::Descending) => v.reverse(),
            Ok(v) => v,
            Err(e) => {
                error_signal = Err(e);
                Ordering::Less
            }
        }
    });

    error_signal?;

    Ok(!options.contains(SortOptions::UniqueSort) || unique_sort_satisfied)
}

fn compare_string_case_sensitive<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    a: Value<'gc>,
    b: Value<'gc>,
) -> Result<Ordering, Error> {
    let string_a = a.coerce_to_string(activation)?;
    let string_b = b.coerce_to_string(activation)?;

    Ok(string_a.cmp(&string_b))
}

fn compare_string_case_insensitive<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    a: Value<'gc>,
    b: Value<'gc>,
) -> Result<Ordering, Error> {
    let string_a = a.coerce_to_string(activation)?.to_lowercase();
    let string_b = b.coerce_to_string(activation)?.to_lowercase();

    Ok(string_a.cmp(&string_b))
}

fn compare_numeric<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    a: Value<'gc>,
    b: Value<'gc>,
) -> Result<Ordering, Error> {
    let num_a = a.coerce_to_number(activation)?;
    let num_b = b.coerce_to_number(activation)?;

    if num_a.is_nan() && num_b.is_nan() {
        Ok(Ordering::Equal)
    } else if num_a.is_nan() {
        Ok(Ordering::Greater)
    } else if num_b.is_nan() {
        Ok(Ordering::Less)
    } else {
        Ok(num_a.partial_cmp(&num_b).unwrap())
    }
}

/// Take a sorted set of values and produce the result requested by the caller.
fn sort_postprocess<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    options: EnumSet<SortOptions>,
    unique_satisfied: bool,
    values: Vec<(usize, Value<'gc>)>,
) -> Result<Value<'gc>, Error> {
    if unique_satisfied {
        if options.contains(SortOptions::ReturnIndexedArray) {
            return build_array(
                activation,
                ArrayStorage::from_storage(
                    values
                        .iter()
                        .map(|(i, _v)| Some(i.clone().into()))
                        .collect(),
                ),
            );
        } else {
            if let Some(mut old_array) = this.as_array_storage_mut(activation.context.gc_context) {
                let mut new_vec = Vec::new();

                for (src, v) in values.iter() {
                    if old_array.get(*src).is_none() && !matches!(v, Value::Undefined) {
                        new_vec.push(Some(v.clone()));
                    } else {
                        new_vec.push(old_array.get(*src).clone());
                    }
                }

                let mut new_array = ArrayStorage::from_storage(new_vec);

                swap(&mut *old_array, &mut new_array);
            }

            return Ok(this.into());
        }
    }

    Ok(0.into())
}

/// Given a value, extract it's array values.
///
/// If the value is not an array, this function yields `None`.
fn extract_array_values<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<Option<Vec<Value<'gc>>>, Error> {
    let object = value.coerce_to_object(activation).ok();
    let holey_vec = if let Some(object) = object {
        if let Some(field_array) = object.as_array_storage() {
            let mut array = Vec::new();

            for v in field_array.iter() {
                array.push(v);
            }

            array
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    let mut unholey_vec = Vec::new();
    for (i, v) in holey_vec.iter().enumerate() {
        unholey_vec.push(resolve_array_hole(
            activation,
            object.unwrap(),
            i,
            v.clone(),
        )?);
    }

    Ok(Some(unholey_vec))
}

/// Impl `Array.sort`
pub fn sort<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let (compare_fnc, options) = if args.len() > 1 {
            (
                Some(
                    args.get(0)
                        .cloned()
                        .unwrap_or(Value::Undefined)
                        .coerce_to_object(activation)?,
                ),
                args.get(1)
                    .cloned()
                    .unwrap_or_else(|| 0.into())
                    .coerce_to_enumset(activation)?,
            )
        } else {
            (
                None,
                args.get(0)
                    .cloned()
                    .unwrap_or_else(|| 0.into())
                    .coerce_to_enumset(activation)?,
            )
        };

        let mut values = if let Some(values) = extract_array_values(activation, this.into())? {
            values
                .iter()
                .enumerate()
                .map(|(i, v)| (i, v.clone()))
                .collect::<Vec<(usize, Value<'gc>)>>()
        } else {
            return Ok(0.into());
        };

        let unique_satisfied = if let Some(v) = compare_fnc {
            sort_inner(
                activation,
                &mut values,
                options,
                constrain(|activation, a, b| {
                    let order = v
                        .call(None, &[a, b], activation, None)?
                        .coerce_to_number(activation)?;

                    if order > 0.0 {
                        Ok(Ordering::Greater)
                    } else if order < 0.0 {
                        Ok(Ordering::Less)
                    } else {
                        Ok(Ordering::Equal)
                    }
                }),
            )?
        } else if options.contains(SortOptions::Numeric) {
            sort_inner(activation, &mut values, options, compare_numeric)?
        } else if options.contains(SortOptions::CaseInsensitive) {
            sort_inner(
                activation,
                &mut values,
                options,
                compare_string_case_insensitive,
            )?
        } else {
            sort_inner(
                activation,
                &mut values,
                options,
                compare_string_case_sensitive,
            )?
        };

        return sort_postprocess(activation, this, options, unique_satisfied, values);
    }

    Ok(0.into())
}

/// Given a value, extract it's array values.
///
/// If the value is not an array, it will be returned as if it was present in a
/// one-element array containing itself. This is intended for use with parsing
/// parameters which are optionally arrays.
fn extract_maybe_array_values<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<Vec<Value<'gc>>, Error> {
    Ok(extract_array_values(activation, value.clone())?.unwrap_or_else(|| vec![value]))
}

/// Given a value, extract it's array values and coerce them to strings.
///
/// If the value is not an array, it will be returned as if it was present in a
/// one-element array containing itself. This is intended for use with parsing
/// parameters which are optionally arrays. The returned value will still be
/// coerced into a string in this case.
fn extract_maybe_array_strings<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<Vec<AvmString<'gc>>, Error> {
    let mut out = Vec::new();

    for value in extract_maybe_array_values(activation, value)? {
        out.push(value.coerce_to_string(activation)?);
    }

    Ok(out)
}

/// Given a value, extract it's array values and coerce them to enumsets.
///
/// If the value is not an array, it will be returned as if it was present in a
/// one-element array containing itself. This is intended for use with parsing
/// parameters which are optionally arrays. The returned value will still be
/// coerced into a string in this case.
fn extract_maybe_array_enumsets<'gc, E>(
    activation: &mut Activation<'_, 'gc, '_>,
    value: Value<'gc>,
) -> Result<Vec<EnumSet<E>>, Error>
where
    E: EnumSetType,
{
    let mut out = Vec::new();

    for value in extract_maybe_array_values(activation, value)? {
        out.push(value.coerce_to_enumset(activation)?);
    }

    Ok(out)
}

/// Impl `Array.sortOn`
pub fn sort_on<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(field_names_value) = args.get(0).cloned() {
            let field_names = extract_maybe_array_strings(activation, field_names_value)?;
            let mut options = extract_maybe_array_enumsets(
                activation,
                args.get(1).cloned().unwrap_or_else(|| 0.into()),
            )?;

            let first_option = options
                .get(0)
                .cloned()
                .unwrap_or_else(EnumSet::empty)
                .intersection(SortOptions::UniqueSort | SortOptions::ReturnIndexedArray);
            let mut values = if let Some(values) = extract_array_values(activation, this.into())? {
                values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, v.clone()))
                    .collect::<Vec<(usize, Value<'gc>)>>()
            } else {
                return Ok(0.into());
            };

            if options.len() < field_names.len() {
                options.resize(
                    field_names.len(),
                    options.last().cloned().unwrap_or_else(EnumSet::empty),
                );
            }

            let unique_satisfied = sort_inner(
                activation,
                &mut values,
                first_option,
                constrain(|activation, a, b| {
                    for (field_name, options) in field_names.iter().zip(options.iter()) {
                        let mut a_object = a.coerce_to_object(activation)?;
                        let a_field = a_object.get_property(
                            a_object,
                            &QName::new(Namespace::public_namespace(), *field_name),
                            activation,
                        )?;

                        let mut b_object = b.coerce_to_object(activation)?;
                        let b_field = b_object.get_property(
                            b_object,
                            &QName::new(Namespace::public_namespace(), *field_name),
                            activation,
                        )?;

                        let ord = if options.contains(SortOptions::Numeric) {
                            compare_numeric(activation, a_field, b_field)?
                        } else if options.contains(SortOptions::CaseInsensitive) {
                            compare_string_case_insensitive(activation, a_field, b_field)?
                        } else {
                            compare_string_case_sensitive(activation, a_field, b_field)?
                        };

                        if matches!(ord, Ordering::Equal) {
                            continue;
                        }

                        if options.contains(SortOptions::Descending) {
                            return Ok(ord.reverse());
                        } else {
                            return Ok(ord);
                        }
                    }

                    Ok(Ordering::Equal)
                }),
            )?;

            return sort_postprocess(activation, this, first_option, unique_satisfied, values);
        }
    }

    Ok(0.into())
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

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "sort"),
        Method::from_builtin(sort),
    ));

    class.write(mc).define_instance_trait(Trait::from_method(
        QName::new(Namespace::public_namespace(), "sortOn"),
        Method::from_builtin(sort_on),
    ));

    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public_namespace(), "CASEINSENSITIVE"),
        Multiname::from(QName::new(Namespace::public_namespace(), "uint")),
        Some(EnumSet::from(SortOptions::CaseInsensitive).as_u32().into()),
    ));

    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public_namespace(), "DESCENDING"),
        Multiname::from(QName::new(Namespace::public_namespace(), "uint")),
        Some(EnumSet::from(SortOptions::Descending).as_u32().into()),
    ));

    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public_namespace(), "NUMERIC"),
        Multiname::from(QName::new(Namespace::public_namespace(), "uint")),
        Some(EnumSet::from(SortOptions::Numeric).as_u32().into()),
    ));

    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public_namespace(), "RETURNINDEXEDARRAY"),
        Multiname::from(QName::new(Namespace::public_namespace(), "uint")),
        Some(
            EnumSet::from(SortOptions::ReturnIndexedArray)
                .as_u32()
                .into(),
        ),
    ));

    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public_namespace(), "UNIQUESORT"),
        Multiname::from(QName::new(Namespace::public_namespace(), "uint")),
        Some(EnumSet::from(SortOptions::UniqueSort).as_u32().into()),
    ));

    class
}
