//! Definition of commands

use ruffle_core::debugable::DValue;

/// Commands that the debugger client can send the the current debuggee
#[derive(Debug, Clone)]
pub enum Command {
    /// Pause at the start of the next frame
    Pause,

    /// Resume execution of the next frame
    Play,

    /// Get information about the display object at the given path
    Info { path: String },

    /// Get the children of the display object at the given depth
    GetChildren { path: String },

    /// Get the properties on this object
    GetProps { path: String },

    /// Get the value of a property
    GetPropValue { path: String, name: String },

    /// Set the value of a property
    SetPropValue {
        path: String,
        name: String,
        value: DValue,
    },

    /// Stop the current display object
    StopDO { path: String },

    /// Break the execution of AVM1
    Avm1Break,

    /// Get the state of the AVM1 stack
    Avm1Stack,

    /// Execute the next instruction, stepping into function calls
    Avm1StepInto,

    /// Execute until the current scope returns
    Avm1StepOut,

    /// Add a breakpoint that will break when `name` is called, either as a function or a method
    Avm1FunctionBreak { name: String },

    /// Remove any breakpoint on `name`
    Avm1FunctionBreakDelete { name: String },

    /// Continue execution
    Avm1Continue,

    /// Push a value onto the stack
    Avm1Push { val: DValue },

    /// Pop a value from the stack
    Avm1Pop,

    /// Get all the current breakpoints
    Avm1BreakpointsGet,

    /// Get the value of a avm1 variable
    Avm1VariableGet { path: String },

    /// Set the value of a avm1 variable
    Avm1VariableSet { path: String, value: DValue },

    /// Get the sub-properties of an avm1 variable
    Avm1SubpropGet { path: String },

    /// Get the avm1 backtrace
    Avm1Backtrace,

    /// Get the avm1 registers
    Avm1Registers,

    /// Get global variables
    Avm1Globals,

    /// Get local variables
    Avm1Locals,
}

impl Command {
    /// Does this command require that the vm is not currently executing, for now that's basically
    /// all avm commands
    pub fn requires_paused_vm(&self) -> bool {
        matches!(
            self,
            Self::Avm1StepOut
                | Self::Avm1StepInto
                | Self::Avm1VariableSet { .. }
                | Self::Avm1Backtrace
                | Self::Avm1Locals
                | Self::Avm1Globals
                | Self::Avm1Continue
                | Self::Avm1Pop
                | Self::Avm1Push { .. }
                | Self::Avm1Registers
                | Self::Avm1Stack
                | Self::Avm1SubpropGet { .. }
        )
    }
}
