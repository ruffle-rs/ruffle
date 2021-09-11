//! `flash.display.IBitmapDrawable` builtin

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Emulates attempts to execute bodiless methods.
pub fn bodiless_method<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("Cannot execute non-native method without body".into())
}

/// Implements `flash.display.IBitmapDrawable`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Construct `IBitmapDrawable`'s class.
pub fn create_interface<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "IBitmapDrawable"),
        None,
        Method::from_builtin(
            bodiless_method,
            "<IBitmapDrawable instance initializer>",
            mc,
        ),
        Method::from_builtin(class_init, "<IBitmapDrawable interface initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::INTERFACE);

    class
}
