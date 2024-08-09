//! `Namespace` impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::e4x::is_xml_name;
use crate::avm2::error::make_error_1098;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{namespace_allocator, FunctionObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::string::AvmString;

// All of these methods will be defined as both
// AS3 instance methods and methods on the `Namespace` class prototype.
const PUBLIC_INSTANCE_AND_PROTO_METHODS: &[(&str, NativeMethodImpl)] = &[("toString", uri)];

/// Implements `Namespace`'s instance initializer.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_namespace_object() {
        let api_version = activation.avm2().root_api_version;

        let (prefix, namespace) = match args {
            [prefix, uri] => {
                let namespace_uri = if let Value::Object(Object::QNameObject(qname)) = uri {
                    qname.uri().unwrap_or_else(|| AvmString::from(""))
                } else {
                    uri.coerce_to_string(activation)?
                };
                let namespace =
                    Namespace::package(namespace_uri, api_version, &mut activation.borrow_gc());
                let prefix_str = prefix.coerce_to_string(activation)?;

                // The order is important here to match Flash
                let mut resulting_prefix = if matches!(prefix, Value::Undefined | Value::Null) {
                    None
                } else {
                    Some(prefix_str)
                };
                // The only allowed prefix if the uri is empty is the literal empty string
                if namespace.as_uri().is_empty() && resulting_prefix != Some("".into()) {
                    return Err(make_error_1098(activation, &prefix_str));
                }
                if !prefix_str.is_empty() && !is_xml_name(prefix_str) {
                    resulting_prefix = None;
                }
                (resulting_prefix, namespace)
            }
            [Value::Object(Object::QNameObject(qname))] => {
                let ns = qname
                    .uri()
                    .map(|uri| Namespace::package(uri, api_version, &mut activation.borrow_gc()))
                    .unwrap_or_else(|| Namespace::any(activation.context.gc_context));
                if ns.as_uri().is_empty() {
                    (Some("".into()), ns)
                } else {
                    (None, ns)
                }
            }
            [Value::Object(Object::NamespaceObject(ns))] => (ns.prefix(), ns.namespace()),
            [val] => {
                let ns = Namespace::package(
                    val.coerce_to_string(activation)?,
                    api_version,
                    &mut activation.borrow_gc(),
                );
                if ns.as_uri().is_empty() {
                    (Some("".into()), ns)
                } else {
                    (None, ns)
                }
            }
            _ => (
                Some("".into()),
                activation.avm2().public_namespace_base_version,
            ),
        };

        this.init_namespace(activation.context.gc_context, namespace);
        this.set_prefix(activation.context.gc_context, prefix);
    }
    Ok(Value::Undefined)
}

fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation
        .avm2()
        .classes()
        .namespace
        .construct(activation, args)?
        .into())
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let proto = this_class.prototype();

    for (name, method) in PUBLIC_INSTANCE_AND_PROTO_METHODS {
        proto.set_string_property_local(
            *name,
            FunctionObject::from_method(
                activation,
                Method::from_builtin(*method, name, gc_context),
                scope,
                None,
                Some(this_class),
            )
            .into(),
            activation,
        )?;
        proto.set_local_property_is_enumerable(gc_context, (*name).into(), false);
    }
    Ok(Value::Undefined)
}

/// Implements `Namespace.prefix`'s getter
pub fn prefix<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(o) = this.as_namespace_object() {
        if let Some(prefix) = o.prefix() {
            return Ok(prefix.into());
        }
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
pub fn create_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "Namespace"),
        Some(activation.avm2().classes().object.inner_class_definition()),
        Method::from_builtin(instance_init, "<Namespace instance initializer>", mc),
        Method::from_builtin(class_init, "<Namespace class initializer>", mc),
        activation.avm2().classes().class.inner_class_definition(),
        mc,
    );

    class.set_instance_allocator(mc, namespace_allocator);
    class.set_native_instance_init(
        mc,
        Method::from_builtin(
            native_instance_init,
            "<Namespace native instance initializer>",
            mc,
        ),
    );
    class.set_call_handler(
        mc,
        Method::from_builtin(class_call, "<Namespace call handler>", mc),
    );

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("prefix", Some(prefix), None), ("uri", Some(uri), None)];
    class.define_builtin_instance_properties(
        mc,
        activation.avm2().public_namespace_base_version,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    const CONSTANTS_INT: &[(&str, i32)] = &[("length", 2)];
    class.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CONSTANTS_INT,
        activation,
    );

    class.define_builtin_instance_methods(
        mc,
        activation.avm2().as3_namespace,
        PUBLIC_INSTANCE_AND_PROTO_METHODS,
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
