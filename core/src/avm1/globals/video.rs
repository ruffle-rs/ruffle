//! Video class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::{NativeObject, Object, TObject};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::value::Value;
use crate::avm1::ScriptObject;
use crate::context::GcContext;
use crate::display_object::{TDisplayObject, Video};

macro_rules! video_method {
    ( $fn: expr ) => {
        |activation, this, args| {
            if let Some(display_object) = this.as_display_object() {
                if let Some(video) = display_object.as_video() {
                    return $fn(video, activation, args);
                }
            }
            Ok(Value::Undefined)
        }
    };
}

/// Implements `Video`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "attachVideo" => method(video_method!(attach_video); DONT_ENUM | DONT_DELETE | VERSION_6);
};

pub fn attach_video<'gc>(
    video: Video<'gc>,
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let source = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object(activation);

    if let NativeObject::NetStream(ns) = source.native() {
        video.attach_netstream(activation.context, ns);
    } else {
        tracing::warn!("Cannot use object of type {:?} as video source", source);
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
