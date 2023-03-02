//! `QName` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{qname_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use gc_arena::GcCell;

/// Implements `QName`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|t| t.as_qname_object()) {
        let namespace = if args.len() > 1 {
            let ns_arg = args.get(0).cloned().unwrap();
            let local_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

            let namespace = match ns_arg {
                Value::Object(o) if o.as_namespace().is_some() => {
                    o.as_namespace().as_deref().copied()
                }
                Value::Undefined | Value::Null => None,
                v => Some(Namespace::package(
                    v.coerce_to_string(activation)?,
                    activation.context.gc_context,
                )),
            };
            if let Value::Object(Object::QNameObject(qname)) = local_arg {
                this.set_local_name(activation.context.gc_context, qname.local_name());
            } else {
                this.set_local_name(
                    activation.context.gc_context,
                    local_arg.coerce_to_string(activation)?,
                );
            }
            namespace
        } else {
            let qname_arg = args.get(0).cloned().unwrap_or(Value::Undefined);
            if let Value::Object(Object::QNameObject(qname_obj)) = qname_arg {
                this.init_name(activation.context.gc_context, qname_obj.name().clone());
                return Ok(Value::Undefined);
            }
            let local = qname_arg.coerce_to_string(activation)?;
            if &*local != b"*" {
                this.set_local_name(activation.context.gc_context, local);
                Some(activation.avm2().public_namespace)
            } else {
                None
            }
        };

        if let Some(namespace) = namespace {
            this.set_namespace(activation.context.gc_context, namespace)
        }
    }

    Ok(Value::Undefined)
}

/// Implements `QName`'s class initializer.
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let scope = activation.create_scopechain();
    let this_class = this.as_class_object().unwrap();
    let qname_proto = this_class.prototype();

    qname_proto.set_string_property_local(
        "toString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toString", activation.context.gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;

    qname_proto.set_string_property_local(
        "valueOf",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(value_of, "valueOf", activation.context.gc_context),
            scope,
            None,
            Some(this_class),
        )
        .into(),
        activation,
    )?;

    qname_proto.set_local_property_is_enumerable(
        activation.context.gc_context,
        "toString".into(),
        false,
    );
    qname_proto.set_local_property_is_enumerable(
        activation.context.gc_context,
        "valueOf".into(),
        false,
    );

    Ok(Value::Undefined)
}

/// Implements `QName.localName`'s getter
pub fn local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|t| t.as_qname_object()) {
        return Ok(this.local_name().into());
    }

    Ok(Value::Undefined)
}

/// Implements `QName.uri`'s getter
pub fn uri<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|t| t.as_qname_object()) {
        return Ok(this.uri().map(Value::from).unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `QName.AS3::toString` and `QName.prototype.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.and_then(|t| t.as_qname_object()) {
        return Ok(this.name().as_uri(activation.context.gc_context).into());
    }

    Ok(Value::Undefined)
}

/// Implements `QName.AS3::valueOf` and `QName.prototype.valueOf`
pub fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        return Ok(this.into());
    }

    Ok(Value::Undefined)
}

/// Construct `QName`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace, "QName"),
        Some(Multiname::new(activation.avm2().public_namespace, "Object")),
        Method::from_builtin(instance_init, "<QName instance initializer>", mc),
        Method::from_builtin(class_init, "<QName class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);
    write.set_instance_allocator(qname_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("localName", Some(local_name), None),
        ("uri", Some(uri), None),
    ];
    write.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("toString", to_string), ("valueOf", value_of)];
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    class
}
