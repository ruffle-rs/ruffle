//! Activation records

use crate::avm1::error::Error;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::scope::Scope;
use crate::avm1::stack_frame::StackFrame;
use crate::avm1::{Object, Value};
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use crate::tag_utils::SwfSlice;
use gc_arena::{GcCell, MutationContext};
use smallvec::SmallVec;
use std::cell::{Ref, RefMut};
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

    /// The currently in use constant pool.
    constant_pool: GcCell<'gc, Vec<String>>,

    /// The immutable value of `this`.
    this: Object<'gc>,

    /// The arguments this function was called by.
    arguments: Option<Object<'gc>>,

    /// The return value of the activation.
    return_value: Option<Value<'gc>>,

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

    /// Flags that the current activation frame is being executed and has a
    /// reader object copied from it. Taking out two readers on the same
    /// activation frame is a programming error.
    is_executing: bool,

    /// The base clip of this stack frame.
    /// This will be the movieclip that contains the bytecode.
    base_clip: DisplayObject<'gc>,

    /// The current target display object of this stack frame.
    /// This can be changed with `tellTarget` (via `ActionSetTarget` and `ActionSetTarget2`).
    target_clip: Option<DisplayObject<'gc>>,
}

unsafe impl<'gc> gc_arena::Collect for Activation<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.scope.trace(cc);
        self.constant_pool.trace(cc);
        self.this.trace(cc);
        self.arguments.trace(cc);
        self.return_value.trace(cc);
        self.local_registers.trace(cc);
        self.base_clip.trace(cc);
        self.target_clip.trace(cc);
    }
}

