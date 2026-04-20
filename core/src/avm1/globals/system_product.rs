use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Attribute, Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "isRunning" => method(is_running);
    "isInstalled" => method(is_installed);
    "launch" => method(launch);
    "download" => method(download);
    "validate" => method(validate);
    "installedVersion" => method(installed_version);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get(0).cloned().unwrap_or(Value::Undefined);
    this.define_value(
        activation.gc(),
        istr!("name"),
        name,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    Ok(Value::Undefined)
}

fn is_running<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.Product", "isRunning");
    Ok(Value::Bool(false))
}

fn is_installed<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.Product", "isInstalled");
    Ok(Value::Bool(false))
}

fn launch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.Product", "launch");
    Ok(Value::Undefined)
}

fn download<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.Product", "download");
    Ok(Value::Undefined)
}

fn validate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.Product", "validate");
    Ok(Value::Undefined)
}

fn installed_version<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "System.Product", "installedVersion");
    Ok(Value::Undefined)
}
