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
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};

/// Implements `Error`'s instance initializer.
fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this.and_then(|this| this.as_error_object()) {
        let message = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| AvmString::default().into());
        let id = args.get(1).cloned().unwrap_or_else(|| 0.into());

        this.set_property(&QName::dynamic_name("message").into(), message, activation)?;
        this.set_property(&QName::dynamic_name("id").into(), id, activation)?;
        this.set_property(
            &QName::dynamic_name("name").into(),
            "Error".into(),
            activation,
        )?;
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

fn get_stack_trace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if cfg!(not(feature = "avm_debug")) {
        return Ok(Value::Null);
    }

    if let Some(this) = this.and_then(|this| this.as_error_object()) {
        return Ok(this.display_full(activation)?.into());
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

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("getStackTrace", get_stack_trace)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
