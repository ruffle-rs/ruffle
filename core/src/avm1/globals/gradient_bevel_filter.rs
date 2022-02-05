//! flash.filters.GradientBevelFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::bevel_filter::BevelFilterType;
use crate::avm1::object::gradient_bevel_filter::GradientBevelFilterObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::string::{AvmString, WStr};
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "distance" => property(distance, set_distance);
    "angle" => property(angle, set_angle);
    "colors" => property(colors, set_colors);
    "alphas" => property(alphas, set_alphas);
    "ratios" => property(ratios, set_ratios);
    "blurX" => property(blur_x, set_blur_x);
    "blurY" => property(blur_y, set_blur_y);
    "strength" => property(strength, set_strength);
    "quality" => property(quality, set_quality);
    "type" => property(get_type, set_type);
    "knockout" => property(knockout, set_knockout);
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_distance(activation, this, args.get(0..1).unwrap_or_default())?;
    set_angle(activation, this, args.get(1..2).unwrap_or_default())?;
    set_colors(activation, this, args.get(2..3).unwrap_or_default())?;
    set_alphas(activation, this, args.get(3..4).unwrap_or_default())?;
    set_ratios(activation, this, args.get(4..5).unwrap_or_default())?;
    set_blur_x(activation, this, args.get(5..6).unwrap_or_default())?;
    set_blur_y(activation, this, args.get(6..7).unwrap_or_default())?;
    set_strength(activation, this, args.get(7..8).unwrap_or_default())?;
    set_quality(activation, this, args.get(8..9).unwrap_or_default())?;
    set_type(activation, this, args.get(9..10).unwrap_or_default())?;
    set_knockout(activation, this, args.get(10..11).unwrap_or_default())?;

    Ok(this.into())
}

pub fn distance<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.distance().into());
    }

    Ok(Value::Undefined)
}

pub fn set_distance<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let distance = args.get(0).unwrap_or(&4.into()).coerce_to_f64(activation)?;

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_distance(activation.context.gc_context, distance);
    }

    Ok(Value::Undefined)
}

pub fn angle<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.angle().into());
    }

    Ok(Value::Undefined)
}

pub fn set_angle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let angle = args
        .get(0)
        .unwrap_or(&44.9999999772279.into())
        .coerce_to_f64(activation)?;

    let clamped_angle = if angle.is_sign_negative() {
        -(angle.abs() % 360.0)
    } else {
        angle % 360.0
    };

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_angle(activation.context.gc_context, clamped_angle);
    }

    Ok(Value::Undefined)
}

pub fn colors<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_gradient_bevel_filter_object() {
        return Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            filter.colors().iter().map(|&x| x.into()),
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn set_colors<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let colors = args.get(0).unwrap_or(&Value::Undefined);

    if let Value::Object(obj) = colors {
        if let Some(filter) = this.as_gradient_bevel_filter_object() {
            let arr_len = obj.length(activation)? as usize;
            let mut colors_arr = Vec::with_capacity(arr_len);

            let old_alphas = filter.alphas();
            let mut alphas_arr = Vec::with_capacity(arr_len);

            for i in 0..arr_len {
                let col = obj
                    .get_element(activation, i as i32)
                    .coerce_to_u32(activation)?;

                let alpha = if let Some(alpha) = old_alphas.get(i) {
                    *alpha
                } else if col >> 24 == 0 {
                    0.0
                } else {
                    255.0 / (col >> 24) as f64
                };

                colors_arr.push(col & 0xFFFFFF);
                alphas_arr.push(alpha);
            }

            filter.set_colors(activation.context.gc_context, colors_arr);
            filter.set_alphas(activation.context.gc_context, alphas_arr);

            let ratios = filter.ratios().into_iter().take(arr_len).collect();
            filter.set_ratios(activation.context.gc_context, ratios);
        }
    }

    Ok(Value::Undefined)
}

