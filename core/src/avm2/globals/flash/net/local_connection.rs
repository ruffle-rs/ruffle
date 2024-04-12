use crate::avm2::error::{argument_error, make_error_2007};
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Avm2, Error, Object, Value};
use crate::string::AvmString;

use crate::avm2_stub_method;

pub use crate::avm2::object::local_connection_allocator;

/// Implements `domain` getter
pub fn get_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = &activation.context.swf;

    let domain = if let Ok(url) = url::Url::parse(movie.url()) {
        if url.scheme() == "file" {
            "localhost".into()
        } else if let Some(domain) = url.domain() {
            AvmString::new_utf8(activation.context.gc_context, domain)
        } else {
            // no domain?
            "localhost".into()
        }
    } else {
        tracing::error!("LocalConnection::domain: Unable to parse movie URL");
        return Ok(Value::Null);
    };

    Ok(Value::String(domain))
}

/// Implements `LocalConnection.send`
pub fn send_internal<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Already null-checked by the AS wrapper `LocalConnection.send`
    let connection_name = args.get_value(0);

    let connection_name = connection_name.coerce_to_string(activation)?;

    let event_name = if activation
        .context
        .local_connections
        .all_by_name(connection_name)
        .is_empty()
    {
        "error"
    } else {
        avm2_stub_method!(activation, "flash.net.LocalConnection", "send");

        "status"
    };

    let event = activation.avm2().classes().statusevent.construct(
        activation,
        &[
            "status".into(),
            false.into(),
            false.into(),
            Value::Null,
            event_name.into(),
        ],
    )?;

    Avm2::dispatch_event(&mut activation.context, event, this);

    Ok(Value::Undefined)
}

/// Implements `LocalConnection.connect`
pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let connection_name = args.get_value(0);
    if matches!(connection_name, Value::Null) {
        return Err(make_error_2007(activation, "connectionName"));
    };

    if let Some(local_connection) = this.as_local_connection_object() {
        if local_connection.is_connected() {
            return Err(Error::AvmError(argument_error(
                activation,
                "Error #2082: Connect failed because the object is already connected.",
                2082,
            )?));
        }

        let connection_name = connection_name.coerce_to_string(activation)?;
        local_connection.connect(activation, connection_name);
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