impl<'gc> Activation<'gc> {
    pub fn from_action(
        swf_version: u8,
        code: SwfSlice,
        scope: GcCell<'gc, Scope<'gc>>,
        constant_pool: GcCell<'gc, Vec<String>>,
        base_clip: DisplayObject<'gc>,
        this: Object<'gc>,
        arguments: Option<Object<'gc>>,
    ) -> Activation<'gc> {
        Activation {
            swf_version,
            data: code,
            pc: 0,
            scope,
            constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            this,
            arguments,
            return_value: None,
            is_function: false,
            local_registers: None,
            is_executing: false,
        }
    }

    pub fn from_function(
        swf_version: u8,
        code: SwfSlice,
        scope: GcCell<'gc, Scope<'gc>>,
        constant_pool: GcCell<'gc, Vec<String>>,
        base_clip: DisplayObject<'gc>,
        this: Object<'gc>,
        arguments: Option<Object<'gc>>,
    ) -> Activation<'gc> {
        Activation {
            swf_version,
            data: code,
            pc: 0,
            scope,
            constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            this,
            arguments,
            return_value: None,
            is_function: true,
            local_registers: None,
            is_executing: false,
        }
    }

    /// Construct an empty stack frame with no code.
    ///
    /// This is used by tests and by callback methods (`onEnterFrame`) to create a base
    /// activation frame with access to the global context.
    pub fn from_nothing(
        swf_version: u8,
        globals: Object<'gc>,
        mc: MutationContext<'gc, '_>,
        base_clip: DisplayObject<'gc>,
    ) -> Activation<'gc> {
        use crate::tag_utils::SwfMovie;

        let global_scope = GcCell::allocate(mc, Scope::from_global_object(globals));
        let child_scope = GcCell::allocate(mc, Scope::new_local_scope(global_scope, mc));
        let empty_constant_pool = GcCell::allocate(mc, Vec::new());

        Activation {
            swf_version,
            data: SwfSlice {
                movie: Arc::new(SwfMovie::empty(swf_version)),
                start: 0,
                end: 0,
            },
            pc: 0,
            scope: child_scope,
            constant_pool: empty_constant_pool,
            base_clip,
            target_clip: Some(base_clip),
            this: globals,
            arguments: None,
            return_value: None,
            is_function: false,
            local_registers: None,
            is_executing: false,
        }
    }

    /// Create a new activation to run a block of code with a given scope.
    pub fn to_rescope(&self, code: SwfSlice, scope: GcCell<'gc, Scope<'gc>>) -> Self {
        Activation {
            swf_version: self.swf_version,
            data: code,
            pc: 0,
            scope,
            constant_pool: self.constant_pool,
            base_clip: self.base_clip,
            target_clip: self.target_clip,
            this: self.this,
            arguments: self.arguments,
            return_value: None,
            is_function: false,
            local_registers: self.local_registers,
            is_executing: false,
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
    #[allow(dead_code)]
    pub fn set_data(&mut self, new_data: SwfSlice) {
        self.data = new_data;
    }

    /// Determines if a stack frame references the same function as a given
    /// SwfSlice.
    #[allow(dead_code)]
    pub fn is_identical_fn(&self, other: &SwfSlice) -> bool {
        Arc::ptr_eq(&self.data.movie, &other.movie)
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
    #[allow(dead_code)]
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

    /// Gets the base clip of this stack frame.
    /// This is the movie clip that contains the executing bytecode.
    pub fn base_clip(&self) -> DisplayObject<'gc> {
        self.base_clip
    }

    /// Gets the current target clip of this stack frame.
    /// This is the movie clip to which `GotoFrame` and other actions apply.
    /// Changed via `ActionSetTarget`/`ActionSetTarget2`.
    pub fn target_clip(&self) -> Option<DisplayObject<'gc>> {
        self.target_clip
    }

    /// Changes the target clip.
    pub fn set_target_clip(&mut self, value: Option<DisplayObject<'gc>>) {
        self.target_clip = value;
    }

    /// Indicates whether or not the end of this scope should return a value.
    pub fn can_return(&self) -> bool {
        self.is_function
    }

    /// Resolve a particular named local variable within this activation.
    ///
    /// Because scopes are object chains, the same rules for `Object::get`
    /// still apply here.
    pub fn resolve(
        &self,
        name: &str,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error<'gc>> {
        if name == "this" {
            return Ok(Value::Object(self.this).into());
        }

        if name == "arguments" && self.arguments.is_some() {
            return Ok(Value::Object(self.arguments.unwrap()).into());
        }

        self.scope().resolve(name, activation, context, self.this)
    }

    /// Check if a particular property in the scope chain is defined.
    pub fn is_defined(
        &self,
        activation: &mut StackFrame<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        if name == "this" {
            return true;
        }

        if name == "arguments" && self.arguments.is_some() {
            return true;
        }

        self.scope().is_defined(activation, context, name)
    }

    /// Define a named local variable within this activation.
    pub fn define(&self, name: &str, value: impl Into<Value<'gc>>, mc: MutationContext<'gc, '_>) {
        self.scope().define(name, value, mc)
    }

    /// Returns value of `this` as a reference.
    pub fn this_cell(&self) -> Object<'gc> {
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

    pub fn constant_pool(&self) -> GcCell<'gc, Vec<String>> {
        self.constant_pool
    }

    pub fn set_constant_pool(&mut self, constant_pool: GcCell<'gc, Vec<String>>) {
        self.constant_pool = constant_pool;
    }

    /// Attempts to lock the activation frame for execution.
    ///
    /// If this frame is already executing, that is an error condition.
    pub fn lock(&mut self) -> Result<(), Error<'gc>> {
        if self.is_executing {
            return Err(Error::AlreadyExecutingFrame);
        }

        self.is_executing = true;

        Ok(())
    }

    /// Unlock the activation object. This allows future execution to run on it
    /// again.
    pub fn unlock_execution(&mut self) {
        self.is_executing = false;
    }

    /// Retrieve the return value from a completed activation, if the function
    /// has already returned.
    #[allow(dead_code)]
    pub fn return_value(&self) -> Option<Value<'gc>> {
        self.return_value.clone()
    }

    /// Set the return value.
    pub fn set_return_value(&mut self, value: Value<'gc>) {
        self.return_value = Some(value);
    }
}
