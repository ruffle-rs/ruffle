//! Microphone object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{ArrayBuilder, Object, Value};
use crate::avm1_stub;

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
