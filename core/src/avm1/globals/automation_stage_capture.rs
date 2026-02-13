use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "capture" => method(CAPTURE; DONT_ENUM | DONT_DELETE);
    "cancel" => method(CANCEL; DONT_ENUM | DONT_DELETE);
    "getFileNameBase" => method(GET_FILE_NAME_BASE; DONT_ENUM | DONT_DELETE);
    "setFileNameBase" => method(SET_FILE_NAME_BASE; DONT_ENUM | DONT_DELETE);
    "getClipRect" => method(GET_CLIP_RECT; DONT_ENUM | DONT_DELETE);
    "setClipRect" => method(SET_CLIP_RECT; DONT_ENUM | DONT_DELETE);
    "listenForStageCapture" => method(LISTEN_FOR_STAGE_CAPTURE; DONT_ENUM | DONT_DELETE);
    "valueOf" => method(VALUE_OF; DONT_DELETE);
    "toString" => method(TO_STRING; DONT_DELETE);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(table_constructor!(method), None, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

pub mod method {
    pub const CAPTURE: u16 = 0;
    pub const CANCEL: u16 = 1;
    pub const GET_FILE_NAME_BASE: u16 = 2;
    pub const SET_FILE_NAME_BASE: u16 = 3;
    pub const GET_CLIP_RECT: u16 = 4;
    pub const SET_CLIP_RECT: u16 = 5;
    pub const LISTEN_FOR_STAGE_CAPTURE: u16 = 6;
    pub const VALUE_OF: u16 = 7;
    pub const TO_STRING: u16 = 8;
    pub const CONSTRUCTOR: u16 = 100;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;
    const CNAME: &str = "flash.automation.StageCapture";

    if index == CONSTRUCTOR {
        avm1_stub!(activation, CNAME);
        return Ok(this.into());
    }

    match index {
        CAPTURE => avm1_stub!(activation, CNAME, "capture"),
        CANCEL => avm1_stub!(activation, CNAME, "cancel"),
        GET_FILE_NAME_BASE => avm1_stub!(activation, CNAME, "getFileNameBase"),
        SET_FILE_NAME_BASE => avm1_stub!(activation, CNAME, "setFileNameBase"),
        GET_CLIP_RECT => avm1_stub!(activation, CNAME, "getClipRect"),
        SET_CLIP_RECT => avm1_stub!(activation, CNAME, "setClipRect"),
        LISTEN_FOR_STAGE_CAPTURE => avm1_stub!(activation, CNAME, "listenForStageCapture"),
        VALUE_OF => avm1_stub!(activation, CNAME, "valueOf"),
        TO_STRING => avm1_stub!(activation, CNAME, "toString"),
        _ => (),
    }

    Ok(Value::Undefined)
}
