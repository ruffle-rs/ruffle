//! `Error` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::error_allocator;
use crate::avm2::object::Object;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `Error`'s instance initializer.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        if let Some(message) = args.get(0) {
            let id = args
                .get(1)
                .unwrap_or(&0i32.into())
                .coerce_to_i32(activation)?;
            this.set_message(
                activation.context.gc_context,
                message.coerce_to_string(activation)?,
            );
            this.set_id(activation.context.gc_context, id);
        }
    }
    Ok(Value::Undefined)
}

/// Implements `Error`'s class initializer.
fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

fn error_id<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        return Ok(this.id().into());
    }
    Ok(Value::Undefined)
}

fn name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        return Ok(this.name().into());
    }
    Ok(Value::Undefined)
}

fn set_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        if let Some(name) = args.get(0) {
            this.set_name(
                activation.context.gc_context,
                name.coerce_to_string(activation)?,
            );
        }
    }
    Ok(Value::Undefined)
}

fn message<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        return Ok(this.message().into());
    }
    Ok(Value::Undefined)
}

fn set_message<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        if let Some(message) = args.get(0) {
            this.set_message(
                activation.context.gc_context,
                message.coerce_to_string(activation)?,
            );
        }
    }
    Ok(Value::Undefined)
}

/// Construct `Error`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "Error"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Error instance initializer>", mc),
        Method::from_builtin(class_init, "<Error class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(error_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("errorID", Some(error_id), None),
        ("name", Some(name), Some(set_name)),
        ("message", Some(message), Some(set_message)),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