pub fn alphas<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_gradient_bevel_filter_object() {
        return Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            filter.alphas().iter().map(|&x| x.into()),
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn set_alphas<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let alphas = args.get(0).unwrap_or(&Value::Undefined);

    if let Value::Object(obj) = alphas {
        if let Some(filter) = this.as_gradient_bevel_filter_object() {
            let length = (obj.length(activation)? as usize).min(filter.colors().len());

            let alphas: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    Ok(obj
                        .get_element(activation, i as i32)
                        .coerce_to_f64(activation)?
                        .clamp(0.0, 1.0))
                })
                .collect();
            let alphas = alphas?;

            let colors = filter.colors().into_iter().take(length).collect();
            filter.set_colors(activation.context.gc_context, colors);

            let ratios = filter.ratios().into_iter().take(length).collect();
            filter.set_ratios(activation.context.gc_context, ratios);

            filter.set_alphas(activation.context.gc_context, alphas);
        }
    }

    Ok(Value::Undefined)
}

pub fn ratios<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_gradient_bevel_filter_object() {
        return Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            filter.ratios().iter().map(|&x| x.into()),
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn set_ratios<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let ratios = args.get(0).unwrap_or(&Value::Undefined);

    if let Value::Object(obj) = ratios {
        if let Some(filter) = this.as_gradient_bevel_filter_object() {
            let length = (obj.length(activation)? as usize).min(filter.colors().len());

            let ratios: Result<Vec<_>, Error<'gc>> = (0..length)
                .map(|i| {
                    Ok(obj
                        .get_element(activation, i as i32)
                        .coerce_to_i32(activation)?
                        .clamp(0, 255) as u8)
                })
                .collect();
            let ratios = ratios?;

            let colors = filter.colors().into_iter().take(length).collect();
            filter.set_colors(activation.context.gc_context, colors);

            let alphas = filter.alphas().into_iter().take(length).collect();
            filter.set_alphas(activation.context.gc_context, alphas);

            filter.set_ratios(activation.context.gc_context, ratios);
        }
    }

    Ok(Value::Undefined)
}

pub fn blur_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.blur_x().into());
    }

    Ok(Value::Undefined)
}

pub fn set_blur_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_x = args
        .get(0)
        .unwrap_or(&4.into())
        .coerce_to_f64(activation)
        .map(|x| x.clamp(0.0, 255.0))?;

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_blur_x(activation.context.gc_context, blur_x);
    }

    Ok(Value::Undefined)
}

pub fn blur_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.blur_y().into());
    }

    Ok(Value::Undefined)
}

pub fn set_blur_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_y = args
        .get(0)
        .unwrap_or(&4.into())
        .coerce_to_f64(activation)
        .map(|x| x.clamp(0.0, 255.0))?;

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_blur_y(activation.context.gc_context, blur_y);
    }

    Ok(Value::Undefined)
}

pub fn strength<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.strength().into());
    }

    Ok(Value::Undefined)
}

pub fn set_strength<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let strength = args
        .get(0)
        .unwrap_or(&1.into())
        .coerce_to_f64(activation)
        .map(|x| x.clamp(0.0, 255.0))?;

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_strength(activation.context.gc_context, strength);
    }

    Ok(Value::Undefined)
}

pub fn quality<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.quality().into());
    }

    Ok(Value::Undefined)
}

pub fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let quality = args
        .get(0)
        .unwrap_or(&1.into())
        .coerce_to_i32(activation)
        .map(|x| x.clamp(0, 15))?;

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_quality(activation.context.gc_context, quality);
    }

    Ok(Value::Undefined)
}

pub fn get_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(filter) = this.as_gradient_bevel_filter_object() {
        let type_: &WStr = filter.get_type().into();
        return Ok(AvmString::new(activation.context.gc_context, type_).into());
    }

    Ok(Value::Undefined)
}

pub fn set_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let type_: BevelFilterType = args
        .get(0)
        .unwrap_or(&"inner".into())
        .coerce_to_string(activation)
        .map(|s| s.as_wstr().into())?;

    if let Some(filter) = this.as_gradient_bevel_filter_object() {
        filter.set_type(activation.context.gc_context, type_);
    }

    Ok(Value::Undefined)
}

pub fn knockout<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_gradient_bevel_filter_object() {
        return Ok(object.knockout().into());
    }

    Ok(Value::Undefined)
}

pub fn set_knockout<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let knockout = args
        .get(0)
        .unwrap_or(&false.into())
        .as_bool(activation.swf_version());

    if let Some(object) = this.as_gradient_bevel_filter_object() {
        object.set_knockout(activation.context.gc_context, knockout);
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let color_matrix_filter = GradientBevelFilterObject::empty_object(gc_context, Some(proto));
    let object = color_matrix_filter.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    color_matrix_filter.into()
}
