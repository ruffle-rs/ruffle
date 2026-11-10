//! Microphone object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{ArrayBuilder, Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "setSilenceLevel" => method(SET_SILENCE_LEVEL; DONT_ENUM | DONT_DELETE | VERSION_6);
    "setRate" => method(SET_RATE; DONT_ENUM | DONT_DELETE | VERSION_6);
    "setGain" => method(SET_GAIN; DONT_ENUM | DONT_DELETE | VERSION_6);
    "setUseEchoSuppression" => method(SET_USE_ECHO_SUPPRESSION; DONT_ENUM | DONT_DELETE | VERSION_6);
    "setCodec" => method(SET_CODEC; DONT_ENUM | DONT_DELETE | VERSION_10);
    "setFramesPerPacket" => method(SET_FRAMES_PER_PACKET; DONT_ENUM | DONT_DELETE | VERSION_10);
    "setEncodeQuality" => method(SET_ENCODE_QUALITY; DONT_ENUM | DONT_DELETE | VERSION_10);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    // In playerglobals.swf, this is a bytecode function delegating to the internal native impl.
    "get" => function(|a, this, args| method(a, this, args, method::INTERNAL_GET));
    use fn method;
    "names" => property(GET_NAMES);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto, PropertyOrder::PrototypeLast);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

pub mod method {
    pub const SET_SILENCE_LEVEL: u16 = 0;
    pub const SET_RATE: u16 = 1;
    pub const SET_GAIN: u16 = 2;
    pub const SET_USE_ECHO_SUPPRESSION: u16 = 3;
    pub const SET_CODEC: u16 = 4;
    pub const SET_FRAMES_PER_PACKET: u16 = 5;
    pub const SET_ENCODE_QUALITY: u16 = 6;

    pub const INTERNAL_GET: u16 = 200;
    pub const GET_NAMES: u16 = 201;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;
    const CNAME: &str = "Microphone";

    match index {
        INTERNAL_GET => {
            avm1_stub!(activation, CNAME, "get");
            // Microphone.get() returns null when there's no microphone.
            return Ok(Value::Null);
        }
        GET_NAMES => {
            avm1_stub!(activation, CNAME, "names");
            return Ok(ArrayBuilder::empty(activation).into());
        }
        SET_SILENCE_LEVEL => avm1_stub!(activation, CNAME, "setSilenceLevel"),
        SET_RATE => avm1_stub!(activation, CNAME, "setRate"),
        SET_GAIN => avm1_stub!(activation, CNAME, "setGain"),
        SET_USE_ECHO_SUPPRESSION => avm1_stub!(activation, CNAME, "setUseEchoSuppression"),
        SET_CODEC => avm1_stub!(activation, CNAME, "setCodec"),
        SET_FRAMES_PER_PACKET => avm1_stub!(activation, CNAME, "setFramesPerPacket"),
        SET_ENCODE_QUALITY => avm1_stub!(activation, CNAME, "setEncodeQuality"),
        _ => (),
    }

    Ok(Value::Undefined)
}
