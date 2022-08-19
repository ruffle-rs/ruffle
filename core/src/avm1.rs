#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
pub mod function;
#[macro_use]
pub mod property_decl;

pub mod activation;
mod callable_value;
pub mod debug;
pub mod error;
mod fscommand;
pub mod globals;
pub mod object;
pub mod property;
pub mod property_map;
pub mod runtime;
mod scope;
mod value;

#[cfg(test)]
mod tests;

pub use crate::avm1::activation::{Activation, ActivationIdentifier};
pub use crate::avm1::error::Error;
use crate::string::AvmString;
pub use globals::SystemPrototypes;
pub use object::array_object::ArrayObject;
pub use object::script_object::ScriptObject;
pub use object::sound_object::SoundObject;
pub use object::stage_object::StageObject;
pub use object::{Object, ObjectPtr, TObject};
pub use value::Value;

#[macro_export]
macro_rules! avm_warn {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            log::warn!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            log::warn!($($arg)*)
        }
    )
}

#[macro_export]
macro_rules! avm_error {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            log::error!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            log::error!($($arg)*)
        }
    )
}
