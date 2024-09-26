//! `Namespace` impl

use crate::avm2::activation::Activation;
use crate::avm2::e4x::is_xml_name;
use crate::avm2::error::make_error_1098;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;

pub use crate::avm2::object::namespace_allocator;

/// Implements `Namespace`'s `init` method, which is called from the constructor.
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arguments_object = args[0].as_object().unwrap();
    let arguments_list = arguments_object.as_array_storage().unwrap();
    let arguments_list = arguments_list
        .iter()
        .map(|v| v.unwrap()) // Arguments should be array with no holes
        .collect::<Vec<_>>();

    let this = this.as_namespace_object().unwrap();

    let api_version = activation.avm2().root_api_version;
    let namespaces = activation.avm2().namespaces;

    let (prefix, namespace) = match arguments_list.as_slice() {
        [prefix, uri] => {
            let namespace_uri = if let Value::Object(Object::QNameObject(qname)) = uri {
                qname
                    .uri(activation.strings())
                    .unwrap_or_else(|| activation.strings().empty())
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
        [Value::Object(Object::QNameObject(qname))] => {
            let uri = qname.uri(activation.strings());
            let ns = uri.map_or_else(Namespace::any, |uri| {
                Namespace::package(uri, api_version, activation.strings())
            });
            let prefix = match uri {
                Some(name) if !name.is_empty() => None,
                _ => Some(activation.strings().empty()),
            };
            (prefix, ns)
        }
        [Value::Object(Object::NamespaceObject(ns))] => (ns.prefix(), ns.namespace()),
        [val] => {
            let name = val.coerce_to_string(activation)?;
            let ns = Namespace::package(name, api_version, activation.strings());
            let prefix = name.is_empty().then(|| activation.strings().empty());
            (prefix, ns)
        }
        _ => (Some(activation.strings().empty()), namespaces.public_all()),
    };

    this.init_namespace(activation.context.gc_context, namespace);
    this.set_prefix(activation.context.gc_context, prefix);

    Ok(Value::Undefined)
}

pub fn call_handler<'gc>(
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

/// Implements `Namespace.prefix`'s getter
pub fn get_prefix<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_namespace_object().unwrap();

    Ok(this.namespace().as_uri(activation.strings()).into())
}
