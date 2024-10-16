//! Activation frames

use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::e4x::{escape_attribute_value, escape_element_value};
use crate::avm2::error::{
    make_error_1065, make_error_1127, make_error_1506, make_null_or_undefined_error, type_error,
};
use crate::avm2::method::{BytecodeMethod, Method, ResolvedParamConfig};
use crate::avm2::object::{
    ArrayObject, ByteArrayObject, ClassObject, FunctionObject, NamespaceObject, ScriptObject,
    XmlListObject,
};
use crate::avm2::object::{Object, TObject};
use crate::avm2::op::Op;
use crate::avm2::scope::{search_scope_stack, Scope, ScopeChain};
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use crate::string::{AvmAtom, AvmString, StringContext};
use crate::tag_utils::SwfMovie;
use gc_arena::Gc;
use smallvec::SmallVec;
use std::cmp::{min, Ordering};
use std::sync::Arc;
use swf::avm2::types::{
    Exception, Index, Method as AbcMethod, MethodFlags as AbcMethodFlags, Namespace as AbcNamespace,
};

use super::error::make_mismatch_error;

/// Represents a particular register set.
///
/// This type exists primarily because SmallVec isn't garbage-collectable.
pub struct RegisterSet<'gc>(SmallVec<[Value<'gc>; 8]>);

unsafe impl gc_arena::Collect for RegisterSet<'_> {
    #[inline]
    fn trace(&self, cc: &gc_arena::Collection) {
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

    pub fn get_unchecked(&self, num: u32) -> Value<'gc> {
        self.0[num as usize]
    }

    pub fn get_unchecked_mut(&mut self, num: u32) -> &mut Value<'gc> {
        self.0.get_mut(num as usize).unwrap()
    }
}

#[derive(Clone)]
enum FrameControl<'gc> {
    Continue,
    Return(Value<'gc>),
}

/// Represents a single activation of a given AVM2 function or keyframe.
pub struct Activation<'a, 'gc: 'a> {
    /// The instruction index.
    ip: i32,

    /// Amount of actions performed since the last timeout check
    actions_since_timeout_check: u16,

    /// Local registers.
    ///
    /// All activations have local registers, but it is possible for multiple
    /// activations (such as a rescope) to execute from the same register set.
    local_registers: RegisterSet<'gc>,

    /// This represents the outer scope of the method that is executing.
    ///
    /// The outer scope gives an activation access to the "outer world", including
    /// the current Domain.
    outer: ScopeChain<'gc>,

    /// The domain of the original AS3 caller.
    ///
    /// This is intended exclusively for builtin methods to access the domain of the
    /// bytecode method that called it.
    ///
    /// If this activation was not made for a builtin method, this will be the
    /// current domain instead.
    caller_domain: Option<Domain<'gc>>,

    /// The movie that called this builtin method.
    /// This is intended to be used only for builtin methods- if this activation's method
    /// is a bytecode method, the movie will instead be the movie that the bytecode method came from.
    caller_movie: Option<Arc<SwfMovie>>,

    /// The superclass of the class that yielded the currently executing method.
    ///
    /// This is used to maintain continuity when multiple methods supercall
    /// into one another. For example, if a class method supercalls a
    /// grandparent class's method, then this value will be the grandparent's
    /// class object. Then, if we supercall again, we look up supermethods from
    /// the great-grandparent class, preventing us from accidentally calling
    /// the same method again.
    ///
    /// This will not be available outside of method, setter, or getter calls.
    bound_superclass_object: Option<ClassObject<'gc>>,

    bound_class: Option<Class<'gc>>,

    /// The class of all objects returned from `newactivation`.
    ///
    /// In method calls that call for an activation object, this will be
    /// configured as the anonymous class whose traits match the method's
    /// declared traits.
    ///
    /// If this is `None`, then the method did not ask for an activation object
    /// and we will not allocate a class for one.
    activation_class: Option<ClassObject<'gc>>,

    /// The index where the stack frame starts.
    stack_depth: usize,

    /// The index where the scope frame starts.
    scope_depth: usize,

    pub context: &'a mut UpdateContext<'gc>,
}

