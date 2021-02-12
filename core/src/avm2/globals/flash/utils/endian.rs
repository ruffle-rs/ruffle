use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.utils.Endian`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.utils.Endian`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.utils"), "Endian"),
        None,
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    class
        .write(mc)
        .set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);
    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "LITTLE_ENDIAN"),
        QName::new(Namespace::public(), "String").into(),
        Some("littleEndian".into()),
    ));

    class.write(mc).define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "BIG_ENDIAN"),
        QName::new(Namespace::public(), "String").into(),
        Some("bigEndian".into()),
    ));
    class
}
