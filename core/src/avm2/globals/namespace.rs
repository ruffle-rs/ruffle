//! `Namespace` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{namespace_allocator, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::{avm2_stub_constructor, avm2_stub_getter};
use gc_arena::GcCell;

/// Implements `Namespace`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_namespace_object() {
        let uri_value = match args {
            [_prefix, uri] => {
                avm2_stub_constructor!(activation, "Namespace", "Namespace prefix not supported");
                Some(*uri)
            }
            [uri] => Some(*uri),
            _ => None,
        };

        let api_version = activation.avm2().root_api_version;

        let namespace = match uri_value {
            Some(Value::Object(Object::QNameObject(qname))) => qname
                .uri()
                .map(|uri| Namespace::package(uri, api_version, &mut activation.borrow_gc()))
                .unwrap_or_else(|| Namespace::any(activation.context.gc_context)),
            Some(val) => Namespace::package(
                val.coerce_to_string(activation)?,
                api_version,
                &mut activation.borrow_gc(),
            ),
            None => activation.avm2().public_namespace_base_version,
        };

        this.init_namespace(activation.context.gc_context, namespace);
    }
    Ok(Value::Undefined)
}

fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_constructor!(activation, "Namespace");
    Err("Namespace constructor is a stub.".into())
}

/// Implements `Namespace`'s native instance initializer.
pub fn native_instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation.super_init(this, args)?;

    Ok(Value::Undefined)
}

/// Implements `Namespace`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `Namespace.prefix`'s getter
pub fn prefix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if this.as_namespace_object().is_some() {
        avm2_stub_getter!(activation, "Namespace", "prefix");
        return Ok("".into());
    }

    Ok(Value::Undefined)
}

/// Implements `Namespace.uri`'s getter
pub fn uri<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(o) = this.as_namespace_object() {
        return Ok(o.namespace().as_uri().into());
    }

    Ok(Value::Undefined)
}

/// Construct `Namespace`'s class.
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> GcCell<'gc, Class<'gc>> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "Namespace"),
        Some(Multiname::new(
            activation.avm2().public_namespace_base_version,
            "Object",
        )),
        Method::from_builtin(instance_init, "<Namespace instance initializer>", mc),
        Method::from_builtin(class_init, "<Namespace class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_allocator(namespace_allocator);
    write.set_native_instance_init(Method::from_builtin(
        native_instance_init,
        "<Namespace native instance initializer>",
        mc,
    ));
    write.set_call_handler(Method::from_builtin(
        class_call,
        "<Namespace call handler>",
        mc,
    ));

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("prefix", Some(prefix), None), ("uri", Some(uri), None)];
    write.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace_base_version,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const CONSTANTS_INT: &[(&str, i32)] = &[("length", 2)];
    write.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CONSTANTS_INT,
        activation,
    );

    const AS3_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[("toString", uri)];
    write.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        AS3_INSTANCE_METHODS,
    );

    class
}
