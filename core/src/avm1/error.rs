use std::fmt;

use crate::avm1::Value;
use thiserror::Error;

#[derive(Error)]
pub enum Error<'gc> {
    #[error("Prototype recursion limit has been exceeded")]
    PrototypeRecursionLimit,

    #[error("A script in this movie has taken too long to execute and has been terminated.")]
    ExecutionTimeout,

    #[error("{0} levels of function recursion were exceeded in one action list. This is probably an infinite loop.")]
    FunctionRecursionLimit(u16),

    #[error("66 levels of special recursion were exceeded in one action list. This is probably an infinite loop.")]
    SpecialRecursionLimit,

    #[error("Couldn't parse SWF")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("A script has thrown a custom error.")]
    ThrownValue(Value<'gc>),
}

impl<'gc> fmt::Debug for Error<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrototypeRecursionLimit => write!(f, "PrototypeRecursionLimit"),
            Self::ExecutionTimeout => write!(f, "ExecutionTimeout"),
            Self::FunctionRecursionLimit(err) => {
                f.debug_tuple("FunctionRecursionLimit").field(err).finish()
            }
            Self::SpecialRecursionLimit => write!(f, "SpecialRecursionLimit"),
            Self::InvalidSwf(err) => f.debug_tuple("InvalidSwf").field(err).finish(),
            Self::ThrownValue(_) => write!(f, "ThrownValue(_)"),
        }
    }
}
