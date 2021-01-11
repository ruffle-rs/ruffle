//! flash.geom.Rectangle

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::point::{construct_new_point, point_to_object, value_to_point};
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::f64::NAN;

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        this.set("x", 0.into(), activation)?;
        this.set("y", 0.into(), activation)?;
        this.set("width", 0.into(), activation)?;
        this.set("height", 0.into(), activation)?;
    } else {
        this.set(
            "x",
            args.get(0).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
        this.set(
            "y",
            args.get(1).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
        this.set(
            "width",
            args.get(2).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
        this.set(
            "height",
            args.get(3).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
    }

    Ok(this.into())
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?;
    let y = this.get("y", activation)?;
    let width = this.get("width", activation)?;
    let height = this.get("height", activation)?;

    Ok(AvmString::new(
        activation.context.gc_context,
        format!(
            "(x={}, y={}, w={}, h={})",
            x.coerce_to_string(activation)?,
            y.coerce_to_string(activation)?,
            width.coerce_to_string(activation)?,
            height.coerce_to_string(activation)?
        ),
    )
    .into())
}

pub fn create_rectangle_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    rectangle_proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        rectangle_proto,
    )
}

fn is_empty<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;
    Ok((width <= 0.0 || height <= 0.0 || width.is_nan() || height.is_nan()).into())
}

fn set_empty<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set("x", 0.into(), activation)?;
    this.set("y", 0.into(), activation)?;
    this.set("width", 0.into(), activation)?;
    this.set("height", 0.into(), activation)?;
    Ok(Value::Undefined)
}

fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let args = [
        this.get("x", activation)?,
        this.get("y", activation)?,
        this.get("width", activation)?,
        this.get("height", activation)?,
    ];
    let constructor = activation.context.avm1.prototypes.rectangle_constructor;
    let cloned = constructor.construct(activation, &args)?;
    Ok(cloned)
}

fn contains<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: This arbitrarily should return `false` or `undefined` for different invalid-values.
    // I can't find any rhyme or reason for it.
    let x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_f64(activation)?;
    let y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_f64(activation)?;
    if x.is_nan() || y.is_nan() {
        return Ok(Value::Undefined);
    }

    let left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let right = left + this.get("width", activation)?.coerce_to_f64(activation)?;
    let top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let bottom = top + this.get("height", activation)?.coerce_to_f64(activation)?;

    Ok((x >= left && x < right && y >= top && y < bottom).into())
}

fn contains_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (x, y) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;
    if x.is_nan() || y.is_nan() {
        return Ok(Value::Undefined);
    }

    let left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let right = left + this.get("width", activation)?.coerce_to_f64(activation)?;
    let top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let bottom = top + this.get("height", activation)?.coerce_to_f64(activation)?;

    Ok((x >= left && x < right && y >= top && y < bottom).into())
}

fn contains_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let other = if let Some(Value::Object(other)) = args.get(0) {
        other
    } else {
        return Ok(Value::Undefined);
    };

    let this_left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let this_top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let this_right = this_left + this.get("width", activation)?.coerce_to_f64(activation)?;
    let this_bottom = this_top + this.get("height", activation)?.coerce_to_f64(activation)?;

    let other_left = other.get("x", activation)?.coerce_to_f64(activation)?;
    let other_top = other.get("y", activation)?.coerce_to_f64(activation)?;
    let other_right = other_left + other.get("width", activation)?.coerce_to_f64(activation)?;
    let other_bottom = other_top + other.get("height", activation)?.coerce_to_f64(activation)?;

    if other_left.is_nan() || other_top.is_nan() || other_right.is_nan() || other_bottom.is_nan() {
        return Ok(Value::Undefined);
    }

    Ok((other_left >= this_left
        && other_right <= this_right
        && other_top >= this_top
        && other_bottom <= this_bottom)
        .into())
}

