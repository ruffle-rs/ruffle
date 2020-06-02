//! flash.geom.Rectangle

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::point::{construct_new_point, point_to_object, value_to_point};
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::f64::NAN;

fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if args.is_empty() {
        this.set("x", 0.into(), avm, context)?;
        this.set("y", 0.into(), avm, context)?;
        this.set("width", 0.into(), avm, context)?;
        this.set("height", 0.into(), avm, context)?;
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
        this.set(
            "width",
            args.get(2).unwrap_or(&Value::Undefined).to_owned(),
            avm,
            context,
        )?;
        this.set(
            "height",
            args.get(3).unwrap_or(&Value::Undefined).to_owned(),
            avm,
            context,
        )?;
    }

    Ok(Value::Undefined.into())
}

fn to_string<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .coerce_to_string(avm, context)?;

    Ok(format!("(x={}, y={}, w={}, h={})", x, y, width, height).into())
}

pub fn create_rectangle_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    rectangle_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        rectangle_proto,
    )
}

fn is_empty<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    Ok((width <= 0.0 || height <= 0.0 || width.is_nan() || height.is_nan()).into())
}

fn set_empty<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    this.set("x", 0.into(), avm, context)?;
    this.set("y", 0.into(), avm, context)?;
    this.set("width", 0.into(), avm, context)?;
    this.set("height", 0.into(), avm, context)?;
    Ok(Value::Undefined.into())
}

fn clone<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let proto = context.system_prototypes.rectangle;
    let args = [
        this.get("x", avm, context)?.resolve(avm, context)?,
        this.get("y", avm, context)?.resolve(avm, context)?,
        this.get("width", avm, context)?.resolve(avm, context)?,
        this.get("height", avm, context)?.resolve(avm, context)?,
    ];
    let cloned = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, cloned, &args)?;
    Ok(cloned.into())
}

fn contains<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    // TODO: This arbitrarily should return `false` or `undefined` for different invalid-values.
    // I can't find any rhyme or reason for it.
    let x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_number(avm, context)?;
    let y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_number(avm, context)?;
    if x.is_nan() || y.is_nan() {
        return Ok(Value::Undefined.into());
    }

    let left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let right = left
        + this
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let bottom = top
        + this
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    Ok((x >= left && x < right && y >= top && y < bottom).into())
}

fn contains_point<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (x, y) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        avm,
        context,
    )?;
    if x.is_nan() || y.is_nan() {
        return Ok(Value::Undefined.into());
    }

    let left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let right = left
        + this
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let bottom = top
        + this
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    Ok((x >= left && x < right && y >= top && y < bottom).into())
}

fn contains_rectangle<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let other = if let Some(Value::Object(other)) = args.get(0) {
        other
    } else {
        return Ok(Value::Undefined.into());
    };

    let this_left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_right = this_left
        + this
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let this_bottom = this_top
        + this
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    let other_left = other
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let other_top = other
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let other_right = other_left
        + other
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let other_bottom = other_top
        + other
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    if other_left.is_nan() || other_top.is_nan() || other_right.is_nan() || other_bottom.is_nan() {
        return Ok(Value::Undefined.into());
    }

    Ok((other_left >= this_left
        && other_right <= this_right
        && other_top >= this_top
        && other_bottom <= this_bottom)
        .into())
}

fn intersects<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let other = if let Some(Value::Object(other)) = args.get(0) {
        other
    } else {
        return Ok(false.into());
    };

    let this_left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_right = this_left
        + this
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let this_bottom = this_top
        + this
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    let other_left = other
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let other_top = other
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let other_right = other_left
        + other
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let other_bottom = other_top
        + other
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    Ok((this_left < other_right
        && this_right > other_left
        && this_top < other_bottom
        && this_bottom > other_top)
        .into())
}

