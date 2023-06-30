//! flash.filters.BitmapFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Attribute, Object, ScriptObject, TObject, Value};
use crate::context::GcContext;
use ruffle_render::filters::Filter;

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
    Ok(create_instance(activation, native, proto).into())
}

pub fn avm1_to_filter(_object: Object) -> Option<Filter> {
    // TODO

    // Invalid filters are silently dropped/ignored, no errors are thrown.
    None
}

pub fn filter_to_avm1<'gc>(_activation: &mut Activation<'_, 'gc>, _filter: Filter) -> Value<'gc> {
    // TODO

    // Unrepresentable filters (eg Shader) will just return as Null.
    // Not sure there's a way to even get to that state though, they can only be added in avm2.
    Value::Null
}

pub fn create_instance<'gc>(
    activation: &mut Activation<'_, 'gc>,
    native: NativeObject<'gc>,
    proto: Option<Value<'gc>>,
) -> ScriptObject<'gc> {
    let result = ScriptObject::new(activation.context.gc_context, None);
    // Set `__proto__` manually since `ScriptObject::new()` doesn't support primitive prototypes.
    // TODO: Pass `proto` to `ScriptObject::new()` once possible.
    if let Some(proto) = proto {
        result.define_value(
            activation.context.gc_context,
            "__proto__",
            proto,
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );
    }
    result.set_native(activation.context.gc_context, native);
    result
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
