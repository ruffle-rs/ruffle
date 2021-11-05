//! Activation frames

use crate::avm2::array::ArrayStorage;
use crate::avm2::class::Class;
use crate::avm2::domain::Domain;
use crate::avm2::method::{BytecodeMethod, Method, ParamConfig};
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::object::{
    ArrayObject, ByteArrayObject, ClassObject, FunctionObject, NamespaceObject, ScriptObject,
};
use crate::avm2::object::{Object, TObject};
use crate::avm2::scope::{Scope, ScopeChain, ScopeStack};
use crate::avm2::script::Script;
use crate::avm2::value::Value;
use crate::avm2::{value, Avm2, Error};
use crate::context::UpdateContext;
use crate::string::AvmString;
use crate::swf::extensions::ReadSwfExt;
use gc_arena::{Gc, GcCell, MutationContext};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cmp::{min, Ordering};
use swf::avm2::read::Reader;
use swf::avm2::types::{
    Class as AbcClass, Index, Method as AbcMethod, Multiname as AbcMultiname,
    Namespace as AbcNamespace, Op,
};

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

#[derive(Debug, Clone)]
enum FrameControl<'gc> {
    Continue,
    Return(Value<'gc>),
}

/// Represents a single activation of a given AVM2 function or keyframe.
pub struct Activation<'a, 'gc: 'a, 'gc_context: 'a> {
    /// The immutable value of `this`.
    #[allow(dead_code)]
    this: Option<Object<'gc>>,

    /// The arguments this function was called by.
    #[allow(dead_code)]
    arguments: Option<Object<'gc>>,

    /// Flags that the current activation frame is being executed and has a
    /// reader object copied from it. Taking out two readers on the same
    /// activation frame is a programming error.
    is_executing: bool,

    /// Amount of actions performed since the last timeout check
    actions_since_timeout_check: u16,

    /// Local registers.
    ///
    /// All activations have local registers, but it is possible for multiple
    /// activations (such as a rescope) to execute from the same register set.
    local_registers: GcCell<'gc, RegisterSet<'gc>>,

    /// What was returned from the function.
    ///
    /// A return value of `None` indicates that the called function is still
    /// executing. Functions that do not return instead return `Undefined`.
    #[allow(dead_code)]
    return_value: Option<Value<'gc>>,

    /// The current scope stack.
    scope_stack: ScopeStack<'gc>,

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
    caller_domain: Domain<'gc>,

    /// The class that yielded the currently executing method.
    ///
    /// This is used to maintain continuity when multiple methods supercall
    /// into one another. For example, if a class method supercalls a
    /// grandparent class's method, then this value will be the grandparent's
    /// class object. Then, if we supercall again, we look up supermethods from
    /// the great-grandparent class, preventing us from accidentally calling
    /// the same method again.
    ///
    /// This will not be available outside of method, setter, or getter calls.
    subclass_object: Option<ClassObject<'gc>>,

    /// The class of all objects returned from `newactivation`.
    ///
    /// In method calls that call for an activation object, this will be
    /// configured as the anonymous class whose traits match the method's
    /// declared traits.
    ///
    /// If this is `None`, then the method did not ask for an activation object
    /// and we will not allocate a class for one.
    activation_class: Option<ClassObject<'gc>>,

    pub context: UpdateContext<'a, 'gc, 'gc_context>,
}

impl<'a, 'gc, 'gc_context> Activation<'a, 'gc, 'gc_context> {
    /// Construct an activation that does not represent any particular scope.
    ///
    /// This exists primarily for non-AVM2 related manipulations of the
    /// interpreter environment that require an activation. For example,
    /// loading traits into an object, or running tests.
    ///
    /// It is a logic error to attempt to run AVM2 code in a nothing
    /// `Activation`.
    pub fn from_nothing(context: UpdateContext<'a, 'gc, 'gc_context>) -> Self {
        let local_registers = GcCell::allocate(context.gc_context, RegisterSet::new(0));

        Self {
            this: None,
            arguments: None,
            is_executing: false,
            actions_since_timeout_check: 0,
            local_registers,
            return_value: None,
            scope_stack: ScopeStack::new(),
            outer: ScopeChain::new(context.avm2.globals),
            caller_domain: context.avm2.globals,
            subclass_object: None,
            activation_class: None,
            context,
        }
    }

    /// Construct an activation for the execution of a particular script's
    /// initializer method.
    pub fn from_script(
        context: UpdateContext<'a, 'gc, 'gc_context>,
        script: Script<'gc>,
    ) -> Result<Self, Error> {
        let (method, global_object, domain) = script.init();

        let num_locals = match method {
            Method::Native { .. } => 0,
            Method::Bytecode(bytecode) => {
                let body: Result<_, Error> = bytecode.body().ok_or_else(|| {
                    "Cannot execute non-native method (for script) without body".into()
                });
                body?.num_locals
            }
        };
        let local_registers =
            GcCell::allocate(context.gc_context, RegisterSet::new(num_locals + 1));

        *local_registers
            .write(context.gc_context)
            .get_mut(0)
            .unwrap() = global_object.into();

        Ok(Self {
            this: Some(global_object),
            arguments: None,
            is_executing: false,
            actions_since_timeout_check: 0,
            local_registers,
            return_value: None,
            scope_stack: ScopeStack::new(),
            outer: ScopeChain::new(domain),
            caller_domain: domain,
            subclass_object: None,
            activation_class: None,
            context,
        })
    }

