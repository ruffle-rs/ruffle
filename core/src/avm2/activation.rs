//! Activation frames

use crate::avm2::function::Avm2MethodEntry;
use crate::avm2::object::Object;
use crate::avm2::scope::Scope;
use crate::avm2::script::Script;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell, MutationContext};
use smallvec::SmallVec;

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
    pub fn new(num: u32) -> Self {
        Self(smallvec![Value::Undefined; num as usize])
    }

    /// Return a reference to a given register, if it exists.
    pub fn get(&self, num: u32) -> Option<&Value<'gc>> {
        self.0.get(num as usize)
    }

    /// Return a mutable reference to a given register, if it exists.
    pub fn get_mut(&mut self, num: u32) -> Option<&mut Value<'gc>> {
        self.0.get_mut(num as usize)
    }
}

/// Represents a single activation of a given AVM2 function or keyframe.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Activation<'gc> {
    /// The AVM method entry we're executing code out of.
    method: Avm2MethodEntry<'gc>,

    /// The current location of the instruction stream being executed.
    pc: usize,

    /// The immutable value of `this`.
    this: Option<Object<'gc>>,

    /// The arguments this function was called by.
    arguments: Option<Object<'gc>>,

    /// Flags that the current activation frame is being executed and has a
    /// reader object copied from it. Taking out two readers on the same
    /// activation frame is a programming error.
    is_executing: bool,

    /// Local registers.
    ///
    /// All activations have local registers, but it is possible for multiple
    /// activations (such as a rescope) to execute from the same register set.
    local_registers: GcCell<'gc, RegisterSet<'gc>>,

    /// What was returned from the function.
    ///
    /// A return value of `None` indicates that the called function is still
    /// executing. Functions that do not return instead return `Undefined`.
    return_value: Option<Value<'gc>>,

    /// The current local scope, implemented as a bare object.
    local_scope: Object<'gc>,

    /// The current scope stack.
    ///
    /// A `scope` of `None` indicates that the scope stack is empty.
    scope: Option<GcCell<'gc, Scope<'gc>>>,

    /// The base prototype of `this`.
    ///
    /// This will not be available if this is not a method call.
    base_proto: Option<Object<'gc>>,
}

impl<'gc> Activation<'gc> {
    pub fn from_script(
        context: &mut UpdateContext<'_, 'gc, '_>,
        script: GcCell<'gc, Script<'gc>>,
        global: Object<'gc>,
    ) -> Result<Self, Error> {
        let method = script.read().init().into_entry()?;
        let scope = Some(Scope::push_scope(None, global, context.gc_context));
        let num_locals = method.body().num_locals;
        let local_registers =
            GcCell::allocate(context.gc_context, RegisterSet::new(num_locals + 1));

        *local_registers
            .write(context.gc_context)
            .get_mut(0)
            .unwrap() = global.into();

        Ok(Self {
            method,
            pc: 0,
            this: Some(global),
            arguments: None,
            is_executing: false,
            local_registers,
            return_value: None,
            local_scope: ScriptObject::bare_object(context.gc_context),
            scope,
            base_proto: None,
        })
    }

    pub fn from_action(
        context: &mut UpdateContext<'_, 'gc, '_>,
        method: Avm2MethodEntry<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        this: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        base_proto: Option<Object<'gc>>,
    ) -> Result<Self, Error> {
        let num_locals = method.body().num_locals;
        let num_declared_arguments = method.method().params.len() as u32;
        let local_registers = GcCell::allocate(
            context.gc_context,
            RegisterSet::new(num_locals + num_declared_arguments + 1),
        );

        {
            let mut write = local_registers.write(context.gc_context);
            *write.get_mut(0).unwrap() = this.map(|t| t.into()).unwrap_or(Value::Null);

            for i in 0..num_declared_arguments {
                *write.get_mut(1 + i).unwrap() = arguments
                    .get(i as usize)
                    .cloned()
                    .unwrap_or(Value::Undefined);
            }
        }

        Ok(Self {
            method,
            pc: 0,
            this,
            arguments: None,
            is_executing: false,
            local_registers,
            return_value: None,
            local_scope: ScriptObject::bare_object(context.gc_context),
            scope,
            base_proto,
        })
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

    /// Obtain a reference to the method being executed.
    pub fn method(&self) -> &Avm2MethodEntry<'gc> {
        &self.method
    }

    /// Get the PC value.
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Set the PC value.
    pub fn set_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }

    /// Retrieve a local register.
    pub fn local_register(&self, id: u32) -> Option<Value<'gc>> {
        self.local_registers.read().get(id).cloned()
    }

    /// Get the current scope stack.
    pub fn scope(&self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.scope
    }

    /// Set a new scope stack.
    pub fn set_scope(&mut self, new_scope: Option<GcCell<'gc, Scope<'gc>>>) {
        self.scope = new_scope;
    }

    /// Set a local register.
    ///
    /// Returns `true` if the set was successful; `false` otherwise
    pub fn set_local_register(
        &mut self,
        id: u32,
        value: impl Into<Value<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> bool {
        if let Some(r) = self.local_registers.write(mc).get_mut(id) {
            *r = value.into();

            true
        } else {
            false
        }
    }

    /// Set the return value.
    pub fn set_return_value(&mut self, value: Value<'gc>) {
        self.return_value = Some(value);
    }

    /// Get the base prototype of the object that the currently executing
    /// method was retrieved from, if one exists.
    pub fn base_proto(&self) -> Option<Object<'gc>> {
        self.base_proto
    }
}
