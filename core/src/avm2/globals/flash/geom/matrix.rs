//! `flash.geom.Matrix` builtin/prototype

use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::{Activation, Error, Namespace, Object, QName, TObject, Value};
use gc_arena::{GcCell, MutationContext};

#[allow(clippy::too_many_arguments)]
fn set_values<'gc>(
    this: &mut Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
) -> Result<(), Error> {
    this.set_property(
        &QName::new(Namespace::public(), "a").into(),
        a.into(),
        activation,
    )?;
    this.set_property(
        &QName::new(Namespace::public(), "b").into(),
        b.into(),
        activation,
    )?;
    this.set_property(
        &QName::new(Namespace::public(), "c").into(),
        c.into(),
        activation,
    )?;
    this.set_property(
        &QName::new(Namespace::public(), "d").into(),
        d.into(),
        activation,
    )?;
    this.set_property(
        &QName::new(Namespace::public(), "tx").into(),
        tx.into(),
        activation,
    )?;
    this.set_property(
        &QName::new(Namespace::public(), "ty").into(),
        ty.into(),
        activation,
    )?;

    Ok(())
}

/// Implements `flash.geom.Matrix`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let a = args
            .get(0)
            .unwrap_or(&1f64.into())
            .coerce_to_number(activation)?;

        let b = args
            .get(1)
            .unwrap_or(&0f64.into())
            .coerce_to_number(activation)?;

        let c = args
            .get(2)
            .unwrap_or(&0f64.into())
            .coerce_to_number(activation)?;

        let d = args
            .get(3)
            .unwrap_or(&1f64.into())
            .coerce_to_number(activation)?;

        let tx = args
            .get(4)
            .unwrap_or(&0f64.into())
            .coerce_to_number(activation)?;

        let ty = args
            .get(5)
            .unwrap_or(&0f64.into())
            .coerce_to_number(activation)?;

        set_values(&mut this, activation, a, b, c, d, tx, ty)?;
    }
    Ok(Value::Undefined)
}

/// Implements `flash.geom.Matrix`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `identity'
pub fn identity<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        set_values(&mut this, activation, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
    }
    Ok(Value::Undefined)
}

/// Implements `scale'
pub fn scale<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let sx = if let Some(sx) = args.get(0) {
            sx.coerce_to_number(activation)?
        } else {
            return Err(format!(
                "Expected 2 arguments in flash.geom::Matrix/scale(), got {}",
                args.len()
            )
            .into());
        };
        let sy = if let Some(sy) = args.get(1) {
            sy.coerce_to_number(activation)?
        } else {
            return Err(format!(
                "Expected 2 arguments in flash.geom::Matrix/scale(), got {}",
                args.len()
            )
            .into());
        };

        let a = this
            .get_property(&QName::new(Namespace::public(), "a").into(), activation)?
            .coerce_to_number(activation)?;
        let d = this
            .get_property(&QName::new(Namespace::public(), "d").into(), activation)?
            .coerce_to_number(activation)?;

        this.set_property(
            &QName::new(Namespace::public(), "a").into(),
            (sx * a).into(),
            activation,
        )?;

        this.set_property(
            &QName::new(Namespace::public(), "d").into(),
            (sy * d).into(),
            activation,
        )?;
    }
    Ok(Value::Undefined)
}

/// Construct `Matrix`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.geom"), "Matrix"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Matrix instance initializer>", mc),
        Method::from_builtin(class_init, "<Matrix class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    const PUBLIC_INSTANCE_NUMBER_SLOTS: &[(&str, Option<f64>)] = &[
        ("a", None),
        ("b", None),
        ("c", None),
        ("d", None),
        ("tx", None),
        ("ty", None),
    ];
    write.define_public_slot_number_instance_traits(PUBLIC_INSTANCE_NUMBER_SLOTS);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] =
        &[("identity", identity), ("scale", scale)];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
