use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, Error, NativeObject, Object, Value};
use crate::avm1_stub;
use crate::streams::NetStream;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let netstream = NetStream::new_avm1(activation.gc(), this);
    this.set_native(activation.gc(), NativeObject::NetStream(netstream));

    Ok(Value::Undefined)
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "publish" => method(publish; DONT_ENUM | DONT_DELETE);
    "play" => method(play; DONT_ENUM | DONT_DELETE);
    "play2" => method(play2; DONT_ENUM | DONT_DELETE);
    "receiveAudio" => method(receive_audio; DONT_ENUM | DONT_DELETE);
    "receiveVideo" => method(receive_video; DONT_ENUM | DONT_DELETE);
    "pause" => method(pause; DONT_ENUM | DONT_DELETE);
    "seek" => method(seek; DONT_ENUM | DONT_DELETE);
    "onPeerConnect" => method(on_peer_connect; DONT_ENUM | DONT_DELETE);
    "close" => method(close; DONT_ENUM | DONT_DELETE);
    "attachAudio" => method(attach_audio; DONT_ENUM | DONT_DELETE);
    "attachVideo" => method(attach_video; DONT_ENUM | DONT_DELETE);
    "send" => method(send; DONT_ENUM | DONT_DELETE);
    "setBufferTime" => method(set_buffer_time; DONT_ENUM | DONT_DELETE);
    "getInfo" => method(get_info; DONT_ENUM | DONT_DELETE);
    "checkPolicyFile" => property(check_policy_file; DONT_ENUM | DONT_DELETE);
    "maxPauseBufferTime" => property(max_pause_buffer_time; DONT_ENUM | DONT_DELETE);
    "backBufferTime" => property(back_buffer_time; DONT_ENUM | DONT_DELETE);

    // TODO The following shouldn't be built-in properties.
    "bufferLength" => property(get_buffer_length);
    "bufferTime" => property(get_buffer_time);
    "bytesLoaded" => property(get_bytes_loaded);
    "bytesTotal" => property(get_bytes_total);
    "time" => property(get_time);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.class(constructor, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

fn get_buffer_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        avm1_stub!(activation, "NetStream", "bufferLength");

        return Ok(ns.buffer_time().into());
    }

    Ok(Value::Undefined)
}

fn get_buffer_time<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        return Ok(ns.buffer_time().into());
    }

    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        return Ok(ns.bytes_loaded().into());
    }

    Ok(Value::Undefined)
}

fn get_bytes_total<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        return Ok(ns.bytes_total().into());
    }

    Ok(Value::Undefined)
}

fn play<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        let name = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;

        ns.play(activation.context, Some(name));
    }

    Ok(Value::Undefined)
}

fn pause<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        let action = args.get(0).cloned().unwrap_or(Value::Undefined);
        let is_pause = action.as_bool(activation.swf_version());

        if matches!(action, Value::Undefined) {
            ns.toggle_paused(activation.context);
        } else if is_pause {
            ns.pause(activation.context, true);
        } else {
            ns.resume(activation.context);
        }
    }

    Ok(Value::Undefined)
}

fn seek<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        let offset = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_f64(activation)?;

        ns.seek(activation.context, offset * 1000.0, false);
    }

    Ok(Value::Undefined)
}

fn set_buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        avm1_stub!(activation, "NetStream", "setBufferTime");

        let buffer_time = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_f64(activation)?;

        ns.set_buffer_time(buffer_time);
    }

    Ok(Value::Undefined)
}

fn get_time<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::NetStream(ns) = this.native() {
        return Ok((ns.time() / 1000.0).into());
    }

    Ok(Value::Undefined)
}

fn publish<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "publish");
    Ok(Value::Undefined)
}

fn play2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "play2");
    Ok(Value::Undefined)
}

fn receive_audio<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "receiveAudio");
    Ok(Value::Undefined)
}

fn receive_video<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "receiveVideo");
    Ok(Value::Undefined)
}

fn on_peer_connect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO This might be a settable property instead, but it needs more
    //   testing to confirm.
    avm1_stub!(activation, "NetStream", "onPeerConnect");
    Ok(Value::Bool(true))
}

fn close<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "close");
    Ok(Value::Undefined)
}

fn attach_audio<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "attachAudio");
    Ok(Value::Undefined)
}

fn attach_video<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "attachVideo");
    Ok(Value::Undefined)
}

fn send<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "send");
    Ok(Value::Undefined)
}

fn get_info<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "getInfo");
    Ok(Value::Undefined)
}

fn check_policy_file<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "checkPolicyFile");
    Ok(Value::Undefined)
}

fn max_pause_buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "maxPauseBufferTime");
    Ok(Value::Undefined)
}

fn back_buffer_time<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "NetStream", "backBufferTime");
    Ok(Value::Undefined)
}
