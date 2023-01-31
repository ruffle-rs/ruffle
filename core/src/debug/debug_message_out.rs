use crate::debug::debug_value::DValue;
use crate::debug::display_object_info::DisplayObjectInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DebugMessageOut {
    /// A generic success / fail message
    GenericResult {
        success: bool,
    },

    /// Result sent when a value is retrieved
    GetValueResult {
        path: String,
        value: DValue,
    },

    /// Result sent when requesting sub-properties of an object
    GetSubpropsResult {
        path: String,
        props: Vec<String>,
    },

    /// Result sent when requesting a backtrace
    GetBacktraceResult {
        backtrace: Vec<String>,
    },

    /// Result send when requesting registers
    GetRegisterResult {
        regs: Vec<DValue>,
    },

    /// Result sent when requesting locals
    GetLocalsResult {
        locals: Vec<String>,
    },

    /// Result sent when requesting globals
    GetGlobalsResult {
        globals: Vec<String>,
    },

    /// Message sent when a trace is logged
    LogTrace(String),

    State {
        playing: bool,
    },
    BreakpointHit {
        name: String,
    },
    GetVarResult {
        value: String,
    },
    DisplayObjectInfo(DisplayObjectInfo),
    GetPropsResult {
        keys: Vec<String>,
    },
    BreakpointList {
        bps: Vec<String>,
    },
}
