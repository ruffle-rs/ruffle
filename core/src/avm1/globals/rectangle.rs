//! flash.geom.Rectangle

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::point::{construct_new_point, point_to_object, value_to_point};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::string::{AvmString, StringContext};
use ruffle_macros::istr;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "toString" => method(to_string);
    "isEmpty" => method(is_empty);
    "setEmpty" => method(set_empty);
    "clone" => method(clone);
    "contains" => method(contains);
    "containsPoint" => method(contains_point);
    "containsRectangle" => method(contains_rectangle);
    "intersects" => method(intersects);
    "union" => method(union);
    "inflate" => method(inflate);
    "inflatePoint" => method(inflate_point);
    "offset" => method(offset);
    "offsetPoint" => method(offset_point);
    "intersection" => method(intersection);
    "equals" => method(equals);
    "left" => property(get_left, set_left);
    "top" => property(get_top, set_top);
    "right" => property(get_right, set_right);
    "bottom" => property(get_bottom, set_bottom);
    "size" => property(get_size, set_size);
    "topLeft" => property(get_top_left, set_top_left);
    "bottomRight" => property(get_bottom_right, set_bottom_right);
};

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        this.set(istr!("x"), 0.into(), activation)?;
        this.set(istr!("y"), 0.into(), activation)?;
        this.set(istr!("width"), 0.into(), activation)?;
        this.set(istr!("height"), 0.into(), activation)?;
    } else {
        this.set(
            istr!("x"),
            args.get(0).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
        this.set(
            istr!("y"),
            args.get(1).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
        this.set(
            istr!("width"),
            args.get(2).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
        this.set(
            istr!("height"),
            args.get(3).unwrap_or(&Value::Undefined).to_owned(),
            activation,
        )?;
    }

    Ok(this.into())
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get(istr!("x"), activation)?;
    let y = this.get(istr!("y"), activation)?;
    let width = this.get(istr!("width"), activation)?;
    let height = this.get(istr!("height"), activation)?;

    Ok(AvmString::new_utf8(
        activation.gc(),
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
    context: &mut StringContext<'gc>,
    rectangle_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        rectangle_proto,
    )
}

fn is_empty<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;
    Ok((width <= 0.0 || height <= 0.0 || width.is_nan() || height.is_nan()).into())
}

fn set_empty<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.set(istr!("x"), 0.into(), activation)?;
    this.set(istr!("y"), 0.into(), activation)?;
    this.set(istr!("width"), 0.into(), activation)?;
    this.set(istr!("height"), 0.into(), activation)?;
    Ok(Value::Undefined)
}

fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let args = [
        this.get(istr!("x"), activation)?,
        this.get(istr!("y"), activation)?,
        this.get(istr!("width"), activation)?,
        this.get(istr!("height"), activation)?,
    ];
    let constructor = activation.context.avm1.prototypes().rectangle_constructor;
    let cloned = constructor.construct(activation, &args)?;
    Ok(cloned)
}

fn contains<'gc>(
    activation: &mut Activation<'_, 'gc>,
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

    let left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let right = left
        + this
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let bottom = top
        + this
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    Ok((x >= left && x < right && y >= top && y < bottom).into())
}

fn contains_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
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

    let left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let right = left
        + this
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let bottom = top
        + this
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    Ok((x >= left && x < right && y >= top && y < bottom).into())
}

fn contains_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let other = if let Some(Value::Object(other)) = args.get(0) {
        other
    } else {
        return Ok(Value::Undefined);
    };

    let this_left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let this_top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let this_right = this_left
        + this
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let this_bottom = this_top
        + this
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    let other_left = other
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let other_top = other
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let other_right = other_left
        + other
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let other_bottom = other_top
        + other
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let other = if let Some(Value::Object(other)) = args.get(0) {
        other
    } else {
        return Ok(false.into());
    };

    let this_left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let this_top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let this_right = this_left
        + this
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let this_bottom = this_top
        + this
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    let other_left = other
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let other_top = other
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let other_right = other_left
        + other
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let other_bottom = other_top
        + other
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    Ok((this_left < other_right
        && this_right > other_left
        && this_top < other_bottom
        && this_bottom > other_top)
        .into())
}

