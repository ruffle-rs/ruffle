//! `uint` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::{make_error_1003, make_error_1004};
use crate::avm2::globals::number::print_with_radix;
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::{AvmString, Error, Multiname, QName};

/// Implements `uint`'s instance initializer.
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
                .coerce_to_u32(activation)?
                .into();
        }
    }

    Ok(Value::Undefined)
}

/// Implements `uint`'s native instance initializer.
fn super_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implements `uint`'s class initializer.
fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let uint_proto = this_class.prototype();

    uint_proto.set_string_property_local(
        "toExponential",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_exponential, "toExponential", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toFixed",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_fixed, "toFixed", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toPrecision",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_precision, "toPrecision", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toLocaleString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toLocaleString", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "toString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toString", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    uint_proto.set_string_property_local(
        "valueOf",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(value_of, "valueOf", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;

    uint_proto.set_local_property_is_enumerable(gc_context, "toExponential".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toFixed".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toPrecision".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toLocaleString".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
    uint_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(0))
        .coerce_to_u32(activation)?
        .into())
}

/// Implements `uint.toExponential`
use crate::avm2::globals::number::to_exponential;

/// Implements `uint.toFixed`
use crate::avm2::globals::number::to_fixed;

/// Implements `uint.toPrecision`
use crate::avm2::globals::number::to_precision;

/// Implements `uint.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let uint_proto = activation.avm2().classes().uint.prototype();
    if Object::ptr_eq(uint_proto, this) {
        return Ok("0".into());
    }

    let number = if let Some(this) = this.as_primitive() {
        match *this {
            Value::Integer(o) => o as f64,
            Value::Number(o) => o,
            _ => return Err(make_error_1004(activation, "uint.prototype.toString")),
        }
    } else {
        return Err(make_error_1004(activation, "uint.prototype.toString"));
    };

    let radix = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(10))
        .coerce_to_i32(activation)?;

    if radix < 2 || radix > 36 {
        return Err(make_error_1003(activation, radix));
    }

    Ok(print_with_radix(activation, number, radix as usize)?.into())
}

/// Implements `uint.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let uint_proto = activation.avm2().classes().uint.prototype();
    if Object::ptr_eq(uint_proto, this) {
        return Ok(0.into());
    }

    if let Some(this) = this.as_primitive() {
        match *this {
            Value::Integer(_) => Ok(*this),
            _ => Err(make_error_1004(activation, "uint.prototype.valueOf")),
        }
    } else {
        Err(make_error_1004(activation, "uint.prototype.valueOf"))
    }
}

/// Construct `uint`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "uint"),
        Some(activation.avm2().class_defs().object),
        Method::from_builtin_and_params(
            instance_init,
            "<uint instance initializer>",
            vec![ParamConfig {
                param_name: AvmString::new_utf8(activation.context.gc_context, "value"),
                param_type_name: Multiname::any(),
                default_value: Some(Value::Integer(0)),
            }],
            Multiname::any(),
            true,
            mc,
        ),
        Method::from_builtin(class_init, "<uint class initializer>", mc),
        activation.avm2().class_defs().class,
        mc,
    );

    class.set_attributes(mc, ClassAttributes::FINAL | ClassAttributes::SEALED);
    class.set_instance_allocator(mc, primitive_allocator);
    class.set_super_init(
        mc,
        Method::from_builtin(super_init, "<uint native instance initializer>", mc),
    );
    class.set_call_handler(
        mc,
        Method::from_builtin(call_handler, "<uint call handler>", mc),
    );

    const CLASS_CONSTANTS_UINT: &[(&str, u32)] =
        &[("MAX_VALUE", u32::MAX), ("MIN_VALUE", u32::MIN)];
    class.define_constant_uint_class_traits(
        activation.avm2().public_namespace_base_version,
        CLASS_CONSTANTS_UINT,
        activation,
    );

    const CLASS_CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    class.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CLASS_CONSTANTS_INT,
        activation,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("toExponential", to_exponential),
        ("toFixed", to_fixed),
        ("toPrecision", to_precision),
        ("toString", to_string),
        ("valueOf", value_of),
    ];
    class.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}
