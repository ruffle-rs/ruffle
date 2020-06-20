use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Couldn't parse SWF")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("No stack frame to execute")]
    NoStackFrame,

    #[error("Attempted to run a frame not on the current interpreter stack")]
    FrameNotOnStack,

    #[error("Attempted to execute the same frame twice")]
    AlreadyExecutingFrame,

    #[error("Script error")]
    ScriptError(Box<dyn std::error::Error>),
}
