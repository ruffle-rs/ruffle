#[cfg(test)]
#[macro_use]
mod test_utils;

#[macro_use]
mod function;
#[macro_use]
mod property_decl;

mod activation;
mod callable_value;
mod clamp;
mod debug;
mod error;
mod flv;
mod fscommand;
pub(crate) mod globals;
mod object;
mod object_reference;
mod property;
mod property_map;
mod runtime;
mod scope;
mod value;

#[cfg(test)]
mod tests;

pub use activation::{Activation, ActivationIdentifier};
pub use debug::VariableDumper;
pub use error::Error;
pub use flv::FlvValueAvm1Ext;
pub use function::{Executable, ExecutionReason};
pub use globals::context_menu::make_context_menu_state;
pub use globals::sound::start as start_sound;
pub use globals::system::SystemProperties;
pub use object::array_object::ArrayObject;
pub use object::script_object::ScriptObject;
pub use object::sound_object::SoundObject;
pub use object::stage_object::StageObject;
pub use object::{NativeObject, Object, ObjectPtr, TObject};
pub use property::Attribute;
pub use property_map::PropertyMap;
pub use runtime::Avm1;
pub use value::Value;

#[macro_export]
macro_rules! avm_warn {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            tracing::warn!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            tracing::warn!($($arg)*)
        }
    )
}

#[macro_export]
macro_rules! avm_error {
    ($activation: ident, $($arg:tt)*) => (
        if cfg!(feature = "avm_debug") {
            tracing::error!("{} -- in {}", format!($($arg)*), $activation.id)
        } else {
            tracing::error!($($arg)*)
        }
    )
}

#[macro_export]
macro_rules! avm1_stub {
    ($activation: ident, $class: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm1Constructor { class: $class };
        $activation.context.stub_tracker.encounter(&STUB);
    };
    ($activation: ident, $class: literal, $method: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm1Method {
            class: $class,
            method: $method,
            specifics: None,
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
    ($activation: ident, $class: literal, $method: literal, $specifics: literal) => {
        #[cfg_attr(
            feature = "known_stubs",
            linkme::distributed_slice($crate::stub::KNOWN_STUBS)
        )]
        static STUB: $crate::stub::Stub = $crate::stub::Stub::Avm1Method {
            class: $class,
            method: $method,
            specifics: Some($specifics),
        };
        $activation.context.stub_tracker.encounter(&STUB);
    };
}
