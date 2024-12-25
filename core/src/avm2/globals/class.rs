//! `Class` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error::type_error;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;

pub fn class_allocator<'gc>(
    _class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Err(Error::AvmError(type_error(
        activation,
        "Error #1115: Class$ is not a constructor.",
        1115,
    )?))
}

/// Implements `Class`'s instance initializer.
/// This can only be called by subclasses (if at all), so in practice it's a noop.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implement's `Class`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn prototype<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if let Some(class) = this.as_class_object() {
        return Ok(class.prototype().into());
    }

    Ok(Value::Undefined)
}

/// Construct `Class`'s i_class.
pub fn create_i_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object_i_class: Class<'gc>,
) -> Class<'gc> {
    let gc_context = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let class_i_class = Class::custom_new(
        QName::new(namespaces.public_all(), "Class"),
        Some(object_i_class),
        Method::from_builtin(instance_init, "<Class instance initializer>", gc_context),
        gc_context,
    );
    // The documentation and playerglobals are wrong; attempting to extend Class
    // throws a VerifyError
    class_i_class.set_attributes(gc_context, ClassAttributes::FINAL);

    class_i_class.set_instance_allocator(gc_context, class_allocator);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[("prototype", Some(prototype), None)];
    class_i_class.define_builtin_instance_properties(
        gc_context,
        namespaces.public_all(),
        PUBLIC_INSTANCE_PROPERTIES,
    );

    class_i_class.mark_traits_loaded(activation.gc());
    class_i_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class_i_class
}

/// Construct `Class`'s c_class.
pub fn create_c_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class_i_class: Class<'gc>,
) -> Class<'gc> {
    let gc_context = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let class_c_class = Class::custom_new(
        QName::new(namespaces.public_all(), "Class$"),
        Some(class_i_class),
        Method::from_builtin(class_init, "<Class class initializer>", gc_context),
        gc_context,
    );
    class_c_class.set_attributes(gc_context, ClassAttributes::FINAL);

    // 'length' is a weird undocumented constant in Class.
    // We need to define it, since it shows up in 'describeType'
    const CLASS_CONSTANTS: &[(&str, i32)] = &[("length", 1)];
    class_c_class.define_constant_int_instance_traits(
        namespaces.public_all(),
        CLASS_CONSTANTS,
        activation,
    );

    class_c_class.mark_traits_loaded(activation.gc());
    class_c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class_c_class
}
