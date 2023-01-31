use crate::debug::debug_value::DValue;
use serde::{Deserialize, Serialize};

/// Debug messages that are handled by a specifically targeted display object
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TargetedMsg {
    /// Get information about this object display object
    GetInfo,

    /// Get children of this display object
    GetChildren,

    /// Get properties on this object
    GetProps,

    /// Get the value of the given prop
    GetPropValue { name: String },

    /// Set the value of the given prop
    SetPropValue { name: String, value: DValue },

    /// Stop this clip
    /// TODO: this only works on clips, should we have a custom(str) msg that allows do-specific behaviour, or should they all be in this enum with a msg that allows getting which ones are available
    Stop,
}