impl<'a, 'gc> Activation<'a, 'gc> {
    /// Convenience method to retrieve the current GC context. Note that explicitly writing
    /// `self.context.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc gc_arena::Mutation<'gc> {
        self.context.gc_context
    }

    #[inline(always)]
    pub fn strings(&mut self) -> &mut StringContext<'gc> {
        &mut self.context.strings
    }

    /// Construct an activation that does not represent any particular scope.
    ///
    /// This exists primarily for non-AVM2 related manipulations of the
    /// interpreter environment that require an activation. For example,
    /// loading traits into an object, or running tests.
    ///
    /// It is a logic error to attempt to run AVM2 code in a nothing
    /// `Activation`.
    pub fn from_nothing(context: &'a mut UpdateContext<'gc>) -> Self {
        let local_registers = RegisterSet::new(0);

        Self {
            ip: 0,
            actions_since_timeout_check: 0,
            local_registers,
            outer: ScopeChain::new(context.avm2.stage_domain),
            caller_domain: None,
            caller_movie: None,
            bound_superclass_object: None,
            bound_class: None,
            activation_class: None,
            stack_depth: context.avm2.stack.len(),
            scope_depth: context.avm2.scope_stack.len(),
            context,
        }
    }

    /// Like `from_nothing`, but with a specified domain.
    ///
    /// This should be used when you actually need to run AVM2 code, but
    /// don't have a particular scope to run it in. For example, this is
    /// used to run frame scripts for AVM2 movies.
    ///
    /// The 'Domain' should come from the SwfMovie associated with whatever
    /// action you're performing. When running frame scripts, this is the
    /// `SwfMovie` associated with the `MovieClip` being processed.
    pub fn from_domain(context: &'a mut UpdateContext<'gc>, domain: Domain<'gc>) -> Self {
        let local_registers = RegisterSet::new(0);

        Self {
            ip: 0,
            actions_since_timeout_check: 0,
            local_registers,
            outer: ScopeChain::new(context.avm2.stage_domain),
            caller_domain: Some(domain),
            caller_movie: None,
            bound_superclass_object: None,
            bound_class: None,
            activation_class: None,
            stack_depth: context.avm2.stack.len(),
            scope_depth: context.avm2.scope_stack.len(),
            context,
        }
    }

    /// Construct an activation for the execution of a particular script's
    /// initializer method.
    pub fn from_script(
        context: &'a mut UpdateContext<'gc>,
        script: Script<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let (method, global_object, domain) = script.init();

        let num_locals = match method {
            Method::Native { .. } => 0,
            Method::Bytecode(bytecode) => {
                let body = bytecode
                    .body()
                    .ok_or("Cannot execute non-native method (for script) without body")?;

                body.num_locals
            }
        };
        let mut local_registers = RegisterSet::new(num_locals + 1);

        *local_registers.get_unchecked_mut(0) = global_object.into();

        let activation_class = if let Method::Bytecode(method) = method {
            let body = method
                .body()
                .ok_or("Cannot execute non-native method (for script) without body")?;

            BytecodeMethod::get_or_init_activation_class(method, context.gc_context, || {
                let translation_unit = method.translation_unit();
                let abc_method = method.method();
                let mut dummy_activation = Activation::from_domain(context, domain);
                dummy_activation.set_outer(ScopeChain::new(domain));
                let activation_class = Class::for_activation(
                    &mut dummy_activation,
                    translation_unit,
                    abc_method,
                    body,
                )?;

                ClassObject::from_class(&mut dummy_activation, activation_class, None)
            })?
        } else {
            None
        };

        let mut created_activation = Self {
            ip: 0,
            actions_since_timeout_check: 0,
            local_registers,
            outer: ScopeChain::new(domain),
            caller_domain: Some(domain),
            caller_movie: script.translation_unit().map(|t| t.movie()),
            bound_superclass_object: Some(context.avm2.classes().object), // The script global class extends Object
            bound_class: Some(script.global_class()),
            activation_class,
            stack_depth: context.avm2.stack.len(),
            scope_depth: context.avm2.scope_stack.len(),
            context,
        };

        // Run verifier for bytecode methods
        if let Method::Bytecode(method) = method {
            if method.verified_info.borrow().is_none() {
                BytecodeMethod::verify(method, &mut created_activation)?;
            }
        }

        Ok(created_activation)
    }

    /// Finds an object on either the current or outer scope of this activation by definition.
    pub fn find_definition(
        &mut self,
        name: &Multiname<'gc>,
    ) -> Result<Option<Object<'gc>>, Error<'gc>> {
        let outer_scope = self.outer;

        if let Some(obj) = search_scope_stack(self.scope_frame(), name, outer_scope.is_empty())? {
            Ok(Some(obj))
        } else if let Some(obj) = outer_scope.find(name, self)? {
            Ok(Some(obj))
        } else if let Some(global) = self.global_scope() {
            if global.base().has_own_dynamic_property(name) {
                Ok(Some(global))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Resolve a single parameter value.
    ///
    /// Given an individual parameter value and the associated parameter's
    /// configuration, return what value should be stored in the called
    /// function's local registers (or an error, if the parameter violates the
    /// signature of the current called method).
    fn resolve_parameter(
        &mut self,
        method: Method<'gc>,
        value: Option<&Value<'gc>>,
        param_config: &ResolvedParamConfig<'gc>,
        user_arguments: &[Value<'gc>],
        bound_class: Option<Class<'gc>>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let arg = if let Some(value) = value {
            value
        } else if let Some(default_value) = &param_config.default_value {
            default_value
        } else if param_config.param_type.is_none() {
            // TODO: FP's system of allowing missing arguments
            // is a more complicated than this.
            return Ok(Value::Undefined);
        } else {
            return Err(Error::AvmError(make_mismatch_error(
                self,
                method,
                user_arguments,
                bound_class,
            )?));
        };

        if let Some(param_class) = param_config.param_type {
            arg.coerce_to_type(self, param_class)
        } else {
            Ok(*arg)
        }
    }

    /// Statically resolve all of the parameters for a given method.
    ///
    /// This function makes no attempt to enforce a given method's parameter
    /// count limits or to package variadic arguments.
    ///
    /// The returned list of parameters will be coerced to the stated types in
    /// the signature, with missing parameters filled in with defaults.
    pub fn resolve_parameters(
        &mut self,
        method: Method<'gc>,
        user_arguments: &[Value<'gc>],
        signature: &[ResolvedParamConfig<'gc>],
        bound_class: Option<Class<'gc>>,
    ) -> Result<Vec<Value<'gc>>, Error<'gc>> {
        let mut arguments_list = Vec::new();
        for (arg, param_config) in user_arguments.iter().zip(signature.iter()) {
            arguments_list.push(self.resolve_parameter(
                method,
                Some(arg),
                param_config,
                user_arguments,
                bound_class,
            )?);
        }

        match user_arguments.len().cmp(&signature.len()) {
            Ordering::Greater => {
                //Variadic parameters exist, just push them into the list
                arguments_list.extend_from_slice(&user_arguments[signature.len()..])
            }
            Ordering::Less => {
                //Apply remaining default parameters
                for param_config in signature[user_arguments.len()..].iter() {
                    arguments_list.push(self.resolve_parameter(
                        method,
                        None,
                        param_config,
                        user_arguments,
                        bound_class,
                    )?);
                }
            }
            _ => {}
        }

        Ok(arguments_list)
    }

    /// Construct an activation for the execution of a particular bytecode
    /// method.
    /// NOTE: this is intended to be used immediately after from_nothing(),
    /// as a more efficient replacement for direct `Activation::from_method()`
    #[allow(clippy::too_many_arguments)]
    pub fn init_from_method(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        outer: ScopeChain<'gc>,
        this: Object<'gc>,
        user_arguments: &[Value<'gc>],
        bound_superclass_object: Option<ClassObject<'gc>>,
        bound_class: Option<Class<'gc>>,
        callee: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let body: Result<_, Error<'gc>> = method
            .body()
            .ok_or_else(|| "Cannot execute non-native method without body".into());
        let body = body?;
        let num_locals = body.num_locals;
        let has_rest_or_args = method.is_variadic();

        let mut local_registers = RegisterSet::new(num_locals + 1);
        *local_registers.get_unchecked_mut(0) = this.into();

        let activation_class =
            BytecodeMethod::get_or_init_activation_class(method, self.context.gc_context, || {
                let translation_unit = method.translation_unit();
                let abc_method = method.method();
                let mut dummy_activation = Activation::from_domain(self.context, outer.domain());
                dummy_activation.set_outer(outer);
                let activation_class = Class::for_activation(
                    &mut dummy_activation,
                    translation_unit,
                    abc_method,
                    body,
                )?;
                ClassObject::from_class(&mut dummy_activation, activation_class, None)
            })?;

        self.ip = 0;
        self.actions_since_timeout_check = 0;
        self.local_registers = local_registers;
        self.outer = outer;
        self.caller_domain = Some(outer.domain());
        self.caller_movie = Some(method.owner_movie());
        self.bound_superclass_object = bound_superclass_object;
        self.bound_class = bound_class;
        self.activation_class = activation_class;
        self.stack_depth = self.context.avm2.stack.len();
        self.scope_depth = self.context.avm2.scope_stack.len();

        // Everything is now setup for the verifier to run
        if method.verified_info.borrow().is_none() {
            BytecodeMethod::verify(method, self)?;
        }

        let verified_info = method.verified_info.borrow();
        let signature = &verified_info.as_ref().unwrap().param_config;

        if user_arguments.len() > signature.len() && !has_rest_or_args {
            return Err(Error::AvmError(make_mismatch_error(
                self,
                Method::Bytecode(method),
                user_arguments,
                bound_class,
            )?));
        }

        // Statically verify all non-variadic, provided parameters.
        let arguments_list = self.resolve_parameters(
            Method::Bytecode(method),
            user_arguments,
            signature,
            bound_class,
        )?;

        {
            for (i, arg) in arguments_list[0..min(signature.len(), arguments_list.len())]
                .iter()
                .enumerate()
            {
                *self.local_registers.get_unchecked_mut(1 + i as u32) = *arg;
            }
        }

        if has_rest_or_args {
            let args_array = if method
                .method()
                .flags
                .contains(AbcMethodFlags::NEED_ARGUMENTS)
            {
                // note: resolve_parameters ensures that arguments_list length is >= user_arguments
                ArrayStorage::from_args(&arguments_list[..user_arguments.len()])
            } else if method.method().flags.contains(AbcMethodFlags::NEED_REST) {
                if let Some(rest_args) = arguments_list.get(signature.len()..) {
                    ArrayStorage::from_args(rest_args)
                } else {
                    ArrayStorage::new(0)
                }
            } else {
                unreachable!();
            };

            let args_object = ArrayObject::from_storage(self, args_array)?;

            if method
                .method()
                .flags
                .contains(AbcMethodFlags::NEED_ARGUMENTS)
            {
                args_object.set_string_property_local("callee", callee.into(), self)?;
                args_object.set_local_property_is_enumerable(
                    self.context.gc_context,
                    "callee".into(),
                    false,
                );
            }

            *self
                .local_registers
                .get_unchecked_mut(1 + signature.len() as u32) = args_object.into();
        }

        Ok(())
    }

    /// Construct an activation for the execution of a builtin method.
    ///
    /// It is a logic error to attempt to execute builtins within the same
    /// activation as the method or script that called them. You must use this
    /// function to construct a new activation for the builtin so that it can
    /// properly supercall.
    pub fn from_builtin(
        context: &'a mut UpdateContext<'gc>,
        bound_superclass_object: Option<ClassObject<'gc>>,
        bound_class: Option<Class<'gc>>,
        outer: ScopeChain<'gc>,
        caller_domain: Option<Domain<'gc>>,
        caller_movie: Option<Arc<SwfMovie>>,
    ) -> Self {
        let local_registers = RegisterSet::new(0);

        Self {
            ip: 0,
            actions_since_timeout_check: 0,
            local_registers,
            outer,
            caller_domain,
            caller_movie,
            bound_superclass_object,
            bound_class,
            activation_class: None,
            stack_depth: context.avm2.stack.len(),
            scope_depth: context.avm2.scope_stack.len(),
            context,
        }
    }

    /// Call the superclass's instance initializer.
    pub fn super_init(
        &mut self,
        receiver: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let bound_superclass_object = self
            .bound_superclass_object
            .expect("Superclass object is required to run super_init");

        bound_superclass_object.call_super_init(receiver.into(), args, self)
    }

    /// Retrieve a local register.
    pub fn local_register(&self, id: u32) -> Value<'gc> {
        // Verification guarantees that this is valid
        self.local_registers.get_unchecked(id)
    }

    /// Set a local register.
    pub fn set_local_register(&mut self, id: u32, value: impl Into<Value<'gc>>) {
        // Verification guarantees that this is valid
        *self.local_registers.get_unchecked_mut(id) = value.into();
    }

    /// Retrieve the outer scope of this activation
    pub fn outer(&self) -> ScopeChain<'gc> {
        self.outer
    }

    /// Sets the outer scope of this activation
    pub fn set_outer(&mut self, new_outer: ScopeChain<'gc>) {
        self.outer = new_outer;
    }

    /// Creates a new ScopeChain by chaining the current state of this
    /// activation's scope stack with the outer scope.
    pub fn create_scopechain(&self) -> ScopeChain<'gc> {
        self.outer
            .chain(self.context.gc_context, self.scope_frame())
    }

    /// Returns the domain of the original AS3 caller. This will be `None`
    /// if this activation was constructed with `from_nothing`
    pub fn caller_domain(&self) -> Option<Domain<'gc>> {
        self.caller_domain
    }

    /// Returns the movie of the original AS3 caller. This will be `None`
    /// if this activation was constructed with `from_nothing`
    pub fn caller_movie(&self) -> Option<Arc<SwfMovie>> {
        self.caller_movie.clone()
    }

    /// Like `caller_movie()`, but returns the root movie if `caller_movie`
    /// is `None`. This matches what FP does in most cases.
    pub fn caller_movie_or_root(&self) -> Arc<SwfMovie> {
        self.caller_movie().unwrap_or(self.context.swf.clone())
    }

    /// Returns the global scope of this activation.
    ///
    /// The global scope refers to scope at the bottom of the
    /// outer scope. If the outer scope is empty, we use the bottom
    /// of the current scope stack instead.
    ///
    /// A return value of `None` implies that both the outer scope, and
    /// the current scope stack were both empty.
    pub fn global_scope(&self) -> Option<Object<'gc>> {
        let outer_scope = self.outer;
        outer_scope
            .get(0)
            .or_else(|| self.scope_frame().first().copied())
            .map(|scope| scope.values())
    }

    pub fn avm2(&mut self) -> &mut Avm2<'gc> {
        self.context.avm2
    }

    pub fn scope_frame(&self) -> &[Scope<'gc>] {
        &self.context.avm2.scope_stack[self.scope_depth..]
    }

    /// Pushes a value onto the operand stack.
    #[inline]
    pub fn push_stack(&mut self, value: impl Into<Value<'gc>>) {
        self.avm2().push(value.into());
    }

    /// Pushes a value onto the operand stack, without running some checks.
    #[inline]
    pub fn push_raw(&mut self, value: impl Into<Value<'gc>>) {
        self.avm2().push_raw(value.into());
    }

    /// Pops a value off the operand stack.
    #[inline]
    #[must_use]
    pub fn pop_stack(&mut self) -> Value<'gc> {
        self.avm2().pop()
    }

    /// Pops multiple values off the operand stack.
    #[inline]
    #[must_use]
    pub fn pop_stack_args(&mut self, arg_count: u32) -> Vec<Value<'gc>> {
        self.avm2().pop_args(arg_count)
    }

    /// Pushes a scope onto the scope stack.
    #[inline]
    pub fn push_scope(&mut self, scope: Scope<'gc>) {
        self.avm2().push_scope(scope);
    }

    /// Pops a scope off of the scope stack.
    #[inline]
    pub fn pop_scope(&mut self) {
        self.avm2().pop_scope();
    }

    /// Clears the operand stack used by this activation.
    #[inline]
    pub fn clear_stack(&mut self) {
        let stack_depth = self.stack_depth;
        self.avm2().truncate_stack(stack_depth);
    }

    /// Clears the scope stack used by this activation.
    #[inline]
    pub fn clear_scope(&mut self) {
        let scope_depth = self.scope_depth;
        self.avm2().scope_stack.truncate(scope_depth);
    }

    /// Get the superclass of the class that defined the currently-executing
    /// method, if it exists.
    ///
    /// If the currently-executing method is not part of a class, or the class
    /// does not have a superclass, then this panics. The `name` parameter
    /// allows you to provide the name of a property you were attempting to
    /// access on the object.
    pub fn bound_superclass_object(&self, name: &Multiname<'gc>) -> ClassObject<'gc> {
        self.bound_superclass_object.unwrap_or_else(|| {
            panic!(
                "Cannot call supermethod {} without a superclass",
                name.to_qualified_name(self.context.gc_context),
            )
        })
    }

    /// Get the class that defined the currently-executing method, if it exists.
    pub fn bound_class(&self) -> Option<Class<'gc>> {
        self.bound_class
    }

    /// Retrieve a namespace from the current constant pool.
    fn pool_namespace(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcNamespace>,
    ) -> Result<Namespace<'gc>, Error<'gc>> {
        method.translation_unit().pool_namespace(self, index)
    }

    /// Retrieve a method entry from the current ABC file's method table.
    fn table_method(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMethod>,
        is_function: bool,
    ) -> Result<Method<'gc>, Error<'gc>> {
        method
            .translation_unit()
            .load_method(index, is_function, self)
    }

    pub fn run_actions(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // The method must be verified at this point

        let verified_info = method.verified_info.borrow();
        let verified_code = verified_info.as_ref().unwrap().parsed_code.as_slice();

        self.ip = 0;

        let val = loop {
            let result = self.do_next_opcode(method, verified_code);
            match result {
                Ok(FrameControl::Return(value)) => break Ok(value),
                Ok(FrameControl::Continue) => {}
                Err(e) => break Err(e),
            }
        };

        self.clear_stack();
        self.clear_scope();
        val
    }

    /// If a local exception handler exists for the error, use it to handle
    /// the error. Otherwise pass the error down the stack.
    fn handle_err(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        error: Error<'gc>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let error = match error {
            Error::AvmError(err) => err,
            Error::RustError(_) => return Err(error),
        };

        let verified_info = method.verified_info.borrow();
        let exception_list = &verified_info.as_ref().unwrap().exceptions;

        // Use `coerce_to_object` so that we handle primitives correctly.
        let err_object = error.coerce_to_object(self);
        let last_ip = self.ip - 1;
        for e in exception_list {
            if last_ip >= e.from_offset as i32 && last_ip < e.to_offset as i32 {
                let mut matches = false;
                // A typeless catch block (e.g. `catch(er) { ... }`) will
                // always match.
                if e.target_class.is_none() {
                    matches = true;
                } else if let Ok(err_object) = err_object {
                    let target_class = e.target_class.expect("Just confirmed to be non-None");

                    matches = err_object.is_of_type(target_class);
                }

                if matches {
                    #[cfg(feature = "avm_debug")]
                    tracing::info!(target: "avm_caught", "Caught exception: {:?}", Error::AvmError(error));

                    self.clear_stack();
                    self.push_stack(error);

                    self.clear_scope();
                    self.ip = e.target_offset as i32;
                    return Ok(FrameControl::Continue);
                }
            }
        }

        Err(Error::AvmError(error))
    }

    /// Run a single action from a given action reader.
    #[inline(always)]
    fn do_next_opcode(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        opcodes: &[Op<'gc>],
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.actions_since_timeout_check += 1;
        if self.actions_since_timeout_check >= 64000 {
            self.actions_since_timeout_check = 0;
            if self.context.update_start.elapsed() >= self.context.max_execution_duration {
                return Err(
                    "A script in this movie has taken too long to execute and has been terminated."
                        .into(),
                );
            }
        }

        let op = &opcodes[self.ip as usize];
        self.ip += 1;
        avm_debug!(self.avm2(), "Opcode: {op:?}");

        {
            let result = match op {
                Op::PushByte { value } => self.op_push_byte(*value),
                Op::PushDouble { value } => self.op_push_double(*value),
                Op::PushFalse => self.op_push_false(),
                Op::PushInt { value } => self.op_push_int(*value),
                Op::PushNamespace { value } => self.op_push_namespace(method, *value),
                Op::PushNaN => self.op_push_nan(),
                Op::PushNull => self.op_push_null(),
                Op::PushShort { value } => self.op_push_short(*value),
                Op::PushString { string } => self.op_push_string(*string),
                Op::PushTrue => self.op_push_true(),
                Op::PushUint { value } => self.op_push_uint(*value),
                Op::PushUndefined => self.op_push_undefined(),
                Op::Pop => self.op_pop(),
                Op::Dup => self.op_dup(),
                Op::GetLocal { index } => self.op_get_local(*index),
                Op::SetLocal { index } => self.op_set_local(*index),
                Op::Kill { index } => self.op_kill(*index),
                Op::Call { num_args } => self.op_call(*num_args),
                Op::CallMethod {
                    index,
                    num_args,
                    push_return_value,
                } => self.op_call_method(*index, *num_args, *push_return_value),
                Op::CallProperty {
                    multiname,
                    num_args,
                } => self.op_call_property(*multiname, *num_args),
                Op::CallPropLex {
                    multiname,
                    num_args,
                } => self.op_call_prop_lex(*multiname, *num_args),
                Op::CallPropVoid {
                    multiname,
                    num_args,
                } => self.op_call_prop_void(*multiname, *num_args),
                Op::CallStatic { index, num_args } => {
                    self.op_call_static(method, *index, *num_args)
                }
                Op::CallSuper {
                    multiname,
                    num_args,
                } => self.op_call_super(*multiname, *num_args),
                Op::CallSuperVoid {
                    multiname,
                    num_args,
                } => self.op_call_super_void(*multiname, *num_args),
                Op::ReturnValue => self.op_return_value(method),
                Op::ReturnValueNoCoerce => self.op_return_value_no_coerce(),
                Op::ReturnVoid => self.op_return_void(),
                Op::GetProperty { multiname } => self.op_get_property(*multiname),
                Op::SetProperty { multiname } => self.op_set_property(*multiname),
                Op::InitProperty { multiname } => self.op_init_property(*multiname),
                Op::DeleteProperty { multiname } => self.op_delete_property(*multiname),
                Op::GetSuper { multiname } => self.op_get_super(*multiname),
                Op::SetSuper { multiname } => self.op_set_super(*multiname),
                Op::In => self.op_in(),
                Op::PushScope => self.op_push_scope(),
                Op::NewCatch { index } => self.op_newcatch(method, *index),
                Op::PushWith => self.op_push_with(),
                Op::PopScope => self.op_pop_scope(),
                Op::GetOuterScope { index } => self.op_get_outer_scope(*index),
                Op::GetScopeObject { index } => self.op_get_scope_object(*index),
                Op::GetGlobalScope => self.op_get_global_scope(),
                Op::FindDef { multiname } => self.op_find_def(*multiname),
                Op::FindProperty { multiname } => self.op_find_property(*multiname),
                Op::FindPropStrict { multiname } => self.op_find_prop_strict(*multiname),
                Op::GetScriptGlobals { script } => self.op_get_script_globals(*script),
                Op::GetDescendants { multiname } => self.op_get_descendants(*multiname),
                Op::GetSlot { index } => self.op_get_slot(*index),
                Op::SetSlot { index } => self.op_set_slot(*index),
                Op::SetSlotNoCoerce { index } => self.op_set_slot_no_coerce(*index),
                Op::GetGlobalSlot { index } => self.op_get_global_slot(*index),
                Op::SetGlobalSlot { index } => self.op_set_global_slot(*index),
                Op::Construct { num_args } => self.op_construct(*num_args),
                Op::ConstructProp {
                    multiname,
                    num_args,
                } => self.op_construct_prop(*multiname, *num_args),
                Op::ConstructSuper { num_args } => self.op_construct_super(*num_args),
                Op::NewActivation => self.op_new_activation(),
                Op::NewObject { num_args } => self.op_new_object(*num_args),
                Op::NewFunction { index } => self.op_new_function(method, *index),
                Op::NewClass { class } => self.op_new_class(*class),
                Op::ApplyType { num_types } => self.op_apply_type(*num_types),
                Op::NewArray { num_args } => self.op_new_array(*num_args),
                Op::CoerceA => Ok(FrameControl::Continue),
                Op::CoerceB => self.op_coerce_b(),
                Op::CoerceD => self.op_coerce_d(),
                Op::CoerceDSwapPop => self.op_coerce_d_swap_pop(),
                Op::CoerceI => self.op_coerce_i(),
                Op::CoerceISwapPop => self.op_coerce_i_swap_pop(),
                Op::CoerceO => self.op_coerce_o(),
                Op::CoerceS => self.op_coerce_s(),
                Op::CoerceU => self.op_coerce_u(),
                Op::CoerceUSwapPop => self.op_coerce_u_swap_pop(),
                Op::ConvertO => self.op_convert_o(),
                Op::ConvertS => self.op_convert_s(),
                Op::Add => self.op_add(),
                Op::AddI => self.op_add_i(),
                Op::BitAnd => self.op_bitand(),
                Op::BitNot => self.op_bitnot(),
                Op::BitOr => self.op_bitor(),
                Op::BitXor => self.op_bitxor(),
                Op::DecLocal { index } => self.op_declocal(*index),
                Op::DecLocalI { index } => self.op_declocal_i(*index),
                Op::Decrement => self.op_decrement(),
                Op::DecrementI => self.op_decrement_i(),
                Op::Divide => self.op_divide(),
                Op::IncLocal { index } => self.op_inclocal(*index),
                Op::IncLocalI { index } => self.op_inclocal_i(*index),
                Op::Increment => self.op_increment(),
                Op::IncrementI => self.op_increment_i(),
                Op::LShift => self.op_lshift(),
                Op::Modulo => self.op_modulo(),
                Op::Multiply => self.op_multiply(),
                Op::MultiplyI => self.op_multiply_i(),
                Op::Negate => self.op_negate(),
                Op::NegateI => self.op_negate_i(),
                Op::RShift => self.op_rshift(),
                Op::Subtract => self.op_subtract(),
                Op::SubtractI => self.op_subtract_i(),
                Op::Swap => self.op_swap(),
                Op::URShift => self.op_urshift(),
                Op::Jump { offset } => self.op_jump(*offset),
                Op::IfTrue { offset } => self.op_if_true(*offset),
                Op::IfFalse { offset } => self.op_if_false(*offset),
                Op::IfStrictEq { offset } => self.op_if_strict_eq(*offset),
                Op::IfStrictNe { offset } => self.op_if_strict_ne(*offset),
                Op::IfEq { offset } => self.op_if_eq(*offset),
                Op::IfNe { offset } => self.op_if_ne(*offset),
                Op::IfGe { offset } => self.op_if_ge(*offset),
                Op::IfGt { offset } => self.op_if_gt(*offset),
                Op::IfLe { offset } => self.op_if_le(*offset),
                Op::IfLt { offset } => self.op_if_lt(*offset),
                Op::IfNge { offset } => self.op_if_nge(*offset),
                Op::IfNgt { offset } => self.op_if_ngt(*offset),
                Op::IfNle { offset } => self.op_if_nle(*offset),
                Op::IfNlt { offset } => self.op_if_nlt(*offset),
                Op::StrictEquals => self.op_strict_equals(),
                Op::Equals => self.op_equals(),
                Op::GreaterEquals => self.op_greater_equals(),
                Op::GreaterThan => self.op_greater_than(),
                Op::LessEquals => self.op_less_equals(),
                Op::LessThan => self.op_less_than(),
                Op::Nop => Ok(FrameControl::Continue),
                Op::Not => self.op_not(),
                Op::HasNext => self.op_has_next(),
                Op::HasNext2 {
                    object_register,
                    index_register,
                } => self.op_has_next_2(*object_register, *index_register),
                Op::NextName => self.op_next_name(),
                Op::NextValue => self.op_next_value(),
                Op::IsType { class } => self.op_is_type(*class),
                Op::IsTypeLate => self.op_is_type_late(),
                Op::AsType { class } => self.op_as_type(*class),
                Op::AsTypeLate => self.op_as_type_late(),
                Op::InstanceOf => self.op_instance_of(),
                Op::Debug {
                    is_local_register,
                    register_name,
                    register,
                } => self.op_debug(*is_local_register, *register_name, *register),
                Op::DebugFile { file_name } => self.op_debug_file(*file_name),
                Op::DebugLine { line_num } => self.op_debug_line(*line_num),
                Op::Bkpt => self.op_bkpt(),
                Op::BkptLine { line_num } => self.op_bkpt_line(*line_num),
                Op::Timestamp => self.op_timestamp(),
                Op::TypeOf => self.op_type_of(),
                Op::EscXAttr => self.op_esc_xattr(),
                Op::EscXElem => self.op_esc_elem(),
                Op::LookupSwitch(ref lookup_switch) => {
                    self.op_lookup_switch(lookup_switch.default_offset, &lookup_switch.case_offsets)
                }
                Op::Coerce { class } => self.op_coerce(*class),
                Op::CoerceSwapPop { class } => self.op_coerce_swap_pop(*class),
                Op::CheckFilter => self.op_check_filter(),
                Op::Si8 => self.op_si8(),
                Op::Si16 => self.op_si16(),
                Op::Si32 => self.op_si32(),
                Op::Sf32 => self.op_sf32(),
                Op::Sf64 => self.op_sf64(),
                Op::Li8 => self.op_li8(),
                Op::Li16 => self.op_li16(),
                Op::Li32 => self.op_li32(),
                Op::Lf32 => self.op_lf32(),
                Op::Lf64 => self.op_lf64(),
                Op::Sxi1 => self.op_sxi1(),
                Op::Sxi8 => self.op_sxi8(),
                Op::Sxi16 => self.op_sxi16(),
                Op::Throw => self.op_throw(),
                _ => {
                    tracing::info!("Encountered unimplemented AVM2 opcode {:?}", op);

                    return Err("Unknown op".into());
                }
            };

            if let Err(error) = result {
                return self.handle_err(method, error);
            }
            result
        }
    }

    fn op_push_byte(&mut self, value: i8) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(value as i32);
        Ok(FrameControl::Continue)
    }

    fn op_push_double(&mut self, value: f64) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(value);
        Ok(FrameControl::Continue)
    }

    fn op_push_false(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(false);
        Ok(FrameControl::Continue)
    }

    fn op_push_int(&mut self, value: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(value);
        Ok(FrameControl::Continue)
    }

    fn op_push_namespace(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        value: Index<AbcNamespace>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let ns = self.pool_namespace(method, value)?;
        let ns_object = NamespaceObject::from_namespace(self, ns)?;

        self.push_stack(ns_object);
        Ok(FrameControl::Continue)
    }

    fn op_push_nan(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(f64::NAN);
        Ok(FrameControl::Continue)
    }

    fn op_push_null(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(Value::Null);
        Ok(FrameControl::Continue)
    }

    fn op_push_short(&mut self, value: i16) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(value);
        Ok(FrameControl::Continue)
    }

    fn op_push_string(&mut self, string: AvmAtom<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(string);
        Ok(FrameControl::Continue)
    }

    fn op_push_true(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(true);
        Ok(FrameControl::Continue)
    }

    fn op_push_uint(&mut self, value: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(value);
        Ok(FrameControl::Continue)
    }

    fn op_push_undefined(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(Value::Undefined);
        Ok(FrameControl::Continue)
    }

    fn op_pop(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let _ = self.pop_stack();

        Ok(FrameControl::Continue)
    }

    fn op_dup(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.avm2().peek(0);
        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_get_local(&mut self, register_index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(self.local_register(register_index));
        Ok(FrameControl::Continue)
    }

    fn op_set_local(&mut self, register_index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        self.set_local_register(register_index, value);

        Ok(FrameControl::Continue)
    }

    fn op_kill(&mut self, register_index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.set_local_register(register_index, Value::Undefined);

        Ok(FrameControl::Continue)
    }

    fn op_call(&mut self, arg_count: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let receiver = self.pop_stack();
        let function = self
            .pop_stack()
            .as_callable(self, None, Some(receiver), false)?;
        let value = function.call(receiver, &args, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_method(
        &mut self,
        index: u32,
        arg_count: u32,
        push_return_value: bool,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // The entire implementation of VTable assumes that
        // call_method is never encountered in user code. (see the long comment there)
        // This was also the conclusion from analysing avmplus behavior - they
        // unconditionally VerifyError upon noticing it.

        // However, the optimizer can still generate it.

        let args = self.pop_stack_args(arg_count);
        let receiver = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        let value = receiver.call_method(index, &args, self)?;

        if push_return_value {
            self.push_stack(value);
        }

        Ok(FrameControl::Continue)
    }

    fn op_call_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        let value = receiver.call_property(&multiname, &args, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_prop_lex(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;
        let function = receiver.get_property(&multiname, self)?.as_callable(
            self,
            Some(&multiname),
            Some(receiver.into()),
            false,
        )?;
        let value = function.call(Value::Null, &args, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_prop_void(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        receiver.call_property(&multiname, &args, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_call_static(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMethod>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let receiver = self.pop_stack();
        let method = self.table_method(method, index, false)?;
        // TODO: What scope should the function be executed with?
        let scope = self.create_scopechain();
        let function = FunctionObject::from_method(self, method, scope, None, None, None);
        let value = function.call(receiver, &args, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_super(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        let bound_superclass_object = self.bound_superclass_object(&multiname);

        let value = bound_superclass_object.call_super(&multiname, receiver, &args, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_super_void(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        let bound_superclass_object = self.bound_superclass_object(&multiname);

        bound_superclass_object.call_super(&multiname, receiver, &args, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_return_value(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let return_value = self.pop_stack();
        let return_type = method.resolved_return_type();

        let coerced = if let Some(return_type) = return_type {
            return_value.coerce_to_type(self, return_type)?
        } else {
            return_value
        };

        Ok(FrameControl::Return(coerced))
    }

    fn op_return_value_no_coerce(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let return_value = self.pop_stack();

        Ok(FrameControl::Return(return_value))
    }

    fn op_return_void(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        Ok(FrameControl::Return(Value::Undefined))
    }

    fn op_get_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // default path for static names
        if !multiname.has_lazy_component() {
            let object = self.pop_stack();
            let object = object.coerce_to_object_or_typeerror(self, Some(&multiname))?;
            let value = object.get_property(&multiname, self)?;
            self.push_stack(value);
            return Ok(FrameControl::Continue);
        }

        // (fast) side path for dictionary/array-likes
        // NOTE: FP behaves differently here when the public namespace isn't
        // included in the multiname's namespace set
        if multiname.has_lazy_name() && !multiname.has_lazy_ns() {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.context.avm2.peek(0);
            let object_value = self.context.avm2.peek(1);

            if let Value::Object(object) = object_value {
                match name_value {
                    Value::Integer(name_int) if name_int >= 0 => {
                        if let Some(value) = object.get_index_property(name_int as usize) {
                            let _ = self.pop_stack();
                            let _ = self.pop_stack();
                            self.push_stack(value);

                            return Ok(FrameControl::Continue);
                        }
                    }
                    Value::Object(name_object) => {
                        if let Some(dictionary) = object.as_dictionary_object() {
                            let _ = self.pop_stack();
                            let _ = self.pop_stack();
                            let value = dictionary.get_property_by_object(name_object);
                            self.push_stack(value);

                            return Ok(FrameControl::Continue);
                        }
                    }
                    _ => {}
                }
            }
        }

        // main path for dynamic names
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack();
        let object = object.coerce_to_object_or_typeerror(self, Some(&multiname))?;
        let value = object.get_property(&multiname, self)?;
        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        // default path for static names
        if !multiname.has_lazy_component() {
            let object = self.pop_stack();
            let object = object.coerce_to_object_or_typeerror(self, Some(&multiname))?;
            object.set_property(&multiname, value, self)?;
            return Ok(FrameControl::Continue);
        }

        // side path for dictionary/arrays (TODO)
        // NOTE: FP behaves differently here when the public namespace isn't
        // included in the multiname's namespace set
        if multiname.has_lazy_name() && !multiname.has_lazy_ns() {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.context.avm2.peek(0);
            let object_value = self.context.avm2.peek(1);

            if let Value::Object(object) = object_value {
                match name_value {
                    Value::Integer(name_int) if name_int >= 0 => {
                        if let Some(mut array) =
                            object.as_array_storage_mut(self.context.gc_context)
                        {
                            let _ = self.pop_stack();
                            let _ = self.pop_stack();
                            array.set(name_int as usize, value);

                            return Ok(FrameControl::Continue);
                        }
                    }
                    Value::Object(name_object) => {
                        if let Some(dictionary) = object.as_dictionary_object() {
                            let _ = self.pop_stack();
                            let _ = self.pop_stack();
                            dictionary.set_property_by_object(
                                name_object,
                                value,
                                self.context.gc_context,
                            );

                            return Ok(FrameControl::Continue);
                        }
                    }
                    _ => {}
                }
            }
        }

        // main path for dynamic names
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack();
        let object = object.coerce_to_object_or_typeerror(self, Some(&multiname))?;
        object.set_property(&multiname, value, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_init_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        let multiname = multiname.fill_with_runtime_params(self)?;

        let object = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        object.init_property(&multiname, value, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_delete_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // default path for static names
        if !multiname.has_lazy_component() {
            let object = self.pop_stack();
            let object = object.coerce_to_object_or_typeerror(self, Some(&multiname))?;
            let did_delete = object.delete_property(self, &multiname)?;
            self.push_raw(did_delete);
            return Ok(FrameControl::Continue);
        }

        // side path for dictionary/arrays (TODO)
        if multiname.has_lazy_name() && !multiname.has_lazy_ns() {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.context.avm2.peek(0);
            let object = self.context.avm2.peek(1);
            if !name_value.is_primitive() {
                let object = object.coerce_to_object_or_typeerror(self, None)?;
                if let Some(dictionary) = object.as_dictionary_object() {
                    let _ = self.pop_stack();
                    let _ = self.pop_stack();
                    dictionary.delete_property_by_object(
                        name_value.as_object().unwrap(),
                        self.context.gc_context,
                    );

                    self.push_raw(true);
                    return Ok(FrameControl::Continue);
                }
            }
        }

        // main path for dynamic names
        if multiname.has_lazy_name() {
            let name_value = self.context.avm2.peek(0);
            if matches!(name_value, Value::Object(Object::XmlListObject(_))) {
                // ECMA-357 11.3.1 The delete Operator
                // If the type of the operand is XMLList, then a TypeError exception is thrown.
                return Err(Error::AvmError(type_error(
                    self,
                    "Error #1119: Delete operator is not supported with operand of type XMLList.",
                    1119,
                )?));
            }
        }
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack();
        let object = object.coerce_to_object_or_typeerror(self, Some(&multiname))?;
        let did_delete = object.delete_property(self, &multiname)?;

        self.push_raw(did_delete);

        Ok(FrameControl::Continue)
    }

    fn op_get_super(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        let bound_superclass_object = self.bound_superclass_object(&multiname);

        let value = bound_superclass_object.get_super(&multiname, object, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_super(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        let bound_superclass_object = self.bound_superclass_object(&multiname);

        bound_superclass_object.set_super(&multiname, value, object, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_in(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let obj = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;
        let name_value = self.pop_stack();

        if let Some(dictionary) = obj.as_dictionary_object() {
            if !name_value.is_primitive() {
                let obj_key = name_value.as_object().unwrap();
                self.push_raw(dictionary.has_property_by_object(obj_key));

                return Ok(FrameControl::Continue);
            }
        }

        let name = name_value.coerce_to_string(self)?;
        let multiname = Multiname::new(self.avm2().find_public_namespace(), name);
        let has_prop = obj.has_property_via_in(self, &multiname)?;

        self.push_raw(has_prop);

        Ok(FrameControl::Continue)
    }

    fn op_newcatch(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<Exception>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let verified_info = method.verified_info.borrow();
        let exception_list = &verified_info.as_ref().unwrap().exceptions;

        let ex = &exception_list[index.0 as usize];
        let vname = ex.variable_name;

        let so = if let Some(vname) = vname {
            ScriptObject::catch_scope(self, &vname)
        } else {
            // for `finally` scopes, FP just creates a normal object.
            self.avm2().classes().object.construct(self, &[])?
        };

        self.push_stack(so);

        Ok(FrameControl::Continue)
    }

    fn op_push_scope(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;
        self.push_scope(Scope::new(object));

        Ok(FrameControl::Continue)
    }

    fn op_push_with(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;
        self.push_scope(Scope::new_with(object));

        Ok(FrameControl::Continue)
    }

    fn op_pop_scope(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.pop_scope();

        Ok(FrameControl::Continue)
    }

    fn op_get_outer_scope(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Verifier ensures that this points to a valid outer scope

        let scope = self.outer.get_unchecked(index as usize);

        self.push_stack(scope.values());

        Ok(FrameControl::Continue)
    }

    fn op_get_scope_object(&mut self, index: u8) -> Result<FrameControl<'gc>, Error<'gc>> {
        let scope = self.scope_frame().get(index as usize).copied();

        if let Some(scope) = scope {
            self.push_stack(scope.values());
        } else {
            self.push_stack(Value::Undefined);
        };

        Ok(FrameControl::Continue)
    }

    fn op_get_global_scope(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.push_stack(
            self.global_scope()
                .map(|gs| gs.into())
                .unwrap_or(Value::Null),
        );

        Ok(FrameControl::Continue)
    }

    fn op_find_def(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        // Verifier ensures that multiname is non-lazy

        avm_debug!(self.avm2(), "Resolving {:?}", *multiname);
        let (_, script) = self.domain().find_defining_script(self, &multiname)?;
        let obj = script.globals(self.context)?;
        self.push_stack(obj);
        Ok(FrameControl::Continue)
    }

    fn op_find_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        avm_debug!(self.context.avm2, "Resolving {:?}", *multiname);

        let multiname = multiname.fill_with_runtime_params(self)?;
        let result = self
            .find_definition(&multiname)?
            .or_else(|| self.global_scope());

        self.push_stack(result.map(|o| o.into()).unwrap_or(Value::Undefined));

        Ok(FrameControl::Continue)
    }

    fn op_find_prop_strict(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        avm_debug!(self.context.avm2, "Resolving {:?}", *multiname);

        let multiname = multiname.fill_with_runtime_params(self)?;
        let found: Result<Object<'gc>, Error<'gc>> = self
            .find_definition(&multiname)?
            .ok_or_else(|| make_error_1065(self, &multiname));
        let result: Value<'gc> = found?.into();

        self.push_stack(result);

        Ok(FrameControl::Continue)
    }

    fn op_get_script_globals(
        &mut self,
        script: Script<'gc>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let globals = script.globals(self.context)?;

        self.push_stack(globals);

        Ok(FrameControl::Continue)
    }

    fn op_get_descendants(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;
        if let Some(descendants) = object.xml_descendants(self, &multiname) {
            self.push_stack(descendants);
        } else {
            // Even if it's an object with the "descendants" property, we won't support it.
            let class_name = object
                .instance_class()
                .name()
                .to_qualified_name_err_message(self.context.gc_context);
            return Err(Error::AvmError(type_error(
                self,
                &format!(
                    "Error #1016: Descendants operator (..) not supported on type {}",
                    class_name
                ),
                1016,
            )?));
        }

        Ok(FrameControl::Continue)
    }

    fn op_get_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;
        let value = object.get_slot(index);

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        object.set_slot(index, value, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_set_slot_no_coerce(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        object.set_slot_no_coerce(index, value, self.context.gc_context);

        Ok(FrameControl::Continue)
    }

    fn op_get_global_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self
            .global_scope()
            .map(|global| global.get_slot(index))
            .unwrap_or(Value::Undefined);

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_global_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        self.global_scope()
            .map(|global| global.set_slot(index, value, self))
            .transpose()?;

        Ok(FrameControl::Continue)
    }

    fn op_construct(&mut self, arg_count: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let ctor = self.pop_stack().as_callable(self, None, None, true)?;

        let object = ctor.construct(self, &args)?;

        self.push_stack(object);

        Ok(FrameControl::Continue)
    }

    fn op_construct_prop(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let source = self
            .pop_stack()
            .coerce_to_object_or_typeerror(self, Some(&multiname))?;

        let object = source.construct_prop(&multiname, &args, self)?;

        self.push_stack(object);

        Ok(FrameControl::Continue)
    }

    fn op_construct_super(&mut self, arg_count: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let receiver = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        self.super_init(receiver, &args)?;

        Ok(FrameControl::Continue)
    }

    fn op_new_activation(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let instance = self
            .activation_class
            .expect("Activation class should exist for bytecode")
            .construct(self, &[])?;

        self.push_stack(instance);

        Ok(FrameControl::Continue)
    }

    fn op_new_object(&mut self, num_args: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let object = self.context.avm2.classes().object.construct(self, &[])?;

        for _ in 0..num_args {
            let value = self.pop_stack();
            let name = self.pop_stack();

            object.set_public_property(name.coerce_to_string(self)?, value, self)?;
        }

        self.push_stack(object);

        Ok(FrameControl::Continue)
    }

    fn op_new_function(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMethod>,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let method_entry = self.table_method(method, index, true)?;
        let scope = self.create_scopechain();

        let new_fn = FunctionObject::from_function(self, method_entry, scope)?;

        self.push_stack(new_fn);

        Ok(FrameControl::Continue)
    }

    fn op_new_class(&mut self, class: Class<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        let base_value = self.pop_stack();
        let base_class = match base_value {
            Value::Object(o) => match o.as_class_object() {
                Some(cls) => Some(cls),
                None => return Err("Base class for new class is not a class.".into()),
            },
            Value::Null => None,
            _ => return Err("Base class for new class is not Object or null.".into()),
        };

        let new_class = ClassObject::from_class(self, class, base_class)?;

        self.push_raw(new_class);

        Ok(FrameControl::Continue)
    }

    fn op_apply_type(&mut self, num_types: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(num_types);
        let base = self
            .pop_stack()
            .as_object()
            .ok_or_else(|| make_error_1127(self))?;

        let applied = base.apply(self, &args)?;

        self.push_stack(applied);

        Ok(FrameControl::Continue)
    }

    fn op_new_array(&mut self, num_args: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let args = self.pop_stack_args(num_args);
        let array = ArrayStorage::from_args(&args[..]);
        let array_obj = ArrayObject::from_storage(self, array)?;

        self.push_stack(array_obj);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_b(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_boolean();

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_d(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_d_swap_pop(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;
        let _ = self.pop_stack();

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_i_swap_pop(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;
        let _ = self.pop_stack();

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_o(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        let coerced = match value {
            Value::Undefined | Value::Null => Value::Null,
            _ => value,
        };

        self.push_stack(coerced);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_s(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        let coerced = match value {
            Value::Undefined | Value::Null => Value::Null,
            Value::String(_) => value,
            _ => value.coerce_to_string(self)?.into(),
        };

        self.push_raw(coerced);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_u(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_u32(self)?;

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_u_swap_pop(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_u32(self)?;
        let _ = self.pop_stack();

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_convert_o(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();
        if matches!(value, Value::Null | Value::Undefined) {
            return Err(make_null_or_undefined_error(self, value, None));
        }
        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_convert_s(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_string(self)?;

        self.push_raw(value);

        Ok(FrameControl::Continue)
    }

    fn op_check_filter(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let xml = self.avm2().class_defs().xml;
        let xml_list = self.avm2().class_defs().xml_list;
        let value = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        if value.is_of_type(xml) || value.is_of_type(xml_list) {
            self.push_stack(value);
        } else {
            return Err(Error::AvmError(type_error(
                self,
                &format!(
                    "Error #1123: Filter operator not supported on type {}.",
                    value.instance_of_class_name(self.context.gc_context)
                ),
                1123,
            )?));
        }
        Ok(FrameControl::Continue)
    }

    fn op_add(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let sum_value = match (value1, value2) {
            // note: with not-yet-guaranteed assumption that Integer < 1<<28, this won't overflow.
            (Value::Integer(n1), Value::Integer(n2)) => (n1 + n2).into(),
            (Value::Number(n1), Value::Number(n2)) => (n1 + n2).into(),
            (Value::String(s), value2) => Value::String(AvmString::concat(
                self.context.gc_context,
                s,
                value2.coerce_to_string(self)?,
            )),
            (value1, Value::String(s)) => Value::String(AvmString::concat(
                self.context.gc_context,
                value1.coerce_to_string(self)?,
                s,
            )),
            (Value::Object(value1), Value::Object(value2))
                if (value1.as_xml_list_object().is_some() || value1.as_xml_object().is_some())
                    && (value2.as_xml_list_object().is_some()
                        || value2.as_xml_object().is_some()) =>
            {
                let list = XmlListObject::new(self, None, None);
                // NOTE: Use append here since that correctly sets target property/object.
                list.append(value1.into(), self.gc());
                list.append(value2.into(), self.gc());
                list.into()
            }
            (value1, value2) => {
                let prim_value1 = value1.coerce_to_primitive(None, self)?;
                let prim_value2 = value2.coerce_to_primitive(None, self)?;

                match (prim_value1, prim_value2) {
                    (Value::String(s), value2) => Value::String(AvmString::concat(
                        self.context.gc_context,
                        s,
                        value2.coerce_to_string(self)?,
                    )),
                    (value1, Value::String(s)) => Value::String(AvmString::concat(
                        self.context.gc_context,
                        value1.coerce_to_string(self)?,
                        s,
                    )),
                    (value1, value2) => Value::Number(
                        value1.coerce_to_number(self)? + value2.coerce_to_number(self)?,
                    ),
                }
            }
        };

        self.push_stack(sum_value);

        Ok(FrameControl::Continue)
    }

    fn op_add_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1.wrapping_add(value2));

        Ok(FrameControl::Continue)
    }

    fn op_bitand(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1 & value2);

        Ok(FrameControl::Continue)
    }

    fn op_bitnot(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(!value1);

        Ok(FrameControl::Continue)
    }

    fn op_bitor(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1 | value2);

        Ok(FrameControl::Continue)
    }

    fn op_bitxor(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1 ^ value2);

        Ok(FrameControl::Continue)
    }

    fn op_declocal(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.local_register(index).coerce_to_number(self)?;

        self.set_local_register(index, value - 1.0);

        Ok(FrameControl::Continue)
    }

    fn op_declocal_i(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.local_register(index).coerce_to_i32(self)?;

        self.set_local_register(index, value.wrapping_sub(1));

        Ok(FrameControl::Continue)
    }

    fn op_decrement(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(value - 1.0);

        Ok(FrameControl::Continue)
    }

    fn op_decrement_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value.wrapping_sub(1));

        Ok(FrameControl::Continue)
    }

    fn op_divide(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_number(self)?;
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(value1 / value2);

        Ok(FrameControl::Continue)
    }

    fn op_inclocal(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.local_register(index).coerce_to_number(self)?;

        self.set_local_register(index, value + 1.0);

        Ok(FrameControl::Continue)
    }

    fn op_inclocal_i(&mut self, index: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.local_register(index).coerce_to_i32(self)?;

        self.set_local_register(index, value.wrapping_add(1));

        Ok(FrameControl::Continue)
    }

    fn op_increment(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(value + 1.0);

        Ok(FrameControl::Continue)
    }

    fn op_increment_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value.wrapping_add(1));

        Ok(FrameControl::Continue)
    }

    fn op_lshift(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_u32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1 << (value2 & 0x1F));

        Ok(FrameControl::Continue)
    }

    fn op_modulo(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_number(self)?;
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(value1 % value2);

        Ok(FrameControl::Continue)
    }

    fn op_multiply(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_number(self)?;
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(value1 * value2);

        Ok(FrameControl::Continue)
    }

    fn op_multiply_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1.wrapping_mul(value2));

        Ok(FrameControl::Continue)
    }

    fn op_negate(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_raw(-value1);

        Ok(FrameControl::Continue)
    }

    fn op_negate_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1.wrapping_neg());

        Ok(FrameControl::Continue)
    }

    fn op_rshift(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_u32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1 >> (value2 & 0x1F));

        Ok(FrameControl::Continue)
    }

    fn op_subtract(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let sub_value: Value<'gc> = match (value1, value2) {
            // note: with not-yet-guaranteed assumption that Integer < 1<<28, this won't underflow.
            (Value::Integer(n1), Value::Integer(n2)) => (n1 - n2).into(),
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).into(),
            _ => {
                let value2 = value2.coerce_to_number(self)?;
                let value1 = value1.coerce_to_number(self)?;
                (value1 - value2).into()
            }
        };

        self.push_stack(sub_value);

        Ok(FrameControl::Continue)
    }

    fn op_subtract_i(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_raw(value1.wrapping_sub(value2));

        Ok(FrameControl::Continue)
    }

    fn op_swap(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        self.push_stack(value2);
        self.push_stack(value1);

        Ok(FrameControl::Continue)
    }

    fn op_urshift(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_u32(self)?;
        let value1 = self.pop_stack().coerce_to_u32(self)?;

        self.push_raw(value1 >> (value2 & 0x1F));

        Ok(FrameControl::Continue)
    }

    fn op_jump(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        self.ip += offset;

        Ok(FrameControl::Continue)
    }

    fn op_if_true(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_boolean();

        if value {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_false(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_boolean();

        if !value {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_strict_eq(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value1.strict_eq(&value2) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_strict_ne(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if !value1.strict_eq(&value2) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_eq(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value1.abstract_eq(&value2, self)? {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_ne(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if !value1.abstract_eq(&value2, self)? {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_ge(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value1.abstract_lt(&value2, self)? == Some(false) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_gt(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value2.abstract_lt(&value1, self)? == Some(true) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_le(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value2.abstract_lt(&value1, self)? == Some(false) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_lt(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value1.abstract_lt(&value2, self)? == Some(true) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_nge(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value1.abstract_lt(&value2, self)?.unwrap_or(true) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_ngt(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if !value2.abstract_lt(&value1, self)?.unwrap_or(false) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_nle(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if value2.abstract_lt(&value1, self)?.unwrap_or(true) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_nlt(&mut self, offset: i32) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        if !value1.abstract_lt(&value2, self)?.unwrap_or(false) {
            self.ip += offset;
        }

        Ok(FrameControl::Continue)
    }

    fn op_strict_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_raw(value1.strict_eq(&value2));

        Ok(FrameControl::Continue)
    }

    fn op_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1.abstract_eq(&value2, self)?;

        self.push_raw(result);

        Ok(FrameControl::Continue)
    }

    fn op_greater_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = !value1.abstract_lt(&value2, self)?.unwrap_or(true);

        self.push_raw(result);

        Ok(FrameControl::Continue)
    }

    fn op_greater_than(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value2.abstract_lt(&value1, self)?.unwrap_or(false);

        self.push_raw(result);

        Ok(FrameControl::Continue)
    }

    fn op_less_equals(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = !value2.abstract_lt(&value1, self)?.unwrap_or(true);

        self.push_raw(result);

        Ok(FrameControl::Continue)
    }

    fn op_less_than(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1.abstract_lt(&value2, self)?.unwrap_or(false);

        self.push_raw(result);

        Ok(FrameControl::Continue)
    }

    fn op_not(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack().coerce_to_boolean();

        self.push_raw(!value);

        Ok(FrameControl::Continue)
    }

    fn op_has_next(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let cur_index = self.pop_stack().coerce_to_u32(self)?;

        let object = self.pop_stack();
        if matches!(object, Value::Undefined | Value::Null) {
            self.push_raw(0.0);
        } else {
            let object = object.coerce_to_object(self)?;
            if let Some(next_index) = object.get_next_enumerant(cur_index, self)? {
                self.push_raw(next_index);
            } else {
                self.push_raw(0.0);
            }
        }

        Ok(FrameControl::Continue)
    }
    fn op_has_next_2(
        &mut self,
        object_register: u32,
        index_register: u32,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let mut cur_index = self.local_register(index_register).coerce_to_u32(self)?;

        let object = self.local_register(object_register);

        let mut object = if matches!(object, Value::Undefined | Value::Null) {
            None
        } else {
            Some(object.coerce_to_object(self)?)
        };

        while let Some(cur_object) = object {
            if let Some(index) = cur_object.get_next_enumerant(cur_index, self)? {
                cur_index = index;
                break;
            } else {
                cur_index = 0;
                object = cur_object.proto();
            }
        }

        if object.is_none() {
            cur_index = 0;
        }

        self.push_raw(cur_index != 0);
        self.set_local_register(index_register, cur_index);
        self.set_local_register(
            object_register,
            object.map(|v| v.into()).unwrap_or(Value::Null),
        );

        Ok(FrameControl::Continue)
    }

    fn op_next_name(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let cur_index = self.pop_stack().coerce_to_number(self)?;
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        let name = object.get_enumerant_name(cur_index as u32, self)?;

        self.push_stack(name);

        Ok(FrameControl::Continue)
    }

    fn op_next_value(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let cur_index = self.pop_stack().coerce_to_number(self)?;
        let object = self.pop_stack().coerce_to_object_or_typeerror(self, None)?;

        let value = object.get_enumerant_value(cur_index as u32, self)?;

        self.push_stack(value);

        Ok(FrameControl::Continue)
    }

    fn op_is_type(&mut self, class: Class<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        let is_instance_of = value.is_of_type(self, class);
        self.push_raw(is_instance_of);

        Ok(FrameControl::Continue)
    }

    fn op_is_type_late(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let Some(type_object) = self
            .pop_stack()
            .as_object()
            .and_then(|o| o.as_class_object())
        else {
            return Err(Error::AvmError(type_error(
                self,
                "Error #1041: The right-hand side of operator must be a class.",
                1041,
            )?));
        };
        let value = self.pop_stack();

        let is_instance_of = value.is_of_type(self, type_object.inner_class_definition());
        self.push_raw(is_instance_of);

        Ok(FrameControl::Continue)
    }

    fn op_as_type(&mut self, class: Class<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        if value.is_of_type(self, class) {
            self.push_stack(value);
        } else {
            self.push_raw(Value::Null);
        }

        Ok(FrameControl::Continue)
    }

    fn op_as_type_late(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let class = self.pop_stack();

        if matches!(class, Value::Undefined) {
            return Err(make_null_or_undefined_error(self, class, None));
        }

        if let Some(class) = class.as_object() {
            let Some(class) = class.as_class_object() else {
                return Err(Error::AvmError(type_error(
                    self,
                    "Error #1041: The right-hand side of operator must be a class.",
                    1041,
                )?));
            };
            let value = self.pop_stack();

            if value.is_of_type(self, class.inner_class_definition()) {
                self.push_stack(value);
            } else {
                self.push_raw(Value::Null);
            }

            Ok(FrameControl::Continue)
        } else {
            // Primitive values and null both throw this error
            Err(make_null_or_undefined_error(self, Value::Null, None))
        }
    }

    fn op_instance_of(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let Some(type_object) = self.pop_stack().as_object() else {
            return Err(Error::AvmError(type_error(
                self,
                "Error #1040: The right-hand side of instanceof must be a class or function.",
                1040,
            )?));
        };

        if type_object.as_class_object().is_none() && type_object.as_function_object().is_none() {
            return Err(Error::AvmError(type_error(
                self,
                "Error #1040: The right-hand side of instanceof must be a class or function.",
                1040,
            )?));
        };

        let value = self.pop_stack();

        if let Ok(value) = value.coerce_to_object(self) {
            let is_instance_of = value.is_instance_of(self, type_object)?;

            self.push_raw(is_instance_of);
        } else if matches!(value, Value::Undefined) {
            // undefined
            return Err(make_null_or_undefined_error(self, value, None));
        } else {
            // null
            self.push_raw(false);
        }

        Ok(FrameControl::Continue)
    }

    fn op_type_of(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let value = self.pop_stack();

        let type_name = match value {
            Value::Undefined => "undefined",
            Value::Null => "object",
            Value::Bool(_) => "boolean",
            Value::Number(_) | Value::Integer(_) => "number",
            Value::Object(o) => {
                let classes = self.avm2().class_defs();

                match o {
                    Object::FunctionObject(_) => {
                        if o.instance_class() == classes.function {
                            "function"
                        } else {
                            // Subclasses always have a typeof = "object"
                            "object"
                        }
                    }
                    Object::XmlObject(_) | Object::XmlListObject(_) => {
                        if o.instance_class() == classes.xml_list
                            || o.instance_class() == classes.xml
                        {
                            "xml"
                        } else {
                            // Subclasses always have a typeof = "object"
                            "object"
                        }
                    }
                    _ => "object",
                }
            }
            Value::String(_) => "string",
        };

        self.push_raw(Value::String(type_name.into()));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::EscXAttr`
    fn op_esc_xattr(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let s = self.pop_stack().coerce_to_string(self)?;

        // Implementation of `EscapeAttributeValue` from ECMA-357(10.2.1.2)
        let r = escape_attribute_value(s);
        self.push_raw(AvmString::new(self.context.gc_context, r));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::EscXElem`
    fn op_esc_elem(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let r = match self.pop_stack() {
            // We explicitly call toXMLString on Xml/XmlListObject since the toString of these objects have special handling for simple content, which is not used here.
            Value::Object(Object::XmlObject(x)) => x.as_xml_string(self),
            Value::Object(Object::XmlListObject(x)) => x.as_xml_string(self),
            // contrary to the avmplus documentation, this escapes the value on the top of the stack using EscapeElementValue from ECMA-357 *NOT* EscapeAttributeValue.
            x => AvmString::new(self.gc(), escape_element_value(x.coerce_to_string(self)?)),
        };

        self.push_raw(r);

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::LookupSwitch`
    fn op_lookup_switch(
        &mut self,
        default_offset: i32,
        case_offsets: &[i32],
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        let index = self.pop_stack().coerce_to_i32(self).map_err(|_| {
            Error::from(
                "VerifyError: Invalid value type on stack (should have been int) for LookupSwitch!",
            )
        })?;

        let offset = case_offsets
            .get(index as usize)
            .copied()
            .unwrap_or(default_offset);

        self.ip += offset;
        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Coerce`
    fn op_coerce(&mut self, class: Class<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.pop_stack();
        let x = val.coerce_to_type(self, class)?;

        self.push_stack(x);
        Ok(FrameControl::Continue)
    }

    fn op_coerce_swap_pop(&mut self, class: Class<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.pop_stack();
        let _ = self.pop_stack();

        let x = val.coerce_to_type(self, class)?;

        self.push_stack(x);
        Ok(FrameControl::Continue)
    }

    pub fn domain(&self) -> Domain<'gc> {
        self.outer.domain()
    }

    fn domain_memory(&self) -> ByteArrayObject<'gc> {
        self.outer.domain().domain_memory()
    }

    /// Implements `Op::Si8`
    fn op_si8(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_i32(self)?;
        let val = self.pop_stack().coerce_to_i32(self)? as i8;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut()
            .expect("Bytearray storage should exist");

        let Ok(address) = usize::try_from(address) else {
            return Err(make_error_1506(self));
        };

        if address >= dm.len() {
            return Err(make_error_1506(self));
        }

        dm.set_nongrowing(address, val as u8);

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Si16`
    fn op_si16(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_i32(self)?;
        let val = self.pop_stack().coerce_to_i32(self)? as i16;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut()
            .expect("Bytearray storage should exist");

        let Ok(address) = usize::try_from(address) else {
            return Err(make_error_1506(self));
        };
        if address > dm.len() - 2 {
            return Err(make_error_1506(self));
        }
        dm.write_at_nongrowing(&val.to_le_bytes(), address)
            .map_err(|e| e.to_avm(self))?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Si32`
    fn op_si32(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_i32(self)?;
        let val = self.pop_stack().coerce_to_i32(self)?;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut()
            .expect("Bytearray storage should exist");

        let Ok(address) = usize::try_from(address) else {
            return Err(make_error_1506(self));
        };
        if address > dm.len() - 4 {
            return Err(make_error_1506(self));
        }
        dm.write_at_nongrowing(&val.to_le_bytes(), address)
            .map_err(|e| e.to_avm(self))?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sf32`
    fn op_sf32(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_i32(self)?;
        let val = self.pop_stack().coerce_to_number(self)? as f32;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut()
            .expect("Bytearray storage should exist");

        let Ok(address) = usize::try_from(address) else {
            return Err(make_error_1506(self));
        };
        if address > dm.len() - 4 {
            return Err(make_error_1506(self));
        }
        dm.write_at_nongrowing(&val.to_le_bytes(), address)
            .map_err(|e| e.to_avm(self))?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sf64`
    fn op_sf64(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_i32(self)?;
        let val = self.pop_stack().coerce_to_number(self)?;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut()
            .expect("Bytearray storage should exist");

        let Ok(address) = usize::try_from(address) else {
            return Err(make_error_1506(self));
        };
        if address > dm.len() - 8 {
            return Err(make_error_1506(self));
        }
        dm.write_at_nongrowing(&val.to_le_bytes(), address)
            .map_err(|e| e.to_avm(self))?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Li8`
    fn op_li8(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        let val = dm.get(address);

        if let Some(val) = val {
            self.push_raw(val);
        } else {
            return Err(make_error_1506(self));
        }

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Li16`
    fn op_li16(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 2 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(2, address).map_err(|e| e.to_avm(self))?;
        self.push_raw(u16::from_le_bytes(val.try_into().unwrap()));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Li32`
    fn op_li32(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 4 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(4, address).map_err(|e| e.to_avm(self))?;
        self.push_raw(i32::from_le_bytes(val.try_into().unwrap()));
        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Lf32`
    fn op_lf32(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 4 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(4, address).map_err(|e| e.to_avm(self))?;
        self.push_raw(f32::from_le_bytes(val.try_into().unwrap()));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Lf64`
    fn op_lf64(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 8 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(8, address).map_err(|e| e.to_avm(self))?;
        self.push_raw(f64::from_le_bytes(val.try_into().unwrap()));
        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sxi1`
    fn op_sxi1(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.pop_stack().coerce_to_i32(self)?;

        let val = val.wrapping_shl(31).wrapping_shr(31);

        self.push_raw(Value::Integer(val));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sxi8`
    fn op_sxi8(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.pop_stack().coerce_to_i32(self)?;

        let val = (val.wrapping_shl(23).wrapping_shr(23) & 0xFF) as i8 as i32;

        self.push_raw(Value::Integer(val));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sxi16`
    fn op_sxi16(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let val = self.pop_stack().coerce_to_i32(self)?;

        let val = (val.wrapping_shl(15).wrapping_shr(15) & 0xFFFF) as i16 as i32;

        self.push_raw(Value::Integer(val));

        Ok(FrameControl::Continue)
    }

    #[cfg(feature = "avm_debug")]
    fn op_debug(
        &mut self,
        is_local_register: bool,
        register_name: AvmAtom<'gc>,
        register: u8,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        if is_local_register {
            if (register as usize) < self.local_registers.0.len() {
                let value = self.local_register(register as u32);

                avm_debug!(self.avm2(), "Debug: {register_name} = {value:?}");
            } else {
                avm_debug!(
                    self.avm2(),
                    "Debug: {register_name} = <out-of-bounds register #{register}>",
                );
            }
        } else {
            avm_debug!(self.avm2(), "Unknown debugging mode!");
        }

        Ok(FrameControl::Continue)
    }

    #[cfg(not(feature = "avm_debug"))]
    fn op_debug(
        &mut self,
        _is_local_register: bool,
        _register_name: AvmAtom<'gc>,
        _register: u8,
    ) -> Result<FrameControl<'gc>, Error<'gc>> {
        Ok(FrameControl::Continue)
    }

    #[cfg(feature = "avm_debug")]
    fn op_debug_file(&mut self, file_name: AvmAtom<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        avm_debug!(self.avm2(), "File: {file_name}");

        Ok(FrameControl::Continue)
    }

    #[cfg(not(feature = "avm_debug"))]
    fn op_debug_file(&mut self, _file_name: AvmAtom<'gc>) -> Result<FrameControl<'gc>, Error<'gc>> {
        Ok(FrameControl::Continue)
    }

    fn op_debug_line(&mut self, line_num: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        avm_debug!(self.avm2(), "Line: {line_num}");

        Ok(FrameControl::Continue)
    }

    fn op_bkpt(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // while a debugger is not attached, this is a no-op
        Ok(FrameControl::Continue)
    }

    fn op_bkpt_line(&mut self, _line_num: u32) -> Result<FrameControl<'gc>, Error<'gc>> {
        // while a debugger is not attached, this is a no-op
        Ok(FrameControl::Continue)
    }

    fn op_timestamp(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        // while a debugger is not attached, this is a no-op
        Ok(FrameControl::Continue)
    }

    fn op_throw(&mut self) -> Result<FrameControl<'gc>, Error<'gc>> {
        let error_val = self.pop_stack();
        Err(Error::AvmError(error_val))
    }
}
