//! `global` constructor
//!
//! Globals are an undocumented Flash class that don't appear to have any
//! public methods, but are the class that the script global scope is an
//! instance of.

use crate::avm2::activation::Activation;
use crate::avm2::class::{BuiltinType, Class, ClassAttributes};
use crate::avm2::error::Error;
use crate::avm2::method::Method;
use crate::avm2::traits::Trait;
use crate::avm2::QName;
use ruffle_macros::istr;

/// Construct `global`'s class.
pub fn create_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    init_method: Method<'gc>,
    traits: Box<[Trait<'gc>]>,
) -> Result<Class<'gc>, Error<'gc>> {
    let mc = activation.gc();
    let class = Class::custom_new(
        QName::new(activation.avm2().namespaces.public_all(), istr!("global")),
        Some(activation.avm2().class_defs().object),
        Some(init_method),
        traits,
        mc,
    );

    class.set_attributes(ClassAttributes::FINAL);

    class.validate_class(activation, true)?;
    class.validate_signatures(activation)?;

    // `global` classes have no interfaces, so use `init_vtable_with_interfaces`
    // and pass an empty list
    class.init_vtable_with_interfaces(activation.context, Box::new([]));

    class.mark_builtin_type(BuiltinType::ScriptTraits);

    Ok(class)
}
