use crate::avm2::amf::serialize_value;
use crate::avm2::error::{argument_error, make_error_2004, make_error_2085, Error2004Type};
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, Value};
use crate::string::AvmString;
use flash_lso::types::{AMFVersion, Value as AmfValue};

pub use crate::avm2::object::local_connection_allocator;
use crate::local_connection::LocalConnections;

/// Implements `domain` getter
pub fn get_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = &activation.context.swf;
    let domain = LocalConnections::get_domain(movie.url());

    Ok(Value::String(AvmString::new_utf8(
        activation.context.gc_context,
        domain,
    )))
}

/// Implements `LocalConnection.send`
pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection_name = args.get_string_non_null(activation, 0, "connectionName")?;
    let method_name = args.get_string_non_null(activation, 1, "methodName")?;

    if connection_name.is_empty() {
        return Err(make_error_2085(activation, "connectionName"));
    }
    if method_name.is_empty() {
        return Err(make_error_2085(activation, "methodName"));
    }
    if &method_name == b"send"
        || &method_name == b"connect"
        || &method_name == b"close"
        || &method_name == b"allowDomain"
        || &method_name == b"allowInsecureDomain"
        || &method_name == b"domain"
    {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    let mut amf_arguments = Vec::with_capacity(args.len() - 2);
    for arg in &args[2..] {
        amf_arguments.push(
            serialize_value(activation, *arg, AMFVersion::AMF0, &mut Default::default())
                .unwrap_or(AmfValue::Undefined),
        );
    }

    if let Some(local_connection) = this.as_local_connection_object() {
        activation.context.local_connections.send(
            &LocalConnections::get_domain(activation.context.swf.url()),
            (activation.domain(), local_connection),
            connection_name,
            method_name,
            amf_arguments,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `LocalConnection.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection_name = args.get_string_non_null(activation, 0, "connectionName")?;
    if connection_name.is_empty() {
        return Err(make_error_2085(activation, "connectionName"));
    }
    if connection_name.contains(b':') {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    if let Some(local_connection) = this.as_local_connection_object() {
        if !local_connection.connect(activation, connection_name) {
            // This triggers both if this object is already connected, OR there's something else taking the name
            // (The error message is misleading, in that case!)
            return Err(Error::AvmError(argument_error(
                activation,
                "Error #2082: Connect failed because the object is already connected.",
                2082,
            )?));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `LocalConnection.close`
pub fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(local_connection) = this.as_local_connection_object() {
        if !local_connection.is_connected() {
            return Err(Error::AvmError(argument_error(
                activation,
                "Error #2083: Close failed because the object is not connected.",
                2083,
            )?));
        }

        local_connection.disconnect(activation);
    }

    Ok(Value::Undefined)
}

pub fn get_client<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(local_connection) = this.as_local_connection_object() {
        Ok(local_connection.client().into())
    } else {
        Ok(Value::Undefined)
    }
}

pub fn set_client<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(local_connection) = this.as_local_connection_object() {
        let client_obj = args.try_get_object(activation, 0);

        if let Some(client_obj) = client_obj {
            local_connection.set_client(activation.gc(), client_obj);
        } else {
            return Err(make_error_2004(activation, Error2004Type::TypeError));
        }
    }

    Ok(Value::Undefined)
}
