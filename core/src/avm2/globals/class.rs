//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;

/// Implements `Class`'s instance initializer.
///
/// Notably, you cannot construct new classes this way, so this returns an
/// error.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Err("Classes cannot be constructed.".into())
}

/// Implement's `Class`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn prototype<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(class) = this.as_class_object() {
        return Ok(class.prototype().into());
    }

    Ok(Value::Undefined)
}

/// Construct `Class`'s class.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object_class: Class<'gc>,
) -> Class<'gc> {
    let gc_context = activation.context.gc_context;
    let class_class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "Class"),
        Some(object_class),
        Method::from_builtin(instance_init, "<Class instance initializer>", gc_context),
        Method::from_builtin(class_init, "<Class class initializer>", gc_context),
        gc_context,
    );

    // 'length' is a weird undocumented constant in Class.
    // We need to define it, since it shows up in 'describeType'
    const CLASS_CONSTANTS: &[(&str, i32)] = &[("length", 1)];
    class_class.define_constant_int_class_traits(
        activation.avm2().public_namespace_base_version,
        CLASS_CONSTANTS,
        activation,
    );

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("prototype", Some(prototype), None)];
    class_class.define_builtin_instance_properties(
        gc_context,
        activation.avm2().public_namespace_base_version,
        PUBLIC_INSTANCE_PROPERTIES,
    );

    class_class.mark_traits_loaded(activation.context.gc_context);
    class_class
        .init_vtable(&mut activation.context)
        .expect("Native class's vtable should initialize");

    class_class
}