    /// Finds an object on either the current or outer scope of this activation by definition.
    pub fn find_definition(&mut self, name: &Multiname<'gc>) -> Result<Option<Object<'gc>>, Error> {
        let outer_scope = self.outer;

        if let Some(obj) = self.scope_stack.find(name, outer_scope.is_empty())? {
            Ok(Some(obj))
        } else if let Some(obj) = outer_scope.find(name, self)? {
            Ok(Some(obj))
        } else {
            Ok(None)
        }
    }

    /// Resolves a definition using either the current or outer scope of this activation.
    pub fn resolve_definition(
        &mut self,
        name: &Multiname<'gc>,
    ) -> Result<Option<Value<'gc>>, Error> {
        let outer_scope = self.outer;

        if let Some(obj) = self.scope_stack.find(name, outer_scope.is_empty())? {
            Ok(Some(obj.get_property(obj, name, self)?))
        } else if let Some(result) = outer_scope.resolve(name, self)? {
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Resolve a type name to a class.
    ///
    /// This returns an error if a type is named but does not exist; or if the
    /// typed named is not a class object.
    fn resolve_type(
        &mut self,
        type_name: Multiname<'gc>,
    ) -> Result<Option<ClassObject<'gc>>, Error> {
        if type_name.is_any() {
            return Ok(None);
        }

        let class = self
            .resolve_definition(&type_name)?
            .ok_or_else(|| format!("Could not resolve parameter type {:?}", type_name))?
            .coerce_to_object(self)?;

        let class = class
            .as_class_object()
            .ok_or_else(|| format!("Resolved parameter type {:?} is not a class", type_name))?;

        // Type parameters should specialize the returned class.
        // Unresolvable parameter types are treated as Any, which is treated as
        // Object.
        if !type_name.params().is_empty() {
            let mut param_types = Vec::with_capacity(type_name.params().len());

            for param in type_name.params() {
                param_types.push(match self.resolve_type(param.clone())? {
                    Some(o) => Value::Object(o.into()),
                    None => Value::Null,
                });
            }

            return Ok(Some(class.apply(self, &param_types[..])?));
        }

        Ok(Some(class))
    }

    /// Resolve a single parameter value.
    ///
    /// Given an individual parameter value and the associated parameter's
    /// configuration, return what value should be stored in the called
    /// function's local registers (or an error, if the parameter violates the
    /// signature of the current called method).
    fn resolve_parameter(
        &mut self,
        method_name: &str,
        value: Option<&Value<'gc>>,
        param_config: &ParamConfig<'gc>,
        index: usize,
    ) -> Result<Value<'gc>, Error> {
        let arg = if let Some(value) = value {
            Cow::Borrowed(value)
        } else if let Some(default) = &param_config.default_value {
            Cow::Borrowed(default)
        } else if param_config.param_type_name.is_any() {
            return Ok(Value::Undefined);
        } else {
            return Err(format!(
                "Param {} (index {}) was missing when calling {}",
                param_config.param_name, index, method_name
            )
            .into());
        };

        let type_name = param_config.param_type_name.clone();
        let param_type = self.resolve_type(type_name)?;

        if let Some(param_type) = param_type {
            arg.coerce_to_type(self, param_type)
        } else {
            Ok(arg.into_owned())
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
        method_name: &str,
        user_arguments: &[Value<'gc>],
        signature: &[ParamConfig<'gc>],
    ) -> Result<Vec<Value<'gc>>, Error> {
        let mut arguments_list = Vec::new();
        for (i, (arg, param_config)) in user_arguments.iter().zip(signature.iter()).enumerate() {
            arguments_list.push(self.resolve_parameter(method_name, Some(arg), param_config, i)?);
        }

        match user_arguments.len().cmp(&signature.len()) {
            Ordering::Greater => {
                //Variadic parameters exist, just push them into the list
                arguments_list.extend_from_slice(&user_arguments[signature.len()..])
            }
            Ordering::Less => {
                //Apply remaining default parameters
                for (i, param_config) in signature[user_arguments.len()..].iter().enumerate() {
                    arguments_list.push(self.resolve_parameter(
                        method_name,
                        None,
                        param_config,
                        i + user_arguments.len(),
                    )?);
                }
            }
            _ => {}
        }

        Ok(arguments_list)
    }

    /// Construct an activation for the execution of a particular bytecode
    /// method.
    pub fn from_method(
        mut context: UpdateContext<'a, 'gc, 'gc_context>,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        outer: ScopeChain<'gc>,
        this: Option<Object<'gc>>,
        user_arguments: &[Value<'gc>],
        subclass_object: Option<ClassObject<'gc>>,
        callee: Object<'gc>,
    ) -> Result<Self, Error> {
        let body: Result<_, Error> = method
            .body()
            .ok_or_else(|| "Cannot execute non-native method without body".into());
        let body = body?;
        let num_locals = body.num_locals;
        let has_rest_or_args = method.is_variadic();
        let arg_register = if has_rest_or_args { 1 } else { 0 };

        let signature = method.signature();
        if user_arguments.len() > signature.len() && !has_rest_or_args {
            return Err(format!(
                "Attempted to call {:?} with {} arguments (more than {} is prohibited)",
                method.method_name(),
                user_arguments.len(),
                signature.len()
            )
            .into());
        }

        let num_declared_arguments = signature.len() as u32;

        let local_registers = GcCell::allocate(
            context.gc_context,
            RegisterSet::new(num_locals + num_declared_arguments + arg_register + 1),
        );

        {
            let mut write = local_registers.write(context.gc_context);
            *write.get_mut(0).unwrap() = this.map(|t| t.into()).unwrap_or(Value::Null);
        }

        let activation_class = if method.method().needs_activation {
            let translation_unit = method.translation_unit();
            let abc_method = method.method();
            let mut dummy_activation = Activation::from_nothing(context.reborrow());
            dummy_activation.set_outer(outer);
            let activation_class =
                Class::for_activation(&mut dummy_activation, translation_unit, abc_method, body)?;
            let activation_class_object =
                ClassObject::from_class(&mut dummy_activation, activation_class, None)?;

            drop(dummy_activation);

            Some(activation_class_object)
        } else {
            None
        };

        let mut activation = Self {
            this,
            arguments: None,
            is_executing: false,
            actions_since_timeout_check: 0,
            local_registers,
            return_value: None,
            scope_stack: ScopeStack::new(),
            outer,
            caller_domain: outer.domain(),
            subclass_object,
            activation_class,
            context,
        };

        //Statically verify all non-variadic, provided parameters.
        let arguments_list =
            activation.resolve_parameters(method.method_name(), user_arguments, signature)?;

        {
            let mut write = local_registers.write(activation.context.gc_context);
            for (i, arg) in arguments_list[0..min(signature.len(), arguments_list.len())]
                .iter()
                .enumerate()
            {
                *write.get_mut(1 + i as u32).unwrap() = arg.clone();
            }
        }

        if has_rest_or_args {
            let args_array = if method.method().needs_arguments_object {
                ArrayStorage::from_args(&arguments_list)
            } else if method.method().needs_rest {
                if let Some(rest_args) = arguments_list.get(signature.len()..) {
                    ArrayStorage::from_args(rest_args)
                } else {
                    ArrayStorage::new(0)
                }
            } else {
                unreachable!();
            };

            let mut args_object = ArrayObject::from_storage(&mut activation, args_array)?;

            if method.method().needs_arguments_object {
                args_object.set_property(
                    args_object,
                    &QName::new(Namespace::public(), "callee").into(),
                    callee.into(),
                    &mut activation,
                )?;
            }

            *local_registers
                .write(activation.context.gc_context)
                .get_mut(1 + num_declared_arguments)
                .unwrap() = args_object.into();
        }

        Ok(activation)
    }

    /// Construct an activation for the execution of a builtin method.
    ///
    /// It is a logic error to attempt to execute builtins within the same
    /// activation as the method or script that called them. You must use this
    /// function to construct a new activation for the builtin so that it can
    /// properly supercall.
    pub fn from_builtin(
        context: UpdateContext<'a, 'gc, 'gc_context>,
        this: Option<Object<'gc>>,
        subclass_object: Option<ClassObject<'gc>>,
        outer: ScopeChain<'gc>,
        caller_domain: Domain<'gc>,
    ) -> Result<Self, Error> {
        let local_registers = GcCell::allocate(context.gc_context, RegisterSet::new(0));

        Ok(Self {
            this,
            arguments: None,
            is_executing: false,
            actions_since_timeout_check: 0,
            local_registers,
            return_value: None,
            scope_stack: ScopeStack::new(),
            outer,
            caller_domain,
            subclass_object,
            activation_class: None,
            context,
        })
    }

    /// Execute a script initializer.
    pub fn run_stack_frame_for_script(&mut self, script: Script<'gc>) -> Result<(), Error> {
        let init = script.init().0.into_bytecode()?;

        self.run_actions(init)?;

        Ok(())
    }

    /// Call the superclass's instance initializer.
    pub fn super_init(
        &mut self,
        receiver: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error> {
        let superclass_object = self
            .subclass_object()
            .and_then(|c| c.superclass_object())
            .ok_or_else(|| {
                Error::from("Attempted to call super constructor without a superclass.")
            });
        let superclass_object = superclass_object?;

        superclass_object.call_native_init(Some(receiver), args, self)
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

    /// Retrieve a local register.
    pub fn local_register(&self, id: u32) -> Result<Value<'gc>, Error> {
        self.local_registers
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Out of bounds register read: {}", id).into())
    }

    /// Set a local register.
    ///
    /// Returns `true` if the set was successful; `false` otherwise
    pub fn set_local_register(
        &mut self,
        id: u32,
        value: impl Into<Value<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(r) = self.local_registers.write(mc).get_mut(id) {
            *r = value.into();

            Ok(())
        } else {
            Err(format!("Out of bounds register write: {}", id).into())
        }
    }

    /// Sets the outer scope of this activation
    pub fn set_outer(&mut self, new_outer: ScopeChain<'gc>) {
        self.outer = new_outer;
    }

    /// Creates a new ScopeChain by chaining the current state of this
    /// activation's scope stack with the outer scope.
    pub fn create_scopechain(&self) -> ScopeChain<'gc> {
        self.outer
            .chain(self.context.gc_context, self.scope_stack.scopes())
    }

    /// Returns the domain of the original AS3 caller.
    pub fn caller_domain(&self) -> Domain<'gc> {
        self.caller_domain
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
            .or_else(|| self.scope_stack.get(0))
            .map(|scope| scope.values())
    }

    pub fn avm2(&mut self) -> &mut Avm2<'gc> {
        self.context.avm2
    }

    /// Set the return value.
    pub fn set_return_value(&mut self, value: Value<'gc>) {
        self.return_value = Some(value);
    }

    /// Get the class that defined the currently-executing method, if it
    /// exists.
    ///
    /// If the currently-executing method is not part of an ES4 class, then
    /// this yields `None`.
    pub fn subclass_object(&self) -> Option<ClassObject<'gc>> {
        self.subclass_object
    }

    /// Retrieve a int from the current constant pool.
    fn pool_int(
        &self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<i32>,
    ) -> Result<i32, Error> {
        value::abc_int(method.translation_unit(), index)
    }

    /// Retrieve a int from the current constant pool.
    fn pool_uint(
        &self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<u32>,
    ) -> Result<u32, Error> {
        value::abc_uint(method.translation_unit(), index)
    }

    /// Retrieve a double from the current constant pool.
    fn pool_double(
        &self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<f64>,
    ) -> Result<f64, Error> {
        value::abc_double(method.translation_unit(), index)
    }

