use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};
use crate::avm1_stub;
use crate::prelude::TDisplayObject;
use crate::string::AvmString;

use ruffle_common::sandbox::SandboxType;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "allowDomain" => method(allow_domain);
    "allowInsecureDomain" => method(allow_insecure_domain);
    "loadPolicyFile" => method(load_policy_file);
    "chooseLocalSwfPath" => property(get_choose_local_swf_path);
    "escapeDomain" => method(escape_domain);
    "sandboxType" => property(get_sandbox_type);
    "PolicyFileResolver" => method(policy_file_resolver);
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let security = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(security, OBJECT_DECLS(context));
    security
}

fn allow_domain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.security", "allowDomain");
    Ok(Value::Bool(args.get(0).is_some()))
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
        activation.gc(),
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
