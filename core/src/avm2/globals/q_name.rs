//! `QName` impl

use crate::avm2::activation::Activation;
use crate::avm2::api_version::ApiVersion;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Namespace;

pub use crate::avm2::object::q_name_allocator;

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
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
    Ok(activation
        .avm2()
        .classes()
        .qname
        .construct(activation, args)?
        .into())
}

/// Implements `QName`'s `init` method, which is called from the constructor.
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

    let this = this.as_qname_object().unwrap();
    let namespace = if arguments_list.get(1).is_some() {
        let ns_arg = arguments_list[0];
        let mut local_arg = arguments_list[1];

        if matches!(local_arg, Value::Undefined) {
            local_arg = "".into();
        }

        let api_version = activation.avm2().root_api_version;

        let namespace = match ns_arg {
            Value::Object(Object::NamespaceObject(ns)) => Some(ns.namespace()),
            Value::Object(Object::QNameObject(qname)) => qname.uri().map(|uri| {
                Namespace::package(uri, ApiVersion::AllVersions, &mut activation.borrow_gc())
            }),
            Value::Null => None,
            Value::Undefined => Some(Namespace::package(
                "",
                api_version,
                &mut activation.borrow_gc(),
            )),
            v => Some(Namespace::package(
                v.coerce_to_string(activation)?,
                api_version,
                &mut activation.borrow_gc(),
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
        let qname_arg = arguments_list.get(0).copied().unwrap_or(Value::Undefined);
        if let Value::Object(Object::QNameObject(qname_obj)) = qname_arg {
            this.init_name(activation.context.gc_context, qname_obj.name().clone());
            return Ok(Value::Undefined);
        }

        let local = if qname_arg == Value::Undefined {
            "".into()
        } else {
            qname_arg.coerce_to_string(activation)?
        };

        if &*local != b"*" {
            this.set_local_name(activation.context.gc_context, local);
            Some(activation.avm2().find_public_namespace())
        } else {
            None
        }
    };

    if let Some(namespace) = namespace {
        this.set_namespace(activation.context.gc_context, namespace);
        this.set_is_qname(activation.context.gc_context, true);
    }

    Ok(Value::Undefined)
}

/// Implements `QName.localName`'s getter
pub fn get_local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_qname_object() {
        return Ok(this.local_name().into());
    }

    Ok(Value::Undefined)
}

/// Implements `QName.uri`'s getter
pub fn get_uri<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_qname_object() {
        return Ok(this.uri().map(Value::from).unwrap_or(Value::Null));
    }

    Ok(Value::Undefined)
}

/// Implements `QName.AS3::toString` and `QName.prototype.toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_qname_object() {
        return Ok(this.name().as_uri(activation.context.gc_context).into());
    }

    Ok(Value::Undefined)
}
