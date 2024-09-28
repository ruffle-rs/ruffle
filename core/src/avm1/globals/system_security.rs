use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::Object;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ScriptObject, Value};
use crate::avm1_stub;
use crate::prelude::TDisplayObject;
use crate::sandbox::SandboxType;
use crate::string::{AvmString, StringContext};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "PolicyFileResolver" => method(policy_file_resolver);
    "allowDomain" => method(allow_domain);
    "allowInsecureDomain" => method(allow_insecure_domain);
    "loadPolicyFile" => method(load_policy_file);
    "escapeDomain" => method(escape_domain);
    "sandboxType" => property(get_sandbox_type);
    "chooseLocalSwfPath" => property(get_choose_local_swf_path);
};

fn allow_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "allowDomain");
    Ok(Value::Undefined)
}

fn allow_insecure_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "allowInsecureDomain");
    Ok(Value::Undefined)
}

fn load_policy_file<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "loadPolicyFile");
    Ok(Value::Undefined)
}

fn escape_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "escapeDomain");
    Ok(Value::Undefined)
}

fn get_sandbox_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let movie = activation.base_clip().movie();
    Ok(AvmString::new_utf8(
        activation.context.gc_context,
        match movie.sandbox_type() {
            SandboxType::Remote => "remote",
            SandboxType::LocalWithFile => "localWithFile",
            SandboxType::LocalWithNetwork => "localWithNetwork",
            SandboxType::LocalTrusted | SandboxType::Application => "localTrusted",
        },
    )
    .into())
}

fn get_choose_local_swf_path<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "chooseLocalSwfPath");
    Ok(Value::Undefined)
}

fn policy_file_resolver<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "chooseLocalSwfPath");
    Ok(Value::Undefined)
}

pub fn create<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let security = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(OBJECT_DECLS, context, security, fn_proto);
    security.into()
}