fn union<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let this_top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let this_right = this_left
        + this
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let this_bottom = this_top
        + this
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    let (other_left, other_top, other_width, other_height) =
        if let Some(Value::Object(other)) = args.get(0) {
            (
                other
                    .get(istr!("x"), activation)?
                    .coerce_to_f64(activation)?,
                other
                    .get(istr!("y"), activation)?
                    .coerce_to_f64(activation)?,
                other
                    .get(istr!("width"), activation)?
                    .coerce_to_f64(activation)?,
                other
                    .get(istr!("height"), activation)?
                    .coerce_to_f64(activation)?,
            )
        } else {
            (f64::NAN, f64::NAN, f64::NAN, f64::NAN)
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

    let constructor = activation.context.avm1.prototypes().rectangle_constructor;
    let result = constructor.construct(
        activation,
        &[left.into(), top.into(), width.into(), height.into()],
    )?;
    Ok(result)
}

fn inflate<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;
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

    this.set(istr!("x"), (x - horizontal).into(), activation)?;
    this.set(istr!("y"), (y - vertical).into(), activation)?;
    this.set(
        istr!("width"),
        (width + horizontal * 2.0).into(),
        activation,
    )?;
    this.set(
        istr!("height"),
        (height + vertical * 2.0).into(),
        activation,
    )?;

    Ok(Value::Undefined)
}

fn inflate_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;
    let (horizontal, vertical) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;

    this.set(istr!("x"), (x - horizontal).into(), activation)?;
    this.set(istr!("y"), (y - vertical).into(), activation)?;
    this.set(
        istr!("width"),
        (width + horizontal * 2.0).into(),
        activation,
    )?;
    this.set(
        istr!("height"),
        (height + vertical * 2.0).into(),
        activation,
    )?;

    Ok(Value::Undefined)
}

fn offset<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
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

    this.set(istr!("x"), (x + horizontal).into(), activation)?;
    this.set(istr!("y"), (y + vertical).into(), activation)?;

    Ok(Value::Undefined)
}

fn offset_point<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let (horizontal, vertical) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;

    this.set(istr!("x"), (x + horizontal).into(), activation)?;
    this.set(istr!("y"), (y + vertical).into(), activation)?;

    Ok(Value::Undefined)
}

fn intersection<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this_left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let this_top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let this_right = this_left
        + this
            .get(istr!("width"), activation)?
            .coerce_to_f64(activation)?;
    let this_bottom = this_top
        + this
            .get(istr!("height"), activation)?
            .coerce_to_f64(activation)?;

    let (other_left, other_top, other_width, other_height) =
        if let Some(Value::Object(other)) = args.get(0) {
            (
                other
                    .get(istr!("x"), activation)?
                    .coerce_to_f64(activation)?,
                other
                    .get(istr!("y"), activation)?
                    .coerce_to_f64(activation)?,
                other
                    .get(istr!("width"), activation)?
                    .coerce_to_f64(activation)?,
                other
                    .get(istr!("height"), activation)?
                    .coerce_to_f64(activation)?,
            )
        } else {
            (f64::NAN, f64::NAN, f64::NAN, f64::NAN)
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

    let constructor = activation.context.avm1.prototypes().rectangle_constructor;
    let result = constructor.construct(
        activation,
        &[
            left.into(),
            top.into(),
            (right - left).into(),
            (bottom - top).into(),
        ],
    )?;
    Ok(result)
}

