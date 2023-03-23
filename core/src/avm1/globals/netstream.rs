use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::{NativeObject, Object, TObject};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, ScriptObject, Value};
use crate::streams::NetStream;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let netstream = NetStream::new(activation.context.gc_context);
    this.set_native(
        activation.context.gc_context,
        NativeObject::NetStream(netstream),
    );

    Ok(Value::Undefined)
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "bytesLoaded" => property(get_bytes_loaded);
    "bytesTotal" => property(get_bytes_total);
    "play" => method(play; DONT_ENUM | DONT_DELETE);
};

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
        return Ok(ns.bytes_loaded().into());
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

        ns.play(&mut activation.context, Some(name));
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    object.into()
}

pub fn create_class<'gc>(
    gc_context: MutationContext<'gc, '_>,
    netstream_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        netstream_proto,
    )
}
