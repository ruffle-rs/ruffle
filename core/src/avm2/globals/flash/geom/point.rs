//! `flash.geom.Point` builtin/prototype

use crate::avm1::AvmString;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::traits::Trait;
use crate::avm2::{Activation, Error, Namespace, Object, QName, TObject, Value};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.geom.Point`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(mut this) = this {
        let x = args
            .get(0)
            .unwrap_or(&0.into())
            .as_number(activation.context.gc_context)?;
        let y = args
            .get(1)
            .unwrap_or(&0.into())
            .as_number(activation.context.gc_context)?;

        this.set_property(
            this,
            &QName::new(Namespace::public(), "x"),
            x.into(),
            activation,
        )?;
        this.set_property(
            this,
            &QName::new(Namespace::public(), "y"),
            y.into(),
            activation,
        )?;
    }
    Ok(Value::Undefined)
}

fn coords<'gc>(
    this: &mut Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<(f64, f64), Error> {
    let x = this
        .get_property(*this, &QName::new(Namespace::public(), "x"), activation)?
        .as_number(activation.context.gc_context)?;
    let y = this
        .get_property(*this, &QName::new(Namespace::public(), "y"), activation)?
        .as_number(activation.context.gc_context)?;
    Ok((x, y))
}

/// Implements `flash.geom.Point`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

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

            let proto = activation.context.avm2.prototypes().point;
            let args = [Value::Number(our_x + their_x), Value::Number(our_y + their_y)];
            let new_point = proto.construct(activation, &args)?;
            instance_init(activation, Some(new_point), &args)?;

            return Ok(new_point.into());
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
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    // write.set_attributes(ClassAttributes::SEALED);

    //TODO: check namespaces
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "length"),
        Method::from_builtin(length),
    ));
    // write.define_instance_trait(Trait::from_getter(
    //     QName::new(Namespace::public(), "x"),
    //     Method::from_builtin(x),
    // ));
    // write.define_instance_trait(Trait::from_getter(
    //     QName::new(Namespace::public(), "y"),
    //     Method::from_builtin(y),
    // ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "add"),
        Method::from_builtin(add),
    ));
    //TODO:do we need this
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "clone"),
    //     Method::from_builtin(),
    // ));
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "copyFrom"),
    //     Method::from_builtin(),
    // ));
    // //TODO: static
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "distance"),
    //     Method::from_builtin(),
    // ));
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "equals"),
    //     Method::from_builtin(),
    // ));
    // //TODO: static
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "interpolate"),
    //     Method::from_builtin(),
    // ));
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "normalize"),
    //     Method::from_builtin(),
    // ));
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "offset"),
    //     Method::from_builtin(),
    // ));
    // //TODO: static
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "polar"),
    //     Method::from_builtin(),
    // ));
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "setTo"),
    //     Method::from_builtin(),
    // ));
    // write.define_instance_trait(Trait::from_method(
    //     QName::new(Namespace::public(), "subtract"),
    //     Method::from_builtin(),
    // ));
    write.define_instance_trait(Trait::from_method(
        QName::new(Namespace::public(), "toString"),
        Method::from_builtin(to_string),
    ));

    class
}
