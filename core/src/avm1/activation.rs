//! Activation records

use std::sync::Arc;
use std::cell::{Ref, RefMut};
use gc_arena::{GcCell, MutationContext};
use crate::tag_utils::SwfSlice;
use crate::avm1::scope::Scope;
use crate::avm1::Value;

/// Represents a single activation of a given AVM1 function or keyframe.
pub struct Activation<'gc> {
    /// Represents the SWF version of a given function.
    /// 
    /// Certain AVM1 operations change behavior based on the version of the SWF
    /// file they were defined in. For example, case sensitivity changes based
    /// on the SWF version.
    swf_version: u8,

    /// Action data being executed by the reader below.
    data: SwfSlice,

    /// The current location of the instruction stream being executed.
    pc: usize,

    /// The opcode stack values for the current stack frame.
    stack: Vec<Value<'gc>>,

    /// All defined local variables in this stack frame.
    scope: GcCell<'gc, Scope<'gc>>,
}

unsafe impl<'gc> gc_arena::Collect for Activation<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.stack.trace(cc);
        self.scope.trace(cc);
    }
}

impl<'gc> Activation<'gc> {
    pub fn from_action(swf_version: u8, code: SwfSlice, scope: GcCell<'gc, Scope<'gc>>) -> Activation<'gc> {
        Activation {
            swf_version: swf_version,
            data: code,
            pc: 0,
            stack: Vec::new(),
            scope: scope
        }
    }

    /// Returns the SWF version of the action or function being executed.
    pub fn swf_version(&self) -> u8 {
        self.swf_version
    }

    /// Returns the data this stack frame executes from.
    pub fn data(&self) -> SwfSlice {
        self.data.clone()
    }

    /// Determines if a stack frame references the same function as a given
    /// SwfSlice.
    pub fn is_identical_fn(&self, other: &SwfSlice) -> bool {
        Arc::ptr_eq(&self.data.data, &other.data)
    }

    /// Returns a mutable reference to the current data offset.
    pub fn pc(&self) -> usize {
        self.pc
    }
    
    /// Change the current PC.
    pub fn set_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }

    /// Returns the value stack.
    pub fn stack(&self) -> &Vec<Value<'gc>> {
        &self.stack
    }

    /// Returns the value stack for mutation.
    pub fn stack_mut(&mut self) -> &mut Vec<Value<'gc>> {
        &mut self.stack
    }

    /// Returns AVM local variable scope.
    pub fn scope(&self) -> Ref<Scope<'gc>> {
        self.scope.read()
    }

    /// Returns AVM local variable scope for mutation.
    pub fn scope_mut(&mut self, mc: MutationContext<'gc, '_>) -> RefMut<Scope<'gc>> {
        self.scope.write(mc)
    }

    /// Resolve a particular named local variable within this activation.
    pub fn resolve(&self, name: &str) -> Value<'gc> {
        self.scope().resolve(name)
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(&self, name: &str) -> bool {
        self.scope().is_defined(name)
    }

    /// Define a named local variable within this activation.
    pub fn define(&self, name: &str, value: Value<'gc>, mc: MutationContext<'gc, '_>) {
        self.scope().define(name, value, mc)
    }
}