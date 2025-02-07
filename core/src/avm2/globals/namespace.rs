//! `Namespace` impl

use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::e4x::is_xml_name;
use crate::avm2::error::make_error_1098;
use crate::avm2::object::{NamespaceObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;

/// Implements a custom constructor for `Namespace`.
pub fn namespace_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let api_version = activation.avm2().root_api_version;
    let namespaces = activation.avm2().namespaces;

    let (prefix, namespace) = match args.len() {
        0 => (Some(istr!("")), namespaces.public_all()),
        1 => {
            // These cases only activate with exactly one argument passed
            match args[0] {
                Value::Object(Object::QNameObject(qname)) => {
                    let uri = qname.uri(activation.strings());
                    let ns = uri.map_or_else(Namespace::any, |uri| {
                        Namespace::package(uri, api_version, activation.strings())
                    });
                    let prefix = match uri {
                        Some(name) if !name.is_empty() => None,
                        _ => Some(istr!("")),
                    };
                    (prefix, ns)
                }
                Value::Object(Object::NamespaceObject(ns)) => (ns.prefix(), ns.namespace()),
                val => {
                    let name = val.coerce_to_string(activation)?;
                    let ns = Namespace::package(name, api_version, activation.strings());
                    let prefix = name.is_empty().then(|| istr!(""));
                    (prefix, ns)
                }
            }
        }
        2.. => {
            let prefix = args[0];
            let uri = args[1];

            let namespace_uri = if let Value::Object(Object::QNameObject(qname)) = uri {
                qname.uri(activation.strings()).unwrap_or_else(|| istr!(""))
            } else {
                uri.coerce_to_string(activation)?
            };
            let namespace = Namespace::package(namespace_uri, api_version, activation.strings());
            let prefix_str = prefix.coerce_to_string(activation)?;

            // The order is important here to match Flash
            let mut resulting_prefix = if matches!(prefix, Value::Undefined | Value::Null) {
                None
            } else {
                Some(prefix_str)
            };
            // The only allowed prefix if the uri is empty is the literal empty string
            if namespace_uri.is_empty() && !resulting_prefix.is_some_and(|s| s.is_empty()) {
                return Err(make_error_1098(activation, &prefix_str));
            }
            if !prefix_str.is_empty() && !is_xml_name(prefix_str) {
                resulting_prefix = None;
            }
            (resulting_prefix, namespace)
        }
    };

    Ok(NamespaceObject::from_ns_and_prefix(activation, namespace, prefix).into())
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    activation
        .avm2()
        .classes()
        .namespace
        .construct(activation, args)
}

/// Implements `Namespace.prefix`'s getter
pub fn get_prefix<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_namespace_object().unwrap();

    if let Some(prefix) = this.prefix() {
        Ok(prefix.into())
    } else {
        Ok(Value::Undefined)
    }
}

/// Implements `Namespace.uri`'s getter
pub fn get_uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let this = this.as_namespace_object().unwrap();

    Ok(this.namespace().as_uri(activation.strings()).into())
}
