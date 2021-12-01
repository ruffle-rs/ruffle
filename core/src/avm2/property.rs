//! Property data structures

use crate::avm2::Error;
use bitflags::bitflags;
use gc_arena::Collect;

bitflags! {
    /// Attributes of properties in the AVM runtime.
    ///
    /// TODO: Replace with AVM2 properties for traits
    #[derive(Collect)]
    #[collect(require_static)]
    pub struct Attribute: u8 {
        /// Property cannot be deleted in user code.
        const DONT_DELETE = 1 << 0;

        /// Property cannot be set.
        const READ_ONLY   = 1 << 1;
    }
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub enum Property {
    Virtual {
        get: Option<u32>,
        set: Option<u32>,
        attributes: Attribute,
    },
    Method {
        disp_id: u32,
        attributes: Attribute,
    },
    Slot {
        slot_id: u32,
        attributes: Attribute,
    },
}

impl Property {
    /// Convert a function into a method.
    ///
    /// This applies READ_ONLY/DONT_DELETE to the property.
    pub fn new_method(disp_id: u32) -> Self {
        let attributes = Attribute::DONT_DELETE | Attribute::READ_ONLY;

        Property::Method {
            disp_id,
            attributes,
        }
    }

    /// Create a new, unconfigured virtual property item.
    pub fn new_virtual() -> Self {
        let attributes = Attribute::DONT_DELETE | Attribute::READ_ONLY;

        Property::Virtual {
            get: None,
            set: None,
            attributes,
        }
    }

    /// Create a new slot property.
    pub fn new_slot(slot_id: u32) -> Self {
        Property::Slot {
            slot_id,
            attributes: Attribute::DONT_DELETE,
        }
    }

    /// Install a getter into this property.
    ///
    /// This function errors if attempting to install executables into a
    /// non-virtual property.
    ///
    /// The implementation must be a valid function, otherwise the VM will
    /// panic when the property is accessed.
    pub fn install_virtual_getter(&mut self, getter_impl: u32) -> Result<(), Error> {
        match self {
            Property::Virtual { get, .. } => *get = Some(getter_impl),
            _ => return Err("Not a virtual property".into()),
        };

        Ok(())
    }

    /// Install a setter into this property.
    ///
    /// This function errors if attempting to install executables into a
    /// non-virtual property.
    ///
    /// The implementation must be a valid function, otherwise the VM will
    /// panic when the property is accessed.
    pub fn install_virtual_setter(&mut self, setter_impl: u32) -> Result<(), Error> {
        match self {
            Property::Virtual { set, .. } => *set = Some(setter_impl),
            _ => return Err("Not a virtual property".into()),
        };

        Ok(())
    }

    /// Retrieve the slot ID of a property.
    ///
    /// This function yields `None` if this property is not a slot.
    pub fn slot_id(&self) -> Option<u32> {
        match self {
            Property::Slot { slot_id, .. } => Some(*slot_id),
            _ => None,
        }
    }

    pub fn can_delete(&self) -> bool {
        let attributes = match self {
            Property::Virtual { attributes, .. } => attributes,
            Property::Method { attributes, .. } => attributes,
            Property::Slot { attributes, .. } => attributes,
        };
        !attributes.contains(Attribute::DONT_DELETE)
    }

    pub fn is_overwritable(&self) -> bool {
        let attributes = match self {
            Property::Virtual {
                attributes, set, ..
            } => {
                if set.is_none() {
                    return false;
                }
                attributes
            }
            Property::Method { attributes, .. } => attributes,
            Property::Slot { attributes, .. } => attributes,
        };
        !attributes.contains(Attribute::READ_ONLY)
    }
}
