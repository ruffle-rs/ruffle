//! flash.filter.BlurFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::blur_filter::BlurFilterObject;
use crate::avm1::{Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_x = args
        .get(0)
        .unwrap_or(&4.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    let blur_y = args
        .get(1)
        .unwrap_or(&4.into())
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    let quality = args
        .get(2)
        .unwrap_or(&1.into())
        .coerce_to_i32(activation)
        .map(|x| x.max(0).min(15))?;

    let blur_filter = this.as_blur_filter_object().unwrap();

    blur_filter.set_blur_x(activation.context.gc_context, blur_x);
    blur_filter.set_blur_y(activation.context.gc_context, blur_y);
    blur_filter.set_quality(activation.context.gc_context, quality);

    Ok(Value::Undefined)
}

pub fn get_blur_x<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_blur_filter_object().unwrap().get_blur_x().into())
}

pub fn set_blur_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_blur_filter_object()
        .unwrap()
        .set_blur_x(activation.context.gc_context, blur_x);

    Ok(Value::Undefined)
}

pub fn get_blur_y<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_blur_filter_object().unwrap().get_blur_y().into())
}

pub fn set_blur_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_y = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(activation)
        .map(|x| x.max(0.0).min(255.0))?;

    this.as_blur_filter_object()
        .unwrap()
        .set_blur_y(activation.context.gc_context, blur_y);

    Ok(Value::Undefined)
}

pub fn get_quality<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_blur_filter_object().unwrap().get_quality().into())
}

pub fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_y = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)
        .map(|x| x.max(0).min(15))?;

    this.as_blur_filter_object()
        .unwrap()
        .set_quality(activation.context.gc_context, blur_y);

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let blur_filter = BlurFilterObject::empty_object(gc_context, Some(proto));
    let object = blur_filter.as_script_object().unwrap();

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

    blur_filter.into()
}
