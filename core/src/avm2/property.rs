//! Property data structures

use crate::avm2::Activation;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::TranslationUnit;
use crate::avm2::Value;
use crate::string::{AvmString, StringContext};
use gc_arena::{Collect, Gc};

use super::class::Class;

#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum Property {
    Virtual { get: Option<u32>, set: Option<u32> },
    Method { disp_id: u32 },
    Slot { slot_id: u32 },
    ConstSlot { slot_id: u32 },
}

/// The type of a `Slot`/`ConstSlot` property, represented
/// as a lazily-resolved class. This also implements the
/// property-specific coercion logic applied when setting
/// or initializing a property.
///
/// The class resolution needs to be lazy, since we can have
/// a cycle of property type references between classes
/// (e.g. Class1 has `var prop1:Class2`, and Class2
/// has `var prop2:Class1`).
///
/// Additionally, property class resolution uses special
/// logic, different from normal "runtime" class resolution,
/// that allows private types to be referenced.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum PropertyClass<'gc> {
    /// The type `*`. This allows `Value::Undefined`, so it needs to
    /// be distinguished from the `Object` class
    Any,
    Class(Class<'gc>),
    Name(Gc<'gc, Multiname<'gc>>, Option<TranslationUnit<'gc>>),
}

impl<'gc> PropertyClass<'gc> {
    pub fn name(name: Option<Gc<'gc, Multiname<'gc>>>, unit: Option<TranslationUnit<'gc>>) -> Self {
        if let Some(name) = name {
            PropertyClass::Name(name, unit)
        } else {
            PropertyClass::Any
        }
    }

    /// Returns `value` coerced to the type of this `PropertyClass`.
    /// The bool is `true` if this `PropertyClass` was just modified
    /// to cache a class resolution, and `false` if it was not modified.
    pub fn coerce(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: Value<'gc>,
    ) -> Result<(Value<'gc>, bool), Error<'gc>> {
        let (class, changed) = match self {
            PropertyClass::Class(class) => (Some(*class), false),
            PropertyClass::Name(name, unit) => {
                // Note - we look up the class in the domain by name, which allows us to look up private classes.
                // This also has the advantage of letting us coerce to a class while the `ClassObject`
                // is still being constructed (since the `Class` will already exist in the domain).

                // We should only be missing a translation unit when performing a lookup from playerglobals,
                // so use that domain if we don't have a translation unit.
                let domain = unit.map_or(activation.avm2().playerglobals_domain, |u| u.domain());
                if let Some(class) = domain.get_class(activation.context, name) {
                    *self = PropertyClass::Class(class);
                    (Some(class), true)
                } else {
                    return Err(
                        format!("Could not resolve class {name:?} for property coercion").into(),
                    );
                }
            }
            PropertyClass::Any => (None, false),
        };

        if let Some(class) = class {
            Ok((value.coerce_to_type(activation, class)?, changed))
        } else {
            // We have a type of '*' ("any"), so don't
            // perform any coercions
            Ok((value, changed))
        }
    }

    pub fn get_class(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Option<Class<'gc>>, Error<'gc>> {
        match self {
            PropertyClass::Class(class) => Ok(Some(*class)),
            PropertyClass::Name(name, unit) => {
                let domain = unit.map_or(activation.avm2().playerglobals_domain, |u| u.domain());
                if let Some(class) = domain.get_class(activation.context, name) {
                    *self = PropertyClass::Class(class);
                    Ok(Some(class))
                } else {
                    Err(
                        format!("Could not resolve class {name:?} for property class lookup")
                            .into(),
                    )
                }
            }
            PropertyClass::Any => Ok(None),
        }
    }

    pub fn get_name(&self, context: &mut StringContext<'gc>) -> AvmString<'gc> {
        match self {
            PropertyClass::Class(class) => class.name().to_qualified_name(context.gc_context),
            PropertyClass::Name(name, _) => name.to_qualified_name_or_star(context),
            PropertyClass::Any => context.ascii_char(b'*'),
        }
    }
}

impl Property {
    pub fn new_method(disp_id: u32) -> Self {
        Property::Method { disp_id }
    }

    pub fn new_getter(disp_id: u32) -> Self {
        Property::Virtual {
            get: Some(disp_id),
            set: None,
        }
    }

    pub fn new_setter(disp_id: u32) -> Self {
        Property::Virtual {
            get: None,
            set: Some(disp_id),
        }
    }

    pub fn new_slot(slot_id: u32) -> Self {
        Property::Slot { slot_id }
    }

    pub fn new_const_slot(slot_id: u32) -> Self {
        Property::ConstSlot { slot_id }
    }
}
