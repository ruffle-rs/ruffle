//! LocalConnection class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::display_object::TDisplayObject;
use crate::string::AvmString;
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "domain" => method(domain; DONT_DELETE | READ_ONLY);
};

pub fn domain<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = if let Some(movie) = activation.base_clip().movie() {
        movie
    } else {
        log::error!("LocalConnection::domain: Movie was None");
        return Ok(Value::Null);
    };

    let domain = if let Some(url) = movie.url() {
        if let Ok(url) = url::Url::parse(url) {
            if url.scheme() == "file" {
                "localhost".into()
            } else if let Some(domain) = url.domain() {
                AvmString::new_utf8(activation.context.gc_context, domain)
            } else {
                // no domain?
                "localhost".into()
            }
        } else {
            log::error!("LocalConnection::domain: Unable to parse movie URL");
            return Ok(Value::Null);
        }
    } else {
        // No URL (loading local data).
        "localhost".into()
    };

    Ok(Value::String(domain))
}

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    object.into()
}
