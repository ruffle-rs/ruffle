//! `uint` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::globals::number::{print_with_precision, print_with_radix};
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2::{AvmString, Error};
use gc_arena::{GcCell, MutationContext};

/// Implements `uint`'s instance initializer.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(mut prim) = this.as_primitive_mut(activation.context.gc_context) {
            if matches!(*prim, Value::Undefined | Value::Null) {
                *prim = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Undefined)
                    .coerce_to_u32(activation)?
                    .into();
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `uint`'s native instance initializer.
fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?;
    }

    Ok(Value::Undefined)
}

/// Implements `uint`'s class initializer.
fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let scope = activation.create_scopechain();
        let gc_context = activation.context.gc_context;
        let this_class = this.as_class_object().unwrap();
        let uint_proto = this_class.prototype();

        uint_proto.set_property_local(
            &Multiname::public("toExponential"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_exponential, "toExponential", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        uint_proto.set_property_local(
            &Multiname::public("toFixed"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_fixed, "toFixed", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        uint_proto.set_property_local(
            &Multiname::public("toPrecision"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_precision, "toPrecision", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        uint_proto.set_property_local(
            &Multiname::public("toString"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_string, "toString", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        uint_proto.set_property_local(
            &Multiname::public("valueOf"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(value_of, "valueOf", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        uint_proto.set_local_property_is_enumerable(gc_context, "toExponential".into(), false);
        uint_proto.set_local_property_is_enumerable(gc_context, "toFixed".into(), false);
        uint_proto.set_local_property_is_enumerable(gc_context, "toPrecision".into(), false);
        uint_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
        uint_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);
    }

    Ok(Value::Undefined)
}

/// Implements `uint.toExponential`
fn to_exponential<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Integer(number) = *this {
                let digits = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Integer(0))
                    .coerce_to_u32(activation)? as usize;

                if digits > 20 {
                    return Err("toExponential can only print with 0 through 20 digits.".into());
                }

                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!("{0:.1$e}", number, digits)
                        .replace('e', "e+")
                        .replace("e+-", "e-")
                        .replace("e+0", ""),
                )
                .into());
            }
        }
    }

    Err("uint.prototype.toExponential has been called on an incompatible object".into())
}

/// Implements `uint.toFixed`
fn to_fixed<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Integer(number) = *this {
                let digits = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Integer(0))
                    .coerce_to_u32(activation)? as usize;

                if digits > 20 {
                    return Err("toFixed can only print with 0 through 20 digits.".into());
                }

                return Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!("{0:.1$}", number as f64, digits),
                )
                .into());
            }
        }
    }

    Err("uint.prototype.toFixed has been called on an incompatible object".into())
}

/// Implements `uint.toPrecision`
fn to_precision<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Integer(number) = *this {
                let wanted_digits = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Integer(0))
                    .coerce_to_u32(activation)? as usize;

                if wanted_digits < 1 || wanted_digits > 21 {
                    return Err("toPrecision can only print with 1 through 21 digits.".into());
                }

                return Ok(print_with_precision(activation, number as f64, wanted_digits)?.into());
            }
        }
    }

    Err("uint.prototype.toPrecision has been called on an incompatible object".into())
}

/// Implements `uint.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            if let Value::Integer(number) = *this {
                let radix = args
                    .get(0)
                    .cloned()
                    .unwrap_or(Value::Integer(10))
                    .coerce_to_u32(activation)? as usize;

                if radix < 2 || radix > 36 {
                    return Err("toString can only print in bases 2 thru 36.".into());
                }

                return Ok(print_with_radix(activation, number as f64, radix)?.into());
            }
        }
    }

    Err("uint.prototype.toString has been called on an incompatible object".into())
}

/// Implements `uint.valueOf`
fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            return Ok(*this);
        }
    }

    Ok(Value::Undefined)
}

/// Construct `uint`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "uint"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<uint instance initializer>", mc),
        Method::from_builtin(class_init, "<uint class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(primitive_allocator);
    write.set_native_instance_init(Method::from_builtin_and_params(
        native_instance_init,
        "<uint native instance initializer>",
        vec![ParamConfig::of_type(
            "num",
            QName::new(Namespace::public(), "Object").into(),
        )],
        false,
        mc,
    ));

    const CLASS_CONSTANTS: &[(&str, u32)] = &[("MAX_VALUE", u32::MAX), ("MIN_VALUE", u32::MIN)];
    write.define_public_constant_uint_class_traits(CLASS_CONSTANTS);

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
        ("toString", to_string),
        ("valueOf", value_of),
    ];
    write.define_as3_builtin_instance_methods(mc, AS3_INSTANCE_METHODS);

    class
}
