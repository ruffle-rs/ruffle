//! Activation frames

use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::e4x::{escape_attribute_value, escape_element_value};
use crate::avm2::error::{
    make_error_1065, make_error_1127, make_error_1506, make_null_or_undefined_error, type_error,
};
use crate::avm2::function::FunctionArgs;
use crate::avm2::method::{Method, NativeMethodImpl, ResolvedParamConfig};
use crate::avm2::object::{
    ArrayObject, ByteArrayObject, ClassObject, FunctionObject, NamespaceObject, ScriptObject,
    XmlListObject,
};
use crate::avm2::object::{Object, TObject};
use crate::avm2::op::{LookupSwitch, Op};
use crate::avm2::scope::{search_scope_stack, Scope, ScopeChain};
use crate::avm2::script::Script;
use crate::avm2::stack::StackFrame;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use crate::string::{AvmAtom, AvmString, HasStringContext, StringContext};
use crate::tag_utils::SwfMovie;
use gc_arena::Gc;
use ruffle_macros::istr;
use std::cmp::{min, Ordering};
use std::sync::Arc;
use swf::avm2::types::{Index, Method as AbcMethod, MethodFlags as AbcMethodFlags};

use super::error::make_mismatch_error;

/// Represents a single activation of a given AVM2 function or keyframe.
pub struct Activation<'a, 'gc: 'a> {
    /// The number of locals this method uses.
    num_locals: usize,

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

    /// The stack frame.
    stack: StackFrame<'a, 'gc>,

    /// The index where the scope frame starts.
    scope_depth: usize,

    /// In avmplus, some behavior differs slightly depending on whether the JIT
    /// or the interpreter is used. Most methods are executed in "JIT mode", but
    /// in some cases "interpreter mode" is used instead: for example, script
    /// initializers always execute in "interpreter mode". We keep track of
    /// whether the current method would be interpreted or JITted in this flag.
    is_interpreter: bool,

    pub context: &'a mut UpdateContext<'gc>,
}

impl<'gc> HasStringContext<'gc> for Activation<'_, 'gc> {
    #[inline(always)]
    fn strings_ref(&self) -> &StringContext<'gc> {
        &self.context.strings
    }
}

impl<'a, 'gc> Activation<'a, 'gc> {
    /// Convenience method to retrieve the current GC context. Note that explicitly writing
    /// `self.context.gc_context` can be sometimes necessary to satisfy the borrow checker.
    #[inline(always)]
    pub fn gc(&self) -> &'gc gc_arena::Mutation<'gc> {
        self.context.gc()
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
        Self {
            num_locals: 0,
            outer: ScopeChain::new(context.avm2.stage_domain),
            caller_domain: None,
            caller_movie: None,
            bound_superclass_object: None,
            bound_class: None,
            stack: StackFrame::empty(),
            scope_depth: context.avm2.scope_stack.len(),
            is_interpreter: false,
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
        Self {
            num_locals: 0,
            outer: ScopeChain::new(context.avm2.stage_domain),
            caller_domain: Some(domain),
            caller_movie: None,
            bound_superclass_object: None,
            bound_class: None,
            stack: StackFrame::empty(),
            scope_depth: context.avm2.scope_stack.len(),
            is_interpreter: false,
            context,
        }
    }

    /// Construct an activation for the execution of a particular script's
    /// initializer method. This is intended to be used immediately after
    /// from_nothing(), to make it easier to clean up after the Activation runs.
    pub fn init_from_script(
        &mut self,
        script: Script<'gc>,
        stack_frame: StackFrame<'a, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let (method, global_object, domain) = script.init();

        let body = method
            .body()
            .expect("Cannot execute method for script without body");

        let num_locals = body.num_locals as usize;

        self.num_locals = num_locals;
        self.outer = ScopeChain::new(domain);
        self.caller_domain = Some(domain);
        self.caller_movie = Some(script.translation_unit().movie());
        self.bound_superclass_object = Some(self.context.avm2.classes().object);
        self.bound_class = Some(script.global_class());
        self.stack = stack_frame;
        self.scope_depth = self.context.avm2.scope_stack.len();
        self.is_interpreter = true; // Script initializers are always in interpreter mode

        // Resolve signature
        method.resolve_info(self)?;

        // Run verifier for bytecode methods
        method.verify(self)?;

        // Create locals- script init methods are run with no parameters passed
        self.push_stack(global_object);
        for _ in 0..num_locals - 1 {
            self.push_stack(Value::Undefined);
        }

        Ok(())
    }

    /// Finds an object on either the current or outer scope of this activation by definition.
    pub fn find_definition(
        &mut self,
        name: &Multiname<'gc>,
    ) -> Result<Option<Value<'gc>>, Error<'gc>> {
        let outer_scope = self.outer;

        if let Some(obj) = search_scope_stack(self, name, outer_scope.is_empty()) {
            Ok(Some(obj))
        } else if let Some(obj) = outer_scope.find(name, self)? {
            Ok(Some(obj))
        } else {
            let global = self.global_scope();

            if global
                .as_object()
                .is_some_and(|o| o.base().has_own_dynamic_property(name))
            {
                Ok(Some(global))
            } else {
                Ok(None)
            }
        }
    }

    /// Statically resolve all of the parameters for a native method.
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
            let coerced_arg = if let Some(param_class) = param_config.param_type {
                arg.coerce_to_type(self, param_class)?
            } else {
                *arg
            };

            arguments_list.push(coerced_arg);
        }

        match user_arguments.len().cmp(&signature.len()) {
            Ordering::Greater => {
                // Variadic parameters exist, just push them into the list
                arguments_list.extend_from_slice(&user_arguments[signature.len()..])
            }
            Ordering::Less => {
                // Apply remaining default parameters
                for param_config in signature[user_arguments.len()..].iter() {
                    let arg = if let Some(default_value) = &param_config.default_value {
                        *default_value
                    } else {
                        return Err(Error::avm_error(make_mismatch_error(
                            self,
                            method,
                            user_arguments.len(),
                            bound_class,
                        )?));
                    };

                    let coerced_arg = if let Some(param_class) = param_config.param_type {
                        arg.coerce_to_type(self, param_class)?
                    } else {
                        arg
                    };

                    arguments_list.push(coerced_arg);
                }
            }
            _ => {}
        }

