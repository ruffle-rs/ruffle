//! `int` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::{make_error_1003, make_error_1004};
use crate::avm2::globals::number::print_with_radix;
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::{AvmString, Error, Multiname, QName};

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
        "toLocaleString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toLocaleString", gc_context),
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
    int_proto.set_local_property_is_enumerable(gc_context, "toLocaleString".into(), false);
    int_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
    int_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

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
        .coerce_to_i32(activation)?
        .into())
}

/// Implements `int.toExponential`
use crate::avm2::globals::number::to_exponential;

/// Implements `int.toFixed`
use crate::avm2::globals::number::to_fixed;

/// Implements `int.toPrecision`
use crate::avm2::globals::number::to_precision;

/// Implements `int.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let int_proto = activation.avm2().classes().int.prototype();
    if Object::ptr_eq(int_proto, this) {
        return Ok("0".into());
    }

    let number = if let Some(this) = this.as_primitive() {
        match *this {
            Value::Integer(o) => o,
            _ => return Err(make_error_1004(activation, "int.prototype.toString")),
        }
    } else {
        return Err(make_error_1004(activation, "int.prototype.toString"));
    };

    let radix = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Integer(10))
        .coerce_to_i32(activation)?;

    if radix < 2 || radix > 36 {
        return Err(make_error_1003(activation, radix));
    }

    Ok(print_with_radix(activation, number as f64, radix as usize)?.into())
}

/// Implements `int.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let int_proto = activation.avm2().classes().int.prototype();
    if Object::ptr_eq(int_proto, this) {
        return Ok(0.into());
    }

    if let Some(this) = this.as_primitive() {
        match *this {
            Value::Integer(_) => Ok(*this),
            _ => Err(make_error_1004(activation, "int.prototype.valueOf")),
        }
    } else {
        Err(make_error_1004(activation, "int.prototype.valueOf"))
    }
}

/// Construct `int`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "int"),
        Some(activation.avm2().classes().object.inner_class_definition()),
        Method::from_builtin_and_params(
            instance_init,
            "<int instance initializer>",
            vec![ParamConfig {
                param_name: AvmString::new_utf8(activation.context.gc_context, "value"),
                param_type_name: Multiname::any(activation.context.gc_context),
                default_value: Some(Value::Integer(0)),
            }],
            Multiname::any(activation.context.gc_context),
            true,
            mc,
        ),
        Method::from_builtin(class_init, "<int class initializer>", mc),
        activation.avm2().classes().class.inner_class_definition(),
        mc,
    );

    class.set_attributes(mc, ClassAttributes::FINAL | ClassAttributes::SEALED);
    class.set_instance_allocator(mc, primitive_allocator);
    class.set_native_instance_init(
        mc,
        Method::from_builtin(
            native_instance_init,
            "<int native instance initializer>",
            mc,
        ),
    );
    class.set_call_handler(
        mc,
        Method::from_builtin(call_handler, "<int call handler>", mc),
    );

    // 'length' is a weird undocumented constant in int.
    // We need to define it, since it shows up in 'describeType'
    const CLASS_CONSTANTS: &[(&str, i32)] = &[
        ("MAX_VALUE", i32::MAX),
        ("MIN_VALUE", i32::MIN),
        ("length", 1),
    ];
    class.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
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
