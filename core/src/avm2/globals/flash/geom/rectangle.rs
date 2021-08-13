//! `flash.geom.Rectangle` builtin/prototype

use crate::avm1::AvmString;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::globals::flash::geom::point::create_point;
use crate::avm2::{Activation, Error, Namespace, Object, QName, TObject, Value};
use gc_arena::{GcCell, MutationContext};

pub fn create_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    properties: (f64, f64, f64, f64),
) -> Result<Value<'gc>, Error> {
    let rectangle_class = activation.context.avm2.classes().rectangle;

    let args = [Value::Number(properties.0), Value::Number(properties.1), Value::Number(properties.2), Value::Number(properties.3)];
    let new_rectangle = rectangle_class.construct(activation, &args)?;

    Ok(new_rectangle.into())
}

/// Implements `flash.geom.Rectangle`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let _ = set_to(activation, this, args)?;
    Ok(Value::Undefined)
}

/// Implements `flash.geom.Rectangle`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

fn set_properties<'gc>(
    this: &mut Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    value: (f64, f64, f64, f64),
) -> Result<(), Error> {
    let x = value.0;
    let y = value.1;
    let width = value.2;
    let height = value.3;

    let bottom = y + height;
    let top = y;

    let left = x;
    let right = x + width;

    let size = create_point(activation, (width, height)).unwrap();
    let top_left = create_point(activation, (x, y)).unwrap();
    let bottom_right = create_point(activation, (x + width, y + height)).unwrap();
    
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "x"),
        x.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "y"),
        y.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "width"),
        width.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "height"),
        height.into(),
        activation,
    )?;
    
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "bottom"),
        bottom.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "top"),
        top.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "left"),
        left.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "right"),
        right.into(),
        activation,
    )?;

    this.set_property(
        *this,
        &QName::new(Namespace::public(), "bottomRight"),
        bottom_right.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "topLeft"),
        top_left.into(),
        activation,
    )?;
    this.set_property(
        *this,
        &QName::new(Namespace::public(), "size"),
        size.into(),
        activation,
    )?;
    Ok(())
}

fn properties<'gc>(
    this: &mut Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<(f64, f64, f64, f64), Error> {
    let x = this
        .get_property(*this, &QName::new(Namespace::public(), "x"), activation)?
        .coerce_to_number(activation)?;
    let y = this
        .get_property(*this, &QName::new(Namespace::public(), "y"), activation)?
        .coerce_to_number(activation)?;
    let width = this
        .get_property(*this, &QName::new(Namespace::public(), "width"), activation)?
        .coerce_to_number(activation)?;
    let height = this
        .get_property(
            *this,
            &QName::new(Namespace::public(), "height"),
            activation,
        )?
        .coerce_to_number(activation)?;

    Ok((x, y, width, height))
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

        let width = args
            .get(2)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;
        let height = args
            .get(3)
            .unwrap_or(&0.into())
            .coerce_to_number(activation)?;

        set_properties(&mut this, activation, (x, y, width, height))?;
    }

    Ok(Value::Undefined)
}

/// Implements `toString`
pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let x = this
            .get_property(this, &QName::new(Namespace::public(), "x"), activation)?
            .coerce_to_number(activation)?;
        let y = this
            .get_property(this, &QName::new(Namespace::public(), "y"), activation)?
            .coerce_to_number(activation)?;
        let width = this
            .get_property(this, &QName::new(Namespace::public(), "width"), activation)?
            .coerce_to_number(activation)?;
        let height = this
            .get_property(
                this,
                &QName::new(Namespace::public(), "height"),
                activation,
            )?
            .coerce_to_number(activation)?;

        return Ok(AvmString::new(
            activation.context.gc_context,
            format!("(x={}, y={}, w={}, h={})", x, y, width, height),
        )
        .into());
    }

    Ok(Value::Undefined)
}

/// Construct `Rectangle`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.geom"), "Rectangle"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Rectangle instance initializer>", mc),
        Method::from_builtin(class_init, "<Rectangle class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("setTo", set_to), ("toString", to_string)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);
    class
}