fn union<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this_left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_right = this_left
        + this
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let this_bottom = this_top
        + this
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    let (other_left, other_top, other_width, other_height) =
        if let Some(Value::Object(other)) = args.get(0) {
            (
                other
                    .get("x", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
                other
                    .get("y", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
                other
                    .get("width", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
                other
                    .get("height", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
            )
        } else {
            (NAN, NAN, NAN, NAN)
        };
    let other_right = other_left + other_width;
    let other_bottom = other_top + other_height;

    let left = if this_left.is_nan() {
        this_left
    } else if other_left.is_nan() {
        other_left
    } else {
        this_left.min(other_left)
    };
    let top = if this_top.is_nan() {
        this_top
    } else if other_top.is_nan() {
        other_top
    } else {
        this_top.min(other_top)
    };
    let width = if this_right.is_nan() {
        this_right
    } else if other_right.is_nan() {
        other_right
    } else {
        this_right.max(other_right)
    } - left;
    let height = if this_bottom.is_nan() {
        this_bottom
    } else if other_bottom.is_nan() {
        other_bottom
    } else {
        this_bottom.max(other_bottom)
    } - top;

    let proto = context.system_prototypes.rectangle;
    let args = [
        Value::Number(left),
        Value::Number(top),
        Value::Number(width),
        Value::Number(height),
    ];
    let result = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, result, &args)?;
    Ok(result.into())
}

fn inflate<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let horizontal = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_number(avm, context)?;
    let vertical = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_number(avm, context)?;

    this.set("x", Value::Number(x - horizontal), avm, context)?;
    this.set("y", Value::Number(y - vertical), avm, context)?;
    this.set(
        "width",
        Value::Number(width + horizontal * 2.0),
        avm,
        context,
    )?;
    this.set(
        "height",
        Value::Number(height + vertical * 2.0),
        avm,
        context,
    )?;

    Ok(Value::Undefined.into())
}

fn inflate_point<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let (horizontal, vertical) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        avm,
        context,
    )?;

    this.set("x", Value::Number(x - horizontal), avm, context)?;
    this.set("y", Value::Number(y - vertical), avm, context)?;
    this.set(
        "width",
        Value::Number(width + horizontal * 2.0),
        avm,
        context,
    )?;
    this.set(
        "height",
        Value::Number(height + vertical * 2.0),
        avm,
        context,
    )?;

    Ok(Value::Undefined.into())
}

fn offset<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let horizontal = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_number(avm, context)?;
    let vertical = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .as_number(avm, context)?;

    this.set("x", Value::Number(x + horizontal), avm, context)?;
    this.set("y", Value::Number(y + vertical), avm, context)?;

    Ok(Value::Undefined.into())
}

fn offset_point<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let (horizontal, vertical) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        avm,
        context,
    )?;

    this.set("x", Value::Number(x + horizontal), avm, context)?;
    this.set("y", Value::Number(y + vertical), avm, context)?;

    Ok(Value::Undefined.into())
}

fn intersection<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let this_left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let this_right = this_left
        + this
            .get("width", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;
    let this_bottom = this_top
        + this
            .get("height", avm, context)?
            .resolve(avm, context)?
            .as_number(avm, context)?;

    let (other_left, other_top, other_width, other_height) =
        if let Some(Value::Object(other)) = args.get(0) {
            (
                other
                    .get("x", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
                other
                    .get("y", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
                other
                    .get("width", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
                other
                    .get("height", avm, context)?
                    .resolve(avm, context)?
                    .as_number(avm, context)?,
            )
        } else {
            (NAN, NAN, NAN, NAN)
        };
    let other_right = other_left + other_width;
    let other_bottom = other_top + other_height;

    let (mut left, mut top, mut right, mut bottom) = if this_left.is_nan()
        || other_left.is_nan()
        || this_top.is_nan()
        || other_top.is_nan()
        || this_right.is_nan()
        || other_right.is_nan()
        || this_bottom.is_nan()
        || other_bottom.is_nan()
    {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        (
            this_left.max(other_left),
            this_top.max(other_top),
            this_right.min(other_right),
            this_bottom.min(other_bottom),
        )
    };

    if right <= left || bottom <= top {
        right = 0.0;
        left = 0.0;
        bottom = 0.0;
        top = 0.0;
    }

    let proto = context.system_prototypes.rectangle;
    let args = [
        Value::Number(left),
        Value::Number(top),
        Value::Number(right - left),
        Value::Number(bottom - top),
    ];
    let result = proto.new(avm, context, proto, &args)?;
    let _ = constructor(avm, context, result, &args)?;
    Ok(result.into())
}

fn equals<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(Value::Object(other)) = args.get(0) {
        let this_x = this.get("x", avm, context)?.resolve(avm, context)?;
        let this_y = this.get("y", avm, context)?.resolve(avm, context)?;
        let this_width = this.get("width", avm, context)?.resolve(avm, context)?;
        let this_height = this.get("height", avm, context)?.resolve(avm, context)?;
        let other_x = other.get("x", avm, context)?.resolve(avm, context)?;
        let other_y = other.get("y", avm, context)?.resolve(avm, context)?;
        let other_width = other.get("width", avm, context)?.resolve(avm, context)?;
        let other_height = other.get("height", avm, context)?.resolve(avm, context)?;
        let proto = context.system_prototypes.rectangle;
        let constructor = context.system_prototypes.rectangle_constructor;
        return Ok((this_x == other_x
            && this_y == other_y
            && this_width == other_width
            && this_height == other_height
            && other.is_instance_of(avm, context, constructor, proto)?)
        .into());
    }

    Ok(false.into())
}

fn get_left<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(this.get("x", avm, context)?.resolve(avm, context)?.into())
}

fn set_left<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_left = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    let old_left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    this.set("x", new_left.clone(), avm, context)?;
    this.set(
        "width",
        Value::Number(width + (old_left - new_left.as_number(avm, context)?)),
        avm,
        context,
    )?;
    Ok(Value::Undefined.into())
}

