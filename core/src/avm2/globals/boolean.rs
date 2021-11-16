//! `Boolean` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{primitive_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `Boolean`'s instance initializer.
pub fn instance_init<'gc>(
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
                    .unwrap_or(Value::Bool(false))
                    .coerce_to_boolean()
                    .into();
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `Boolean`'s native instance initializer.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, args)?;
    }

    Ok(Value::Undefined)
}

/// Implements `Boolean`'s class initializer.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut boolean_proto = this
            .get_property(this, &QName::dynamic_name("prototype").into(), activation)?
            .coerce_to_object(activation)?;
        let scope = activation.create_scopechain();
        let gc_context = activation.context.gc_context;
        let this_class = this.as_class_object().unwrap();

        boolean_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toString"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_string, "toString", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
        boolean_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "valueOf"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(value_of, "valueOf", gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `Boolean.toString`
pub fn to_string<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            match this.clone() {
                Value::Bool(true) => return Ok("true".into()),
                Value::Bool(false) => return Ok("false".into()),
                _ => {}
            };
        }
    }

    Err("Boolean.prototype.toString has been called on an incompatible object".into())
}

/// Implements `Boolean.valueOf`
pub fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(this) = this.as_primitive() {
            return Ok(this.clone());
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Boolean`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "Boolean"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Boolean instance initializer>", mc),
        Method::from_builtin(class_init, "<Boolean class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(primitive_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<Boolean native instance initializer>",
        mc,
    ));

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("toString", to_string), ("valueOf", value_of)];
    write.define_as3_builtin_instance_methods(mc, AS3_INSTANCE_METHODS);

    class
}
