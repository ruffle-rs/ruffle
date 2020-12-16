//! flash.filter.GlowFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::glow_filter::GlowFilterObject;
use crate::avm1::{Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    set_color(activation, this, args.get(0..1).unwrap_or(&[]))?;
    set_alpha(activation, this, args.get(1..2).unwrap_or(&[]))?;
    set_blur_x(activation, this, args.get(2..3).unwrap_or(&[]))?;
    set_blur_y(activation, this, args.get(3..4).unwrap_or(&[]))?;
    set_strength(activation, this, args.get(4..5).unwrap_or(&[]))?;
    set_quality(activation, this, args.get(5..6).unwrap_or(&[]))?;

    Ok(Value::Undefined)
}

pub fn get_alpha<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_alpha().into())
}

pub fn set_alpha<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let alpha = args
        .get(0)
        .unwrap_or(&Value::Number(1.0))
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(1.0))?;

    this.as_glow_filter_object()
        .unwrap()
        .set_alpha(activation.context.gc_context, alpha);

    Ok(Value::Undefined)
}

pub fn get_blur_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_blur_x().into())
}

pub fn set_blur_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_x = args
        .get(0)
        .unwrap_or(&Value::Number(6.0))
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_glow_filter_object()
        .unwrap()
        .set_blur_x(activation.context.gc_context, blur_x);

    Ok(Value::Undefined)
}

pub fn get_blur_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_blur_y().into())
}

pub fn set_blur_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_y = args
        .get(0)
        .unwrap_or(&Value::Number(6.0))
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_glow_filter_object()
        .unwrap()
        .set_blur_y(activation.context.gc_context, blur_y);

    Ok(Value::Undefined)
}

pub fn get_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_color().into())
}

pub fn set_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let color = args
        .get(0)
        .unwrap_or(&Value::Number(0xFF0000.into()))
        .coerce_to_i32(activation)
        .map(|x| x.max(1).min(0xFFFFFF))?;

    this.as_glow_filter_object()
        .unwrap()
        .set_color(activation.context.gc_context, color);

    Ok(Value::Undefined)
}

pub fn get_inner<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_inner().into())
}

pub fn set_inner<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let inner = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .as_bool(activation.current_swf_version());

    this.as_glow_filter_object()
        .unwrap()
        .set_inner(activation.context.gc_context, inner);

    Ok(Value::Undefined)
}

pub fn get_knockout<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_knockout().into())
}

pub fn set_knockout<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let knockout = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .as_bool(activation.current_swf_version());

    this.as_glow_filter_object()
        .unwrap()
        .set_knockout(activation.context.gc_context, knockout);

    Ok(Value::Undefined)
}

pub fn get_quality<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_quality().into())
}

pub fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_y = args
        .get(0)
        .unwrap_or(&Value::Number(1.0))
        .coerce_to_i32(activation)
        .map(|x| x.max(0).min(15))?;

    this.as_glow_filter_object()
        .unwrap()
        .set_quality(activation.context.gc_context, blur_y);

    Ok(Value::Undefined)
}

pub fn get_strength<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_glow_filter_object().unwrap().get_strength().into())
}

pub fn set_strength<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let strength = args
        .get(0)
        .unwrap_or(&Value::Number(2.0))
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_glow_filter_object()
        .unwrap()
        .set_strength(activation.context.gc_context, strength);

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let glow_filter = GlowFilterObject::empty_object(gc_context, Some(proto));
    let object = glow_filter.as_script_object().unwrap();

    object.add_property(
        gc_context,
        "alpha",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_alpha),
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

    object.add_property(
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

    object.add_property(
        gc_context,
        "color",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_color),
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
        "inner",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_inner),
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

    object.add_property(
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

    object.add_property(
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

    glow_filter.into()
}
