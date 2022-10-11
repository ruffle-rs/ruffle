//! flash.filters.BitmapFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Attribute, Object, ScriptObject, TObject, Value};
use gc_arena::{GcCell, MutationContext};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "clone" => method(clone);
};

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let native = match this.native() {
        NativeObject::BlurFilter(blur_filter) => NativeObject::BlurFilter(GcCell::allocate(
            activation.context.gc_context,
            blur_filter.read().clone(),
        )),
        _ => NativeObject::None,
    };
    if !matches!(native, NativeObject::None) {
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
        return Ok(cloned.into());
    }

    if let Some(this) = this.as_bevel_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .bevel_filter_constructor;

        let distance = this.get("distance", activation)?;
        let angle = this.get("angle", activation)?;
        let highlight_color = this.get("highlightColor", activation)?;
        let highlight_alpha = this.get("highlightAlpha", activation)?;
        let shadow_color = this.get("shadowColor", activation)?;
        let shadow_alpha = this.get("shadowAlpha", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;
        let type_ = this.get("type", activation)?;
        let knockout = this.get("knockout", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                distance,
                angle,
                highlight_color,
                highlight_alpha,
                shadow_color,
                shadow_alpha,
                blur_x,
                blur_y,
                strength,
                quality,
                type_,
                knockout,
            ],
        )?;
        return Ok(cloned);
    }

    if let Some(this) = this.as_glow_filter_object() {
        let proto = activation.context.avm1.prototypes().glow_filter_constructor;

        let color = this.get("color", activation)?;
        let alpha = this.get("alpha", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;

        let cloned = proto.construct(
            activation,
            &[color, alpha, blur_x, blur_y, strength, quality],
        )?;
        return Ok(cloned);
    }

    if let Some(this) = this.as_drop_shadow_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .drop_shadow_filter_constructor;

        let distance = this.get("distance", activation)?;
        let angle = this.get("angle", activation)?;
        let color = this.get("color", activation)?;
        let alpha = this.get("alpha", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;
        let inner = this.get("inner", activation)?;
        let knockout = this.get("knockout", activation)?;
        let hide_object = this.get("hide_object", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                distance,
                angle,
                color,
                alpha,
                blur_x,
                blur_y,
                strength,
                quality,
                inner,
                knockout,
                hide_object,
            ],
        )?;
        return Ok(cloned);
    }

    if let Some(this) = this.as_color_matrix_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .color_matrix_filter_constructor;

        let matrix = this.get("matrix", activation)?;

        let cloned = proto.construct(activation, &[matrix])?;

        return Ok(cloned);
    }

    if let Some(this) = this.as_displacement_map_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .displacement_map_filter_constructor;

        let map_bitmap = this.get("mapBitmap", activation)?;
        let map_point = this.get("mapPoint", activation)?;
        let component_x = this.get("componentX", activation)?;
        let component_y = this.get("componentY", activation)?;
        let scale_x = this.get("scaleX", activation)?;
        let scale_y = this.get("scaleY", activation)?;
        let mode = this.get("mode", activation)?;
        let color = this.get("color", activation)?;
        let alpha = this.get("alpha", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                map_bitmap,
                map_point,
                component_x,
                component_y,
                scale_x,
                scale_y,
                mode,
                color,
                alpha,
            ],
        )?;

        return Ok(cloned);
    }

    if let Some(this) = this.as_convolution_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .convolution_filter_constructor;

        let matrix_x = this.get("matrixX", activation)?;
        let matrix_y = this.get("matrixY", activation)?;
        let matrix = this.get("matrix", activation)?;
        let divisor = this.get("divisor", activation)?;
        let bias = this.get("bias", activation)?;
        let preserve_alpha = this.get("preserveAlpha", activation)?;
        let clamp = this.get("clamp", activation)?;
        let color = this.get("color", activation)?;
        let alpha = this.get("alpha", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                matrix_x,
                matrix_y,
                matrix,
                divisor,
                bias,
                preserve_alpha,
                clamp,
                color,
                alpha,
            ],
        )?;

        return Ok(cloned);
    }

    if let Some(this) = this.as_gradient_bevel_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .gradient_bevel_filter_constructor;

        let distance = this.get("distance", activation)?;
        let angle = this.get("angle", activation)?;
        let colors = this.get("colors", activation)?;
        let alphas = this.get("alphas", activation)?;
        let ratios = this.get("ratios", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;
        let type_ = this.get("type", activation)?;
        let knockout = this.get("knockout", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                distance, angle, colors, alphas, ratios, blur_x, blur_y, strength, quality, type_,
                knockout,
            ],
        )?;

        return Ok(cloned);
    }

    if let Some(this) = this.as_gradient_glow_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes()
            .gradient_glow_filter_constructor;

        let distance = this.get("distance", activation)?;
        let angle = this.get("angle", activation)?;
        let colors = this.get("colors", activation)?;
        let alphas = this.get("alphas", activation)?;
        let ratios = this.get("ratios", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;
        let type_ = this.get("type", activation)?;
        let knockout = this.get("knockout", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                distance, angle, colors, alphas, ratios, blur_x, blur_y, strength, quality, type_,
                knockout,
            ],
        )?;

        return Ok(cloned);
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
