//! Microphone object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "setSilenceLevel" => method(set_silence_level; DONT_ENUM | DONT_DELETE);
    "setRate" => method(set_rate; DONT_ENUM | DONT_DELETE);
    "setGain" => method(set_gain; DONT_ENUM | DONT_DELETE);
    "setUseEchoSuppression" => method(set_use_echo_suppression; DONT_ENUM | DONT_DELETE);
    "setCodec" => method(set_codec; DONT_ENUM | DONT_DELETE);
    "setFramesPerPacket" => method(set_frames_per_packet; DONT_ENUM | DONT_DELETE);
    "setEncodeQuality" => method(set_encode_quality; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "get" => method(get);
    "names" => property(get_names);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}
fn get<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "get");
    // Microphone.get() returns null when there's no microphone.
    Ok(Value::Null)
}

fn get_names<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "names");
    activation
        .prototypes()
        .array_constructor
        .construct(activation, &[])
}

fn set_silence_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setSilenceLevel");
    Ok(Value::Undefined)
}

fn set_rate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setRate");
    Ok(Value::Undefined)
}

fn set_gain<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setGain");
    Ok(Value::Undefined)
}

fn set_use_echo_suppression<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setUseEchoSuppression");
    Ok(Value::Undefined)
}

fn set_codec<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setCodec");
    Ok(Value::Undefined)
}

fn set_frames_per_packet<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setFramesPerPacket");
    Ok(Value::Undefined)
}

fn set_encode_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "Microphone", "setEncodeQuality");
    Ok(Value::Undefined)
}
