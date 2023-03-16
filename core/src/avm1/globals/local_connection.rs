//! LocalConnection class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::context::GcContext;
use crate::display_object::TDisplayObject;
use crate::string::AvmString;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "domain" => method(domain; DONT_DELETE | READ_ONLY);
};

pub fn domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.base_clip().movie();

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

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
