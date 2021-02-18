//! Slot contents type

use crate::avm2::property::Attribute;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::Collect;

/// Represents a single slot on an object.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum Slot<'gc> {
    /// An unoccupied slot.
    ///
    /// Attempts to read an unoccupied slot proceed up the prototype chain.
    /// Writing an unoccupied slot will always fail.
    Unoccupied,

    /// An occupied slot.
    ///
    /// TODO: For some reason, rustc believes this variant is unused.
    Occupied {
        value: Value<'gc>,
        attributes: Attribute,
    },
}

impl<'gc> Default for Slot<'gc> {
    fn default() -> Self {
        Self::Unoccupied
    }
}

impl<'gc> Slot<'gc> {
    /// Create a normal slot with a given value.
    pub fn new(value: impl Into<Value<'gc>>) -> Self {
        Self::Occupied {
            value: value.into(),
            attributes: Attribute::empty(),
        }
    }

    /// Create a `const` slot that cannot be overwritten.
    pub fn new_const(value: impl Into<Value<'gc>>) -> Self {
        Self::Occupied {
            value: value.into(),
            attributes: Attribute::READ_ONLY,
        }
    }

    /// Retrieve the value of this slot.
    pub fn get(&self) -> Option<Value<'gc>> {
        match self {
            Self::Unoccupied => None,
            Self::Occupied { value, .. } => Some(value.clone()),
        }
    }

    /// Write the value of this slot.
    pub fn set(&mut self, new_value: impl Into<Value<'gc>>) -> Result<(), Error> {
        match self {
            Self::Unoccupied => Err("Cannot overwrite unoccupied slot".into()),
            Self::Occupied { value, attributes } => {
                if attributes.contains(Attribute::READ_ONLY) {
                    return Err("Cannot overwrite const slot".into());
                }

                //TODO: Type assert

                *value = new_value.into();

                Ok(())
            }
        }
    }

    /// Initialize a slot to a particular value.
    pub fn init(&mut self, new_value: impl Into<Value<'gc>>) -> Result<(), Error> {
        match self {
            Self::Unoccupied => Err("Cannot initialize unoccupied slot".into()),
            Self::Occupied { value, .. } => {
                //TODO: Type assert

                *value = new_value.into();

                Ok(())
            }
        }
    }
}
