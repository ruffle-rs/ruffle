//! flash.geom.Point

use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::f64::NAN;

pub fn point_to_object<'gc>(
    point: (f64, f64),
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let args = [point.0.into(), point.1.into()];
    construct_new_point(&args, avm, context)
}

pub fn construct_new_point<'gc>(
    args: &[Value<'gc>],
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let proto = context.system_prototypes.point;
    let object = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, object, &args)?;
    Ok(object)
}

pub fn value_to_point<'gc>(
    value: Value<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<(f64, f64), Error> {
    let x = value
        .coerce_to_object(avm, context)
        .get("x", avm, context)?
        .coerce_to_f64(avm, context)?;
    let y = value
        .coerce_to_object(avm, context)
        .get("y", avm, context)?
        .coerce_to_f64(avm, context)?;
    Ok((x, y))
}

pub fn object_to_point<'gc>(
    object: Object<'gc>,
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
) -> Result<(f64, f64), Error> {
    let x = object.get("x", avm, context)?.coerce_to_f64(avm, context)?;
    let y = object.get("y", avm, context)?.coerce_to_f64(avm, context)?;
    Ok((x, y))
}

fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        this.set("x", 0.into(), avm, context)?;
        this.set("y", 0.into(), avm, context)?;
    } else {
        this.set(
            "x",
            args.get(0).unwrap_or(&Value::Undefined).to_owned(),
            avm,
            context,
        )?;
        this.set(
            "y",
            args.get(1).unwrap_or(&Value::Undefined).to_owned(),
            avm,
            context,
        )?;
    }

    Ok(Value::Undefined.into())
}

fn clone<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let proto = context.system_prototypes.point;
    let args = [this.get("x", avm, context)?, this.get("y", avm, context)?];
    let cloned = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, cloned, &args)?;

    Ok(cloned.into())
}

fn equals<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(other) = args.get(0) {
        let this_x = this.get("x", avm, context)?;
        let this_y = this.get("y", avm, context)?;
        let other = other.coerce_to_object(avm, context);
        let other_x = other.get("x", avm, context)?;
        let other_y = other.get("y", avm, context)?;
        return Ok((this_x == other_x && this_y == other_y).into());
    }

    Ok(false.into())
}

fn add<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this_x = this.get("x", avm, context)?.coerce_to_f64(avm, context)?;
    let this_y = this.get("y", avm, context)?.coerce_to_f64(avm, context)?;
    let other = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        avm,
        context,
    )?;
    let object = point_to_object((this_x + other.0, this_y + other.1), avm, context)?;
    Ok(object.into())
}

fn subtract<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this_x = this.get("x", avm, context)?.coerce_to_f64(avm, context)?;
    let this_y = this.get("y", avm, context)?.coerce_to_f64(avm, context)?;
    let other = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        avm,
        context,
    )?;
    let object = point_to_object((this_x - other.0, this_y - other.1), avm, context)?;
    Ok(object.into())
}

fn distance<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.len() < 2 {
        return Ok(NAN.into());
    }

    let a = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(avm, context);
    let b = args.get(1).unwrap_or(&Value::Undefined);
    let delta = a.call_method("subtract", &[b.to_owned()], avm, context)?;
    Ok(delta
        .coerce_to_object(avm, context)
        .get("length", avm, context)?
        .into())
}

fn polar<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let length = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let angle = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let point = point_to_object((length * angle.cos(), length * angle.sin()), avm, context)?;
    Ok(point.into())
}

fn interpolate<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.len() < 3 {
        return Ok(point_to_object((NAN, NAN), avm, context)?.into());
    }

    let a = value_to_point(args.get(0).unwrap().to_owned(), avm, context)?;
    let b = value_to_point(args.get(1).unwrap().to_owned(), avm, context)?;
    let f = args.get(2).unwrap().coerce_to_f64(avm, context)?;
    let result = (b.0 - (b.0 - a.0) * f, b.1 - (b.1 - a.1) * f);
    Ok(point_to_object(result, avm, context)?.into())
}

fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this.get("x", avm, context)?;
    let y = this.get("y", avm, context)?;

    Ok(format!(
        "(x={}, y={})",
        x.coerce_to_string(avm, context)?,
        y.coerce_to_string(avm, context)?
    )
    .into())
}

fn length<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let point = value_to_point(this.into(), avm, context)?;
    let length = (point.0 * point.0 + point.1 * point.1).sqrt();
    Ok(length.into())
}

fn normalize<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let current_length = this
        .get("length", avm, context)?
        .coerce_to_f64(avm, context)?;
    if current_length.is_finite() {
        let point = object_to_point(this, avm, context)?;
        let new_length = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(avm, context)?;
        let (x, y) = if current_length == 0.0 {
            (point.0 * new_length, point.1 * new_length)
        } else {
            (
                point.0 / current_length * new_length,
                point.1 / current_length * new_length,
            )
        };

        this.set("x", x.into(), avm, context)?;
        this.set("y", y.into(), avm, context)?;
    }

    Ok(Value::Undefined.into())
}

fn offset<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let point = value_to_point(this.into(), avm, context)?;
    let dx = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;
    let dy = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_f64(avm, context)?;

    this.set("x", (point.0 + dx).into(), avm, context)?;
    this.set("y", (point.1 + dy).into(), avm, context)?;

    Ok(Value::Undefined.into())
}

pub fn create_point_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    point_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let point = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        point_proto,
    );
    let mut object = point.as_script_object().unwrap();

    object.force_set_function("distance", distance, gc_context, EnumSet::empty(), fn_proto);
    object.force_set_function("polar", polar, gc_context, EnumSet::empty(), fn_proto);
    object.force_set_function(
        "interpolate",
        interpolate,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    point
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "equals",
        equals,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("add", add, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "subtract",
        subtract,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "normalize",
        normalize,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "offset",
        offset,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "length",
        Executable::Native(length),
        None,
        Attribute::ReadOnly.into(),
    );

    object.into()
}
