use crate::debug::debug_value::DValue;
use serde::{Deserialize, Serialize};

/// Debug messages that are handled by the AVM1 VM
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Avm1Msg {
    /// Get the current registers
    GetRegisters,

    /// Get the current backtrace
    GetBacktrace,

    /// Execute until the end of the current scope
    StepOut,

    /// Execute a single avm1 opcode, following calls
    StepInto,

    /// Execute a single avm1 opcode, without following calls
    StepOver,

    /// Break execution at the current position
    Break,

    /// Get the current state of the AVM1 stack
    GetStack,

    /// Resume execution
    Continue,

    /// Get the current state of the constant pool
    GetConstantPool,

    /// Break on calling the given function
    BreakFunction { name: String },

    /// Remove the function breakpoint with the given name
    BreakFunctionDelete { name: String },

    /// Get all the breakpoints
    GetBreakpoints,

    /// Push a value onto the stack
    Push { val: DValue },

    /// Pop the top value off of the stack
    Pop,

    /// Get the value at the given path
    GetVariable { path: String },

    /// Set the value at the given path
    SetVariable { path: String, value: DValue },

    /// Get the sub-properties of the value the given path
    GetSubprops { path: String },

    /// Get global variables
    GetGlobals,

    /// Get local variables
    GetLocals,
}
