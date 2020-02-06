//! Activation frames

use crate::avm2::function::Avm2Function;
use crate::avm2::object::Object;
use crate::avm2::Error;
use gc_arena::{Collect, Gc};

/// Represents a single activation of a given AVM2 function or keyframe.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Activation<'gc> {
    /// The function being executed.
    action: Gc<'gc, Avm2Function>,

    /// The current location of the instruction stream being executed.
    pc: usize,

    /// The immutable value of `this`.
    this: Object<'gc>,

    /// The arguments this function was called by.
    arguments: Option<Object<'gc>>,

    /// Flags that the current activation frame is being executed and has a
    /// reader object copied from it. Taking out two readers on the same
    /// activation frame is a programming error.
    is_executing: bool,
}

impl<'gc> Activation<'gc> {
    pub fn from_action(
        action: Gc<'gc, Avm2Function>,
        this: Object<'gc>,
        arguments: Option<Object<'gc>>,
    ) -> Self {
        Self {
            action,
            pc: 0,
            this,
            arguments,
            is_executing: false,
        }
    }

    /// Attempts to lock the activation frame for execution.
    ///
    /// If this frame is already executing, that is an error condition.
    pub fn lock(&mut self) -> Result<(), Error> {
        if self.is_executing {
            return Err("Attempted to execute the same frame twice".into());
        }

        self.is_executing = true;

        Ok(())
    }

    /// Unlock the activation object. This allows future execution to run on it
    /// again.
    pub fn unlock_execution(&mut self) {
        self.is_executing = false;
    }

    /// Obtain a reference to the function being executed.
    pub fn action(&self) -> Gc<'gc, Avm2Function> {
        self.action
    }

    /// Get the PC value.
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Set the PC value.
    pub fn set_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }
}
