//! flash.filters.BitmapFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::bevel_filter::BevelFilter;
use crate::avm1::globals::blur_filter::BlurFilter;
use crate::avm1::globals::color_matrix_filter::ColorMatrixFilter;
use crate::avm1::globals::convolution_filter::ConvolutionFilter;
use crate::avm1::globals::displacement_map_filter::DisplacementMapFilter;
use crate::avm1::globals::drop_shadow_filter::DropShadowFilter;
use crate::avm1::globals::glow_filter::GlowFilter;
use crate::avm1::globals::gradient_filter::GradientFilter;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{Attribute, Object, Value};
use crate::context::UpdateContext;
use ruffle_macros::istr;
use ruffle_render::filters::Filter;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "clone" => method(clone);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    class
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let native = match this.native() {
        NativeObject::BlurFilter(blur_filter) => {
            NativeObject::BlurFilter(blur_filter.duplicate(activation.gc()))
        }
        NativeObject::BevelFilter(bevel_filter) => {
            NativeObject::BevelFilter(bevel_filter.duplicate(activation.gc()))
        }
        NativeObject::GlowFilter(glow_filter) => {
            NativeObject::GlowFilter(glow_filter.duplicate(activation.gc()))
        }
        NativeObject::DropShadowFilter(drop_shadow_filter) => {
            NativeObject::DropShadowFilter(drop_shadow_filter.duplicate(activation.gc()))
        }
        NativeObject::ColorMatrixFilter(color_matrix_filter) => {
            NativeObject::ColorMatrixFilter(color_matrix_filter.duplicate(activation.gc()))
        }
        NativeObject::DisplacementMapFilter(displacement_map_filter) => {
            NativeObject::DisplacementMapFilter(displacement_map_filter.duplicate(activation.gc()))
        }
        NativeObject::ConvolutionFilter(convolution_filter) => {
            NativeObject::ConvolutionFilter(convolution_filter.duplicate(activation.gc()))
        }
        NativeObject::GradientBevelFilter(gradient_bevel_filter) => {
            NativeObject::GradientBevelFilter(gradient_bevel_filter.duplicate(activation.gc()))
        }
        NativeObject::GradientGlowFilter(gradient_glow_filter) => {
            NativeObject::GradientGlowFilter(gradient_glow_filter.duplicate(activation.gc()))
        }
        _ => return Ok(Value::Undefined),
    };
    let proto = this.get_local_stored(istr!("__proto__"), activation);
    Ok(create_instance(activation, native, proto).into())
}

pub fn avm1_to_filter<'gc>(
    object: Object<'gc>,
    context: &mut UpdateContext<'gc>,
) -> Option<Filter> {
    let native = object.native();
    match native {
        NativeObject::BevelFilter(filter) => Some(Filter::BevelFilter(filter.filter())),
        NativeObject::BlurFilter(filter) => Some(Filter::BlurFilter(filter.filter())),
        NativeObject::ColorMatrixFilter(filter) => Some(Filter::ColorMatrixFilter(filter.filter())),
        NativeObject::ConvolutionFilter(filter) => Some(Filter::ConvolutionFilter(filter.filter())),
        NativeObject::GlowFilter(filter) => Some(Filter::GlowFilter(filter.filter())),
        NativeObject::DropShadowFilter(filter) => Some(Filter::DropShadowFilter(filter.filter())),
        NativeObject::DisplacementMapFilter(filter) => {
            Some(Filter::DisplacementMapFilter(filter.filter(context)))
        }
        NativeObject::GradientBevelFilter(filter) => {
            Some(Filter::GradientBevelFilter(filter.filter()))
        }
        NativeObject::GradientGlowFilter(filter) => {
            Some(Filter::GradientGlowFilter(filter.filter()))
        }

        // Invalid filters are silently dropped/ignored, no errors are thrown.
        _ => None,
    }
}

pub fn filter_to_avm1<'gc>(activation: &mut Activation<'_, 'gc>, filter: Filter) -> Value<'gc> {
    let (native, proto) = match filter {
        Filter::BevelFilter(filter) => (
            NativeObject::BevelFilter(BevelFilter::from_filter(activation.gc(), filter)),
            activation.prototypes().bevel_filter,
        ),
        Filter::BlurFilter(filter) => (
            NativeObject::BlurFilter(BlurFilter::from_filter(activation.gc(), filter)),
            activation.prototypes().blur_filter,
        ),
        Filter::ColorMatrixFilter(filter) => (
            NativeObject::ColorMatrixFilter(ColorMatrixFilter::from_filter(
                activation.gc(),
                filter,
            )),
            activation.prototypes().color_matrix_filter,
        ),
        Filter::ConvolutionFilter(filter) => (
            NativeObject::ConvolutionFilter(ConvolutionFilter::from_filter(
                activation.gc(),
                filter,
            )),
            activation.prototypes().convolution_filter,
        ),
        Filter::GlowFilter(filter) => (
            NativeObject::GlowFilter(GlowFilter::from_filter(activation.gc(), filter)),
            activation.prototypes().glow_filter,
        ),
        Filter::DropShadowFilter(filter) => (
            NativeObject::DropShadowFilter(DropShadowFilter::from_filter(activation.gc(), filter)),
            activation.prototypes().drop_shadow_filter,
        ),
        Filter::DisplacementMapFilter(filter) => (
            NativeObject::DisplacementMapFilter(DisplacementMapFilter::from_filter(
                activation.gc(),
                filter,
            )),
            activation.prototypes().displacement_map_filter,
        ),
        Filter::GradientBevelFilter(filter) => (
            NativeObject::GradientBevelFilter(GradientFilter::from_filter(activation.gc(), filter)),
            activation.prototypes().gradient_bevel_filter,
        ),
        Filter::GradientGlowFilter(filter) => (
            NativeObject::GradientGlowFilter(GradientFilter::from_filter(activation.gc(), filter)),
            activation.prototypes().gradient_glow_filter,
        ),
        Filter::ShaderFilter(_) => {
            unreachable!(
                "There should be no way for a ShaderFilter to exist in AVM1-reachable world"
            )
        }
    };

    create_instance(activation, native, Some(proto.into())).into()
}

pub fn create_instance<'gc>(
    activation: &mut Activation<'_, 'gc>,
    native: NativeObject<'gc>,
    proto: Option<Value<'gc>>,
) -> Object<'gc> {
    let result = Object::new(activation.strings(), None);
    // Set `__proto__` manually since `Object::new()` doesn't support primitive prototypes.
    // TODO: Pass `proto` to `Object::new()` once possible.
    if let Some(proto) = proto {
        result.define_value(
            activation.gc(),
            istr!("__proto__"),
            proto,
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );
    }
    result.set_native(activation.gc(), native);
    result
}
