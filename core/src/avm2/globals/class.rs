//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use gc_arena::{GcCell, MutationContext};

/// Implements `Class`'s instance initializer.
///
/// Notably, you cannot construct new classes this way, so this returns an
/// error.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err("Classes cannot be constructed.".into())
}

/// Implement's `Class`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn prototype<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        if let Some(class) = this.as_class_object() {
            return Ok(class.prototype().into());
        }
    }
    Ok(Value::Undefined)
}

/// Construct `Class`'s class.
pub fn create_class<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class_class = Class::new(
        QName::new(Namespace::public(), "Class"),
        Some(Multiname::public("Object")),
        Method::from_builtin(instance_init, "<Class instance initializer>", gc_context),
        Method::from_builtin(class_init, "<Class class initializer>", gc_context),
        gc_context,
    );

    let mut write = class_class.write(gc_context);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("prototype", Some(prototype), None)];
    write.define_public_builtin_instance_properties(gc_context, PUBLIC_INSTANCE_PROPERTIES);

    class_class
}
