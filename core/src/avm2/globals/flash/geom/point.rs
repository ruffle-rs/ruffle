//! `flash.geom.Point` builtin/prototype

use crate::avm1::AvmString;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::{Activation, Error, Namespace, Object, QName, TObject, Value};
use gc_arena::{GcCell, MutationContext};

fn create_point<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    coords: (f64, f64),
) -> Result<Value<'gc>, Error> {
    let point_class = activation.context.avm2.classes().point;

    let args = [Value::Number(coords.0), Value::Number(coords.1)];
    let new_point = point_class.construct(activation, &args)?;

    Ok(new_point.into())
}

/// Implements `flash.geom.Point`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let _ = set_to(activation, this, args)?;
    Ok(Value::Undefined)
}

fn coords<'gc>(
    this: &mut Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<(f64, f64), Error> {
    let x = this
        .get_property(*this, &QName::new(Namespace::public(), "x"), activation)?
        .coerce_to_number(activation)?;
    let y = this
        .get_property(*this, &QName::new(Namespace::public(), "y"), activation)?
        .coerce_to_number(activation)?;
    Ok((x, y))
}

fn set_coords<'gc>(
    this: &mut Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: (f64, f64),
) -> Result<(), Error> {
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "x"),
        value.0.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "y"),
        value.1.into(),
        activation,
    )?;
    Ok(())
}

/// Implements `flash.geom.Point`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements the `length` property
pub fn length<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let (x, y) = coords(&mut this, activation)?;

        return Ok((x * x + y * y).sqrt().into());
    }

    Ok(Value::Undefined)
}

/// Implements `add`
pub fn add<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        if let Some(other) = args.get(0) {
            let mut other_obj = other.coerce_to_object(activation)?;
            let (our_x, our_y) = coords(&mut this, activation)?;
            let (their_x, their_y) = coords(&mut other_obj, activation)?;

            return create_point(activation, (our_x + their_x, our_y + their_y));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `clone`
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let (our_x, our_y) = coords(&mut this, activation)?;

        return create_point(activation, (our_x, our_y));
    }

    Ok(Value::Undefined)
}

/// Implements `copyFrom`
pub fn copy_from<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        if let Some(other) = args.get(0) {
            let mut other_obj = other.coerce_to_object(activation)?;
            let (their_x, their_y) = coords(&mut other_obj, activation)?;

            set_coords(&mut this, activation, (their_x, their_y))?;
        }
    }

    Ok(Value::Undefined)
}

/// Implements `distance`
pub fn distance<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(first) = args.get(0) {
        let mut first_object = first.coerce_to_object(activation)?;
        if let Some(second) = args.get(1) {
            let mut second_obj = second.coerce_to_object(activation)?;
            let (our_x, our_y) = coords(&mut first_object, activation)?;
            let (their_x, their_y) = coords(&mut second_obj, activation)?;

            return Ok(((our_x - their_x).powf(2.0) + (our_y - their_y).powf(2.0))
                .sqrt()
                .into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `equals`
#[allow(clippy::float_cmp)]
pub fn equals<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        if let Some(other) = args.get(0) {
            let mut other_obj = other.coerce_to_object(activation)?;

            let (our_x, our_y) = coords(&mut this, activation)?;
            let (their_x, their_y) = coords(&mut other_obj, activation)?;

            return Ok((our_x == their_x && our_y == their_y).into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `interpolate`
pub fn interpolate<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if args.len() < 3 {
        return create_point(activation, (f64::NAN, f64::NAN));
    }

    let (a_x, a_y) = coords(
        &mut args.get(0).unwrap().coerce_to_object(activation)?,
        activation,
    )?;
    let (b_x, b_y) = coords(
        &mut args.get(1).unwrap().coerce_to_object(activation)?,
        activation,
    )?;
    let f = args.get(2).unwrap().coerce_to_number(activation)?;

    let result = (b_x - (b_x - a_x) * f, b_y - (b_y - a_y) * f);
    create_point(activation, result)
}

/// Implements `normalize`
pub fn normalize<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let thickness = args
            .get(0)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;

        let length = length(activation, Some(this), args)?.coerce_to_number(activation)?;

        if length > 0.0 {
            let inv_d = thickness / length;

            let (old_x, old_y) = coords(&mut this, activation)?;
            set_coords(&mut this, activation, (old_x * inv_d, old_y * inv_d))?;
        }
    }

    Ok(Value::Undefined)
}

/// Implements `offset`
pub fn offset<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let (x, y) = coords(&mut this, activation)?;

        let dx = args
            .get(0)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;
        let dy = args
            .get(1)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;

        set_coords(&mut this, activation, (x + dx, y + dy))?;
    }

    Ok(Value::Undefined)
}

/// Implements `polar`
pub fn polar<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let length = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;
    let angle = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_number(activation)?;

    create_point(activation, (length * angle.cos(), length * angle.sin()))
}

/// Implements `setTo`
pub fn set_to<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let x = args
            .get(0)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;

        set_coords(&mut this, activation, (x, y))?;
    }

    Ok(Value::Undefined)
}

/// Implements `subtract`
pub fn subtract<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        if let Some(other) = args.get(0) {
            let mut other_obj = other.coerce_to_object(activation)?;
            let (our_x, our_y) = coords(&mut this, activation)?;
            let (their_x, their_y) = coords(&mut other_obj, activation)?;

            return create_point(activation, (our_x - their_x, our_y - their_y));
        }
    }

    Ok(Value::Undefined)
}

/// Implements `toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let (x, y) = coords(&mut this, activation)?;
        return Ok(
            AvmString::new(activation.context.gc_context, format!("(x={}, y={})", x, y)).into(),
        );
    }

    Ok(Value::Undefined)
}

/// Construct `Point`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.geom"), "Point"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Point instance initializer>", mc),
        Method::from_builtin(class_init, "<Point class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("length", Some(length), None)];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("distance", distance),
        ("interpolate", interpolate),
        ("polar", polar),
    ];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("add", add),
        ("clone", clone),
        ("copyFrom", copy_from),
        ("equals", equals),
        ("normalize", normalize),
        ("offset", offset),
        ("setTo", set_to),
        ("subtract", subtract),
        ("toString", to_string),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);
    class
}
