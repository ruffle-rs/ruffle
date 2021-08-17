//! User-defined properties

use crate::avm1::{Object, Value};
use bitflags::bitflags;
use core::fmt;
use gc_arena::Collect;

bitflags! {
    /// Attributes of properties in the AVM runtime.
    /// The values are significant and should match the order used by `object::as_set_prop_flags`.
    #[derive(Collect)]
    #[collect(require_static)]
    pub struct Attribute: u8 {
        const DONT_ENUM   = 1 << 0;
        const DONT_DELETE = 1 << 1;
        const READ_ONLY   = 1 << 2;
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum Property<'gc> {
    Virtual {
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    },
    Stored {
        value: Value<'gc>,
        attributes: Attribute,
    },
}

impl<'gc> Property<'gc> {
    /// Set a property slot.
    ///
    /// This function may return an `Executable` of the property's virtual
    /// function, if any happen to exist. It should be resolved, and its value
    /// discarded.
    pub fn set(&mut self, new_value: impl Into<Value<'gc>>) -> Option<Object<'gc>> {
        match self {
            Property::Virtual {
                set: Some(function),
                ..
            } => Some(function.to_owned()),
            Property::Stored {
                value, attributes, ..
            } => {
                if !attributes.contains(Attribute::READ_ONLY) {
                    *value = new_value.into();
                }

                None
            }
            _ => None,
        }
    }

    /// List this property's attributes.
    pub fn attributes(&self) -> Attribute {
        match self {
            Property::Virtual { attributes, .. } => *attributes,
            Property::Stored { attributes, .. } => *attributes,
        }
    }

    /// Re-define this property's attributes.
    pub fn set_attributes(&mut self, new_attributes: Attribute) {
        match self {
            Property::Virtual {
                ref mut attributes, ..
            } => *attributes = new_attributes,
            Property::Stored {
                ref mut attributes, ..
            } => *attributes = new_attributes,
        }
    }

    pub fn can_delete(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(Attribute::DONT_DELETE),
            Property::Stored { attributes, .. } => !attributes.contains(Attribute::DONT_DELETE),
        }
    }

    pub fn is_enumerable(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(Attribute::DONT_ENUM),
            Property::Stored { attributes, .. } => !attributes.contains(Attribute::DONT_ENUM),
        }
    }

    #[allow(dead_code)]
    pub fn is_overwritable(&self) -> bool {
        match self {
            Property::Virtual {
                attributes, set, ..
            } => !attributes.contains(Attribute::READ_ONLY) && !set.is_none(),
            Property::Stored { attributes, .. } => !attributes.contains(Attribute::READ_ONLY),
        }
    }

    pub fn is_virtual(&self) -> bool {
        matches!(self, Property::Virtual { .. })
    }
}

impl fmt::Debug for Property<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Property::Virtual {
                get: _,
                set,
                attributes,
            } => f
                .debug_struct("Property::Virtual")
                .field("get", &true)
                .field("set", &set.is_some())
                .field("attributes", &attributes)
                .finish(),
            Property::Stored { value, attributes } => f
                .debug_struct("Property::Stored")
                .field("value", &value)
                .field("attributes", &attributes)
                .finish(),
        }
    }
}
