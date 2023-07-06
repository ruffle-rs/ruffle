use crate::avm2::{Activation, Error, Object, Value};
use crate::string::AvmString;

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
