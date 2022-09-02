//! `flash.external.ExternalInterface` builtin/prototype

use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::{Activation, Error, Namespace, Object, QName, Value};
use crate::external::{Callback, Value as ExternalValue};
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if args.is_empty() {
        return Ok(Value::Null);
    }

    let name = args.get(0).unwrap().coerce_to_string(activation)?;
    if let Some(method) = activation
        .context
        .external_interface
        .get_method_for(&name.to_utf8_lossy())
    {
        let mut external_args = Vec::with_capacity(args.len() - 1);
        for arg in &args[1..] {
            external_args.push(ExternalValue::from_avm2(arg.to_owned())?);
        }
        Ok(method
            .call(&mut activation.context, &external_args)
            .into_avm2(activation))
    } else {
        Ok(Value::Null)
    }
}

pub fn available<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(activation.context.external_interface.available().into())
}

pub fn add_callback<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let name = args.get(0).unwrap().coerce_to_string(activation)?;
    let method = args.get(1).unwrap();

    if let Value::Object(method) = method {
        activation
            .context
            .external_interface
            .add_callback(name.to_string(), Callback::Avm2 { method: *method });
    }
    Ok(Value::Undefined)
}

/// Construct `ExternalInterface`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.external"), "ExternalInterface"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(
            instance_init,
            "<ExternalInterface instance initializer>",
            mc,
        ),
        Method::from_builtin(class_init, "<ExternalInterface class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] =
        &[("call", call), ("addCallback", add_callback)];

    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("available", Some(available), None)];

    write.define_public_builtin_class_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    class
}
