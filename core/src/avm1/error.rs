use crate::avm1::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<'gc> {
    #[error("Prototype recursion limit has been exceeded")]
    PrototypeRecursionLimit,

    #[error("256 levels of function recursion were exceeded in one action list. This is probably an infinite loop.")]
    FunctionRecursionLimit,

    #[error("66 levels of special recursion were exceeded in one action list. This is probably an infinite loop.")]
    SpecialRecursionLimit,

    #[error("Couldn't parse SWF. This may or may not be a bug in Ruffle, please help us by reporting it to https://github.com/ruffle-rs/ruffle/issues and include the swf that triggered it.")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("A script has thrown a custom error.")]
    ThrownValue(Value<'gc>),
}

impl Error<'_> {
    pub fn is_halting(&self) -> bool {
        match self {
            Error::PrototypeRecursionLimit => true,
            Error::FunctionRecursionLimit => true,
            Error::SpecialRecursionLimit => true,
            Error::InvalidSwf(_) => true,
            Error::ThrownValue(_) => false,
        }
    }
}
