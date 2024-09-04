//! `global` constructor
//!
//! Globals are an undocumented Flash class that don't appear to have any
//! public methods, but are the class that the script global scope is an
//! instance of.

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::object::Object;
use crate::avm2::traits::Trait;
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

/// Construct `global`'s class.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    traits: Vec<Trait<'gc>>,
) -> Class<'gc> {
    let mc = activation.context.gc_context;
    let class = Class::custom_new(
        QName::new(activation.avm2().namespaces.public_all(), "global"),
        Some(activation.avm2().class_defs().object),
        Method::from_builtin(instance_init, "<global instance initializer>", mc),
        mc,
    );

    class.set_traits(mc, traits);
    class.mark_traits_loaded(mc);
    class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    class
}
