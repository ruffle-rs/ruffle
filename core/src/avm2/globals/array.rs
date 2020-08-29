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

/// Implements `Array.join`
pub fn join<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
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

                accum.push(item.coerce_to_string(activation)?.to_string());
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

/// Implements `Array.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join(activation, this, &[",".into()])
}

/// Implements `Array.valueOf`
pub fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    join(activation, this, &[",".into()])
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

    class
}
