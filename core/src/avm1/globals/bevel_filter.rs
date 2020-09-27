//! flash.filter.BevelFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::bevel_filter::{BevelFilterObject, BevelFilterType};
use crate::avm1::{AvmString, Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_distance(activation, this, args.get(0..1).unwrap_or(&[]))?;
    set_angle(activation, this, args.get(1..2).unwrap_or(&[]))?;
    set_highlight_color(activation, this, args.get(2..3).unwrap_or(&[]))?;
    set_highlight_alpha(activation, this, args.get(3..4).unwrap_or(&[]))?;
    set_shadow_color(activation, this, args.get(4..5).unwrap_or(&[]))?;
    set_shadow_alpha(activation, this, args.get(5..6).unwrap_or(&[]))?;
    set_blur_x(activation, this, args.get(6..7).unwrap_or(&[]))?;
    set_blur_y(activation, this, args.get(7..8).unwrap_or(&[]))?;
    set_strength(activation, this, args.get(8..9).unwrap_or(&[]))?;
    set_quality(activation, this, args.get(9..10).unwrap_or(&[]))?;
    set_type(activation, this, args.get(10..11).unwrap_or(&[]))?;
    set_knockout(activation, this, args.get(11..12).unwrap_or(&[]))?;

    Ok(Value::Undefined)
}

pub fn get_distance<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_distance().into())
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

    this.as_bevel_filter_object()
        .unwrap()
        .set_distance(activation.context.gc_context, distance);

    Ok(Value::Undefined)
}

pub fn get_angle<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_angle().into())
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

    this.as_bevel_filter_object()
        .unwrap()
        .set_angle(activation.context.gc_context, clamped_angle);

    Ok(Value::Undefined)
}

pub fn get_highlight_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_bevel_filter_object()
        .unwrap()
        .get_highlight_color()
        .into())
}

pub fn set_highlight_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let color = args
        .get(0)
        .unwrap_or(&0xFFFFFF.into())
        .coerce_to_u32(activation)?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_highlight_color(activation.context.gc_context, color & 0xFFFFFF);

    Ok(Value::Undefined)
}

pub fn get_highlight_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_bevel_filter_object()
        .unwrap()
        .get_highlight_alpha()
        .into())
}

pub fn set_highlight_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let highlight_alpha = args
        .get(0)
        .unwrap_or(&1.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(1.0))?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_highlight_alpha(activation.context.gc_context, highlight_alpha);

    Ok(Value::Undefined)
}

pub fn get_shadow_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_bevel_filter_object()
        .unwrap()
        .get_shadow_color()
        .into())
}

pub fn set_shadow_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let color = args
        .get(0)
        .unwrap_or(&0x000000.into())
        .coerce_to_u32(activation)?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_shadow_color(activation.context.gc_context, color & 0xFFFFFF);

    Ok(Value::Undefined)
}

pub fn get_shadow_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_bevel_filter_object()
        .unwrap()
        .get_shadow_alpha()
        .into())
}

pub fn set_shadow_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let alpha = args
        .get(0)
        .unwrap_or(&1.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(1.0))?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_shadow_alpha(activation.context.gc_context, alpha);

    Ok(Value::Undefined)
}

pub fn get_quality<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_quality().into())
}

pub fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_y = args
        .get(0)
        .unwrap_or(&1.into())
        .coerce_to_i32(activation)
        .map(|x| x.max(0).min(15))?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_quality(activation.context.gc_context, blur_y);

    Ok(Value::Undefined)
}

pub fn get_strength<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_strength().into())
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
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_strength(activation.context.gc_context, strength);

    Ok(Value::Undefined)
}

pub fn get_knockout<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_knockout().into())
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

    this.as_bevel_filter_object()
        .unwrap()
        .set_knockout(activation.context.gc_context, knockout);

    Ok(Value::Undefined)
}

pub fn get_blur_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_blur_x().into())
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
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_blur_x(activation.context.gc_context, blur_x);

    Ok(Value::Undefined)
}

pub fn get_blur_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bevel_filter_object().unwrap().get_blur_y().into())
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
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_bevel_filter_object()
        .unwrap()
        .set_blur_y(activation.context.gc_context, blur_y);

    Ok(Value::Undefined)
}

pub fn get_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let type_: &str = this.as_bevel_filter_object().unwrap().get_type().into();
    Ok(AvmString::new(activation.context.gc_context, type_.to_string()).into())
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

    this.as_bevel_filter_object()
        .unwrap()
        .set_type(activation.context.gc_context, type_);
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = BevelFilterObject::empty_object(gc_context, Some(proto));
    let script_object = object.as_script_object().unwrap();

    script_object.add_property(
        gc_context,
        "distance",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_distance),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_distance),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "angle",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_angle),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_angle),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "highlightColor",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_highlight_color),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_highlight_color),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "highlightAlpha",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_highlight_alpha),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_highlight_alpha),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "shadowColor",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_shadow_color),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_shadow_color),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "shadowAlpha",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_shadow_alpha),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_shadow_alpha),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "quality",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_quality),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_quality),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "strength",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_strength),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_strength),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "knockout",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_knockout),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_knockout),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "blurX",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_blur_x),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_blur_x),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
        gc_context,
        "blurY",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_blur_y),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_blur_y),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );
    script_object.add_property(
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
        EnumSet::empty(),
    );
    object.into()
}
