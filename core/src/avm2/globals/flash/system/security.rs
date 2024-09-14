//! `flash.system.Security` native methods

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2_stub_method;
use crate::sandbox::SandboxType;
use crate::string::AvmString;
use url::Url;

pub fn get_page_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(url) = activation
        .context
        .page_url
        .as_ref()
        .and_then(|page_url| Url::parse(page_url).ok())
    {
        if !url.origin().is_tuple() {
            tracing::warn!("flash.system.Security.pageDomain: Returning null for opaque origin");
            return Ok(Value::Null);
        }

        let mut domain = url.origin().ascii_serialization();
        domain.push('/'); // Add trailing slash that is used by Flash, but isn't part of a standard origin.
        Ok(AvmString::new_utf8(activation.context.gc_context, domain).into())
    } else {
        tracing::warn!("flash.system.Security.pageDomain: No page-url available");
        Ok(Value::Null)
    }
}

pub fn get_sandbox_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation
        .caller_movie()
        .expect("Caller movie expected for sandboxType");
    let sandbox_type = match movie.sandbox_type() {
        SandboxType::Remote => "remote",
        SandboxType::LocalWithFile => "localWithFile",
        SandboxType::LocalWithNetwork => "localWithNetwork",
        SandboxType::LocalTrusted => "localTrusted",
        SandboxType::Application => "application",
    };
    return Ok(AvmString::new_utf8(activation.context.gc_context, sandbox_type).into());
}

pub fn allow_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Security", "allowDomain");
    Ok(Value::Undefined)
}

pub fn allow_insecure_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Security", "allowInsecureDomain");
    Ok(Value::Undefined)
}

pub fn load_policy_file<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Security", "loadPolicyFile");
    Ok(Value::Undefined)
}

pub fn show_settings<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.system.Security", "showSettings");
    Ok(Value::Undefined)
}
