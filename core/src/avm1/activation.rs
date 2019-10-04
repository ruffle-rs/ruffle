//! Activation records

use std::sync::Arc;
use std::cell::{Ref, RefMut};
use gc_arena::{GcCell, MutationContext};
use crate::tag_utils::SwfSlice;
use crate::avm1::scope::Scope;
use crate::avm1::object::Object;
use crate::avm1::Value;

/// Represents a single activation of a given AVM1 function or keyframe.
#[derive(Clone)]
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

    /// All defined local variables in this stack frame.
    scope: GcCell<'gc, Scope<'gc>>,

    /// The immutable value of `this`.
    this: GcCell<'gc, Object<'gc>>,

    /// The arguments this function was called by.
    arguments: Option<GcCell<'gc, Object<'gc>>>,

    /// Indicates if this activation object represents a function or embedded
    /// block (e.g. ActionWith).
    is_function: bool,

    /// Local registers, if any.
    /// 
    /// None indicates a function executing out of the global register set.
    /// Some indicates the existence of local registers, even if none exist.
    /// i.e. None(Vec::new()) means no registers should exist at all.
    /// 
    /// Registers are numbered from 1; r0 does not exist. Therefore this vec,
    /// while nominally starting from zero, actually starts from r1.
    /// 
    /// Registers are stored in a `GcCell` so that rescopes (e.g. with) use the
    /// same register set.
    local_registers: Option<GcCell<'gc, Vec<Value<'gc>>>>
}

unsafe impl<'gc> gc_arena::Collect for Activation<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.scope.trace(cc);
        self.this.trace(cc);
        self.arguments.trace(cc);
        self.local_registers.trace(cc);
    }
}

impl<'gc> Activation<'gc> {
    pub fn from_action(swf_version: u8, code: SwfSlice, scope: GcCell<'gc, Scope<'gc>>, this: GcCell<'gc, Object<'gc>>, arguments: Option<GcCell<'gc, Object<'gc>>>) -> Activation<'gc> {
        Activation {
            swf_version: swf_version,
            data: code,
            pc: 0,
            scope: scope,
            this: this,
            arguments: arguments,
            is_function: false,
            local_registers: None
        }
    }

    pub fn from_function(swf_version: u8, code: SwfSlice, scope: GcCell<'gc, Scope<'gc>>, this: GcCell<'gc, Object<'gc>>, arguments: Option<GcCell<'gc, Object<'gc>>>) -> Activation<'gc> {
        Activation {
            swf_version: swf_version,
            data: code,
            pc: 0,
            scope: scope,
            this: this,
            arguments: arguments,
            is_function: true,
            local_registers: None
        }
    }

    /// Construct an empty stack frame with no code.
    /// 
    /// This is primarily intended for testing purposes: the activation given
    /// will prevent the AVM from panicking without a current activation.
    /// We construct a single scope chain from a global object, and that's about
    /// it.
    pub fn from_nothing(swf_version: u8, globals: GcCell<'gc, Object<'gc>>, mc: MutationContext<'gc, '_>) -> Activation<'gc> {
        let global_scope = GcCell::allocate(mc, Scope::from_global_object(globals));
        let child_scope = GcCell::allocate(mc, Scope::new_local_scope(global_scope, mc));

        Activation {
            swf_version: swf_version,
            data: SwfSlice {
                data: Arc::new(Vec::new()),
                start: 0,
                end: 0
            },
            pc: 0,
            scope: child_scope,
            this: globals,
            arguments: None,
            is_function: false,
            local_registers: None
        }
    }

    /// Create a new activation to run a block of code with a given scope.
    pub fn to_rescope(&self, code: SwfSlice, scope: GcCell<'gc, Scope<'gc>>) -> Self {
        Activation {
            swf_version: self.swf_version,
            data: code,
            pc: 0,
            scope: scope,
            this: self.this,
            arguments: self.arguments,
            is_function: false,
            local_registers: self.local_registers.clone()
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

    /// Change the data being executed.
    pub fn set_data(&mut self, new_data: SwfSlice) {
        self.data = new_data;
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

    /// Returns AVM local variable scope.
    pub fn scope(&self) -> Ref<Scope<'gc>> {
        self.scope.read()
    }

    /// Returns AVM local variable scope for mutation.
    pub fn scope_mut(&mut self, mc: MutationContext<'gc, '_>) -> RefMut<Scope<'gc>> {
        self.scope.write(mc)
    }

    /// Returns AVM local variable scope for reference.
    pub fn scope_cell(&self) -> GcCell<'gc, Scope<'gc>> {
        self.scope.clone()
    }

    /// Indicates whether or not the end of this scope should be handled as an
    /// implicit function return or the end of a block.
    pub fn can_implicit_return(&self) -> bool {
        self.is_function
    }

    /// Resolve a particular named local variable within this activation.
    pub fn resolve(&self, name: &str) -> Value<'gc> {
        if name == "this" {
            return Value::Object(self.this);
        }

        if name == "arguments" && self.arguments.is_some() {
            return Value::Object(self.arguments.unwrap());
        }

        self.scope().resolve(name)
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(&self, name: &str) -> bool {
        if name == "this" {
            return true;
        }

        if name == "arguments" && self.arguments.is_some() {
            return true;
        }

        self.scope().is_defined(name)
    }

    /// Define a named local variable within this activation.
    pub fn define(&self, name: &str, value: Value<'gc>, mc: MutationContext<'gc, '_>) {
        self.scope().define(name, value, mc)
    }

    /// Returns value of `this` as a reference.
    pub fn this_cell(&self) -> GcCell<'gc, Object<'gc>> {
        self.this
    }
    
    /// Returns true if this function was called with a local register set.
    pub fn has_local_registers(&self) -> bool {
        self.local_registers.is_some()
    }

    pub fn allocate_local_registers(&mut self, num: u8, mc: MutationContext<'gc, '_>) {
        self.local_registers = Some(GcCell::allocate(mc, vec![Value::Undefined; num as usize]));
    }

    /// Retrieve a local register.
    pub fn local_register(&self, id: u8) -> Value<'gc> {
        if let Some(local_registers) = self.local_registers {
            local_registers.read().get(id as usize).map(|v| v.clone()).unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }
    }

    /// Set a local register.
    pub fn set_local_register(&mut self, id: u8, value: Value<'gc>, mc: MutationContext<'gc, '_>) {
        if let Some(ref mut local_registers) = self.local_registers {
            local_registers.write(mc).get_mut(id as usize).map(|r| *r = value);
        }
    }
}