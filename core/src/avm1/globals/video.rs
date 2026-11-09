//! Video class

use crate::avm1::Object;
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{
    self, DeclContext, PropertyOrder, StaticDeclarations, SystemClass,
};
use crate::avm1::value::Value;
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

/// Defined lazily, see `define_lazy_proto_properties`.
const LAZY_PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "width" => property(video_method!(width); VERSION_6);
    "height" => property(video_method!(height); VERSION_6);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto, PropertyOrder::PrototypeFirst);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

/// Defines `width` and `height` on `proto`, the prototype of a newly created
/// Video display object (`new Video()` doesn't count).
///
/// See [`property_decl::define_lazy_proto_properties`].
pub fn define_lazy_proto_properties<'gc>(activation: &mut Activation<'_, 'gc>, proto: Object<'gc>) {
    property_decl::define_lazy_proto_properties(activation, proto, LAZY_PROTO_DECLS);
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

fn width<'gc>(
    video: Video<'gc>,
    _activation: &mut Activation<'_, 'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(video.decoded_size().0.into())
}

fn height<'gc>(
    video: Video<'gc>,
    _activation: &mut Activation<'_, 'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(video.decoded_size().1.into())
}
