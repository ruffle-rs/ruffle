//! Property data structures

use gc_arena::Collect;

#[derive(Debug, Collect, Clone, Copy)]
#[collect(require_static)]
pub enum Property {
    Virtual { get: Option<u32>, set: Option<u32> },
    Method { disp_id: u32 },
    Slot { slot_id: u32 },
    ConstSlot { slot_id: u32 },
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
