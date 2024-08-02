//! `global` constructor
//!
//! Globals are an undocumented Flash class that don't appear to have any
//! public methods, but are the class that the script global scope is an
//! instance of.

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;

/// Implements `global`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `global`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Construct `global`'s class.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object_classdef: Class<'gc>,
    class_classdef: Class<'gc>,
) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::new(
        QName::new(activation.avm2().public_namespace_base_version, "global"),
        Some(object_classdef),
        Method::from_builtin(instance_init, "<global instance initializer>", mc),
        Method::from_builtin(class_init, "<global class initializer>", mc),
        class_classdef,
        mc,
    );

    class.mark_traits_loaded(activation.context.gc_context);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    let c_class = class.c_class().expect("Class::new returns an i_class");

    c_class.mark_traits_loaded(activation.context.gc_context);
    c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}