fn equals<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(Value::Object(other)) = args.get(0) {
        let this_x = this.get(istr!("x"), activation)?;
        let this_y = this.get(istr!("y"), activation)?;
        let this_width = this.get(istr!("width"), activation)?;
        let this_height = this.get(istr!("height"), activation)?;
        let other_x = other.get(istr!("x"), activation)?;
        let other_y = other.get(istr!("y"), activation)?;
        let other_width = other.get(istr!("width"), activation)?;
        let other_height = other.get(istr!("height"), activation)?;
        let proto = activation.context.avm1.prototypes().rectangle;
        let constructor = activation.context.avm1.prototypes().rectangle_constructor;
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.get(istr!("x"), activation)
}

fn set_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_left = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    let old_left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    this.set(istr!("x"), new_left, activation)?;
    this.set(
        istr!("width"),
        (width + (old_left - new_left.coerce_to_f64(activation)?)).into(),
        activation,
    )?;
    Ok(Value::Undefined)
}

fn get_top<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.get(istr!("y"), activation)
}

fn set_top<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let new_top = args.get(0).unwrap_or(&Value::Undefined).to_owned();
    let old_top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;
    this.set(istr!("y"), new_top, activation)?;
    this.set(
        istr!("height"),
        (height + (old_top - new_top.coerce_to_f64(activation)?)).into(),
        activation,
    )?;
    Ok(Value::Undefined)
}

fn get_right<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    Ok((x + width).into())
}

fn set_right<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let right = if let Some(arg) = args.get(0) {
        arg.coerce_to_f64(activation)?
    } else {
        f64::NAN
    };
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;

    this.set(istr!("width"), (right - x).into(), activation)?;

    Ok(Value::Undefined)
}

fn get_bottom<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;
    Ok((y + height).into())
}

fn set_bottom<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bottom = if let Some(arg) = args.get(0) {
        arg.coerce_to_f64(activation)?
    } else {
        f64::NAN
    };
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;

    this.set(istr!("height"), (bottom - y).into(), activation)?;

    Ok(Value::Undefined)
}

fn get_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = this.get(istr!("width"), activation)?;
    let height = this.get(istr!("height"), activation)?;
    let point = construct_new_point(&[width, height], activation)?;
    Ok(point)
}

fn set_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (width, height) = if let Some(Value::Object(object)) = args.get(0) {
        (
            object.get(istr!("x"), activation)?,
            object.get(istr!("y"), activation)?,
        )
    } else {
        (Value::Undefined, Value::Undefined)
    };

    this.set(istr!("width"), width, activation)?;
    this.set(istr!("height"), height, activation)?;

    Ok(Value::Undefined)
}

fn get_top_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this.get(istr!("x"), activation)?;
    let y = this.get(istr!("y"), activation)?;
    let point = construct_new_point(&[x, y], activation)?;
    Ok(point)
}

fn set_top_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (new_left, new_top) = if let Some(Value::Object(object)) = args.get(0) {
        (
            object.get(istr!("x"), activation)?,
            object.get(istr!("y"), activation)?,
        )
    } else {
        (Value::Undefined, Value::Undefined)
    };
    let old_left = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    let old_top = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;

    this.set(istr!("x"), new_left, activation)?;
    this.set(istr!("y"), new_top, activation)?;
    this.set(
        istr!("width"),
        (width + (old_left - new_left.coerce_to_f64(activation)?)).into(),
        activation,
    )?;
    this.set(
        istr!("height"),
        (height + (old_top - new_top.coerce_to_f64(activation)?)).into(),
        activation,
    )?;

    Ok(Value::Undefined)
}

fn get_bottom_right<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let y = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;
    let width = this
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)?;
    let height = this
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)?;
    let point = point_to_object((x + width, y + height), activation)?;
    Ok(point)
}

fn set_bottom_right<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (bottom, right) = value_to_point(
        args.get(0).unwrap_or(&Value::Undefined).to_owned(),
        activation,
    )?;
    let top = this
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)?;
    let left = this
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)?;

    this.set(istr!("width"), (bottom - top).into(), activation)?;
    this.set(istr!("height"), (right - left).into(), activation)?;

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, object, fn_proto);
    object.into()
}
