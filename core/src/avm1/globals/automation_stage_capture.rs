use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "capture" => method(capture; DONT_ENUM);
    "cancel" => method(cancel; DONT_ENUM);
    "getFileNameBase" => method(get_file_name_base; DONT_ENUM);
    "setFileNameBase" => method(set_file_name_base; DONT_ENUM);
    "getClipRect" => method(get_clip_rect; DONT_ENUM);
    "setClipRect" => method(set_clip_rect; DONT_ENUM);
    "listenForStageCapture" => method(listen_for_stage_capture; DONT_ENUM);
    "valueOf" => method(value_of);
    "toString" => method(to_string);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn capture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.automation.StageCapture", "capture");
    Ok(Value::Undefined)
}

fn cancel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.automation.StageCapture", "cancel");
    Ok(Value::Undefined)
}

fn get_file_name_base<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.automation.StageCapture",
        "getFileNameBase"
    );
    Ok(Value::Undefined)
}

fn set_file_name_base<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.automation.StageCapture",
        "setFileNameBase"
    );
    Ok(Value::Undefined)
}

fn get_clip_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.automation.StageCapture", "getClipRect");
    Ok(Value::Undefined)
}

fn set_clip_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.automation.StageCapture", "setClipRect");
    Ok(Value::Undefined)
}

fn listen_for_stage_capture<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.automation.StageCapture",
        "listenForStageCapture"
    );
    Ok(Value::Undefined)
}

fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.automation.StageCapture", "valueOf");
    Ok(Value::Undefined)
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.automation.StageCapture", "toString");
    Ok(Value::Undefined)
}
