//! `Boolean` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;
use gc_arena::GcCell;

/// Implements `Boolean`'s instance initializer.
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
                .unwrap_or(Value::Bool(false))
                .coerce_to_boolean()
                .into();
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Boolean`'s native instance initializer.
fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implements `Boolean`'s class initializer.
fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let boolean_proto = this_class.prototype();

    boolean_proto.set_string_property_local(
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
    boolean_proto.set_string_property_local(
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
    boolean_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
    boolean_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(args
        .get(0)
        .cloned()
        .unwrap_or(Value::Bool(false))
        .coerce_to_boolean()
        .into())
}

/// Implements `Boolean.toString`
fn to_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_primitive() {
        match *this {
            Value::Bool(true) => return Ok("true".into()),
            Value::Bool(false) => return Ok("false".into()),
            _ => {}
        };
    }

    Ok("false".into())
}

/// Implements `Boolean.valueOf`
fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_primitive() {
        return Ok(*this);
    }

    Ok(false.into())
}

/// Construct `Boolean`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "Boolean"),
        Some(Multiname::new(
            activation.avm2().public_namespace_base_version,
            "Object",
        )),
        Method::from_builtin(instance_init, "<Boolean instance initializer>", mc),
        Method::from_builtin(class_init, "<Boolean class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);
    write.set_instance_allocator(primitive_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<Boolean native instance initializer>",
        mc,
    ));
    write.set_call_handler(Method::from_builtin(
        call_handler,
        "<Boolean call handler>",
        mc,
    ));

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("toString", to_string), ("valueOf", value_of)];
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    const CONSTANTS_INT: &[(&str, i32)] = &[("length", 1)];
    write.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CONSTANTS_INT,
        activation,
    );

    class
}
