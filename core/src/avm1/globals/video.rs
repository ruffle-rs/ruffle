//! Video class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::value::Value;
use crate::avm1::Object;
use crate::display_object::Video;

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

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "attachVideo" => method(video_method!(attach_video); DONT_ENUM | DONT_DELETE | VERSION_6);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

pub fn attach_video<'gc>(
    video: Video<'gc>,
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let source = args
        .get(0)
        .cloned()
        .unwrap_or(Value::Undefined)
        .coerce_to_object_or_bare(activation)?;

    if let NativeObject::NetStream(ns) = source.native() {
        video.attach_netstream(activation.context, ns);
    } else {
        tracing::warn!("Cannot use object of type {:?} as video source", source);
    }

    Ok(Value::Undefined)
}