fn intersects<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let other = if let Some(Value::Object(other)) = args.get(0) {
        other
    } else {
        return Ok(false.into());
    };

    let this_left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let this_top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let this_right = this_left + this.get("width", activation)?.coerce_to_f64(activation)?;
    let this_bottom = this_top + this.get("height", activation)?.coerce_to_f64(activation)?;

    let other_left = other.get("x", activation)?.coerce_to_f64(activation)?;
    let other_top = other.get("y", activation)?.coerce_to_f64(activation)?;
    let other_right = other_left + other.get("width", activation)?.coerce_to_f64(activation)?;
    let other_bottom = other_top + other.get("height", activation)?.coerce_to_f64(activation)?;

    Ok((this_left < other_right
        && this_right > other_left
        && this_top < other_bottom
        && this_bottom > other_top)
        .into())
}

fn union<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let this_top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let this_right = this_left + this.get("width", activation)?.coerce_to_f64(activation)?;
    let this_bottom = this_top + this.get("height", activation)?.coerce_to_f64(activation)?;

    let (other_left, other_top, other_width, other_height) =
        if let Some(Value::Object(other)) = args.get(0) {
            (
                other.get("x", activation)?.coerce_to_f64(activation)?,
                other.get("y", activation)?.coerce_to_f64(activation)?,
                other.get("width", activation)?.coerce_to_f64(activation)?,
                other.get("height", activation)?.coerce_to_f64(activation)?,
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

    let args = [
        Value::Number(left),
        Value::Number(top),
        Value::Number(width),
        Value::Number(height),
    ];
    let constructor = activation.context.avm1.prototypes.rectangle_constructor;
    let result = constructor.construct(activation, &args)?;
    Ok(result)
}

fn inflate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;
    let horizontal = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_f64(activation)?;
    let vertical = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_f64(activation)?;

    this.set("x", Value::Number(x - horizontal), activation)?;
    this.set("y", Value::Number(y - vertical), activation)?;
    this.set("width", Value::Number(width + horizontal * 2.0), activation)?;
    this.set("height", Value::Number(height + vertical * 2.0), activation)?;

    Ok(Value::Undefined)
}

fn inflate_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;
    let (horizontal, vertical) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;

    this.set("x", Value::Number(x - horizontal), activation)?;
    this.set("y", Value::Number(y - vertical), activation)?;
    this.set("width", Value::Number(width + horizontal * 2.0), activation)?;
    this.set("height", Value::Number(height + vertical * 2.0), activation)?;

    Ok(Value::Undefined)
}

fn offset<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;
    let horizontal = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_f64(activation)?;
    let vertical = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .to_owned()
        .coerce_to_f64(activation)?;

    this.set("x", Value::Number(x + horizontal), activation)?;
    this.set("y", Value::Number(y + vertical), activation)?;

    Ok(Value::Undefined)
}

fn offset_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;
    let (horizontal, vertical) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;

    this.set("x", Value::Number(x + horizontal), activation)?;
    this.set("y", Value::Number(y + vertical), activation)?;

    Ok(Value::Undefined)
}

fn intersection<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let this_top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let this_right = this_left + this.get("width", activation)?.coerce_to_f64(activation)?;
    let this_bottom = this_top + this.get("height", activation)?.coerce_to_f64(activation)?;

    let (other_left, other_top, other_width, other_height) =
        if let Some(Value::Object(other)) = args.get(0) {
            (
                other.get("x", activation)?.coerce_to_f64(activation)?,
                other.get("y", activation)?.coerce_to_f64(activation)?,
                other.get("width", activation)?.coerce_to_f64(activation)?,
                other.get("height", activation)?.coerce_to_f64(activation)?,
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

    let args = [
        Value::Number(left),
        Value::Number(top),
        Value::Number(right - left),
        Value::Number(bottom - top),
    ];
    let constructor = activation.context.avm1.prototypes.rectangle_constructor;
    let result = constructor.construct(activation, &args)?;
    Ok(result)
}

fn equals<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(Value::Object(other)) = args.get(0) {
        let this_x = this.get("x", activation)?;
        let this_y = this.get("y", activation)?;
        let this_width = this.get("width", activation)?;
        let this_height = this.get("height", activation)?;
        let other_x = other.get("x", activation)?;
        let other_y = other.get("y", activation)?;
        let other_width = other.get("width", activation)?;
        let other_height = other.get("height", activation)?;
        let proto = activation.context.avm1.prototypes.rectangle;
        let constructor = activation.context.avm1.prototypes.rectangle_constructor;
        return Ok((this_x == other_x
            && this_y == other_y
            && this_width == other_width
            && this_height == other_height
            && other.is_instance_of(activation, constructor, proto)?)
        .into());
    }

    Ok(false.into())
}

