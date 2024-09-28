use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::{NativeObject, Object, TObject};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, ScriptObject, Value};
use crate::avm1_stub;
use crate::streams::NetStream;
use crate::string::StringContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let netstream = NetStream::new(activation.context.gc_context, Some(this.into()));
    this.set_native(
        activation.context.gc_context,
        NativeObject::NetStream(netstream),
    );

    Ok(this.into())
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "bufferLength" => property(get_buffer_length);
    "bufferTime" => property(get_buffer_time);
    "bytesLoaded" => property(get_bytes_loaded);
    "bytesTotal" => property(get_bytes_total);
    "time" => property(get_time);
    "play" => method(play; DONT_ENUM | DONT_DELETE);
    "pause" => method(pause; DONT_ENUM | DONT_DELETE);
    "seek" => method(seek; DONT_ENUM | DONT_DELETE);
    "setBufferTime" => method(set_buffer_time; DONT_ENUM | DONT_DELETE);
};

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

        ns.set_buffer_time(activation.context.gc_context, buffer_time);
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

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}

pub fn create_class<'gc>(
    context: &mut StringContext<'gc>,
    netstream_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        netstream_proto,
    )
}
