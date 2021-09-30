//! `QName` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{qname_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `QName`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if this.as_qname().is_none() {
            let (namespace, local_arg) = if args.len() > 1 {
                let ns_arg = args.get(0).cloned().unwrap();
                let local_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

                let namespace = match ns_arg {
                    Value::Object(o) if o.as_namespace().is_some() => {
                        o.as_namespace().unwrap().clone()
                    }
                    Value::Undefined | Value::Null => Namespace::Any,
                    v => Namespace::Namespace(v.coerce_to_string(activation)?),
                };

                (namespace, local_arg)
            } else {
                let qname_arg = args.get(0).cloned().unwrap_or(Value::Undefined);
                let namespace = match qname_arg {
                    Value::Object(o) if o.as_qname().is_some() => {
                        o.as_qname().unwrap().namespace().clone()
                    }
                    _ => Namespace::Namespace("".into()),
                };

                (namespace, qname_arg)
            };

            let local_name = match local_arg {
                Value::Object(o) if o.as_qname().is_some() => o.as_qname().unwrap().local_name(),
                v => v.coerce_to_string(activation)?,
            };

            this.init_qname(
                activation.context.gc_context,
                QName::new(namespace, local_name),
            );
        }
    }

    Ok(Value::Undefined)
}

/// Implements `QName`'s class initializer.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = this.unwrap();
    let mut qname_proto = this
        .get_property(this, &QName::dynamic_name("prototype").into(), activation)?
        .coerce_to_object(activation)?;

    qname_proto.set_property(
        qname_proto,
        &QName::dynamic_name("toString").into(),
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toString", activation.context.gc_context),
            qname_proto.get_scope(),
            None,
        )
        .into(),
        activation,
    )?;

    qname_proto.set_property(
        qname_proto,
        &QName::dynamic_name("valueOf").into(),
        FunctionObject::from_method(
            activation,
            Method::from_builtin(value_of, "valueOf", activation.context.gc_context),
            qname_proto.get_scope(),
            None,
        )
        .into(),
        activation,
    )?;

    Ok(Value::Undefined)
}

/// Implements `QName.localName`'s getter
pub fn local_name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(qname) = this.as_qname() {
            return Ok(qname.local_name().into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `QName.uri`'s getter
pub fn uri<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(qname) = this.as_qname() {
            return Ok(match qname.namespace() {
                Namespace::Any => Value::Null,
                ns => ns.as_uri().into(),
            });
        }
    }

    Ok(Value::Undefined)
}

/// Implements `QName.AS3::toString` and `QName.prototype.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(qname) = this.as_qname() {
            return Ok(qname.as_uri(activation.context.gc_context).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `QName.AS3::valueOf` and `QName.prototype.valueOf`
pub fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        return Ok(this.into());
    }

    Ok(Value::Undefined)
}

/// Construct `QName`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::public(), "QName"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<QName instance initializer>", mc),
        Method::from_builtin(class_init, "<QName class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(qname_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("localName", Some(local_name), None),
        ("uri", Some(uri), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("toString", to_string), ("valueOf", value_of)];
    write.define_as3_builtin_instance_methods(mc, AS3_INSTANCE_METHODS);

    class
}
