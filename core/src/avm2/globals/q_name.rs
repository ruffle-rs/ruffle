//! `QName` impl

use ruffle_macros::istr;

use crate::avm2::activation::Activation;
use crate::avm2::api_version::ApiVersion;
use crate::avm2::object::{Object, QNameObject, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() == 1 {
        // 1. If Namespace is not specified and Type(Name) is Object and Name.[[Class]] == “QName”
        if args[0]
            .as_object()
            .and_then(|x| x.as_qname_object())
            .is_some()
        {
            // 1.a. Return Name
            return Ok(args[0]);
        }
    }

    // 2. Create and return a new QName object exactly as if the QName constructor had been called with the
    //    same arguments (section 13.3.2).
    activation
        .avm2()
        .classes()
        .qname
        .construct(activation, args)
}

/// Implements a custom constructor for `QName`.
pub fn q_name_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = QNameObject::new_empty(activation);

    let namespace = if args.len() >= 2 {
        let ns_arg = args[0];
        let mut local_arg = args[1];

        if matches!(local_arg, Value::Undefined) {
            local_arg = istr!("").into();
        }

        let api_version = activation.avm2().root_api_version;

        let namespace = match ns_arg {
            Value::Object(Object::NamespaceObject(ns)) => Some(ns.namespace()),
            Value::Object(Object::QNameObject(qname)) => qname
                .uri(activation.strings())
                .map(|uri| Namespace::package(uri, ApiVersion::AllVersions, activation.strings())),
            Value::Null => None,
            Value::Undefined => Some(Namespace::package(
                istr!(""),
                api_version,
                activation.strings(),
            )),
            v => Some(Namespace::package(
                v.coerce_to_string(activation)?,
                api_version,
                activation.strings(),
            )),
        };

        if let Value::Object(Object::QNameObject(qname)) = local_arg {
            this.set_local_name(activation.gc(), qname.local_name(activation.strings()));
        } else {
            this.set_local_name(activation.gc(), local_arg.coerce_to_string(activation)?);
        }

        namespace
    } else {
        let qname_arg = args.get(0).copied().unwrap_or(Value::Undefined);
        if let Value::Object(Object::QNameObject(qname_obj)) = qname_arg {
            this.init_name(activation.gc(), qname_obj.name().clone());
            return Ok(this.into());
        }

        let local = if qname_arg == Value::Undefined {
            istr!("")
        } else {
            qname_arg.coerce_to_string(activation)?
        };

        if &*local != b"*" {
            this.set_local_name(activation.gc(), local);
            Some(activation.avm2().find_public_namespace())
        } else {
            None
        }
    };

    if let Some(namespace) = namespace {
        this.set_namespace(activation.gc(), namespace);
        this.set_is_qname(activation.gc(), true);
    }

    Ok(this.into())
}

/// Implements `QName.localName`'s getter
pub fn get_local_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_qname_object() {
        return Ok(this.local_name(activation.strings()).into());
    }

    Ok(Value::Undefined)
}

/// Implements `QName.uri`'s getter
pub fn get_uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_qname_object() {
        return Ok(this
            .uri(activation.strings())
            .map_or_else(|| Value::Null, Value::from));
    }

    Ok(Value::Undefined)
}

/// Implements `QName.AS3::toString` and `QName.prototype.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(this) = this.as_qname_object() {
        return Ok(this.name().as_uri(activation.strings()).into());
    }

    Ok(Value::Undefined)
}
