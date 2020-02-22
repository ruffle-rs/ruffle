//! Slot contents type

use crate::avm2::property::Attribute;
use crate::avm2::value::Value;
use crate::avm2::Error;
use enumset::EnumSet;
use gc_arena::{Collect, CollectionContext};

/// Represents a single slot on an object.
#[derive(Clone, Debug)]
pub enum Slot<'gc> {
    /// An unoccupied slot.
    ///
    /// Attempts to read an unoccupied slot proceed up the prototype chain.
    /// Writing an unoccupied slot will always fail.
    Unoccupied,

    /// An occupied slot.
    Occupied {
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    },
}

unsafe impl<'gc> Collect for Slot<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Unoccupied => {}
            Self::Occupied { value, .. } => value.trace(cc),
        }
    }
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
            attributes: EnumSet::empty(),
        }
    }

    /// Create a `const` slot that cannot be overwritten.
    pub fn new_const(value: impl Into<Value<'gc>>) -> Self {
        Self::Occupied {
            value: value.into(),
            attributes: EnumSet::from(Attribute::ReadOnly),
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
                if attributes.contains(Attribute::ReadOnly) {
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
