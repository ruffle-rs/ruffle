use crate::avm1::Value;
use thiserror::Error;

#[derive(Error, Debug)]
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
