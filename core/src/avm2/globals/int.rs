//! `int` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::globals::number::{print_with_precision, print_with_radix};
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::QName;
use crate::avm2::{AvmString, Error};
use gc_arena::GcCell;

/// Implements `int`'s instance initializer.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut prim) = this.as_primitive_mut(activation.context.gc_context) {
        if matches!(*prim, Value::Undefined | Value::Null) {
            *prim = args
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_i32(activation)?
                .into();
        }
    }

    Ok(Value::Undefined)
}

/// Implements `int`'s native instance initializer.
fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implements `int`'s class initializer.
fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let int_proto = this_class.prototype();

    int_proto.set_string_property_local(
        "toExponential",
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
    int_proto.set_string_property_local(
        "toFixed",
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
    int_proto.set_string_property_local(
        "toPrecision",
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
    int_proto.set_string_property_local(
        "toString",
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
    int_proto.set_string_property_local(
        "valueOf",
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

    int_proto.set_local_property_is_enumerable(gc_context, "toExponential".into(), false);
    int_proto.set_local_property_is_enumerable(gc_context, "toFixed".into(), false);
    int_proto.set_local_property_is_enumerable(gc_context, "toPrecision".into(), false);
    int_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
    int_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

    Ok(Value::Undefined)
}

/// Implements `int.toExponential`
fn to_exponential<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
                format!("{number:.digits$e}")
                    .replace('e', "e+")
                    .replace("e+-", "e-")
                    .replace("e+0", ""),
            )
            .into());
        }
    }

    Err("int.prototype.toExponential has been called on an incompatible object".into())
}

/// Implements `int.toFixed`
fn to_fixed<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Err("int.prototype.toFixed has been called on an incompatible object".into())
}

/// Implements `int.toPrecision`
fn to_precision<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Err("int.prototype.toPrecision has been called on an incompatible object".into())
}

/// Implements `int.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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

    Err("int.prototype.toString has been called on an incompatible object".into())
}

/// Implements `int.valueOf`
fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_primitive() {
        return Ok(*this);
    }

    Ok(Value::Undefined)
}

/// Construct `int`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace, "int"),
        Some(Multiname::new(activation.avm2().public_namespace, "Object")),
        Method::from_builtin_and_params(
            instance_init,
            "<int instance initializer>",
            vec![ParamConfig {
                param_name: AvmString::new_utf8(activation.context.gc_context, "value"),
                param_type_name: Multiname::any(activation.context.gc_context),
                default_value: Some(Value::Integer(0)),
            }],
            false,
            mc,
        ),
        Method::from_builtin(class_init, "<int class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);
    write.set_instance_allocator(primitive_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<int native instance initializer>",
        mc,
    ));

    // 'length' is a weird undocumented constant in int.
    // We need to define it, since it shows up in 'describeType'
    const CLASS_CONSTANTS: &[(&str, i32)] = &[
        ("MAX_VALUE", i32::MAX),
        ("MIN_VALUE", i32::MIN),
        ("length", 1),
    ];
    write.define_constant_int_class_traits(
        activation.avm2().public_namespace,
        CLASS_CONSTANTS,
        activation,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
        ("toString", to_string),
        ("valueOf", value_of),
    ];
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    class
}
