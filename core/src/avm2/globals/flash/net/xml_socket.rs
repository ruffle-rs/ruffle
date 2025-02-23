use crate::avm2::error::Error;
use crate::avm2::value::Value;
use crate::avm2::Activation;
use crate::string::AvmString;
use ruffle_macros::istr;

// NOTE: This is used to get the movie domain when null is passed to connect function.
pub fn get_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = &activation.context.swf;

    let domain = if let Ok(url) = url::Url::parse(movie.url()) {
        if url.scheme() == "file" {
            istr!("localhost")
        } else if let Some(domain) = url.domain() {
            AvmString::new_utf8(activation.gc(), domain)
        } else {
            // no domain?
            istr!("localhost")
        }
    } else {
        tracing::error!("XMLSocket::connect: Unable to parse movie URL");
        return Ok(Value::Null);
    };

    Ok(Value::String(domain))
}