    /// Retrieve a string from the current constant pool.
    fn pool_string<'b>(
        &self,
        method: &'b BytecodeMethod<'gc>,
        index: Index<String>,
    ) -> Result<AvmString<'gc>, Error> {
        method
            .translation_unit()
            .pool_string(index.0, self.context.gc_context)
    }

    /// Retrieve a namespace from the current constant pool.
    fn pool_namespace(
        &self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcNamespace>,
    ) -> Result<Namespace<'gc>, Error> {
        Namespace::from_abc_namespace(method.translation_unit(), index, self.context.gc_context)
    }

    /// Retrieve a multiname from the current constant pool.
    fn pool_multiname(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<Multiname<'gc>, Error> {
        Multiname::from_abc_multiname(method.translation_unit(), index, self)
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as an error condition.
    fn pool_multiname_static(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<Multiname<'gc>, Error> {
        Multiname::from_abc_multiname_static(
            method.translation_unit(),
            index,
            self.context.gc_context,
        )
    }

    /// Retrieve a static, or non-runtime, multiname from the current constant
    /// pool.
    ///
    /// This version of the function treats index 0 as the any-type `*`.
    fn pool_multiname_static_any(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<Multiname<'gc>, Error> {
        if index.0 == 0 {
            Ok(Multiname::any())
        } else {
            Multiname::from_abc_multiname_static(
                method.translation_unit(),
                index,
                self.context.gc_context,
            )
        }
    }

    /// Retrieve a method entry from the current ABC file's method table.
    fn table_method(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMethod>,
        is_function: bool,
    ) -> Result<Gc<'gc, BytecodeMethod<'gc>>, Error> {
        BytecodeMethod::from_method_index(method.translation_unit(), index, is_function, self)
    }

    /// Retrieve a class entry from the current ABC file's method table.
    fn table_class(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcClass>,
    ) -> Result<GcCell<'gc, Class<'gc>>, Error> {
        method.translation_unit().load_class(index.0, self)
    }

    pub fn run_actions(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let body: Result<_, Error> = method
            .body()
            .ok_or_else(|| "Cannot execute non-native method without body".into());
        let body = body?;
        let mut reader = Reader::new(&body.code);

        loop {
            let result = self.do_next_opcode(method, &mut reader, &body.code);
            match result {
                Ok(FrameControl::Return(value)) => break Ok(value),
                Ok(FrameControl::Continue) => {}
                Err(e) => break Err(e),
            }
        }
    }

    /// Run a single action from a given action reader.
    fn do_next_opcode<'b>(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        self.actions_since_timeout_check += 1;
        if self.actions_since_timeout_check >= 2000 {
            self.actions_since_timeout_check = 0;
            if self.context.update_start.elapsed() >= self.context.max_execution_duration {
                return Err(
                    "A script in this movie has taken too long to execute and has been terminated."
                        .into(),
                );
            }
        }

        let instruction_start = reader.pos(full_data);
        let op = reader.read_op();
        if let Ok(Some(op)) = op {
            avm_debug!(self.avm2(), "Opcode: {:?}", op);

            let result = match op {
                Op::PushByte { value } => self.op_push_byte(value),
                Op::PushDouble { value } => self.op_push_double(method, value),
                Op::PushFalse => self.op_push_false(),
                Op::PushInt { value } => self.op_push_int(method, value),
                Op::PushNamespace { value } => self.op_push_namespace(method, value),
                Op::PushNaN => self.op_push_nan(),
                Op::PushNull => self.op_push_null(),
                Op::PushShort { value } => self.op_push_short(value),
                Op::PushString { value } => self.op_push_string(method, value),
                Op::PushTrue => self.op_push_true(),
                Op::PushUint { value } => self.op_push_uint(method, value),
                Op::PushUndefined => self.op_push_undefined(),
                Op::Pop => self.op_pop(),
                Op::Dup => self.op_dup(),
                Op::GetLocal { index } => self.op_get_local(index),
                Op::SetLocal { index } => self.op_set_local(index),
                Op::Kill { index } => self.op_kill(index),
                Op::Call { num_args } => self.op_call(num_args),
                Op::CallMethod { index, num_args } => self.op_call_method(index, num_args),
                Op::CallProperty { index, num_args } => {
                    self.op_call_property(method, index, num_args)
                }
                Op::CallPropLex { index, num_args } => {
                    self.op_call_prop_lex(method, index, num_args)
                }
                Op::CallPropVoid { index, num_args } => {
                    self.op_call_prop_void(method, index, num_args)
                }
                Op::CallStatic { index, num_args } => self.op_call_static(method, index, num_args),
                Op::CallSuper { index, num_args } => self.op_call_super(method, index, num_args),
                Op::CallSuperVoid { index, num_args } => {
                    self.op_call_super_void(method, index, num_args)
                }
                Op::ReturnValue => self.op_return_value(),
                Op::ReturnVoid => self.op_return_void(),
                Op::GetProperty { index } => self.op_get_property(method, index),
                Op::SetProperty { index } => self.op_set_property(method, index),
                Op::InitProperty { index } => self.op_init_property(method, index),
                Op::DeleteProperty { index } => self.op_delete_property(method, index),
                Op::GetSuper { index } => self.op_get_super(method, index),
                Op::SetSuper { index } => self.op_set_super(method, index),
                Op::In => self.op_in(),
                Op::PushScope => self.op_push_scope(),
                Op::PushWith => self.op_push_with(),
                Op::PopScope => self.op_pop_scope(),
                Op::GetOuterScope { index } => self.op_get_outer_scope(index),
                Op::GetScopeObject { index } => self.op_get_scope_object(index),
                Op::GetGlobalScope => self.op_get_global_scope(),
                Op::FindProperty { index } => self.op_find_property(method, index),
                Op::FindPropStrict { index } => self.op_find_prop_strict(method, index),
                Op::GetLex { index } => self.op_get_lex(method, index),
                Op::GetSlot { index } => self.op_get_slot(index),
                Op::SetSlot { index } => self.op_set_slot(index),
                Op::GetGlobalSlot { index } => self.op_get_global_slot(index),
                Op::SetGlobalSlot { index } => self.op_set_global_slot(index),
                Op::Construct { num_args } => self.op_construct(num_args),
                Op::ConstructProp { index, num_args } => {
                    self.op_construct_prop(method, index, num_args)
                }
                Op::ConstructSuper { num_args } => self.op_construct_super(num_args),
                Op::NewActivation => self.op_new_activation(),
                Op::NewObject { num_args } => self.op_new_object(num_args),
                Op::NewFunction { index } => self.op_new_function(method, index),
                Op::NewClass { index } => self.op_new_class(method, index),
                Op::ApplyType { num_types } => self.op_apply_type(num_types),
                Op::NewArray { num_args } => self.op_new_array(num_args),
                Op::CoerceA => self.op_coerce_a(),
                Op::CoerceB => self.op_coerce_b(),
                Op::CoerceD => self.op_coerce_d(),
                Op::CoerceI => self.op_coerce_i(),
                Op::CoerceO => self.op_coerce_o(),
                Op::CoerceS => self.op_coerce_s(),
                Op::CoerceU => self.op_coerce_u(),
                Op::ConvertB => self.op_convert_b(),
                Op::ConvertI => self.op_convert_i(),
                Op::ConvertD => self.op_convert_d(),
                Op::ConvertO => self.op_convert_o(),
                Op::ConvertU => self.op_convert_u(),
                Op::ConvertS => self.op_convert_s(),
                Op::Add => self.op_add(),
                Op::AddI => self.op_add_i(),
                Op::BitAnd => self.op_bitand(),
                Op::BitNot => self.op_bitnot(),
                Op::BitOr => self.op_bitor(),
                Op::BitXor => self.op_bitxor(),
                Op::DecLocal { index } => self.op_declocal(index),
                Op::DecLocalI { index } => self.op_declocal_i(index),
                Op::Decrement => self.op_decrement(),
                Op::DecrementI => self.op_decrement_i(),
                Op::Divide => self.op_divide(),
                Op::IncLocal { index } => self.op_inclocal(index),
                Op::IncLocalI { index } => self.op_inclocal_i(index),
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
                Op::Jump { offset } => self.op_jump(offset, reader, full_data),
                Op::IfTrue { offset } => self.op_if_true(offset, reader, full_data),
                Op::IfFalse { offset } => self.op_if_false(offset, reader, full_data),
                Op::IfStrictEq { offset } => self.op_if_strict_eq(offset, reader, full_data),
                Op::IfStrictNe { offset } => self.op_if_strict_ne(offset, reader, full_data),
                Op::IfEq { offset } => self.op_if_eq(offset, reader, full_data),
                Op::IfNe { offset } => self.op_if_ne(offset, reader, full_data),
                Op::IfGe { offset } => self.op_if_ge(offset, reader, full_data),
                Op::IfGt { offset } => self.op_if_gt(offset, reader, full_data),
                Op::IfLe { offset } => self.op_if_le(offset, reader, full_data),
                Op::IfLt { offset } => self.op_if_lt(offset, reader, full_data),
                Op::IfNge { offset } => self.op_if_nge(offset, reader, full_data),
                Op::IfNgt { offset } => self.op_if_ngt(offset, reader, full_data),
                Op::IfNle { offset } => self.op_if_nle(offset, reader, full_data),
                Op::IfNlt { offset } => self.op_if_nlt(offset, reader, full_data),
                Op::StrictEquals => self.op_strict_equals(),
                Op::Equals => self.op_equals(),
                Op::GreaterEquals => self.op_greater_equals(),
                Op::GreaterThan => self.op_greater_than(),
                Op::LessEquals => self.op_less_equals(),
                Op::LessThan => self.op_less_than(),
                Op::Nop => self.op_nop(),
                Op::Not => self.op_not(),
                Op::HasNext => self.op_has_next(),
                Op::HasNext2 {
                    object_register,
                    index_register,
                } => self.op_has_next_2(object_register, index_register),
                Op::NextName => self.op_next_name(),
                Op::NextValue => self.op_next_value(),
                Op::IsType { index } => self.op_is_type(method, index),
                Op::IsTypeLate => self.op_is_type_late(),
                Op::AsType { type_name } => self.op_as_type(method, type_name),
                Op::AsTypeLate => self.op_as_type_late(),
                Op::InstanceOf => self.op_instance_of(),
                Op::Label => Ok(FrameControl::Continue),
                Op::Debug {
                    is_local_register,
                    register_name,
                    register,
                } => self.op_debug(method, is_local_register, register_name, register),
                Op::DebugFile { file_name } => self.op_debug_file(method, file_name),
                Op::DebugLine { line_num } => self.op_debug_line(line_num),
                Op::Bkpt => self.op_bkpt(),
                Op::BkptLine { line_num } => self.op_bkpt_line(line_num),
                Op::Timestamp => self.op_timestamp(),
                Op::TypeOf => self.op_type_of(),
                Op::EscXAttr => self.op_esc_xattr(),
                Op::EscXElem => self.op_esc_elem(),
                Op::LookupSwitch {
                    default_offset,
                    case_offsets,
                } => self.op_lookup_switch(
                    default_offset,
                    &case_offsets,
                    instruction_start,
                    reader,
                    full_data,
                ),
                Op::Coerce { index } => self.op_coerce(method, index),
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
                _ => self.unknown_op(op),
            };

            if let Err(e) = result {
                log::error!("AVM2 error: {}", e);
                return Err(e);
            }
            result
        } else if let Ok(None) = op {
            log::error!("Unknown opcode!");
            Err("Unknown opcode!".into())
        } else if let Err(e) = op {
            log::error!("Parse error: {:?}", e);
            Err(e.into())
        } else {
            unreachable!();
        }
    }

    fn unknown_op(&mut self, op: swf::avm2::types::Op) -> Result<FrameControl<'gc>, Error> {
        log::error!("Unknown AVM2 opcode: {:?}", op);
        Err("Unknown op".into())
    }

    fn op_push_byte(&mut self, value: u8) -> Result<FrameControl<'gc>, Error> {
        //TODO: Adobe Animate CC appears to generate signed byte values, and
        //JPEXS appears to take them.
        self.context.avm2.push(value as i8 as i32);
        Ok(FrameControl::Continue)
    }

    fn op_push_double(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        value: Index<f64>,
    ) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(self.pool_double(method, value)?);
        Ok(FrameControl::Continue)
    }

    fn op_push_false(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(false);
        Ok(FrameControl::Continue)
    }

    fn op_push_int(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        value: Index<i32>,
    ) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(self.pool_int(method, value)?);
        Ok(FrameControl::Continue)
    }

    fn op_push_namespace(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        value: Index<AbcNamespace>,
    ) -> Result<FrameControl<'gc>, Error> {
        let ns = self.pool_namespace(method, value)?;
        let ns_object = NamespaceObject::from_namespace(self, ns)?;

        self.context.avm2.push(ns_object);
        Ok(FrameControl::Continue)
    }

    fn op_push_nan(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(f64::NAN);
        Ok(FrameControl::Continue)
    }

    fn op_push_null(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(Value::Null);
        Ok(FrameControl::Continue)
    }

    fn op_push_short(&mut self, value: i16) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(value);
        Ok(FrameControl::Continue)
    }

    fn op_push_string(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        value: Index<String>,
    ) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(self.pool_string(&method, value)?);
        Ok(FrameControl::Continue)
    }

    fn op_push_true(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(true);
        Ok(FrameControl::Continue)
    }

    fn op_push_uint(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        value: Index<u32>,
    ) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(self.pool_uint(method, value)?);
        Ok(FrameControl::Continue)
    }

    fn op_push_undefined(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(Value::Undefined);
        Ok(FrameControl::Continue)
    }

    fn op_pop(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.pop();

        Ok(FrameControl::Continue)
    }

    fn op_dup(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(
            self.context
                .avm2
                .stack
                .last()
                .cloned()
                .unwrap_or(Value::Undefined),
        );

        Ok(FrameControl::Continue)
    }

    fn op_get_local(&mut self, register_index: u32) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(self.local_register(register_index)?);
        Ok(FrameControl::Continue)
    }

    fn op_set_local(&mut self, register_index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();

        self.set_local_register(register_index, value, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_kill(&mut self, register_index: u32) -> Result<FrameControl<'gc>, Error> {
        self.set_local_register(register_index, Value::Undefined, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_call(&mut self, arg_count: u32) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let receiver = self.context.avm2.pop().coerce_to_object(self).ok();
        let function = self.context.avm2.pop().coerce_to_object(self)?;
        let value = function.call(receiver, &args, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_method(
        &mut self,
        index: Index<AbcMethod>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;

        let value = receiver.call_method(index.0, &args, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_property(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let multiname = self.pool_multiname(method, index)?;
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;

        let value = receiver.call_property(&multiname, &args, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_prop_lex(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let multiname = self.pool_multiname(method, index)?;
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;
        let function = receiver
            .get_property(receiver, &multiname, self)?
            .coerce_to_object(self)?;
        let value = function.call(None, &args, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_prop_void(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let multiname = self.pool_multiname(method, index)?;
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;

        receiver.call_property(&multiname, &args, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_call_static(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMethod>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;
        let method = self.table_method(method, index, false)?;
        // TODO: What scope should the function be executed with?
        let scope = self.create_scopechain();
        let function = FunctionObject::from_method(self, method.into(), scope, None, None);
        let value = function.call(Some(receiver), &args, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_super(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let multiname = self.pool_multiname(method, index)?;
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;

        let superclass_object = self
            .subclass_object()
            .and_then(|bc| bc.superclass_object())
            .ok_or_else(|| Error::from("Attempted to call super method without a superclass."))?;

        let value = superclass_object.call_super(&multiname, receiver, &args, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_call_super_void(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let multiname = self.pool_multiname(method, index)?;
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;

        let superclass_object = self
            .subclass_object()
            .and_then(|bc| bc.superclass_object())
            .ok_or_else(|| Error::from("Attempted to call super method without a superclass."))?;

        superclass_object.call_super(&multiname, receiver, &args, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_return_value(&mut self) -> Result<FrameControl<'gc>, Error> {
        let return_value = self.context.avm2.pop();

        Ok(FrameControl::Return(return_value))
    }

    fn op_return_void(&mut self) -> Result<FrameControl<'gc>, Error> {
        Ok(FrameControl::Return(Value::Undefined))
    }

    fn op_get_property(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let txunit = method.translation_unit();
        let abc = txunit.abc();
        let abc_multiname = Multiname::resolve_multiname_index(&abc, index.clone())?;
        let (multiname, object) = if matches!(
            abc_multiname,
            AbcMultiname::MultinameL { .. } | AbcMultiname::MultinameLA { .. }
        ) {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.context.avm2.pop();
            let object = self.context.avm2.pop().coerce_to_object(self)?;
            if !name_value.is_primitive() {
                if let Some(dictionary) = object.as_dictionary_object() {
                    let value =
                        dictionary.get_property_by_object(name_value.coerce_to_object(self)?);
                    self.context.avm2.push(value);

                    return Ok(FrameControl::Continue);
                }
            }

            (
                Multiname::from_multiname_late(txunit, abc_multiname, name_value, self)?,
                object,
            )
        } else {
            let multiname = self.pool_multiname(method, index)?;
            let object = self.context.avm2.pop().coerce_to_object(self)?;

            (multiname, object)
        };

        let value = object.get_property(object, &multiname, self)?;
        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_property(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();
        let txunit = method.translation_unit();
        let abc = txunit.abc();
        let abc_multiname = Multiname::resolve_multiname_index(&abc, index.clone())?;
        let (multiname, mut object) = if matches!(
            abc_multiname,
            AbcMultiname::MultinameL { .. } | AbcMultiname::MultinameLA { .. }
        ) {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.context.avm2.pop();
            let object = self.context.avm2.pop().coerce_to_object(self)?;
            if !name_value.is_primitive() {
                if let Some(dictionary) = object.as_dictionary_object() {
                    dictionary.set_property_by_object(
                        name_value.coerce_to_object(self)?,
                        value,
                        self.context.gc_context,
                    );

                    return Ok(FrameControl::Continue);
                }
            }

            (
                Multiname::from_multiname_late(txunit, abc_multiname, name_value, self)?,
                object,
            )
        } else {
            let multiname = self.pool_multiname(method, index)?;
            let object = self.context.avm2.pop().coerce_to_object(self)?;

            (multiname, object)
        };

        object.set_property(object, &multiname, value, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_init_property(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();
        let multiname = self.pool_multiname(method, index)?;
        let mut object = self.context.avm2.pop().coerce_to_object(self)?;

        object.init_property(object, &multiname, value, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_delete_property(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let txunit = method.translation_unit();
        let abc = txunit.abc();
        let abc_multiname = Multiname::resolve_multiname_index(&abc, index.clone())?;
        let (multiname, object) = if matches!(
            abc_multiname,
            AbcMultiname::MultinameL { .. } | AbcMultiname::MultinameLA { .. }
        ) {
            // `MultinameL` is the only form of multiname that allows fast-path
            // or alternate-path lookups based on the local name *value*,
            // rather than it's string representation.

            let name_value = self.context.avm2.pop();
            let object = self.context.avm2.pop().coerce_to_object(self)?;
            if !name_value.is_primitive() {
                if let Some(dictionary) = object.as_dictionary_object() {
                    dictionary.delete_property_by_object(
                        name_value.coerce_to_object(self)?,
                        self.context.gc_context,
                    );

                    self.context.avm2.push(true);
                    return Ok(FrameControl::Continue);
                }
            }

            (
                Multiname::from_multiname_late(txunit, abc_multiname, name_value, self)?,
                object,
            )
        } else {
            let multiname = self.pool_multiname(method, index)?;
            let object = self.context.avm2.pop().coerce_to_object(self)?;

            (multiname, object)
        };

        let did_delete = object.delete_property(self, &multiname)?;

        self.context.avm2.push(did_delete);

        Ok(FrameControl::Continue)
    }

    fn op_get_super(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let multiname = self.pool_multiname(method, index)?;
        let object = self.context.avm2.pop().coerce_to_object(self)?;

        let superclass_object = self
            .subclass_object()
            .and_then(|bc| bc.superclass_object())
            .ok_or_else(|| Error::from("Attempted to call super method without a superclass."))?;

        let value = superclass_object.get_super(&multiname, object, self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_super(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();
        let multiname = self.pool_multiname(method, index)?;
        let object = self.context.avm2.pop().coerce_to_object(self)?;

        let superclass_object = self
            .subclass_object()
            .and_then(|bc| bc.superclass_object())
            .ok_or_else(|| Error::from("Attempted to call super method without a superclass."))?;

        superclass_object.set_super(&multiname, value, object, self)?;

        Ok(FrameControl::Continue)
    }

    fn op_in(&mut self) -> Result<FrameControl<'gc>, Error> {
        let obj = self.context.avm2.pop().coerce_to_object(self)?;
        let name_value = self.context.avm2.pop();

        if let Some(dictionary) = obj.as_dictionary_object() {
            if !name_value.is_primitive() {
                let obj_key = name_value.coerce_to_object(self)?;
                self.context
                    .avm2
                    .push(dictionary.has_property_by_object(obj_key));

                return Ok(FrameControl::Continue);
            }
        }

        let name = name_value.coerce_to_string(self)?;
        let qname = QName::new(Namespace::public(), name);
        let has_prop = obj.has_property_via_in(self, &qname)?;

        self.context.avm2.push(has_prop);

        Ok(FrameControl::Continue)
    }

    fn op_push_scope(&mut self) -> Result<FrameControl<'gc>, Error> {
        let object = self.context.avm2.pop().coerce_to_object(self)?;
        self.scope_stack.push(Scope::new(object));

        Ok(FrameControl::Continue)
    }

    fn op_push_with(&mut self) -> Result<FrameControl<'gc>, Error> {
        let object = self.context.avm2.pop().coerce_to_object(self)?;
        self.scope_stack.push(Scope::new_with(object));

        Ok(FrameControl::Continue)
    }

    fn op_pop_scope(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.scope_stack.pop();

        Ok(FrameControl::Continue)
    }

    fn op_get_outer_scope(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let scope = self.outer.get(index as usize);

        if let Some(scope) = scope {
            self.context.avm2.push(scope.values());
        } else {
            self.context.avm2.push(Value::Undefined);
        };

        Ok(FrameControl::Continue)
    }

    fn op_get_scope_object(&mut self, index: u8) -> Result<FrameControl<'gc>, Error> {
        let scope = self.scope_stack.get(index as usize);

        if let Some(scope) = scope {
            self.context.avm2.push(scope.values());
        } else {
            self.context.avm2.push(Value::Undefined);
        };

        Ok(FrameControl::Continue)
    }

    fn op_get_global_scope(&mut self) -> Result<FrameControl<'gc>, Error> {
        self.context.avm2.push(
            self.global_scope()
                .map(|gs| gs.into())
                .unwrap_or(Value::Null),
        );

        Ok(FrameControl::Continue)
    }

    fn op_find_property(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let multiname = self.pool_multiname(method, index)?;
        avm_debug!(self.context.avm2, "Resolving {:?}", multiname);
        let result = self
            .find_definition(&multiname)?
            .or_else(|| self.global_scope());

        self.context
            .avm2
            .push(result.map(|o| o.into()).unwrap_or(Value::Undefined));

        Ok(FrameControl::Continue)
    }

    fn op_find_prop_strict(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let multiname = self.pool_multiname(method, index)?;
        avm_debug!(self.context.avm2, "Resolving {:?}", multiname);
        let found: Result<Object<'gc>, Error> = self
            .find_definition(&multiname)?
            .ok_or_else(|| format!("Property does not exist: {:?}", multiname).into());
        let result: Value<'gc> = found?.into();

        self.context.avm2.push(result);

        Ok(FrameControl::Continue)
    }

    fn op_get_lex(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let multiname = self.pool_multiname_static(method, index)?;
        avm_debug!(self.avm2(), "Resolving {:?}", multiname);
        let found: Result<Value<'gc>, Error> = self
            .resolve_definition(&multiname)?
            .ok_or_else(|| format!("Property does not exist: {:?}", multiname).into());

        self.context.avm2.push(found?);

        Ok(FrameControl::Continue)
    }

    fn op_get_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let object = self.context.avm2.pop().coerce_to_object(self)?;
        let value = object.get_slot(index)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();
        let object = self.context.avm2.pop().coerce_to_object(self)?;

        object.set_slot(index, value, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_get_global_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self
            .global_scope()
            .map(|global| global.get_slot(index))
            .transpose()?
            .unwrap_or(Value::Undefined);

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_set_global_slot(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();

        self.global_scope()
            .map(|global| global.set_slot(index, value, self.context.gc_context))
            .transpose()?;

        Ok(FrameControl::Continue)
    }

    fn op_construct(&mut self, arg_count: u32) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let ctor = self.context.avm2.pop().coerce_to_object(self)?;

        let object = ctor.construct(self, &args)?;

        self.context.avm2.push(object);

        Ok(FrameControl::Continue)
    }

    fn op_construct_prop(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
        arg_count: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let multiname = self.pool_multiname(method, index)?;
        let source = self.context.avm2.pop().coerce_to_object(self)?;

        let object = source.construct_prop(&multiname, &args, self)?;

        self.context.avm2.push(object);

        Ok(FrameControl::Continue)
    }

    fn op_construct_super(&mut self, arg_count: u32) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(arg_count);
        let receiver = self.context.avm2.pop().coerce_to_object(self)?;

        self.super_init(receiver, &args)?;

        Ok(FrameControl::Continue)
    }

    fn op_new_activation(&mut self) -> Result<FrameControl<'gc>, Error> {
        let instance = if let Some(activation_class) = self.activation_class {
            activation_class.construct(self, &[])?
        } else {
            ScriptObject::bare_object(self.context.gc_context)
        };

        self.context.avm2.push(instance);

        Ok(FrameControl::Continue)
    }

    fn op_new_object(&mut self, num_args: u32) -> Result<FrameControl<'gc>, Error> {
        let mut object = self.context.avm2.classes().object.construct(self, &[])?;

        for _ in 0..num_args {
            let value = self.context.avm2.pop();
            let name = self.context.avm2.pop();

            object.set_property(
                object,
                &QName::dynamic_name(name.coerce_to_string(self)?).into(),
                value,
                self,
            )?;
        }

        self.context.avm2.push(object);

        Ok(FrameControl::Continue)
    }

    fn op_new_function(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMethod>,
    ) -> Result<FrameControl<'gc>, Error> {
        let method_entry = self.table_method(method, index, true)?;
        let scope = self.create_scopechain();

        let new_fn = FunctionObject::from_function(self, method_entry.into(), scope)?;

        self.context.avm2.push(new_fn);

        Ok(FrameControl::Continue)
    }

    fn op_new_class(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcClass>,
    ) -> Result<FrameControl<'gc>, Error> {
        let base_value = self.context.avm2.pop();
        let base_class = match base_value {
            Value::Object(o) => match o.as_class_object() {
                Some(cls) => Some(cls),
                None => return Err("Base class for new class is not a class.".into()),
            },
            Value::Null => None,
            _ => return Err("Base class for new class is not Object or null.".into()),
        };

        let class_entry = self.table_class(method, index)?;

        let new_class = ClassObject::from_class(self, class_entry, base_class)?;

        self.context.avm2.push(new_class);

        Ok(FrameControl::Continue)
    }

    fn op_apply_type(&mut self, num_types: u32) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(num_types);
        let base = self.context.avm2.pop().coerce_to_object(self)?;

        if args.len() > 1 {
            return Err(format!(
                "VerifyError: Cannot specialize classes with more than one parameter, {} given",
                args.len()
            )
            .into());
        }

        let applied = base.apply(self, &args[..])?;
        self.context.avm2.push(applied);

        Ok(FrameControl::Continue)
    }

    fn op_new_array(&mut self, num_args: u32) -> Result<FrameControl<'gc>, Error> {
        let args = self.context.avm2.pop_args(num_args);
        let array = ArrayStorage::from_args(&args[..]);
        let array_obj = ArrayObject::from_storage(self, array)?;

        self.context.avm2.push(array_obj);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_a(&mut self) -> Result<FrameControl<'gc>, Error> {
        Ok(FrameControl::Continue)
    }

    fn op_coerce_b(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_boolean();

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_d(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_o(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();

        let coerced = match value {
            Value::Undefined | Value::Null => Value::Null,
            _ => value.coerce_to_object(self)?.into(),
        };

        self.context.avm2.push(coerced);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_s(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();

        let coerced = match value {
            Value::Undefined | Value::Null => Value::Null,
            _ => value.coerce_to_string(self)?.into(),
        };

        self.context.avm2.push(coerced);

        Ok(FrameControl::Continue)
    }

    fn op_coerce_u(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_u32(self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_convert_b(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_boolean();

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_convert_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(Value::Number(value.into()));

        Ok(FrameControl::Continue)
    }

    fn op_convert_d(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(Value::Number(value));

        Ok(FrameControl::Continue)
    }

    fn op_convert_o(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_object(self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_convert_u(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_u32(self)?;

        self.context.avm2.push(Value::Number(value.into()));

        Ok(FrameControl::Continue)
    }

    fn op_convert_s(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_string(self)?;

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_add(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        // TODO: Special handling required for `Date` and ECMA-357/E4X `XML`
        let sum_value = match (value1, value2) {
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 + n2),
            (Value::String(s), value2) => {
                let mut out_s = s.to_string();
                out_s.push_str(&value2.coerce_to_string(self)?);

                Value::String(AvmString::new(self.context.gc_context, out_s))
            }
            (value1, Value::String(s)) => {
                let mut out_s = value1.coerce_to_string(self)?.to_string();
                out_s.push_str(&s);

                Value::String(AvmString::new(self.context.gc_context, out_s))
            }
            (value1, value2) => {
                let prim_value1 = value1.coerce_to_primitive(None, self)?;
                let prim_value2 = value2.coerce_to_primitive(None, self)?;

                match (prim_value1, prim_value2) {
                    (Value::String(s), value2) => {
                        let mut out_s = s.to_string();
                        out_s.push_str(&value2.coerce_to_string(self)?);

                        Value::String(AvmString::new(self.context.gc_context, out_s))
                    }
                    (value1, Value::String(s)) => {
                        let mut out_s = value1.coerce_to_string(self)?.to_string();
                        out_s.push_str(&s);

                        Value::String(AvmString::new(self.context.gc_context, out_s))
                    }
                    (value1, value2) => Value::Number(
                        value1.coerce_to_number(self)? + value2.coerce_to_number(self)?,
                    ),
                }
            }
        };

        self.context.avm2.push(sum_value);

        Ok(FrameControl::Continue)
    }

    fn op_add_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_i32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 + value2);

        Ok(FrameControl::Continue)
    }

    fn op_bitand(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_i32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 & value2);

        Ok(FrameControl::Continue)
    }

    fn op_bitnot(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(!value1);

        Ok(FrameControl::Continue)
    }

    fn op_bitor(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_i32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 | value2);

        Ok(FrameControl::Continue)
    }

    fn op_bitxor(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_i32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 ^ value2);

        Ok(FrameControl::Continue)
    }

    fn op_declocal(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.local_register(index)?.coerce_to_number(self)?;

        self.set_local_register(index, value - 1.0, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_declocal_i(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.local_register(index)?.coerce_to_i32(self)?;

        self.set_local_register(index, value - 1, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_decrement(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value - 1.0);

        Ok(FrameControl::Continue)
    }

    fn op_decrement_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value - 1);

        Ok(FrameControl::Continue)
    }

    fn op_divide(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_number(self)?;
        let value1 = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value1 / value2);

        Ok(FrameControl::Continue)
    }

    fn op_inclocal(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.local_register(index)?.coerce_to_number(self)?;

        self.set_local_register(index, value + 1.0, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_inclocal_i(&mut self, index: u32) -> Result<FrameControl<'gc>, Error> {
        let value = self.local_register(index)?.coerce_to_i32(self)?;

        self.set_local_register(index, value + 1, self.context.gc_context)?;

        Ok(FrameControl::Continue)
    }

    fn op_increment(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value + 1.0);

        Ok(FrameControl::Continue)
    }

    fn op_increment_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value + 1);

        Ok(FrameControl::Continue)
    }

    fn op_lshift(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_u32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 << (value2 & 0x1F));

        Ok(FrameControl::Continue)
    }

    fn op_modulo(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_number(self)?;
        let value1 = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value1 % value2);

        Ok(FrameControl::Continue)
    }

    fn op_multiply(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_number(self)?;
        let value1 = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value1 * value2);

        Ok(FrameControl::Continue)
    }

    fn op_multiply_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_i32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 * value2);

        Ok(FrameControl::Continue)
    }

    fn op_negate(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value1 = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(-value1);

        Ok(FrameControl::Continue)
    }

    fn op_negate_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(-value1);

        Ok(FrameControl::Continue)
    }

    fn op_rshift(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_u32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 >> (value2 & 0x1F));

        Ok(FrameControl::Continue)
    }

    fn op_subtract(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_number(self)?;
        let value1 = self.context.avm2.pop().coerce_to_number(self)?;

        self.context.avm2.push(value1 - value2);

        Ok(FrameControl::Continue)
    }

    fn op_subtract_i(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_i32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_i32(self)?;

        self.context.avm2.push(value1 - value2);

        Ok(FrameControl::Continue)
    }

    fn op_swap(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        self.context.avm2.push(value2);
        self.context.avm2.push(value1);

        Ok(FrameControl::Continue)
    }

    fn op_urshift(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop().coerce_to_u32(self)?;
        let value1 = self.context.avm2.pop().coerce_to_u32(self)?;

        self.context.avm2.push(value1 >> (value2 & 0x1F));

        Ok(FrameControl::Continue)
    }

    fn op_jump<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        reader.seek(full_data, offset);

        Ok(FrameControl::Continue)
    }

    fn op_if_true<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_boolean();

        if value {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_false<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_boolean();

        if !value {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_strict_eq<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value1 == value2 {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_strict_ne<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value1 != value2 {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_eq<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value1.abstract_eq(&value2, self)? {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_ne<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if !value1.abstract_eq(&value2, self)? {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_ge<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value1.abstract_lt(&value2, self)? == Some(false) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_gt<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value2.abstract_lt(&value1, self)? == Some(true) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_le<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value2.abstract_lt(&value1, self)? == Some(false) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_lt<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value1.abstract_lt(&value2, self)? == Some(true) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_nge<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value1.abstract_lt(&value2, self)?.unwrap_or(true) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_ngt<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if !value2.abstract_lt(&value1, self)?.unwrap_or(false) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_nle<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if value2.abstract_lt(&value1, self)?.unwrap_or(true) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_if_nlt<'b>(
        &mut self,
        offset: i32,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        if !value1.abstract_lt(&value2, self)?.unwrap_or(false) {
            reader.seek(full_data, offset);
        }

        Ok(FrameControl::Continue)
    }

    fn op_strict_equals(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        self.context.avm2.push(value1 == value2);

        Ok(FrameControl::Continue)
    }

    fn op_equals(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        let result = value1.abstract_eq(&value2, self)?;

        self.context.avm2.push(result);

        Ok(FrameControl::Continue)
    }

    fn op_greater_equals(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        let result = !value1.abstract_lt(&value2, self)?.unwrap_or(true);

        self.context.avm2.push(result);

        Ok(FrameControl::Continue)
    }

    fn op_greater_than(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        let result = value2.abstract_lt(&value1, self)?.unwrap_or(false);

        self.context.avm2.push(result);

        Ok(FrameControl::Continue)
    }

    fn op_less_equals(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        let result = !value2.abstract_lt(&value1, self)?.unwrap_or(true);

        self.context.avm2.push(result);

        Ok(FrameControl::Continue)
    }

    fn op_less_than(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value2 = self.context.avm2.pop();
        let value1 = self.context.avm2.pop();

        let result = value1.abstract_lt(&value2, self)?.unwrap_or(false);

        self.context.avm2.push(result);

        Ok(FrameControl::Continue)
    }

    fn op_nop(&mut self) -> Result<FrameControl<'gc>, Error> {
        Ok(FrameControl::Continue)
    }

    fn op_not(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_boolean();

        self.context.avm2.push(!value);

        Ok(FrameControl::Continue)
    }

    fn op_has_next(&mut self) -> Result<FrameControl<'gc>, Error> {
        let cur_index = self.context.avm2.pop().coerce_to_u32(self)?;
        let object = self.context.avm2.pop().coerce_to_object(self)?;

        let next_index = cur_index + 1;

        if object.get_enumerant_name(next_index).is_some() {
            self.context.avm2.push(next_index);
        } else {
            self.context.avm2.push(0.0);
        }

        Ok(FrameControl::Continue)
    }

    fn op_has_next_2(
        &mut self,
        object_register: u32,
        index_register: u32,
    ) -> Result<FrameControl<'gc>, Error> {
        let cur_index = self.local_register(index_register)?.coerce_to_u32(self)?;
        let mut object = Some(
            self.local_register(object_register)?
                .coerce_to_object(self)?,
        );

        let mut next_index = cur_index + 1;

        while let Some(cur_object) = object {
            if cur_object.get_enumerant_name(next_index).is_none() {
                next_index = 1;
                object = cur_object.proto();
            } else {
                break;
            }
        }

        if object.is_none() {
            next_index = 0;
        }

        self.context.avm2.push(next_index != 0);
        self.set_local_register(index_register, next_index, self.context.gc_context)?;
        self.set_local_register(
            object_register,
            object.map(|v| v.into()).unwrap_or(Value::Null),
            self.context.gc_context,
        )?;

        Ok(FrameControl::Continue)
    }

    fn op_next_name(&mut self) -> Result<FrameControl<'gc>, Error> {
        let cur_index = self.context.avm2.pop().coerce_to_number(self)?;
        let object = self.context.avm2.pop().coerce_to_object(self)?;

        let name = object.get_enumerant_name(cur_index as u32);

        self.context.avm2.push(name.unwrap_or(Value::Undefined));

        Ok(FrameControl::Continue)
    }

    fn op_next_value(&mut self) -> Result<FrameControl<'gc>, Error> {
        let cur_index = self.context.avm2.pop().coerce_to_number(self)?;
        let object = self.context.avm2.pop().coerce_to_object(self)?;

        let name = object.get_enumerant_name(cur_index as u32);
        let value = if let Some(name) = name {
            let name = name.coerce_to_string(self)?;
            object.get_property(object, &QName::dynamic_name(name).into(), self)?
        } else {
            Value::Undefined
        };

        self.context.avm2.push(value);

        Ok(FrameControl::Continue)
    }

    fn op_is_type(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        type_name_index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();

        let multiname = self.pool_multiname_static(method, type_name_index)?;
        let found: Result<Value<'gc>, Error> =
            self.resolve_definition(&multiname)?.ok_or_else(|| {
                format!(
                    "Attempted to check against nonexistent type {:?}",
                    multiname
                )
                .into()
            });
        let type_object = found?
            .coerce_to_object(self)?
            .as_class_object()
            .ok_or_else(|| {
                Error::from(format!(
                    "Attempted to check against nonexistent type {:?}",
                    multiname
                ))
            })?;

        let is_instance_of = value.is_of_type(self, type_object)?;
        self.context.avm2.push(is_instance_of);

        Ok(FrameControl::Continue)
    }

    fn op_is_type_late(&mut self) -> Result<FrameControl<'gc>, Error> {
        let type_object = self.context.avm2.pop().coerce_to_object(self)?;
        let value = self.context.avm2.pop();

        let type_object = type_object
            .as_class_object()
            .ok_or_else(|| Error::from("Attempted to check against non-type"))?;

        let is_instance_of = value.is_of_type(self, type_object)?;

        self.context.avm2.push(is_instance_of);

        Ok(FrameControl::Continue)
    }

    fn op_as_type(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        type_name_index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop().coerce_to_object(self)?;

        let multiname = self.pool_multiname_static(method, type_name_index)?;
        let found: Result<Value<'gc>, Error> =
            self.resolve_definition(&multiname)?.ok_or_else(|| {
                format!(
                    "Attempted to check against nonexistent type {:?}",
                    multiname
                )
                .into()
            });
        let class = found?
            .coerce_to_object(self)?
            .as_class_object()
            .ok_or_else(|| {
                Error::from("TypeError: The right-hand side of operator must be a class.")
            })?;

        if value.is_of_type(class, self)? {
            self.context.avm2.push(value);
        } else {
            self.context.avm2.push(Value::Null);
        }

        Ok(FrameControl::Continue)
    }

    fn op_as_type_late(&mut self) -> Result<FrameControl<'gc>, Error> {
        let class = self.context.avm2.pop().coerce_to_object(self)?;
        let value = self.context.avm2.pop().coerce_to_object(self)?;

        let class = class.as_class_object().ok_or_else(|| {
            Error::from("TypeError: The right-hand side of operator must be a class.")
        })?;

        if value.is_of_type(class, self)? {
            self.context.avm2.push(value);
        } else {
            self.context.avm2.push(Value::Null);
        }

        Ok(FrameControl::Continue)
    }

    fn op_instance_of(&mut self) -> Result<FrameControl<'gc>, Error> {
        let type_object = self.context.avm2.pop().coerce_to_object(self)?;
        let value = self.context.avm2.pop().coerce_to_object(self).ok();

        if let Some(value) = value {
            let is_instance_of = value.is_instance_of(self, type_object)?;

            self.context.avm2.push(is_instance_of);
        } else {
            self.context.avm2.push(false);
        }

        Ok(FrameControl::Continue)
    }

    fn op_type_of(&mut self) -> Result<FrameControl<'gc>, Error> {
        let value = self.context.avm2.pop();

        let type_name = match value {
            Value::Undefined => "undefined",
            Value::Null => "object",
            Value::Bool(_) => "boolean",
            Value::Number(_) | Value::Integer(_) | Value::Unsigned(_) => "number",
            Value::Object(o) => {
                // Subclasses always have a typeof = "object", must be a subclass if the prototype chain is > 2, or not a subclass if <=2
                let is_not_subclass = matches!(
                    o.proto().and_then(|p| p.proto()).and_then(|p| p.proto()),
                    None
                );

                match o {
                    Object::FunctionObject(_) => {
                        if is_not_subclass {
                            "function"
                        } else {
                            "object"
                        }
                    }
                    Object::XmlObject(_) => {
                        if is_not_subclass {
                            "xml"
                        } else {
                            "object"
                        }
                    }
                    _ => "object",
                }
            }
            Value::String(_) => "string",
        };

        self.context.avm2.push(Value::String(AvmString::new(
            self.context.gc_context,
            type_name,
        )));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::EscXAttr`
    fn op_esc_xattr(&mut self) -> Result<FrameControl<'gc>, Error> {
        let s = self.context.avm2.pop().coerce_to_string(self)?;

        // Implementation of `EscapeAttributeValue` from ECMA-357(10.2.1.2)
        let mut r = String::new();
        for c in s.chars() {
            match c {
                '"' => r += "&quot;",
                '<' => r += "&lt;",
                '&' => r += "&amp;",
                '\u{000A}' => r += "&#xA;",
                '\u{000D}' => r += "&#xD;",
                '\u{0009}' => r += "&#x9;",
                _ => r.push(c),
            }
        }
        self.context
            .avm2
            .push(AvmString::new(self.context.gc_context, r));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::EscXElem`
    fn op_esc_elem(&mut self) -> Result<FrameControl<'gc>, Error> {
        let s = self.context.avm2.pop().coerce_to_string(self)?;

        // contrary to the avmplus documentation, this escapes the value on the top of the stack using EscapeElementValue from ECMA-357 *NOT* EscapeAttributeValue.
        // Implementation of `EscapeElementValue` from ECMA-357(10.2.1.1)
        let mut r = String::new();
        for c in s.chars() {
            match c {
                '<' => r += "&lt;",
                '>' => r += "&gt;",
                '&' => r += "&amp;",
                _ => r.push(c),
            }
        }
        self.context
            .avm2
            .push(AvmString::new(self.context.gc_context, r));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::LookupSwitch`
    fn op_lookup_switch<'b>(
        &mut self,
        default_offset: i32,
        case_offsets: &[i32],
        instruction_start: usize,
        reader: &mut Reader<'b>,
        full_data: &'b [u8],
    ) -> Result<FrameControl<'gc>, Error> {
        let index = self.context.avm2.pop().coerce_to_i32(self)?;

        let offset = case_offsets
            .get(index as usize)
            .copied()
            .unwrap_or(default_offset)
            + instruction_start as i32
            - reader.pos(full_data) as i32;

        reader.seek(full_data, offset);
        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Coerce`
    fn op_coerce(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        index: Index<AbcMultiname>,
    ) -> Result<FrameControl<'gc>, Error> {
        let val = self.context.avm2.pop();
        let type_name = self.pool_multiname_static_any(method, index)?;
        let param_type = self.resolve_type(type_name)?;

        let x = if let Some(param_type) = param_type {
            val.coerce_to_type(self, param_type)?
        } else {
            val
        };

        self.context.avm2.push(x);
        Ok(FrameControl::Continue)
    }

    pub fn domain(&self) -> Domain<'gc> {
        self.outer.domain()
    }

    fn domain_memory(&self) -> ByteArrayObject<'gc> {
        self.outer.domain().domain_memory()
    }

    /// Implements `Op::Si8`
    fn op_si8(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_i32(self)?;
        let val = self.context.avm2.pop().coerce_to_i32(self)?;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut(self.context.gc_context)
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;

        let address =
            usize::try_from(address).map_err(|_| "RangeError: The specified range is invalid")?;
        dm.write_at_nongrowing(&val.to_le_bytes(), address)?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Si16`
    fn op_si16(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_i32(self)?;
        let val = self.context.avm2.pop().coerce_to_i32(self)?;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut(self.context.gc_context)
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;

        let address =
            usize::try_from(address).map_err(|_| "RangeError: The specified range is invalid")?;
        dm.write_at_nongrowing(&val.to_le_bytes(), address)?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Si32`
    fn op_si32(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_i32(self)?;
        let val = self.context.avm2.pop().coerce_to_i32(self)?;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut(self.context.gc_context)
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;

        let address =
            usize::try_from(address).map_err(|_| "RangeError: The specified range is invalid")?;
        dm.write_at_nongrowing(&val.to_le_bytes(), address)?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sf32`
    fn op_sf32(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_i32(self)?;
        let val = self.context.avm2.pop().coerce_to_number(self)? as f32;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut(self.context.gc_context)
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;

        let address =
            usize::try_from(address).map_err(|_| "RangeError: The specified range is invalid")?;
        dm.write_at_nongrowing(&val.to_le_bytes(), address)?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sf64`
    fn op_sf64(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_i32(self)?;
        let val = self.context.avm2.pop().coerce_to_number(self)?;

        let dm = self.domain_memory();
        let mut dm = dm
            .as_bytearray_mut(self.context.gc_context)
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;

        let address =
            usize::try_from(address).map_err(|_| "RangeError: The specified range is invalid")?;
        dm.write_at_nongrowing(&val.to_le_bytes(), address)?;

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Li8`
    fn op_li8(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm
            .as_bytearray()
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;
        let val = dm.get(address);

        if let Some(val) = val {
            self.context.avm2.push(val);
        } else {
            return Err("RangeError: The specified range is invalid".into());
        }

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Li16`
    fn op_li16(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm
            .as_bytearray()
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;
        let val = dm.read_at(2, address)?;
        self.context
            .avm2
            .push(u16::from_le_bytes(val.try_into().unwrap()));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Li32`
    fn op_li32(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm
            .as_bytearray()
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;
        let val = dm.read_at(4, address)?;
        self.context
            .avm2
            .push(i32::from_le_bytes(val.try_into().unwrap()));
        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Lf32`
    fn op_lf32(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm
            .as_bytearray()
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;
        let val = dm.read_at(4, address)?;
        self.context
            .avm2
            .push(f32::from_le_bytes(val.try_into().unwrap()));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Lf64`
    fn op_lf64(&mut self) -> Result<FrameControl<'gc>, Error> {
        let address = self.context.avm2.pop().coerce_to_u32(self)? as usize;

        let dm = self.domain_memory();
        let dm = dm
            .as_bytearray()
            .ok_or_else(|| "Unable to get bytearray storage".to_string())?;
        let val = dm.read_at(8, address)?;
        self.context
            .avm2
            .push(f64::from_le_bytes(val.try_into().unwrap()));
        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sxi1`
    fn op_sxi1(&mut self) -> Result<FrameControl<'gc>, Error> {
        let val = self.context.avm2.pop().coerce_to_i32(self)?;

        let val = val.wrapping_shl(31).wrapping_shr(31);

        self.context.avm2.push(Value::Integer(val));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sxi8`
    fn op_sxi8(&mut self) -> Result<FrameControl<'gc>, Error> {
        let val = self.context.avm2.pop().coerce_to_i32(self)?;

        let val = (val.wrapping_shl(23).wrapping_shr(23) & 0xFF) as i8 as i32;

        self.context.avm2.push(Value::Integer(val));

        Ok(FrameControl::Continue)
    }

    /// Implements `Op::Sxi16`
    fn op_sxi16(&mut self) -> Result<FrameControl<'gc>, Error> {
        let val = self.context.avm2.pop().coerce_to_i32(self)?;

        let val = (val.wrapping_shl(15).wrapping_shr(15) & 0xFFFF) as i16 as i32;

        self.context.avm2.push(Value::Integer(val));

        Ok(FrameControl::Continue)
    }

    #[cfg(avm_debug)]
    fn op_debug(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        is_local_register: bool,
        register_name: Index<String>,
        register: u8,
    ) -> Result<FrameControl<'gc>, Error> {
        if is_local_register {
            let register_name = self.pool_string(method, register_name)?;
            let value = self.local_register(register as u32)?;

            avm_debug!(self.avm2(), "Debug: {} = {:?}", register_name, value);
        } else {
            avm_debug!(self.avm2(), "Unknown debugging mode!");
        }

        Ok(FrameControl::Continue)
    }

    #[cfg(not(avm_debug))]
    fn op_debug(
        &mut self,
        _method: Gc<'gc, BytecodeMethod<'gc>>,
        _is_local_register: bool,
        _register_name: Index<String>,
        _register: u8,
    ) -> Result<FrameControl<'gc>, Error> {
        Ok(FrameControl::Continue)
    }

    #[cfg(avm_debug)]
    fn op_debug_file(
        &mut self,
        method: Gc<'gc, BytecodeMethod<'gc>>,
        file_name: Index<String>,
    ) -> Result<FrameControl<'gc>, Error> {
        let file_name = self.pool_string(method, file_name)?;

        avm_debug!(self.avm2(), "File: {}", file_name);

        Ok(FrameControl::Continue)
    }

    #[cfg(not(avm_debug))]
    fn op_debug_file(
        &mut self,
        _method: Gc<'gc, BytecodeMethod<'gc>>,
        _file_name: Index<String>,
    ) -> Result<FrameControl<'gc>, Error> {
        Ok(FrameControl::Continue)
    }

    fn op_debug_line(&mut self, line_num: u32) -> Result<FrameControl<'gc>, Error> {
        avm_debug!(self.avm2(), "Line: {}", line_num);

        Ok(FrameControl::Continue)
    }

    fn op_bkpt(&mut self) -> Result<FrameControl<'gc>, Error> {
        // while a debugger is not attached, this is a no-op
        Ok(FrameControl::Continue)
    }

    fn op_bkpt_line(&mut self, _line_num: u32) -> Result<FrameControl<'gc>, Error> {
        // while a debugger is not attached, this is a no-op
        Ok(FrameControl::Continue)
    }

    fn op_timestamp(&mut self) -> Result<FrameControl<'gc>, Error> {
        // while a debugger is not attached, this is a no-op
        Ok(FrameControl::Continue)
    }
}
