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

    Ok(ArrayObject::from_array(
        base_array,
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
                let item = item.map(Ok).unwrap_or_else(|| {
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
                });

                accum.push(item?.coerce_to_string(activation)?.to_string());
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

    class
}
