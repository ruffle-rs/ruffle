use crate::avm2::error::type_error;
use crate::avm2::object::TObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Avm2, Error, Object, Value};
use crate::string::AvmString;

use crate::avm2_stub_method;

/// Implements `domain` getter
pub fn get_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.context.swf;

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
pub fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if matches!(args.get_value(0), Value::Null) {
        return Err(Error::AvmError(type_error(
            activation,
            "Error #2007: Parameter connectionName must be non-null.",
            2007,
        )?));
    }

    if matches!(args.get_value(1), Value::Null) {
        return Err(Error::AvmError(type_error(
            activation,
            "Error #2007: Parameter methodName must be non-null.",
            2007,
        )?));
    }

    avm2_stub_method!(activation, "flash.net.LocalConnection", "send");

    let event = activation.avm2().classes().statusevent.construct(
        activation,
        &[
            "status".into(),
            false.into(),
            false.into(),
            Value::Null,
            "error".into(),
        ],
    )?;

    // FIXME: Adding the event listener after calling `send` works in FP.
    Avm2::dispatch_event(&mut activation.context, event, this);

    Ok(Value::Undefined)
}
