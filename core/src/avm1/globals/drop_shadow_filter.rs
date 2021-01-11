//! flash.filter.DropShadowFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::drop_shadow_filter::DropShadowFilterObject;
use crate::avm1::{Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_distance(activation, this, args.get(0..1).unwrap_or_default())?;
    set_angle(activation, this, args.get(1..2).unwrap_or_default())?;
    set_color(activation, this, args.get(2..3).unwrap_or_default())?;
    set_alpha(activation, this, args.get(3..4).unwrap_or_default())?;
    set_blur_x(activation, this, args.get(4..5).unwrap_or_default())?;
    set_blur_y(activation, this, args.get(5..6).unwrap_or_default())?;
    set_strength(activation, this, args.get(6..7).unwrap_or_default())?;
    set_quality(activation, this, args.get(7..8).unwrap_or_default())?;
    set_inner(activation, this, args.get(8..9).unwrap_or_default())?;
    set_knockout(activation, this, args.get(9..10).unwrap_or_default())?;
    set_hide_object(activation, this, args.get(10..11).unwrap_or_default())?;

    Ok(this.into())
}

pub fn alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
        return Ok(object.alpha().into());
    }

    Ok(Value::Undefined)
}

pub fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let alpha = args
        .get(0)
        .unwrap_or(&1.0.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(1.0))?;

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_alpha(activation.context.gc_context, alpha);
    }

    Ok(Value::Undefined)
}

pub fn distance<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_distance(activation.context.gc_context, distance);
    }

    Ok(Value::Undefined)
}

pub fn angle<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_angle(activation.context.gc_context, clamped_angle);
    }

    Ok(Value::Undefined)
}

pub fn blur_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_blur_x(activation.context.gc_context, blur_x);
    }

    Ok(Value::Undefined)
}

pub fn blur_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_blur_y(activation.context.gc_context, blur_y);
    }

    Ok(Value::Undefined)
}

pub fn color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
        return Ok(object.color().into());
    }

    Ok(Value::Undefined)
}

pub fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let color = args
        .get(0)
        .unwrap_or(&0x000000.into())
        .coerce_to_u32(activation)?;

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_color(activation.context.gc_context, color & 0xFFFFFF);
    }

    Ok(Value::Undefined)
}

pub fn hide_object<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
        return Ok(object.hide_object().into());
    }

    Ok(Value::Undefined)
}

pub fn set_hide_object<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let hide_object = args
        .get(0)
        .unwrap_or(&false.into())
        .as_bool(activation.current_swf_version());

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_hide_object(activation.context.gc_context, hide_object);
    }

    Ok(Value::Undefined)
}

pub fn inner<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
        return Ok(object.inner().into());
    }

    Ok(Value::Undefined)
}

pub fn set_inner<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let inner = args
        .get(0)
        .unwrap_or(&false.into())
        .as_bool(activation.current_swf_version());

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_inner(activation.context.gc_context, inner);
    }

    Ok(Value::Undefined)
}

pub fn knockout<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_knockout(activation.context.gc_context, knockout);
    }

    Ok(Value::Undefined)
}

pub fn quality<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_quality(activation.context.gc_context, quality);
    }

    Ok(Value::Undefined)
}

pub fn strength<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(object) = this.as_drop_shadow_filter_object() {
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

    if let Some(object) = this.as_drop_shadow_filter_object() {
        object.set_strength(activation.context.gc_context, strength);
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let drop_shadow_filter = DropShadowFilterObject::empty_object(gc_context, Some(proto));
    let object = drop_shadow_filter.as_script_object().unwrap();

    object.add_property(
        gc_context,
        "alpha",
        FunctionObject::function(
            gc_context,
            Executable::Native(alpha),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_alpha),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
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
        EnumSet::empty(),
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
        EnumSet::empty(),
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
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "color",
        FunctionObject::function(
            gc_context,
            Executable::Native(color),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_color),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

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
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "hideObject",
        FunctionObject::function(
            gc_context,
            Executable::Native(hide_object),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_hide_object),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "inner",
        FunctionObject::function(
            gc_context,
            Executable::Native(inner),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_inner),
            Some(fn_proto),
            fn_proto,
        )),
        EnumSet::empty(),
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
        EnumSet::empty(),
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
        EnumSet::empty(),
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
        EnumSet::empty(),
    );

    drop_shadow_filter.into()
}