        Ok(arguments_list)
    }

    /// Create an `arguments` or `rest` object for a given method. This function
    /// expects the rest of the arguments to already be on the AVM stack.
    #[inline(never)]
    fn create_varargs_object(
        &mut self,
        method: Method<'gc>,
        signature: &[ResolvedParamConfig<'gc>],
        user_arguments: FunctionArgs<'_, 'gc>,
        callee: Value<'gc>,
    ) -> ArrayObject<'gc> {
        let mut all_arguments = Vec::new();

        // Unfortunately we need to allocate now: we need to put all the
        // arguments we just processed into a Vec, so `arguments` or `rest`
        // can put them into an ArrayObject. The missing argument check
        // in `init_from_method` ensures that we have at least `signature.len()`
        // arguments on the stack right now.
        for i in 0..signature.len() {
            let arg = self.stack.peek(signature.len() - i - 1);
            all_arguments.push(arg);
        }

        // We put all the non-variadic arguments into the Vec, but there
        // could have been more passed to the function. Add them now.
        for i in signature.len()..user_arguments.len() {
            let arg = user_arguments.get_at(i);
            all_arguments.push(arg);
        }

        let args_array = if method.method().flags.contains(AbcMethodFlags::NEED_REST) {
            if let Some(rest_args) = all_arguments.get(signature.len()..) {
                ArrayStorage::from_args(rest_args)
            } else {
                ArrayStorage::new(0)
            }
        } else if method
            .method()
            .flags
            .contains(AbcMethodFlags::NEED_ARGUMENTS)
        {
            ArrayStorage::from_args(&all_arguments[..user_arguments.len()])
        } else {
            unreachable!();
        };

        let args_object = ArrayObject::from_storage(self, args_array);

        if method
            .method()
            .flags
            .contains(AbcMethodFlags::NEED_ARGUMENTS)
        {
            let string_callee = istr!(self, "callee");

            args_object.set_dynamic_property(string_callee, callee, self.gc());
            args_object.set_local_property_is_enumerable(self.gc(), string_callee, false);
        }

        args_object
    }

    /// Construct an activation for the execution of a particular bytecode
    /// method.
    /// NOTE: this is intended to be used immediately after from_nothing(),
    /// as a more efficient replacement for direct `Activation::from_method()`
    #[allow(clippy::too_many_arguments)]
    pub fn init_from_method(
        &mut self,
        method: Method<'gc>,
        outer: ScopeChain<'gc>,
        this: Value<'gc>,
        user_arguments: FunctionArgs<'_, 'gc>,
        stack_frame: StackFrame<'a, 'gc>,
        bound_superclass_object: Option<ClassObject<'gc>>,
        bound_class: Option<Class<'gc>>,
        callee: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        let body = method
            .body()
            .expect("Cannot execute non-native method without body");

        let num_locals = body.num_locals as usize;
        let has_rest_or_args = method.is_variadic();

        if let Some(bound_class) = bound_class {
            assert!(this.is_of_type(self, bound_class));
        }

        self.num_locals = num_locals;
        self.outer = outer;
        self.caller_domain = Some(outer.domain());
        self.caller_movie = Some(method.owner_movie());
        self.bound_superclass_object = bound_superclass_object;
        self.bound_class = bound_class;
        self.stack = stack_frame;
        self.scope_depth = self.context.avm2.scope_stack.len();
        self.is_interpreter = false;

        // Resolve parameters and return type
        method.resolve_info(self)?;

        // Everything is now setup for the verifier to run
        method.verify(self)?;

        let signature = method.resolved_param_config();

        if user_arguments.len() > signature.len() && !has_rest_or_args && !method.is_unchecked() {
            return Err(Error::avm_error(make_mismatch_error(
                self,
                method,
                user_arguments.len(),
                bound_class,
            )?));
        }

        // Create locals
        self.push_stack(this);

        // Statically verify all non-variadic, provided parameters.
        let static_arg_count = min(user_arguments.len(), signature.len());
        for (i, param_config) in signature.iter().enumerate().take(static_arg_count) {
            let arg = user_arguments.get_at(i);

            let coerced_arg = if let Some(param_class) = param_config.param_type {
                arg.coerce_to_type(self, param_class)?
            } else {
                arg
            };

            self.push_stack(coerced_arg);
        }

        // Now add missing arguments
        if user_arguments.len() < signature.len() {
            // Apply remaining default parameters
            for param_config in signature[user_arguments.len()..].iter() {
                let arg = if let Some(default_value) = &param_config.default_value {
                    *default_value
                } else if method.is_unchecked() {
                    Value::Undefined
                } else {
                    return Err(Error::avm_error(make_mismatch_error(
                        self,
                        method,
                        user_arguments.len(),
                        bound_class,
                    )?));
                };

                let coerced_arg = if let Some(param_class) = param_config.param_type {
                    arg.coerce_to_type(self, param_class)?
                } else {
                    arg
                };

                self.push_stack(coerced_arg);
            }
        }

        // Finally, handle variadic arguments
        if has_rest_or_args {
            let args_object = self.create_varargs_object(method, signature, user_arguments, callee);

            self.push_stack(args_object);
        }

        // `Stack::get_stack_frame` already initializes all values on the frame
        // to undefined, so we just have to increase the stack pointer
        self.reset_stack();

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
        Self {
            num_locals: 0,
            outer,
            caller_domain,
            caller_movie,
            bound_superclass_object,
            bound_class,
            stack: StackFrame::empty(),
            scope_depth: context.avm2.scope_stack.len(),
            is_interpreter: false,
            context,
        }
    }

    /// Call the superclass's instance initializer.
    ///
    /// This method may panic if called with a Null or Undefined receiver.
    pub fn super_init(
        &mut self,
        receiver: Value<'gc>,
        args: FunctionArgs<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let bound_superclass_object = self
            .bound_superclass_object
            .expect("Superclass object is required to run super_init");

        bound_superclass_object.call_init_with_args(receiver, args, self)
    }

    /// Retrieve a local register.
    pub fn local_register(&mut self, id: u32) -> Value<'gc> {
        // Verification guarantees that this points to a local register
        self.stack.value_at(id as usize)
    }

    /// Set a local register.
    pub fn set_local_register(&mut self, id: u32, value: impl Into<Value<'gc>>) {
        // Verification guarantees that this is valid
        self.stack.set_value_at(id as usize, value.into());
    }

    /// Returns whether this Activation is running in "interpreter mode" as
    /// opposed to "JIT mode". Note that these modes do not actually correspond
    /// to whether the method is being interpreted or JITted.
    pub fn is_interpreter(&self) -> bool {
        self.is_interpreter
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
        self.outer.chain(self.gc(), self.scope_frame())
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
        self.caller_movie().unwrap_or(self.context.root_swf.clone())
    }

    /// Returns the global scope of this activation.
    ///
    /// The global scope refers to scope at the bottom of the
    /// outer scope. If the outer scope is empty, we use the bottom
    /// of the current scope stack instead.
    ///
    /// The verifier guarantees that there is always a global scope
    /// when this function is called.
    pub fn global_scope(&self) -> Value<'gc> {
        let outer_scope = self.outer;
        outer_scope
            .get(0)
            .unwrap_or_else(|| self.scope_frame()[0])
            .values()
    }

    pub fn avm2(&mut self) -> &mut Avm2<'gc> {
        self.context.avm2
    }

    pub fn scope_frame(&self) -> &[Scope<'gc>] {
        &self.context.avm2.scope_stack[self.scope_depth..]
    }

    /// Pushes a value onto the operand stack.
    #[inline(always)]
    pub fn push_stack(&self, value: impl Into<Value<'gc>>) {
        self.stack.push(value.into());
    }

    /// Pops a value off the operand stack.
    #[inline(always)]
    #[must_use]
    pub fn pop_stack(&self) -> Value<'gc> {
        self.stack.pop()
    }

    /// Pops multiple values off the operand stack.
    #[inline]
    #[must_use]
    pub fn pop_stack_args(&self, arg_count: u32) -> Vec<Value<'gc>> {
        self.stack.pop_args(arg_count)
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

    /// Cleans up after this Activation. This removes stack and local entries,
    /// and clears the scope stack. This method must be called after an Activation
    /// created with `Activation::init_from_activation` or `Activation::from_script`
    /// is finished executing.
    ///
    /// This function should take `mut self` instead of `&mut self`, but that
    /// results in worse codegen (the entire Activation is moved).
    #[inline]
    pub fn cleanup(&mut self) {
        self.clear_scope();

        let stack = self.context.avm2.stack;
        stack.dispose_stack_frame(self.stack.take());
    }

    /// Clears the operand stack used by this activation.
    ///
    /// This is called `reset_stack` because it sets the stack pointer to the
    /// first stack entry, which also makes it useful for initializing the stack.
    #[inline]
    fn reset_stack(&self) {
        // This sets the stack pointer to the first stack entry, which is right
        // after all the entries for local registers
        self.stack.set_stack_pointer(self.num_locals);
    }

    /// Clears the scope stack used by this activation.
    #[inline]
    fn clear_scope(&mut self) {
        let scope_depth = self.scope_depth;
        self.avm2().scope_stack.truncate(scope_depth);
    }

    /// Get the superclass of the class that defined the currently-executing
    /// method, if it exists.
    ///
    /// If the currently-executing method is not part of a class, then this
    /// returns `None`.
    pub fn bound_superclass_object(&self) -> Option<ClassObject<'gc>> {
        self.bound_superclass_object
    }

    /// Get the class that defined the currently-executing method, if it exists.
    pub fn bound_class(&self) -> Option<Class<'gc>> {
        self.bound_class
    }

    /// Retrieve a method entry from the current ABC file's method table.
    fn table_method(
        &mut self,
        method: Method<'gc>,
        index: Index<AbcMethod>,
        is_function: bool,
    ) -> Result<Method<'gc>, Error<'gc>> {
        method
            .translation_unit()
            .load_method(index, is_function, self)
    }

    pub fn run_actions(&mut self, method: Method<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        // The method must be verified at this point

        let verified_info = method.get_verified_info();
        let opcodes = verified_info.parsed_code.as_slice();

        self.timeout_check()?;

        let mut ip = 0;

        loop {
            let op = &opcodes[ip];
            ip += 1;
            avm_debug!(self.avm2(), "Opcode: {op:?}");

            let result = match op {
                Op::PushDouble { value } => self.op_push_double(*value),
                Op::PushFalse => self.op_push_false(),
                Op::PushInt { value } => self.op_push_int(*value),
                Op::PushNamespace { namespace } => self.op_push_namespace(*namespace),
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
                Op::CallNative {
                    method,
                    num_args,
                    push_return_value,
                } => self.op_call_native(*method, *num_args, *push_return_value),
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
                Op::GetPropertyStatic { multiname } => self.op_get_property_static(*multiname),
                Op::GetPropertyFast { multiname } => self.op_get_property_fast(*multiname),
                Op::GetPropertySlow { multiname } => self.op_get_property_slow(*multiname),
                Op::SetPropertyStatic { multiname } => self.op_set_property_static(*multiname),
                Op::SetPropertyFast { multiname } => self.op_set_property_fast(*multiname),
                Op::SetPropertySlow { multiname } => self.op_set_property_slow(*multiname),
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
                Op::FindDef { multiname } => self.op_find_def(*multiname),
                Op::FindProperty { multiname } => self.op_find_property(*multiname),
                Op::FindPropStrict { multiname } => self.op_find_prop_strict(*multiname),
                Op::GetScriptGlobals { script } => self.op_get_script_globals(*script),
                Op::GetDescendants { multiname } => self.op_get_descendants(*multiname),
                Op::GetSlot { index } => self.op_get_slot(*index),
                Op::SetSlot { index } => self.op_set_slot(*index),
                Op::SetSlotNoCoerce { index } => self.op_set_slot_no_coerce(*index),
                Op::SetGlobalSlot { index } => self.op_set_global_slot(*index),
                Op::Construct { num_args } => self.op_construct(*num_args),
                Op::ConstructProp {
                    multiname,
                    num_args,
                } => self.op_construct_prop(*multiname, *num_args),
                Op::ConstructSlot { index, num_args } => self.op_construct_slot(*index, *num_args),
                Op::ConstructSuper { num_args } => self.op_construct_super(*num_args),
                Op::NewActivation { activation_class } => self.op_new_activation(*activation_class),
                Op::NewObject { num_args } => self.op_new_object(*num_args),
                Op::NewFunction { index } => self.op_new_function(method, *index),
                Op::NewClass { class } => self.op_new_class(*class),
                Op::ApplyType { num_types } => self.op_apply_type(*num_types),
                Op::NewArray { num_args } => self.op_new_array(*num_args),
                Op::CoerceA => Ok(()),
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
                Op::StrictEquals => self.op_strict_equals(),
                Op::Equals => self.op_equals(),
                Op::GreaterEquals => self.op_greater_equals(),
                Op::GreaterThan => self.op_greater_than(),
                Op::LessEquals => self.op_less_equals(),
                Op::LessThan => self.op_less_than(),
                Op::Nop => Ok(()),
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
                Op::Dxns { .. } => self.op_dxns(),
                Op::DxnsLate => self.op_dxns_late(),
                Op::EscXAttr => self.op_esc_xattr(),
                Op::EscXElem => self.op_esc_elem(),
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

                // Branch ops
                Op::Jump { offset } => {
                    self.timeout_check()?;

                    ip = *offset;

                    continue;
                }
                Op::IfTrue { offset } => {
                    self.timeout_check()?;

                    if self.check_if_true() {
                        ip = *offset;
                    }

                    continue;
                }
                Op::IfFalse { offset } => {
                    self.timeout_check()?;

                    if !self.check_if_true() {
                        ip = *offset;
                    }

                    continue;
                }
                Op::LookupSwitch(lookup_switch) => {
                    self.timeout_check()?;

                    ip = self.lookup_switch(lookup_switch);

                    continue;
                }

                // Return ops
                Op::ReturnValue { return_type } => {
                    let coerce_result = self.return_value(*return_type);
                    match coerce_result {
                        Ok(value) => return Ok(value),
                        Err(error) => Err(error),
                    }
                }
                Op::ReturnVoid { return_type } => {
                    return Ok(self.return_void(*return_type));
                }
            };

            if let Err(error) = result {
                ip = self.handle_err(method, ip, error)?;
            }
        }
    }

    /// If a local exception handler exists for the error, use it to handle
    /// the error. Otherwise pass the error down the stack.
    fn handle_err(
        &mut self,
        method: Method<'gc>,
        ip: usize,
        error: Error<'gc>,
    ) -> Result<usize, Error<'gc>> {
        let error = if let Some(error) = error.as_avm_error() {
            error
        } else {
            return Err(error);
        };

        let verified_info = method.get_verified_info();
        let exception_list = &verified_info.exceptions;

        let last_ip = ip - 1;
        for e in exception_list {
            if last_ip >= e.from_offset && last_ip < e.to_offset {
                let matches = if let Some(target_class) = e.target_class {
                    // This ensures null and undefined don't match
                    error.is_of_type(self, target_class)
                } else {
                    // A typeless catch block (i.e. `catch(err:*) { ... }`) will
                    // always match.
                    true
                };

                if matches {
                    #[cfg(feature = "avm_debug")]
                    tracing::info!(target: "avm_caught", "Caught exception: {:?}", Error::avm_error(error));

                    self.reset_stack();
                    self.push_stack(error);

                    self.clear_scope();
                    return Ok(e.target_offset);
                }
            }
        }

        Err(Error::avm_error(error))
    }

    #[inline(always)]
    fn timeout_check(&mut self) -> Result<(), Error<'gc>> {
        *self.context.actions_since_timeout_check += 1;
        if *self.context.actions_since_timeout_check >= 10000 {
            *self.context.actions_since_timeout_check = 0;
            if self.context.update_start.elapsed() >= self.context.max_execution_duration {
                return Err(
                    "A script in this movie has taken too long to execute and has been terminated."
                        .into(),
                );
            }
        }
        Ok(())
    }

    fn op_push_double(&mut self, value: f64) -> Result<(), Error<'gc>> {
        self.push_stack(value);
        Ok(())
    }

    fn op_push_false(&mut self) -> Result<(), Error<'gc>> {
        self.push_stack(false);
        Ok(())
    }

    fn op_push_int(&mut self, value: i32) -> Result<(), Error<'gc>> {
        self.push_stack(value);
        Ok(())
    }

    fn op_push_namespace(&mut self, namespace: Namespace<'gc>) -> Result<(), Error<'gc>> {
        let ns_object = NamespaceObject::from_namespace(self, namespace);

        self.push_stack(ns_object);
        Ok(())
    }

    fn op_push_null(&mut self) -> Result<(), Error<'gc>> {
        self.push_stack(Value::Null);
        Ok(())
    }

    fn op_push_short(&mut self, value: i16) -> Result<(), Error<'gc>> {
        self.push_stack(value);
        Ok(())
    }

    fn op_push_string(&mut self, string: AvmAtom<'gc>) -> Result<(), Error<'gc>> {
        self.push_stack(string);
        Ok(())
    }

    fn op_push_true(&mut self) -> Result<(), Error<'gc>> {
        self.push_stack(true);
        Ok(())
    }

    fn op_push_uint(&mut self, value: u32) -> Result<(), Error<'gc>> {
        self.push_stack(value);
        Ok(())
    }

    fn op_push_undefined(&mut self) -> Result<(), Error<'gc>> {
        self.push_stack(Value::Undefined);
        Ok(())
    }

    fn op_pop(&mut self) -> Result<(), Error<'gc>> {
        let _ = self.pop_stack();

        Ok(())
    }

    fn op_dup(&mut self) -> Result<(), Error<'gc>> {
        let value = self.stack.peek(0);
        self.push_stack(value);

        Ok(())
    }

    fn op_get_local(&mut self, register_index: u32) -> Result<(), Error<'gc>> {
        let value = self.local_register(register_index);
        self.push_stack(value);

        Ok(())
    }

    fn op_set_local(&mut self, register_index: u32) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        self.set_local_register(register_index, value);

        Ok(())
    }

    fn op_kill(&mut self, register_index: u32) -> Result<(), Error<'gc>> {
        self.set_local_register(register_index, Value::Undefined);

        Ok(())
    }

    fn op_call(&mut self, arg_count: u32) -> Result<(), Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let receiver = self.pop_stack();
        let function = self.pop_stack();

        let value = function.call(self, receiver, &args)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_call_method(
        &mut self,
        index: u32,
        arg_count: u32,
        push_return_value: bool,
    ) -> Result<(), Error<'gc>> {
        // The entire implementation of VTable assumes that
        // call_method is never encountered in user code. (see the long comment there)
        // This was also the conclusion from analysing avmplus behavior - they
        // unconditionally VerifyError upon noticing it.

        // However, the optimizer can still generate it.

        let args = self.stack.get_args(arg_count as usize);
        let receiver = self.pop_stack().null_check(self, None)?;

        let value = receiver.call_method_with_args(index, args, self)?;

        if push_return_value {
            self.push_stack(value);
        }

        Ok(())
    }

    fn op_call_native(
        &mut self,
        method: NativeMethodImpl,
        num_args: u32,
        push_return_value: bool,
    ) -> Result<(), Error<'gc>> {
        let mut args_buf = [Value::Undefined; 2];
        let args = &mut args_buf[..num_args as usize];
        for arg in args.iter_mut().rev() {
            *arg = self.pop_stack();
        }

        let receiver = self.pop_stack().null_check(self, None)?;

        let value = method(self, receiver, args).expect("FastCall methods should not return Err");

        if push_return_value {
            self.push_stack(value);
        }

        Ok(())
    }

    fn op_call_property(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self.pop_stack().null_check(self, Some(&multiname))?;

        let value = receiver.call_property(&multiname, args, self)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_call_prop_lex(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<(), Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self.pop_stack().null_check(self, Some(&multiname))?;
        let function = receiver.get_property(&multiname, self)?;
        let value = function.call(self, Value::Null, &args)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_call_prop_void(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self.pop_stack().null_check(self, Some(&multiname))?;

        receiver.call_property(&multiname, args, self)?;

        Ok(())
    }

    fn op_call_static(
        &mut self,
        method: Method<'gc>,
        index: Index<AbcMethod>,
        arg_count: u32,
    ) -> Result<(), Error<'gc>> {
        let args = self.pop_stack_args(arg_count);
        let receiver = self.pop_stack();
        let method = self.table_method(method, index, false)?;
        // TODO: What scope should the function be executed with?
        let scope = self.create_scopechain();
        let function = FunctionObject::from_method(self, method, scope, None, None, None);
        let value = function.call(self, receiver, &args)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_call_super(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let receiver = self
            .pop_stack()
            .null_check(self, Some(&multiname))?
            .as_object()
            .expect("Super ops should not appear in primitive functions");

        let bound_superclass_object = self
            .bound_superclass_object()
            .expect("Expected a superclass when running callsuper");

        let value = bound_superclass_object.call_super(&multiname, receiver, args, self)?;

        self.push_stack(value);

        Ok(())
    }

    fn return_value(&mut self, return_type: Option<Class<'gc>>) -> Result<Value<'gc>, Error<'gc>> {
        let return_value = self.pop_stack();

        if let Some(return_type) = return_type {
            return_value.coerce_to_type(self, return_type)
        } else {
            Ok(return_value)
        }
    }

    fn return_void(&mut self, return_type: Option<Class<'gc>>) -> Value<'gc> {
        // Manual coerce_to_type

        let class_defs = self.avm2().class_defs();

        if let Some(return_type) = return_type {
            if return_type == class_defs.void {
                Value::Undefined
            } else if return_type == class_defs.int
                || return_type == class_defs.uint
                || return_type == class_defs.number
            {
                0.into()
            } else if return_type == class_defs.boolean {
                false.into()
            } else {
                Value::Null
            }
        } else {
            Value::Undefined
        }
    }

    fn op_get_property_static(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // default path for static names
        let object = self.pop_stack().null_check(self, Some(&multiname))?;

        let value = object.get_property(&multiname, self)?;
        self.push_stack(value);

        Ok(())
    }

    fn op_get_property_fast(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // (fast) side path for dictionary/array-likes

        // `MultinameL` is the only form of multiname that allows fast-path
        // or alternate-path lookups based on the local name *value*,
        // rather than it's string representation.

        let name_value = self.stack.peek(0);
        let object_value = self.stack.peek(1);

        if let Value::Object(object) = object_value {
            match name_value {
                Value::Integer(name_int) if name_int >= 0 => {
                    if let Some(value) = object.get_index_property(name_int as usize) {
                        let _ = self.pop_stack();
                        let _ = self.pop_stack();
                        self.push_stack(value);

                        return Ok(());
                    }
                }
                Value::Object(name_object) => {
                    if let Some(dictionary) = object.as_dictionary_object() {
                        let _ = self.pop_stack();
                        let _ = self.pop_stack();
                        let value = dictionary.get_property_by_object(name_object);
                        self.push_stack(value);

                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        // If it wasn't actually a fast-path access, fall back to the slow version
        self.op_get_property_slow(multiname)
    }

    fn op_get_property_slow(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // main path for dynamic names
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack().null_check(self, Some(&multiname))?;

        let value = object.get_property(&multiname, self)?;
        self.push_stack(value);

        Ok(())
    }

    fn op_set_property_static(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // default path for static names

        let value = self.pop_stack();

        let object = self.pop_stack().null_check(self, Some(&multiname))?;

        object.set_property(&multiname, value, self)?;

        Ok(())
    }

    fn op_set_property_fast(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // side path for dictionary/arrays

        let value = self.stack.peek(0);

        // `MultinameL` is the only form of multiname that allows fast-path
        // or alternate-path lookups based on the local name *value*,
        // rather than it's string representation.

        let name_value = self.stack.peek(1);
        let object_value = self.stack.peek(2);

        if let Value::Object(object) = object_value {
            match name_value {
                Value::Integer(name_int) if name_int >= 0 => {
                    if let Some(result) = object.set_index_property(self, name_int as usize, value)
                    {
                        let _ = self.pop_stack();
                        let _ = self.pop_stack();
                        let _ = self.pop_stack();

                        return result;
                    }
                }
                Value::Object(name_object) => {
                    if let Some(dictionary) = object.as_dictionary_object() {
                        let _ = self.pop_stack();
                        let _ = self.pop_stack();
                        let _ = self.pop_stack();
                        dictionary.set_property_by_object(name_object, value, self.gc());

                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        // If it wasn't actually a fast-path access, fall back to the slow version
        self.op_set_property_slow(multiname)
    }

    fn op_set_property_slow(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // main path for dynamic names

        let value = self.pop_stack();

        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack().null_check(self, Some(&multiname))?;

        object.set_property(&multiname, value, self)?;

        Ok(())
    }

    fn op_init_property(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        let multiname = multiname.fill_with_runtime_params(self)?;

        let object = self.pop_stack().null_check(self, Some(&multiname))?;

        object.init_property(&multiname, value, self)?;

        Ok(())
    }

    fn op_delete_property(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        // default path for static names
        if !multiname.has_lazy_component() {
            let object = self.pop_stack().null_check(self, Some(&multiname))?;

            let did_delete = object.delete_property(self, &multiname)?;
            self.push_stack(did_delete);

            return Ok(());
        }

        // side path for dictionary/arrays (TODO)
        if multiname.has_lazy_name() && !multiname.has_lazy_ns() {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.stack.peek(0);
            let object = self.stack.peek(1);
            if let Some(name_object) = name_value.as_object() {
                if let Some(dictionary) = object.as_object().and_then(|o| o.as_dictionary_object())
                {
                    let _ = self.pop_stack();
                    let _ = self.pop_stack();
                    dictionary.delete_property_by_object(name_object, self.gc());

                    self.push_stack(true);
                    return Ok(());
                }
            }
        }

        // main path for dynamic names
        if multiname.has_lazy_name() {
            let name_value = self.stack.peek(0);
            if matches!(name_value, Value::Object(Object::XmlListObject(_))) {
                // ECMA-357 11.3.1 The delete Operator
                // If the type of the operand is XMLList, then a TypeError exception is thrown.
                return Err(Error::avm_error(type_error(
                    self,
                    "Error #1119: Delete operator is not supported with operand of type XMLList.",
                    1119,
                )?));
            }
        }
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack().null_check(self, Some(&multiname))?;

        let did_delete = object.delete_property(self, &multiname)?;

        self.push_stack(did_delete);

        Ok(())
    }

    fn op_get_super(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self
            .pop_stack()
            .null_check(self, Some(&multiname))?
            .as_object()
            .expect("Super ops should not appear in primitive functions");

        let bound_superclass_object = self
            .bound_superclass_object()
            .expect("Expected a superclass when running getsuper");

        let value = bound_superclass_object.get_super(&multiname, object, self)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_set_super(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self
            .pop_stack()
            .null_check(self, Some(&multiname))?
            .as_object()
            .expect("Super ops should not appear in primitive functions");

        let bound_superclass_object = self
            .bound_superclass_object()
            .expect("Expected a superclass when running setsuper");

        bound_superclass_object.set_super(&multiname, value, object, self)?;

        Ok(())
    }

    fn op_in(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().null_check(self, None)?;
        let name_value = self.pop_stack();

        let has_prop = match value {
            Value::Object(obj) => {
                if let Some(dictionary) = obj.as_dictionary_object() {
                    if let Some(name_object) = name_value.as_object() {
                        self.push_stack(dictionary.has_property_by_object(name_object));

                        return Ok(());
                    }
                }

                let name = name_value.coerce_to_string(self)?;
                let multiname = Multiname::new(self.avm2().find_public_namespace(), name);

                obj.has_property_via_in(self, &multiname)?
            }
            _ => {
                let name = name_value.coerce_to_string(self)?;
                let multiname = Multiname::new(self.avm2().find_public_namespace(), name);

                if value.has_trait(self, &multiname) {
                    true
                } else if let Some(proto) = value.proto(self) {
                    proto.has_property(&multiname)
                } else {
                    // This condition can't be met, since `Value::proto` always
                    // returns `Some` for primitives)
                    unreachable!()
                }
            }
        };

        self.push_stack(has_prop);

        Ok(())
    }

    fn op_newcatch(&mut self, method: Method<'gc>, index: usize) -> Result<(), Error<'gc>> {
        // TODO can we store the catch class in the op?
        let verified_info = method.get_verified_info();
        let exception_list = &verified_info.exceptions;

        let catch_class = &exception_list[index].catch_class;

        let so = if let Some(catch_class) = catch_class {
            // Catch objects don't have prototypes, and we can give it the
            // Class's vtable because the only trait on the vtable is a single
            // const-slot.
            ScriptObject::custom_object(self.gc(), *catch_class, None, catch_class.vtable())
        } else {
            // for `finally` scopes, FP just creates a normal object.
            ScriptObject::new_object(self)
        };

        self.push_stack(so);

        Ok(())
    }

    fn op_push_scope(&mut self) -> Result<(), Error<'gc>> {
        let object = self.pop_stack().null_check(self, None)?;
        self.push_scope(Scope::new(object));

        Ok(())
    }

    fn op_push_with(&mut self) -> Result<(), Error<'gc>> {
        let object = self.pop_stack().null_check(self, None)?;
        self.push_scope(Scope::new_with(object));

        Ok(())
    }

    fn op_pop_scope(&mut self) -> Result<(), Error<'gc>> {
        self.pop_scope();

        Ok(())
    }

    fn op_get_outer_scope(&mut self, index: usize) -> Result<(), Error<'gc>> {
        // Verifier ensures that this points to a valid outer scope

        let scope = self.outer.get_unchecked(index);

        self.push_stack(scope.values());

        Ok(())
    }

    fn op_get_scope_object(&mut self, index: usize) -> Result<(), Error<'gc>> {
        // Verifier ensures that this points to a valid local scope

        let scope = self.scope_frame()[index];

        self.push_stack(scope.values());

        Ok(())
    }

    fn op_find_def(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        // Verifier ensures that multiname is non-lazy

        avm_debug!(self.avm2(), "Resolving {:?}", *multiname);
        let (_, script) = self.domain().find_defining_script(self, &multiname)?;
        let obj = script.globals(self.context)?;
        self.push_stack(obj);
        Ok(())
    }

    fn op_find_property(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        avm_debug!(self.context.avm2, "Resolving {:?}", *multiname);

        let multiname = multiname.fill_with_runtime_params(self)?;
        let result = self
            .find_definition(&multiname)?
            .unwrap_or_else(|| self.global_scope());

        self.push_stack(result);

        Ok(())
    }

    fn op_find_prop_strict(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        avm_debug!(self.context.avm2, "Resolving {:?}", *multiname);

        let multiname = multiname.fill_with_runtime_params(self)?;
        let result = self
            .find_definition(&multiname)?
            .ok_or_else(|| make_error_1065(self, &multiname))?;

        self.push_stack(result);

        Ok(())
    }

    fn op_get_script_globals(&mut self, script: Script<'gc>) -> Result<(), Error<'gc>> {
        let globals = script.globals(self.context)?;

        self.push_stack(globals);

        Ok(())
    }

    fn op_get_descendants(&mut self, multiname: Gc<'gc, Multiname<'gc>>) -> Result<(), Error<'gc>> {
        let multiname = multiname.fill_with_runtime_params(self)?;
        let object = self.pop_stack().null_check(self, None)?;

        if let Some(descendants) = object
            .as_object()
            .and_then(|o| o.xml_descendants(self, &multiname))
        {
            self.push_stack(descendants);
        } else {
            // Even if it's an object with the "descendants" property, we won't support it.
            let class_name = object
                .instance_class(self)
                .name()
                .to_qualified_name_err_message(self.gc());

            return Err(Error::avm_error(type_error(
                self,
                &format!(
                    "Error #1016: Descendants operator (..) not supported on type {class_name}",
                ),
                1016,
            )?));
        }

        Ok(())
    }

    fn op_get_slot(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let stack_top = self.stack.stack_top();

        let object = stack_top
            .get()
            .null_check(self, None)?
            .as_object()
            .expect("Cannot get_slot on primitive");
        let value = object.get_slot(index);

        // We use `stack_top` instead of `pop_stack` and `push_stack` here
        // because it allows us to skip the extra bounds checks and stack pointer
        // changes. This is important here because getslot is one of the hottest
        // ops in most SWFs.
        stack_top.set(value);

        Ok(())
    }

    fn op_set_slot(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();
        let object = self
            .pop_stack()
            .null_check(self, None)?
            .as_object()
            .expect("Cannot set_slot on primitive");

        object.set_slot(index, value, self)?;

        Ok(())
    }

    fn op_set_slot_no_coerce(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();
        let object = self
            .pop_stack()
            .null_check(self, None)?
            .as_object()
            .expect("Cannot set_slot on primitive");

        object.set_slot_no_coerce(index, value, self.gc());

        Ok(())
    }

    fn op_set_global_slot(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        self.global_scope()
            .as_object()
            .unwrap()
            .set_slot(index, value, self)?;

        Ok(())
    }

    fn op_construct(&mut self, arg_count: u32) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let ctor = self.pop_stack();

        let object = ctor.construct(self, args)?;

        self.push_stack(object);

        Ok(())
    }

    fn op_construct_prop(
        &mut self,
        multiname: Gc<'gc, Multiname<'gc>>,
        arg_count: u32,
    ) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let multiname = multiname.fill_with_runtime_params(self)?;
        let source = self.pop_stack().null_check(self, Some(&multiname))?;

        let ctor = source.get_property(&multiname, self)?;
        let constructed_object = ctor.construct(self, args)?;

        self.push_stack(constructed_object);

        Ok(())
    }

    fn op_construct_slot(&mut self, index: u32, arg_count: u32) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let source = self
            .pop_stack()
            .null_check(self, None)?
            .as_object()
            .expect("Cannot get_slot on primitive");

        let ctor = source.get_slot(index);
        let constructed_object = ctor.construct(self, args)?;

        self.push_stack(constructed_object);

        Ok(())
    }

    fn op_construct_super(&mut self, arg_count: u32) -> Result<(), Error<'gc>> {
        let args = self.stack.get_args(arg_count as usize);
        let receiver = self.pop_stack().null_check(self, None)?;

        self.super_init(receiver, args)?;

        Ok(())
    }

    fn op_new_activation(&mut self, activation_class: Class<'gc>) -> Result<(), Error<'gc>> {
        // Create the activation object. Activation objects don't have prototypes,
        // and we can give it the Class's vtable because `Class::for_activation`
        // ensures there are only slots on the vtable.
        let instance = ScriptObject::custom_object(
            self.gc(),
            activation_class,
            None,
            activation_class.vtable(),
        );

        self.push_stack(instance);

        Ok(())
    }

    fn op_new_object(&mut self, num_args: u32) -> Result<(), Error<'gc>> {
        let object = ScriptObject::new_object(self);

        for _ in 0..num_args {
            let value = self.pop_stack();
            let name = self.pop_stack();

            object.set_dynamic_property(name.coerce_to_string(self)?, value, self.gc());
        }

        self.push_stack(object);

        Ok(())
    }

    fn op_new_function(
        &mut self,
        method: Method<'gc>,
        index: Index<AbcMethod>,
    ) -> Result<(), Error<'gc>> {
        let method_entry = self.table_method(method, index, true)?;
        let scope = self.create_scopechain();

        let new_fn = FunctionObject::from_method(self, method_entry, scope, None, None, None);

        self.push_stack(new_fn);

        Ok(())
    }

    fn op_new_class(&mut self, class: Class<'gc>) -> Result<(), Error<'gc>> {
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

        self.push_stack(new_class);

        Ok(())
    }

    fn op_apply_type(&mut self, num_types: u32) -> Result<(), Error<'gc>> {
        let args = self.pop_stack_args(num_types);
        let base = self
            .pop_stack()
            .as_object()
            .ok_or_else(|| make_error_1127(self))?;

        let applied = base.apply(self, &args)?;

        self.push_stack(applied);

        Ok(())
    }

    fn op_new_array(&mut self, num_args: u32) -> Result<(), Error<'gc>> {
        let args = self.pop_stack_args(num_args);
        let array = ArrayStorage::from_args(&args[..]);
        let array_obj = ArrayObject::from_storage(self, array);

        self.push_stack(array_obj);

        Ok(())
    }

    fn op_coerce_b(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_boolean();

        self.push_stack(value);

        Ok(())
    }

    fn op_coerce_d(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_coerce_d_swap_pop(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;
        let _ = self.pop_stack();

        self.push_stack(value);

        Ok(())
    }

    fn op_coerce_i(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_coerce_i_swap_pop(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;
        let _ = self.pop_stack();

        self.push_stack(value);

        Ok(())
    }

    fn op_coerce_o(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        let coerced = match value {
            Value::Undefined | Value::Null => Value::Null,
            _ => value,
        };

        self.push_stack(coerced);

        Ok(())
    }

    fn op_coerce_s(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        let coerced = match value {
            Value::Undefined | Value::Null => Value::Null,
            Value::String(_) => value,
            _ => value.coerce_to_string(self)?.into(),
        };

        self.push_stack(coerced);

        Ok(())
    }

    fn op_coerce_u(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_u32(self)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_coerce_u_swap_pop(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_u32(self)?;
        let _ = self.pop_stack();

        self.push_stack(value);

        Ok(())
    }

    fn op_convert_o(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().null_check(self, None)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_convert_s(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_string(self)?;

        self.push_stack(value);

        Ok(())
    }

    fn op_check_filter(&mut self) -> Result<(), Error<'gc>> {
        let xml = self.avm2().class_defs().xml;
        let xml_list = self.avm2().class_defs().xml_list;
        let value = self.pop_stack().null_check(self, None)?;

        if value.is_of_type(self, xml) || value.is_of_type(self, xml_list) {
            self.push_stack(value);
        } else {
            let class_name = value.instance_of_class_name(self);

            return Err(Error::avm_error(type_error(
                self,
                &format!("Error #1123: Filter operator not supported on type {class_name}."),
                1123,
            )?));
        }
        Ok(())
    }

    fn op_add(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let sum_value = match (value1, value2) {
            // note: with not-yet-guaranteed assumption that Integer < 1<<28, this won't overflow.
            (Value::Integer(n1), Value::Integer(n2)) => (n1 + n2).into(),
            (Value::Number(n1), Value::Number(n2)) => (n1 + n2).into(),
            (Value::String(s), value2) => Value::String(AvmString::concat(
                self.gc(),
                s,
                value2.coerce_to_string(self)?,
            )),
            (value1, Value::String(s)) => Value::String(AvmString::concat(
                self.gc(),
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
                        self.gc(),
                        s,
                        value2.coerce_to_string(self)?,
                    )),
                    (value1, Value::String(s)) => Value::String(AvmString::concat(
                        self.gc(),
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

        Ok(())
    }

    fn op_add_i(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1.wrapping_add(value2));

        Ok(())
    }

    fn op_bitand(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1 & value2);

        Ok(())
    }

    fn op_bitnot(&mut self) -> Result<(), Error<'gc>> {
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(!value1);

        Ok(())
    }

    fn op_bitor(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1 | value2);

        Ok(())
    }

    fn op_bitxor(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1 ^ value2);

        Ok(())
    }

    fn op_declocal(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.local_register(index).coerce_to_number(self)?;

        self.set_local_register(index, value - 1.0);

        Ok(())
    }

    fn op_declocal_i(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.local_register(index).coerce_to_i32(self)?;

        self.set_local_register(index, value.wrapping_sub(1));

        Ok(())
    }

    fn op_decrement(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(value - 1.0);

        Ok(())
    }

    fn op_decrement_i(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value.wrapping_sub(1));

        Ok(())
    }

    fn op_divide(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_number(self)?;
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(value1 / value2);

        Ok(())
    }

    fn op_inclocal(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.local_register(index).coerce_to_number(self)?;

        self.set_local_register(index, value + 1.0);

        Ok(())
    }

    fn op_inclocal_i(&mut self, index: u32) -> Result<(), Error<'gc>> {
        let value = self.local_register(index).coerce_to_i32(self)?;

        self.set_local_register(index, value.wrapping_add(1));

        Ok(())
    }

    fn op_increment(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(value + 1.0);

        Ok(())
    }

    fn op_increment_i(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value.wrapping_add(1));

        Ok(())
    }

    fn op_lshift(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_u32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1 << (value2 & 0x1F));

        Ok(())
    }

    fn op_modulo(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_number(self)?;
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(value1 % value2);

        Ok(())
    }

    fn op_multiply(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_number(self)?;
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(value1 * value2);

        Ok(())
    }

    fn op_multiply_i(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1.wrapping_mul(value2));

        Ok(())
    }

    fn op_negate(&mut self) -> Result<(), Error<'gc>> {
        let value1 = self.pop_stack().coerce_to_number(self)?;

        self.push_stack(-value1);

        Ok(())
    }

    fn op_negate_i(&mut self) -> Result<(), Error<'gc>> {
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1.wrapping_neg());

        Ok(())
    }

    fn op_rshift(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_u32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1 >> (value2 & 0x1F));

        Ok(())
    }

    fn op_subtract(&mut self) -> Result<(), Error<'gc>> {
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

        Ok(())
    }

    fn op_subtract_i(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_i32(self)?;
        let value1 = self.pop_stack().coerce_to_i32(self)?;

        self.push_stack(value1.wrapping_sub(value2));

        Ok(())
    }

    fn op_swap(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        self.push_stack(value2);
        self.push_stack(value1);

        Ok(())
    }

    fn op_urshift(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack().coerce_to_u32(self)?;
        let value1 = self.pop_stack().coerce_to_u32(self)?;

        self.push_stack(value1 >> (value2 & 0x1F));

        Ok(())
    }

    fn check_if_true(&mut self) -> bool {
        let value = self.pop_stack();

        value.coerce_to_boolean()
    }

    fn op_strict_equals(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();
        self.push_stack(value1.strict_eq(&value2));

        Ok(())
    }

    fn op_equals(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1.abstract_eq(&value2, self)?;

        self.push_stack(result);

        Ok(())
    }

    fn op_greater_equals(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = !value1.abstract_lt(&value2, self)?.unwrap_or(true);

        self.push_stack(result);

        Ok(())
    }

    fn op_greater_than(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value2.abstract_lt(&value1, self)?.unwrap_or(false);

        self.push_stack(result);

        Ok(())
    }

    fn op_less_equals(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = !value2.abstract_lt(&value1, self)?.unwrap_or(true);

        self.push_stack(result);

        Ok(())
    }

    fn op_less_than(&mut self) -> Result<(), Error<'gc>> {
        let value2 = self.pop_stack();
        let value1 = self.pop_stack();

        let result = value1.abstract_lt(&value2, self)?.unwrap_or(false);

        self.push_stack(result);

        Ok(())
    }

    fn op_not(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack().coerce_to_boolean();

        self.push_stack(!value);

        Ok(())
    }

    fn op_has_next(&mut self) -> Result<(), Error<'gc>> {
        let cur_index = self.pop_stack().coerce_to_i32(self)?;
        let value = self.pop_stack();

        if cur_index < 0 {
            self.push_stack(0);

            return Ok(());
        }

        let object = match value {
            Value::Undefined | Value::Null => None,
            Value::Object(obj) => Some(obj),
            _ => value.proto(self),
        };

        if let Some(object) = object {
            let next_index = object.get_next_enumerant(cur_index as u32, self)?;

            self.push_stack(next_index);
        } else {
            self.push_stack(0);
        }

        Ok(())
    }
    fn op_has_next_2(
        &mut self,
        object_register: u32,
        index_register: u32,
    ) -> Result<(), Error<'gc>> {
        let cur_index = self.local_register(index_register).coerce_to_i32(self)?;

        if cur_index < 0 {
            self.push_stack(false);

            return Ok(());
        }

        let mut cur_index = cur_index as u32;

        let object_reg = self.local_register(object_register);
        let mut result_value = object_reg;
        let mut object = None;

        match object_reg {
            Value::Undefined | Value::Null => {
                cur_index = 0;
            }
            Value::Object(obj) => {
                object = obj.proto();
                cur_index = obj.get_next_enumerant(cur_index, self)?;
            }
            value => {
                let proto = value.proto(self);
                if let Some(proto) = proto {
                    object = proto.proto();
                    cur_index = proto.get_next_enumerant(cur_index, self)?;
                }
            }
        };

        while let (Some(cur_object), 0) = (object, cur_index) {
            cur_index = cur_object.get_next_enumerant(cur_index, self)?;
            result_value = cur_object.into();

            object = cur_object.proto();
        }

        if cur_index == 0 {
            result_value = Value::Null;
        }

        self.push_stack(cur_index != 0);
        self.set_local_register(index_register, cur_index);
        self.set_local_register(object_register, result_value);

        Ok(())
    }

    fn op_next_name(&mut self) -> Result<(), Error<'gc>> {
        let cur_index = self.pop_stack().coerce_to_i32(self)?;
        let value = self.pop_stack();

        if cur_index <= 0 {
            self.push_stack(Value::Null);

            return Ok(());
        }

        let object = match value.null_check(self, None)? {
            Value::Object(obj) => obj,
            value => value
                .proto(self)
                .expect("Primitives always have a prototype"),
        };

        let name = object.get_enumerant_name(cur_index as u32, self)?;
        self.push_stack(name);

        Ok(())
    }

    fn op_next_value(&mut self) -> Result<(), Error<'gc>> {
        let cur_index = self.pop_stack().coerce_to_i32(self)?;
        let value = self.pop_stack();

        if cur_index <= 0 {
            self.push_stack(Value::Undefined);

            return Ok(());
        }

        let object = match value.null_check(self, None)? {
            Value::Object(obj) => obj,
            value => value
                .proto(self)
                .expect("Primitives always have a prototype"),
        };

        let value = object.get_enumerant_value(cur_index as u32, self)?;
        self.push_stack(value);

        Ok(())
    }

    fn op_is_type(&mut self, class: Class<'gc>) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        let is_instance_of = value.is_of_type(self, class);
        self.push_stack(is_instance_of);

        Ok(())
    }

    fn op_is_type_late(&mut self) -> Result<(), Error<'gc>> {
        let Some(type_object) = self
            .pop_stack()
            .as_object()
            .and_then(|o| o.as_class_object())
        else {
            return Err(Error::avm_error(type_error(
                self,
                "Error #1041: The right-hand side of operator must be a class.",
                1041,
            )?));
        };
        let value = self.pop_stack();

        let is_instance_of = value.is_of_type(self, type_object.inner_class_definition());
        self.push_stack(is_instance_of);

        Ok(())
    }

    fn op_as_type(&mut self, class: Class<'gc>) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        if value.is_of_type(self, class) {
            self.push_stack(value);
        } else {
            self.push_stack(Value::Null);
        }

        Ok(())
    }

    fn op_as_type_late(&mut self) -> Result<(), Error<'gc>> {
        let class = self.pop_stack();

        if matches!(class, Value::Undefined) {
            return Err(make_null_or_undefined_error(self, class, None));
        }

        if let Some(class) = class.as_object() {
            let Some(class) = class.as_class_object() else {
                return Err(Error::avm_error(type_error(
                    self,
                    "Error #1041: The right-hand side of operator must be a class.",
                    1041,
                )?));
            };
            let value = self.pop_stack();

            if value.is_of_type(self, class.inner_class_definition()) {
                self.push_stack(value);
            } else {
                self.push_stack(Value::Null);
            }

            Ok(())
        } else {
            // Primitive values and null both throw this error
            Err(make_null_or_undefined_error(self, Value::Null, None))
        }
    }

    fn op_instance_of(&mut self) -> Result<(), Error<'gc>> {
        let Some(type_object) = self.pop_stack().as_object() else {
            return Err(Error::avm_error(type_error(
                self,
                "Error #1040: The right-hand side of instanceof must be a class or function.",
                1040,
            )?));
        };

        if type_object.as_class_object().is_none() && type_object.as_function_object().is_none() {
            return Err(Error::avm_error(type_error(
                self,
                "Error #1040: The right-hand side of instanceof must be a class or function.",
                1040,
            )?));
        };

        let value = self.pop_stack();

        match value {
            Value::Undefined => return Err(make_null_or_undefined_error(self, value, None)),
            Value::Null => self.push_stack(false),
            value => {
                let is_instance_of = value.is_instance_of(self, type_object);

                self.push_stack(is_instance_of);
            }
        }

        Ok(())
    }

    fn op_type_of(&mut self) -> Result<(), Error<'gc>> {
        let value = self.pop_stack();

        let type_name = match value {
            Value::Undefined => istr!(self, "undefined"),
            Value::Null => istr!(self, "object"),
            Value::Bool(_) => istr!(self, "boolean"),
            Value::Number(_) | Value::Integer(_) => istr!(self, "number"),
            Value::Object(o) => {
                let classes = self.avm2().class_defs();

                match o {
                    Object::FunctionObject(_) => {
                        if o.instance_class() == classes.function {
                            istr!(self, "function")
                        } else {
                            // Subclasses always have a typeof = "object"
                            istr!(self, "object")
                        }
                    }
                    Object::XmlObject(_) | Object::XmlListObject(_) => {
                        if o.instance_class() == classes.xml_list
                            || o.instance_class() == classes.xml
                        {
                            istr!(self, "xml")
                        } else {
                            // Subclasses always have a typeof = "object"
                            istr!(self, "object")
                        }
                    }
                    _ => istr!(self, "object"),
                }
            }
            Value::String(_) => istr!(self, "string"),
        };

        self.push_stack(Value::String(type_name));

        Ok(())
    }

    /// Implements `Op::Dxns`
    fn op_dxns(&mut self) -> Result<(), Error<'gc>> {
        Err("Unimplemented opcode Dxns.".into())
    }

    /// Implements `Op::DxnsLate`
    fn op_dxns_late(&mut self) -> Result<(), Error<'gc>> {
        let _ = self.pop_stack();
        Err("Unimplemented opcode DxnsLate.".into())
    }

    /// Implements `Op::EscXAttr`
    fn op_esc_xattr(&mut self) -> Result<(), Error<'gc>> {
        let s = self.pop_stack().coerce_to_string(self)?;

        // Implementation of `EscapeAttributeValue` from ECMA-357(10.2.1.2)
        let r = escape_attribute_value(s);
        self.push_stack(AvmString::new(self.gc(), r));

        Ok(())
    }

    /// Implements `Op::EscXElem`
    fn op_esc_elem(&mut self) -> Result<(), Error<'gc>> {
        let r = match self.pop_stack() {
            // We explicitly call toXMLString on Xml/XmlListObject since the toString of these objects have special handling for simple content, which is not used here.
            Value::Object(Object::XmlObject(x)) => x.as_xml_string(self),
            Value::Object(Object::XmlListObject(x)) => x.as_xml_string(self),
            // contrary to the avmplus documentation, this escapes the value on the top of the stack using EscapeElementValue from ECMA-357 *NOT* EscapeAttributeValue.
            x => AvmString::new(self.gc(), escape_element_value(x.coerce_to_string(self)?)),
        };

        self.push_stack(r);

        Ok(())
    }

    /// Implements `Op::LookupSwitch`
    fn lookup_switch(&mut self, lookup_switch: &LookupSwitch) -> usize {
        let index = self.pop_stack().as_i32();

        let default_offset = lookup_switch.default_offset;
        let case_offsets = &lookup_switch.case_offsets;

        case_offsets
            .get(index as usize)
            .copied()
            .unwrap_or(default_offset)
    }

    /// Implements `Op::Coerce`
    fn op_coerce(&mut self, class: Class<'gc>) -> Result<(), Error<'gc>> {
        let val = self.pop_stack();
        let x = val.coerce_to_type(self, class)?;

        self.push_stack(x);
        Ok(())
    }

    fn op_coerce_swap_pop(&mut self, class: Class<'gc>) -> Result<(), Error<'gc>> {
        let val = self.pop_stack();
        let _ = self.pop_stack();

        let x = val.coerce_to_type(self, class)?;

        self.push_stack(x);
        Ok(())
    }

    pub fn domain(&self) -> Domain<'gc> {
        self.outer.domain()
    }

    fn domain_memory(&self) -> ByteArrayObject<'gc> {
        self.outer.domain().domain_memory()
    }

    /// Implements `Op::Si8`
    fn op_si8(&mut self) -> Result<(), Error<'gc>> {
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

        Ok(())
    }

    /// Implements `Op::Si16`
    fn op_si16(&mut self) -> Result<(), Error<'gc>> {
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

        Ok(())
    }

    /// Implements `Op::Si32`
    fn op_si32(&mut self) -> Result<(), Error<'gc>> {
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

        Ok(())
    }

    /// Implements `Op::Sf32`
    fn op_sf32(&mut self) -> Result<(), Error<'gc>> {
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

        Ok(())
    }

    /// Implements `Op::Sf64`
    fn op_sf64(&mut self) -> Result<(), Error<'gc>> {
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

        Ok(())
    }

    /// Implements `Op::Li8`
    fn op_li8(&mut self) -> Result<(), Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        let val = dm.get(address);

        if let Some(val) = val {
            self.push_stack(val);
        } else {
            return Err(make_error_1506(self));
        }

        Ok(())
    }

    /// Implements `Op::Li16`
    fn op_li16(&mut self) -> Result<(), Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 2 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(2, address).map_err(|e| e.to_avm(self))?;
        self.push_stack(u16::from_le_bytes(val.try_into().unwrap()));

        Ok(())
    }

    /// Implements `Op::Li32`
    fn op_li32(&mut self) -> Result<(), Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 4 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(4, address).map_err(|e| e.to_avm(self))?;
        self.push_stack(i32::from_le_bytes(val.try_into().unwrap()));
        Ok(())
    }

    /// Implements `Op::Lf32`
    fn op_lf32(&mut self) -> Result<(), Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 4 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(4, address).map_err(|e| e.to_avm(self))?;
        self.push_stack(f32::from_le_bytes(val.try_into().unwrap()));

        Ok(())
    }

    /// Implements `Op::Lf64`
    fn op_lf64(&mut self) -> Result<(), Error<'gc>> {
        let address = self.pop_stack().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm.as_bytearray().expect("Bytearray storage should exist");

        if address > dm.len() - 8 {
            return Err(make_error_1506(self));
        }

        let val = dm.read_at(8, address).map_err(|e| e.to_avm(self))?;
        self.push_stack(f64::from_le_bytes(val.try_into().unwrap()));
        Ok(())
    }

    /// Implements `Op::Sxi1`
    fn op_sxi1(&mut self) -> Result<(), Error<'gc>> {
        let val = self.pop_stack().coerce_to_i32(self)?;

        let val = val.wrapping_shl(31).wrapping_shr(31);

        self.push_stack(Value::Integer(val));

        Ok(())
    }

    /// Implements `Op::Sxi8`
    fn op_sxi8(&mut self) -> Result<(), Error<'gc>> {
        let val = self.pop_stack().coerce_to_i32(self)?;

        let val = (val.wrapping_shl(23).wrapping_shr(23) & 0xFF) as i8 as i32;

        self.push_stack(Value::Integer(val));

        Ok(())
    }

    /// Implements `Op::Sxi16`
    fn op_sxi16(&mut self) -> Result<(), Error<'gc>> {
        let val = self.pop_stack().coerce_to_i32(self)?;

        let val = (val.wrapping_shl(15).wrapping_shr(15) & 0xFFFF) as i16 as i32;

        self.push_stack(Value::Integer(val));

        Ok(())
    }

    #[cfg(feature = "avm_debug")]
    fn op_debug(
        &mut self,
        is_local_register: bool,
        register_name: AvmAtom<'gc>,
        register: u8,
    ) -> Result<(), Error<'gc>> {
        if is_local_register {
            if (register as usize) < self.num_locals {
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

        Ok(())
    }

    #[cfg(not(feature = "avm_debug"))]
    fn op_debug(
        &mut self,
        _is_local_register: bool,
        _register_name: AvmAtom<'gc>,
        _register: u8,
    ) -> Result<(), Error<'gc>> {
        Ok(())
    }

    #[cfg(feature = "avm_debug")]
    fn op_debug_file(&mut self, file_name: AvmAtom<'gc>) -> Result<(), Error<'gc>> {
        avm_debug!(self.avm2(), "File: {file_name}");

        Ok(())
    }

    #[cfg(not(feature = "avm_debug"))]
    fn op_debug_file(&mut self, _file_name: AvmAtom<'gc>) -> Result<(), Error<'gc>> {
        Ok(())
    }

    fn op_debug_line(&mut self, line_num: u32) -> Result<(), Error<'gc>> {
        avm_debug!(self.avm2(), "Line: {line_num}");

        Ok(())
    }

    fn op_bkpt(&mut self) -> Result<(), Error<'gc>> {
        // while a debugger is not attached, this is a no-op
        Ok(())
    }

    fn op_bkpt_line(&mut self, _line_num: u32) -> Result<(), Error<'gc>> {
        // while a debugger is not attached, this is a no-op
        Ok(())
    }

    fn op_timestamp(&mut self) -> Result<(), Error<'gc>> {
        // while a debugger is not attached, this is a no-op
        Ok(())
    }

    fn op_throw(&mut self) -> Result<(), Error<'gc>> {
        let error_val = self.pop_stack();
        Err(Error::avm_error(error_val))
    }
}
