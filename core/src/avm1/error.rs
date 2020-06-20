use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Prototype recursion limit has been exceeded")]
    PrototypeRecursionLimit,

    #[error("Couldn't parse SWF. This may or may not be a bug in Ruffle, please help us by reporting it to https://github.com/ruffle-rs/ruffle/issues and include the swf that triggered it.")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("No stack frame to execute. This is probably a bug in Ruffle, please report it to https://github.com/ruffle-rs/ruffle/issues and include the swf that triggered it.")]
    NoStackFrame,

    #[error("Attempted to run a frame not on the current interpreter stack. This is probably a bug in Ruffle, please report it to https://github.com/ruffle-rs/ruffle/issues and include the swf that triggered it.")]
    FrameNotOnStack,

    #[error("Attempted to execute the same frame twice. This is probably a bug in Ruffle, please report it to https://github.com/ruffle-rs/ruffle/issues and include the swf that triggered it.")]
    AlreadyExecutingFrame,
}

impl Error {
    pub fn is_halting(&self) -> bool {
        match self {
            Error::PrototypeRecursionLimit => true,
            Error::InvalidSwf(_) => true,
            Error::NoStackFrame => true,
            Error::FrameNotOnStack => true,
            Error::AlreadyExecutingFrame => false,
        }
    }
}
