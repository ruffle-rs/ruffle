//! Activation records

use crate::avm1::object::Object;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::scope::Scope;
use crate::avm1::stack_continuation::StackContinuation;
use crate::avm1::{Avm1, Value};
use crate::context::UpdateContext;
use crate::tag_utils::SwfSlice;
use gc_arena::{GcCell, MutationContext};
use smallvec::SmallVec;
use std::cell::{Ref, RefMut};
use std::mem::swap;
use std::sync::Arc;

/// Represents a particular register set.
///
/// This type exists primarily because SmallVec isn't garbage-collectable.
#[derive(Clone)]
pub struct RegisterSet<'gc>(SmallVec<[Value<'gc>; 8]>);

unsafe impl<'gc> gc_arena::Collect for RegisterSet<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        for register in &self.0 {
            register.trace(cc);
        }
    }
}

impl<'gc> RegisterSet<'gc> {
    /// Create a new register set with a given number of specified registers.
    ///
    /// The given registers will be set to `undefined`.
    pub fn new(num: u8) -> Self {
        Self(smallvec![Value::Undefined; num as usize])
    }

    /// Return a reference to a given register, if it exists.
    pub fn get(&self, num: u8) -> Option<&Value<'gc>> {
        self.0.get(num as usize)
    }

    /// Return a mutable reference to a given register, if it exists.
    pub fn get_mut(&mut self, num: u8) -> Option<&mut Value<'gc>> {
        self.0.get_mut(num as usize)
    }

    pub fn len(&self) -> u8 {
        self.0.len() as u8
    }
}

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
    local_registers: Option<GcCell<'gc, RegisterSet<'gc>>>,

    /// Native code to execute when the given activation frame returns.
    ///
    /// This facility exists primarily to allow native code to handle the result
    /// of an AVM function call.
    then_func: Option<Box<dyn StackContinuation<'gc>>>,
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
    pub fn from_action(
        swf_version: u8,
        code: SwfSlice,
        scope: GcCell<'gc, Scope<'gc>>,
        this: GcCell<'gc, Object<'gc>>,
        arguments: Option<GcCell<'gc, Object<'gc>>>,
    ) -> Activation<'gc> {
        Activation {
            swf_version,
            data: code,
            pc: 0,
            scope,
            this,
            arguments,
            is_function: false,
            local_registers: None,
            then_func: None,
        }
    }

    pub fn from_function(
        swf_version: u8,
        code: SwfSlice,
        scope: GcCell<'gc, Scope<'gc>>,
        this: GcCell<'gc, Object<'gc>>,
        arguments: Option<GcCell<'gc, Object<'gc>>>,
    ) -> Activation<'gc> {
        Activation {
            swf_version,
            data: code,
            pc: 0,
            scope,
            this,
            arguments,
            is_function: true,
            local_registers: None,
            then_func: None,
        }
    }

    /// Construct an empty stack frame with no code.
    ///
    /// This is primarily intended for testing purposes: the activation given
    /// will prevent the AVM from panicking without a current activation.
    /// We construct a single scope chain from a global object, and that's about
    /// it.
    pub fn from_nothing(
        swf_version: u8,
        globals: GcCell<'gc, Object<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Activation<'gc> {
        let global_scope = GcCell::allocate(mc, Scope::from_global_object(globals));
        let child_scope = GcCell::allocate(mc, Scope::new_local_scope(global_scope, mc));

        Activation {
            swf_version,
            data: SwfSlice {
                data: Arc::new(Vec::new()),
                start: 0,
                end: 0,
            },
            pc: 0,
            scope: child_scope,
            this: globals,
            arguments: None,
            is_function: false,
            local_registers: None,
            then_func: None,
        }
    }

    /// Create a new activation to run a block of code with a given scope.
    pub fn to_rescope(&self, code: SwfSlice, scope: GcCell<'gc, Scope<'gc>>) -> Self {
        Activation {
            swf_version: self.swf_version,
            data: code,
            pc: 0,
            scope,
            this: self.this,
            arguments: self.arguments,
            is_function: false,
            local_registers: self.local_registers,
            then_func: None,
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
        self.scope
    }

    /// Completely replace the current scope with a new one.
    pub fn set_scope(&mut self, scope: GcCell<'gc, Scope<'gc>>) {
        self.scope = scope;
    }

    /// Indicates whether or not the end of this scope should be handled as an
    /// implicit function return or the end of a block.
    pub fn can_implicit_return(&self) -> bool {
        self.is_function
    }

    /// Resolve a particular named local variable within this activation.
    ///
    /// Because scopes are object chains, the same rules for `Object::get`
    /// still apply here. This function is allowed to yield `None` to indicate
    /// that the result will be calculated on the AVM stack.
    pub fn resolve(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> ReturnValue<'gc> {
        if name == "this" {
            return ReturnValue::Immediate(Value::Object(self.this));
        }

        if name == "arguments" && self.arguments.is_some() {
            return ReturnValue::Immediate(Value::Object(self.arguments.unwrap()));
        }

        self.scope().resolve(name, avm, context, self.this)
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
    pub fn define(&self, name: &str, value: impl Into<Value<'gc>>, mc: MutationContext<'gc, '_>) {
        self.scope().define(name, value, mc)
    }

    /// Returns value of `this` as a reference.
    pub fn this_cell(&self) -> GcCell<'gc, Object<'gc>> {
        self.this
    }

    /// Returns true if this activation has a given local register ID.
    pub fn has_local_register(&self, id: u8) -> bool {
        self.local_registers
            .map(|rs| id < rs.read().len())
            .unwrap_or(false)
    }

    pub fn allocate_local_registers(&mut self, num: u8, mc: MutationContext<'gc, '_>) {
        self.local_registers = match num {
            0 => None,
            num => Some(GcCell::allocate(mc, RegisterSet::new(num))),
        };
    }

    /// Retrieve a local register.
    pub fn local_register(&self, id: u8) -> Option<Value<'gc>> {
        if let Some(local_registers) = self.local_registers {
            local_registers.read().get(id).cloned()
        } else {
            None
        }
    }

    /// Set a local register.
    pub fn set_local_register(
        &mut self,
        id: u8,
        value: impl Into<Value<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) {
        if let Some(ref mut local_registers) = self.local_registers {
            if let Some(r) = local_registers.write(mc).get_mut(id) {
                *r = value.into();
            }
        }
    }

    /// Return the function scheduled to be executed, if any.
    pub fn get_then_func(&mut self) -> Option<&mut dyn StackContinuation<'gc>> {
        match &mut self.then_func {
            Some(f) => Some(f.as_mut()),
            None => None,
        }
    }

    /// Schedule a native function to execute when this stack frame returns.
    ///
    /// Only one native function may be scheduled per activation. It will be
    /// called in lieu of pushing the return value onto the stack, and may
    /// perform any necessary AVM action.
    pub fn and_then(&mut self, func: Box<dyn StackContinuation<'gc>>) {
        self.then_func = Some(func);
    }

    /// Reschedule an already-queued native function from an existing frame.
    ///
    /// The existing advice listed in `and_then` applies here. The previous
    /// activation's continuation will be set to `None`.
    pub fn and_again(
        &mut self,
        previous_activation: GcCell<'gc, Activation<'gc>>,
        context: MutationContext<'gc, '_>,
    ) {
        self.then_func = None;
        swap(
            &mut self.then_func,
            &mut previous_activation.write(context).then_func,
        );
    }
}