fn get_top<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(this.get("y", avm, context)?.resolve(avm, context)?.into())
}

fn set_top<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let new_top = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    let old_top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    this.set("y", new_top.clone(), avm, context)?;
    this.set(
        "height",
        Value::Number(height + (old_top - new_top.as_number(avm, context)?)),
        avm,
        context,
    )?;
    Ok(Value::Undefined.into())
}

fn get_right<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    Ok((x + width).into())
}

fn set_right<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let right = if let Some(arg) = args.get(0) {
        arg.as_number(avm, context)?
    } else {
        NAN
    };
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;

    this.set("width", Value::Number(right - x), avm, context)?;

    Ok(Value::Undefined.into())
}

fn get_bottom<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    Ok((y + height).into())
}

fn set_bottom<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let bottom = if let Some(arg) = args.get(0) {
        arg.as_number(avm, context)?
    } else {
        NAN
    };
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;

    this.set("height", Value::Number(bottom - y), avm, context)?;

    Ok(Value::Undefined.into())
}

fn get_size<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let width = this.get("width", avm, context)?.resolve(avm, context)?;
    let height = this.get("height", avm, context)?.resolve(avm, context)?;
    let point = construct_new_point(&[width, height], avm, context)?;
    Ok(point.into())
}

fn set_size<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (width, height) = if let Some(Value::Object(object)) = args.get(0) {
        (
            object.get("x", avm, context)?.resolve(avm, context)?,
            object.get("y", avm, context)?.resolve(avm, context)?,
        )
    } else {
        (Value::Undefined, Value::Undefined)
    };

    this.set("width", width, avm, context)?;
    this.set("height", height, avm, context)?;

    Ok(Value::Undefined.into())
}

fn get_top_left<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this.get("x", avm, context)?.resolve(avm, context)?;
    let y = this.get("y", avm, context)?.resolve(avm, context)?;
    let point = construct_new_point(&[x, y], avm, context)?;
    Ok(point.into())
}

fn set_top_left<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (new_left, new_top) = if let Some(Value::Object(object)) = args.get(0) {
        (
            object.get("x", avm, context)?.resolve(avm, context)?,
            object.get("y", avm, context)?.resolve(avm, context)?,
        )
    } else {
        (Value::Undefined, Value::Undefined)
    };
    let old_left = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let old_top = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;

    this.set("x", new_left.clone(), avm, context)?;
    this.set("y", new_top.clone(), avm, context)?;
    this.set(
        "width",
        Value::Number(width + (old_left - new_left.as_number(avm, context)?)),
        avm,
        context,
    )?;
    this.set(
        "height",
        Value::Number(height + (old_top - new_top.as_number(avm, context)?)),
        avm,
        context,
    )?;

    Ok(Value::Undefined.into())
}

fn get_bottom_right<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let x = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let y = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let width = this
        .get("width", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let height = this
        .get("height", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let point = point_to_object((x + width, y + height), avm, context)?;
    Ok(point.into())
}

fn set_bottom_right<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let (bottom, right) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        avm,
        context,
    )?;
    let top = this
        .get("x", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;
    let left = this
        .get("y", avm, context)?
        .resolve(avm, context)?
        .as_number(avm, context)?;

    this.set("width", Value::Number(bottom - top), avm, context)?;
    this.set("height", Value::Number(right - left), avm, context)?;

    Ok(Value::Undefined.into())
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

    object.force_set_function(
        "isEmpty",
        is_empty,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "setEmpty",
        set_empty,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "contains",
        contains,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "containsPoint",
        contains_point,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "containsRectangle",
        contains_rectangle,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "intersects",
        intersects,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function("union", union, gc_context, EnumSet::empty(), Some(fn_proto));

    object.force_set_function(
        "inflate",
        inflate,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "inflatePoint",
        inflate_point,
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

    object.force_set_function(
        "offsetPoint",
        offset_point,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "intersection",
        intersection,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "equals",
        equals,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.add_property(
        gc_context,
        "left",
        Executable::Native(get_left),
        Some(Executable::Native(set_left)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "top",
        Executable::Native(get_top),
        Some(Executable::Native(set_top)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "right",
        Executable::Native(get_right),
        Some(Executable::Native(set_right)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "bottom",
        Executable::Native(get_bottom),
        Some(Executable::Native(set_bottom)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "size",
        Executable::Native(get_size),
        Some(Executable::Native(set_size)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "topLeft",
        Executable::Native(get_top_left),
        Some(Executable::Native(set_top_left)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "bottomRight",
        Executable::Native(get_bottom_right),
        Some(Executable::Native(set_bottom_right)),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.into()
}