fn get_left<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.get("x", activation)
}

fn set_left<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_left = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    let old_left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    this.set("x", new_left.clone(), activation)?;
    this.set(
        "width",
        Value::Number(width + (old_left - new_left.coerce_to_f64(activation)?)),
        activation,
    )?;
    Ok(Value::Undefined)
}

fn get_top<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.get("y", activation)
}

fn set_top<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_top = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    let old_top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;
    this.set("y", new_top.clone(), activation)?;
    this.set(
        "height",
        Value::Number(height + (old_top - new_top.coerce_to_f64(activation)?)),
        activation,
    )?;
    Ok(Value::Undefined)
}

fn get_right<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    Ok((x + width).into())
}

fn set_right<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let right = if let Some(arg) = args.get(0) {
        arg.coerce_to_f64(activation)?
    } else {
        NAN
    };
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;

    this.set("width", Value::Number(right - x), activation)?;

    Ok(Value::Undefined)
}

fn get_bottom<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;
    Ok((y + height).into())
}

fn set_bottom<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bottom = if let Some(arg) = args.get(0) {
        arg.coerce_to_f64(activation)?
    } else {
        NAN
    };
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;

    this.set("height", Value::Number(bottom - y), activation)?;

    Ok(Value::Undefined)
}

fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = this.get("width", activation)?;
    let height = this.get("height", activation)?;
    let point = construct_new_point(&[width, height], activation)?;
    Ok(point)
}

fn set_size<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (width, height) = if let Some(Value::Object(object)) = args.get(0) {
        (object.get("x", activation)?, object.get("y", activation)?)
    } else {
        (Value::Undefined, Value::Undefined)
    };

    this.set("width", width, activation)?;
    this.set("height", height, activation)?;

    Ok(Value::Undefined)
}

fn get_top_left<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?;
    let y = this.get("y", activation)?;
    let point = construct_new_point(&[x, y], activation)?;
    Ok(point)
}

fn set_top_left<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (new_left, new_top) = if let Some(Value::Object(object)) = args.get(0) {
        (object.get("x", activation)?, object.get("y", activation)?)
    } else {
        (Value::Undefined, Value::Undefined)
    };
    let old_left = this.get("x", activation)?.coerce_to_f64(activation)?;
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    let old_top = this.get("y", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;

    this.set("x", new_left.clone(), activation)?;
    this.set("y", new_top.clone(), activation)?;
    this.set(
        "width",
        Value::Number(width + (old_left - new_left.coerce_to_f64(activation)?)),
        activation,
    )?;
    this.set(
        "height",
        Value::Number(height + (old_top - new_top.coerce_to_f64(activation)?)),
        activation,
    )?;

    Ok(Value::Undefined)
}

fn get_bottom_right<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get("x", activation)?.coerce_to_f64(activation)?;
    let y = this.get("y", activation)?.coerce_to_f64(activation)?;
    let width = this.get("width", activation)?.coerce_to_f64(activation)?;
    let height = this.get("height", activation)?.coerce_to_f64(activation)?;
    let point = point_to_object((x + width, y + height), activation)?;
    Ok(point)
}

fn set_bottom_right<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (bottom, right) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;
    let top = this.get("x", activation)?.coerce_to_f64(activation)?;
    let left = this.get("y", activation)?.coerce_to_f64(activation)?;

    this.set("width", Value::Number(bottom - top), activation)?;
    this.set("height", Value::Number(right - left), activation)?;

    Ok(Value::Undefined)
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
        FunctionObject::function(
            gc_context,
            Executable::Native(get_left),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_left),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "top",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_top),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_top),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "right",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_right),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_right),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "bottom",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_bottom),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_bottom),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "size",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_size),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_size),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "topLeft",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_top_left),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_top_left),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.add_property(
        gc_context,
        "bottomRight",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_bottom_right),
            Some(fn_proto),
            fn_proto,
        ),
        Some(FunctionObject::function(
            gc_context,
            Executable::Native(set_bottom_right),
            Some(fn_proto),
            fn_proto,
        )),
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.into()
}
