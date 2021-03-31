//! flash.filter.GradientBevelFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::bevel_filter::BevelFilterType;
use crate::avm1::object::gradient_bevel_filter::GradientBevelFilterObject;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use gc_arena::MutationContext;

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
    let distance = args
        .get(0)
        .unwrap_or(&4.0.into())
        .coerce_to_f64(activation)?;

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
        let array = ScriptObject::array(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.array),
        );

        let arr = filter.colors();

        for (index, item) in arr.iter().copied().enumerate() {
            array.set_array_element(index, item.into(), activation.context.gc_context);
        }

        return Ok(array.into());
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
            let arr_len = obj.length();
            let mut colors_arr = Vec::with_capacity(arr_len);

            let old_alphas = filter.alphas();
            let mut alphas_arr = Vec::with_capacity(arr_len);

            for index in 0..arr_len {
                let col = obj.array_element(index).coerce_to_u32(activation)?;

                let alpha = if let Some(alpha) = old_alphas.get(index) {
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
        let array = ScriptObject::array(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.array),
        );

        let arr = filter.alphas();

        for (index, item) in arr.iter().copied().enumerate() {
            array.set_array_element(index, item.into(), activation.context.gc_context);
        }

        return Ok(array.into());
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
            let arr_len = obj.length().min(filter.colors().len());
            let mut arr = Vec::with_capacity(arr_len);

            for index in 0..arr_len {
                arr.push(
                    obj.array_element(index)
                        .coerce_to_f64(activation)?
                        .max(0.0)
                        .min(1.0),
                );
            }

            let colors = filter.colors().into_iter().take(arr_len).collect();
            filter.set_colors(activation.context.gc_context, colors);

            let ratios = filter.ratios().into_iter().take(arr_len).collect();
            filter.set_ratios(activation.context.gc_context, ratios);

            filter.set_alphas(activation.context.gc_context, arr);
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
        let array = ScriptObject::array(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.array),
        );

        let arr = filter.ratios();

        for (index, item) in arr.iter().copied().enumerate() {
            array.set_array_element(index, item.into(), activation.context.gc_context);
        }

        return Ok(array.into());
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
            let arr_len = obj.length().min(filter.colors().len());
            let mut arr = Vec::with_capacity(arr_len);

            for index in 0..arr_len {
                arr.push(
                    obj.array_element(index)
                        .coerce_to_i32(activation)?
                        .max(0)
                        .min(255) as u8,
                );
            }

            let colors = filter.colors().into_iter().take(arr_len).collect();
            filter.set_colors(activation.context.gc_context, colors);

            let alphas = filter.alphas().into_iter().take(arr_len).collect();
            filter.set_alphas(activation.context.gc_context, alphas);

            filter.set_ratios(activation.context.gc_context, arr);
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
        .unwrap_or(&4.0.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

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
        .unwrap_or(&4.0.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

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
        .unwrap_or(&1.0.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

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
        .unwrap_or(&1.0.into())
        .coerce_to_i32(activation)
        .map(|x| x.max(0).min(15))?;

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
        let type_: &str = filter.get_type().into();
        return Ok(AvmString::new(activation.context.gc_context, type_.to_string()).into());
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
        .unwrap_or(&Value::String(AvmString::new(
            activation.context.gc_context,
            "inner".to_string(),
        )))
        .coerce_to_string(activation)
        .map(|s| s.as_str().into())?;

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
        .as_bool(activation.current_swf_version());

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

    object.add_property(
        gc_context,
        "distance",
        FunctionObject::function(
            gc_context,
            Executable::Native(distance),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_distance),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "angle",
        FunctionObject::function(
            gc_context,
            Executable::Native(angle),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_angle),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "colors",
        FunctionObject::function(
            gc_context,
            Executable::Native(colors),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_colors),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "alphas",
        FunctionObject::function(
            gc_context,
            Executable::Native(alphas),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_alphas),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "ratios",
        FunctionObject::function(
            gc_context,
            Executable::Native(ratios),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_ratios),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "blurX",
        FunctionObject::function(
            gc_context,
            Executable::Native(blur_x),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_blur_x),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "blurY",
        FunctionObject::function(
            gc_context,
            Executable::Native(blur_y),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_blur_y),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "strength",
        FunctionObject::function(
            gc_context,
            Executable::Native(strength),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_strength),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "quality",
        FunctionObject::function(
            gc_context,
            Executable::Native(quality),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_quality),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "type",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_type),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_type),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    object.add_property(
        gc_context,
        "knockout",
        FunctionObject::function(
            gc_context,
            Executable::Native(knockout),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_knockout),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::empty(),
    );

    color_matrix_filter.into()
}
