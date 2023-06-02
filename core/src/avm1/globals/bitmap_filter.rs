//! flash.filters.BitmapFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Attribute, Object, ScriptObject, TObject, Value};
use crate::context::GcContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "clone" => method(clone);
};

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let native = match this.native() {
        NativeObject::BlurFilter(blur_filter) => {
            NativeObject::BlurFilter(blur_filter.duplicate(activation.context.gc_context))
        }
        NativeObject::BevelFilter(bevel_filter) => {
            NativeObject::BevelFilter(bevel_filter.duplicate(activation.context.gc_context))
        }
        NativeObject::GlowFilter(glow_filter) => {
            NativeObject::GlowFilter(glow_filter.duplicate(activation.context.gc_context))
        }
        NativeObject::DropShadowFilter(drop_shadow_filter) => NativeObject::DropShadowFilter(
            drop_shadow_filter.duplicate(activation.context.gc_context),
        ),
        NativeObject::ColorMatrixFilter(color_matrix_filter) => NativeObject::ColorMatrixFilter(
            color_matrix_filter.duplicate(activation.context.gc_context),
        ),
        NativeObject::DisplacementMapFilter(displacement_map_filter) => {
            NativeObject::DisplacementMapFilter(
                displacement_map_filter.duplicate(activation.context.gc_context),
            )
        }
        NativeObject::ConvolutionFilter(convolution_filter) => NativeObject::ConvolutionFilter(
            convolution_filter.duplicate(activation.context.gc_context),
        ),
        NativeObject::GradientBevelFilter(gradient_bevel_filter) => {
            NativeObject::GradientBevelFilter(
                gradient_bevel_filter.duplicate(activation.context.gc_context),
            )
        }
        NativeObject::GradientGlowFilter(gradient_glow_filter) => NativeObject::GradientGlowFilter(
            gradient_glow_filter.duplicate(activation.context.gc_context),
        ),
        _ => return Ok(Value::Undefined),
    };
    let proto = this.get_local_stored("__proto__", activation);
    let cloned = ScriptObject::new(activation.context.gc_context, None);
    // Set `__proto__` manually since `ScriptObject::new()` doesn't support primitive prototypes.
    // TODO: Pass `proto` to `ScriptObject::new()` once possible.
    if let Some(proto) = proto {
        cloned.define_value(
            activation.context.gc_context,
            "__proto__",
            proto,
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );
    }
    cloned.set_native(activation.context.gc_context, native);
    Ok(cloned.into())
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
