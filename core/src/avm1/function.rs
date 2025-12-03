//! Code relating to executable functions + calling conventions.

use super::NativeObject;
use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::super_object::SuperObject;
use crate::avm1::object_reference::MovieClipReference;
use crate::avm1::property::Attribute;
use crate::avm1::scope::Scope;
use crate::avm1::value::Value;
use crate::avm1::{ArrayBuilder, Object};
use crate::display_object::TDisplayObject;
use crate::string::{AvmString, StringContext, SwfStrExt as _};
use crate::tag_utils::SwfSlice;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::istr;
use std::{borrow::Cow, cell::Cell, num::NonZeroU8};
use swf::{avm1::types::FunctionFlags, SwfStr};

/// Represents a function defined in Ruffle's code.
pub type NativeFunction = for<'gc> fn(
    &mut Activation<'_, 'gc>,
    Object<'gc>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>>;

/// Represents a function defined in Ruffle's code, compatible with ASnative.
pub type TableNativeFunction = for<'gc> fn(
    &mut Activation<'_, 'gc>,
    Object<'gc>,
    &[Value<'gc>],
    u16,
) -> Result<Value<'gc>, Error<'gc>>;

/// Indicates the reason for an execution
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ExecutionReason {
    /// This execution is a "normal" function call from ActionScript bytecode.
    FunctionCall,

    /// This execution is a "normal" constructor call from ActionScript bytecode.
    ConstructorCall,

    /// This execution is a "special" internal function call from the player,
    /// such as getters, setters, `toString`, or event handlers.
    Special,
}

/// Represents a function defined in the AVM1 runtime, either through
/// `DefineFunction` or `DefineFunction2`.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Avm1Function<'gc> {
    /// The file format version of the SWF that generated this function.
    swf_version: u8,

    /// A reference to the underlying SWF data.
    data: SwfSlice,

    /// The name of the function, if not anonymous.
    name: Option<AvmString<'gc>>,

    /// The number of registers to allocate for this function's private register
    /// set. Any register beyond this ID will be served from the global one.
    register_count: u8,

    /// The parameters of the function.
    params: Vec<Param<'gc>>,

    /// The scope the function was born into.
    scope: Gc<'gc, Scope<'gc>>,

    /// The constant pool the function executes with.
    constant_pool: Gc<'gc, Vec<Value<'gc>>>,

    /// The base movie clip that the function was defined on.
    /// This is the movie clip that contains the bytecode.
    base_clip: MovieClipReference<'gc>,

    /// The flags that define the preloaded registers of the function.
    #[collect(require_static)]
    flags: FunctionFlags,
}

impl<'gc> Avm1Function<'gc> {
    /// Construct a function from a DefineFunction2 action.
    pub fn from_swf_function(
        gc_context: &Mutation<'gc>,
        swf_version: u8,
        actions: SwfSlice,
        swf_function: swf::avm1::types::DefineFunction2,
        scope: Gc<'gc, Scope<'gc>>,
        constant_pool: Gc<'gc, Vec<Value<'gc>>>,
        base_clip: MovieClipReference<'gc>,
    ) -> Self {
        let encoding = SwfStr::encoding_for_version(swf_version);
        let name = if swf_function.name.is_empty() {
            None
        } else {
            Some(AvmString::new(
                gc_context,
                swf_function.name.decode(encoding),
            ))
        };

        let params = swf_function
            .params
            .iter()
            .map(|p| Param {
                register: p.register_index,
                name: AvmString::new(gc_context, p.name.decode(encoding)),
            })
            .collect();

        Avm1Function {
            swf_version,
            data: actions,
            name,
            register_count: swf_function.register_count,
            params,
            scope,
            constant_pool,
            base_clip,
            flags: swf_function.flags,
        }
    }

    pub fn swf_version(&self) -> u8 {
        self.swf_version
    }

    pub fn name(&self) -> Option<AvmString<'gc>> {
        self.name
    }

    pub fn scope(&self) -> Gc<'gc, Scope<'gc>> {
        self.scope
    }

    pub fn register_count(&self) -> u8 {
        self.register_count
    }

    fn debug_string_for_call(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: ExecutionName<'gc>,
        args: &[Value<'gc>],
    ) -> String {
        let mut result = match self.name.map(ExecutionName::Dynamic).unwrap_or(name) {
            ExecutionName::Static(n) => n.to_owned(),
            ExecutionName::Dynamic(n) => n.to_utf8_lossy().into_owned(),
        };
        result.push('(');
        for i in 0..args.len() {
            let arg_type = args.get(i).unwrap().type_of(activation);
            result.push_str(&arg_type.to_string());

            if i < args.len() - 1 {
                result.push_str(", ");
            }
        }
        result.push(')');
        result
    }

    fn load_this(&self, frame: &mut Activation<'_, 'gc>, this: Value<'gc>, preload_r: &mut u8) {
        let preload = self.flags.contains(FunctionFlags::PRELOAD_THIS);
        let suppress = self.flags.contains(FunctionFlags::SUPPRESS_THIS);

        if preload {
            // The register is set to undefined if both flags are set.
            let this = if suppress { Value::Undefined } else { this };
            frame.set_local_register(*preload_r, this);
            *preload_r += 1;
        }
    }

    fn load_arguments(
        &self,
        frame: &mut Activation<'_, 'gc>,
        args: &[Value<'gc>],
        caller: Option<Object<'gc>>,
        preload_r: &mut u8,
    ) {
        let preload = self.flags.contains(FunctionFlags::PRELOAD_ARGUMENTS);
        let suppress = self.flags.contains(FunctionFlags::SUPPRESS_ARGUMENTS);

        if suppress && !preload {
            return;
        }

        let arguments = ArrayBuilder::new(frame).with(args.iter().cloned());

        arguments.define_value(
            frame.gc(),
            istr!(frame, "callee"),
            frame.callee.unwrap().into(),
            Attribute::DONT_ENUM,
        );

        arguments.define_value(
            frame.gc(),
            istr!(frame, "caller"),
            caller.map(Value::from).unwrap_or(Value::Null),
            Attribute::DONT_ENUM,
        );

        let arguments = Value::from(arguments);

        // Contrarily to `this` and `super`, setting both flags is equivalent to just setting `preload`.
        if preload {
            frame.set_local_register(*preload_r, arguments);
            *preload_r += 1;
        } else {
            frame.force_define_local(istr!(frame, "arguments"), arguments);
        }
    }

    fn load_super(
        &self,
        frame: &mut Activation<'_, 'gc>,
        this: Option<Object<'gc>>,
        depth: u8,
        preload_r: &mut u8,
    ) {
        let preload = self.flags.contains(FunctionFlags::PRELOAD_SUPER);
        let suppress = self.flags.contains(FunctionFlags::SUPPRESS_SUPER);

        // TODO: `super` should only be defined if this was a method call (depth > 0?)
        // `f[""]()` emits a CallMethod op, causing `this` to be undefined, but `super` is a function; what is it?
        let zuper = this.filter(|_| !suppress).map(|this| {
            let zuper = NativeObject::Super(SuperObject::new(this, depth));
            Object::new_with_native(frame.strings(), None::<Value<'_>>, zuper).into()
        });

        if preload {
            // The register is set to undefined if both flags are set.
            frame.set_local_register(*preload_r, zuper.unwrap_or(Value::Undefined));
            *preload_r += 1;
        } else if let Some(zuper) = zuper {
            frame.force_define_local(istr!(frame, "super"), zuper);
        }
    }

    fn load_root(&self, frame: &mut Activation<'_, 'gc>, preload_r: &mut u8) {
        if self.flags.contains(FunctionFlags::PRELOAD_ROOT) {
            let root = frame.base_clip().avm1_root().object1_or_undef();
            frame.set_local_register(*preload_r, root);
            *preload_r += 1;
        }
    }

    fn load_parent(&self, frame: &mut Activation<'_, 'gc>, preload_r: &mut u8) {
        if self.flags.contains(FunctionFlags::PRELOAD_PARENT) {
            // If _parent is undefined (because this is a root timeline), it actually does not get pushed,
            // and _global ends up incorrectly taking _parent's register.
            // See test for more info.
            if let Some(parent) = frame.base_clip().avm1_parent() {
                frame.set_local_register(*preload_r, parent.object1_or_undef());
                *preload_r += 1;
            }
        }
    }

    fn load_global(&self, frame: &mut Activation<'_, 'gc>, preload_r: &mut u8) {
        if self.flags.contains(FunctionFlags::PRELOAD_GLOBAL) {
            let global = frame.global_object();
            frame.set_local_register(*preload_r, global.into());
            *preload_r += 1;
        }
    }

    /// Execute the given function bytecode.
    #[expect(clippy::too_many_arguments)]
    fn exec(
        &self,
        name: ExecutionName<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Value<'gc>,
        depth: u8,
        args: &[Value<'gc>],
        reason: ExecutionReason,
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this_obj = match this {
            Value::Object(obj) => Some(obj),
            _ => None,
        };

        let target = activation.target_clip_or_root();
        let is_closure = activation.swf_version() >= 6;

        let this_do = this_obj
            .and_then(|this| this.as_display_object())
            .unwrap_or(target);

        let base_clip = if is_closure || reason == ExecutionReason::Special {
            self.base_clip
                .resolve_reference(activation)
                .map(|(_, _, dobj)| dobj)
                .unwrap_or(this_do)
        } else {
            this_do
        };
        let (swf_version, parent_scope) = if is_closure {
            // Function calls in a v6+ SWF are proper closures, and "close" over the scope that defined the function:
            // * Use the SWF version from the SWF that defined the function.
            // * Use the base clip from when the function was defined.
            // * Close over the scope from when the function was defined.
            (self.swf_version(), self.scope())
        } else {
            // Function calls in a v5 SWF are *not* closures, and will use the settings of
            // `this`, regardless of the function's origin:
            // * Use the SWF version of `this`.
            // * Use the base clip of `this`.
            // * Allocate a new scope using the given base clip. No previous scope is closed over.
            let swf_version = base_clip.swf_version().max(5);
            let base_clip_obj = base_clip.object1().unwrap();
            // TODO: It would be nice to avoid these extra Scope allocs.
            let scope = Gc::new(
                activation.gc(),
                Scope::new(
                    activation.global_scope(),
                    super::scope::ScopeClass::Target,
                    base_clip_obj,
                ),
            );
            (swf_version, scope)
        };

        let child_scope = Gc::new(
            activation.gc(),
            Scope::new_local_scope(parent_scope, activation.gc()),
        );

        // The caller is the previous callee.
        let arguments_caller = activation.callee;

        let name = if cfg!(feature = "avm_debug") {
            Cow::Owned(self.debug_string_for_call(activation, name, args))
        } else {
            Cow::Borrowed("[Anonymous]")
        };

        let is_this_inherited = self
            .flags
            .intersects(FunctionFlags::PRELOAD_THIS | FunctionFlags::SUPPRESS_THIS);
        let local_this = if is_this_inherited {
            activation.this_cell()
        } else {
            this
        };

        let local_registers: Box<[_]> = (0..self.register_count())
            .map(|_| Cell::new(Value::Undefined))
            .collect();

        let max_recursion_depth = activation.context.avm1.max_recursion_depth();
        let mut frame = Activation::from_action(
            activation.context,
            activation.id.function(&name, reason, max_recursion_depth)?,
            swf_version,
            child_scope,
            self.constant_pool,
            base_clip,
            local_this,
            Some(callee),
            &local_registers,
        );

        // Local register #0 is always left free.
        let mut preload_r = 1;
        self.load_this(&mut frame, this, &mut preload_r);
        self.load_arguments(&mut frame, args, arguments_caller, &mut preload_r);
        self.load_super(&mut frame, this_obj, depth, &mut preload_r);
        self.load_root(&mut frame, &mut preload_r);
        self.load_parent(&mut frame, &mut preload_r);
        self.load_global(&mut frame, &mut preload_r);

        // Any unassigned args are set to undefined to prevent assignments from leaking to the parent scope (#2166)
        let args_iter = args
            .iter()
            .cloned()
            .chain(std::iter::repeat(Value::Undefined));

        //TODO: What happens if the argument registers clash with the
        //preloaded registers? What gets done last?
        for (param, value) in self.params.iter().zip(args_iter) {
            if let Some(register) = param.register {
                frame.set_local_register(register.get(), value);
            } else {
                frame.force_define_local(param.name, value);
            }
        }

        Ok(frame.run_actions(self.data.clone())?.value())
    }
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
struct Param<'gc> {
    /// The register the argument will be preloaded into.
    ///
    /// If `register` is `None`, then this parameter will be stored in a named variable in the
    /// function activation and can be accessed using `GetVariable`/`SetVariable`.
    /// Otherwise, the parameter is loaded into a register and must be accessed with
    /// `Push`/`StoreRegister`.
    #[collect(require_static)]
    register: Option<NonZeroU8>,

    /// The name of the parameter.
    name: AvmString<'gc>,
}

/// Represents a function that can be defined in the Ruffle runtime or by the
/// AVM1 bytecode itself.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
enum Executable<'gc> {
    /// A function provided by the Ruffle runtime and implemented in Rust.
    Native(#[collect(require_static)] NativeFunction),

    TableNative {
        #[collect(require_static)]
        native: TableNativeFunction,
        index: u16,
    },

    /// ActionScript data defined by a previous `DefineFunction` or
    /// `DefineFunction2` action.
    Action(Gc<'gc, Avm1Function<'gc>>),
}

/// Indicates the default name to use for this execution in debug builds.
pub enum ExecutionName<'gc> {
    Static(&'static str),
    Dynamic(AvmString<'gc>),
}

impl From<&'static str> for ExecutionName<'_> {
    fn from(string: &'static str) -> Self {
        ExecutionName::Static(string)
    }
}

impl<'gc> From<AvmString<'gc>> for ExecutionName<'gc> {
    fn from(string: AvmString<'gc>) -> Self {
        ExecutionName::Dynamic(string)
    }
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct FunctionObject<'gc> {
    /// The code that will be invoked when this object is called.
    function: Executable<'gc>,
    /// The code that will be invoked when this object is constructed.
    ///
    /// If `None`, falls back to `function`.
    #[collect(require_static)]
    constructor: Option<NativeFunction>,
}

impl<'gc> FunctionObject<'gc> {
    /// Builds a new function object.
    ///
    /// `fn_proto` refers to the implicit proto of the function object, and the
    /// `prototype` refers to the explicit prototype of the function. If the
    /// prototype is present, it will be linked with the newly-created object.
    pub fn build(
        self,
        context: &StringContext<'gc>,
        fn_proto: Object<'gc>,
        prototype: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let obj = Object::new(context, Some(fn_proto));
        let native = NativeObject::Function(Gc::new(context.gc(), self));
        obj.set_native(context.gc(), native);

        if let Some(prototype) = prototype {
            prototype.define_value(
                context.gc(),
                istr!(context, "constructor"),
                Value::Object(obj),
                Attribute::DONT_ENUM,
            );
            obj.define_value(
                context.gc(),
                istr!(context, "prototype"),
                prototype.into(),
                Attribute::DONT_ENUM | Attribute::DONT_DELETE,
            );
        }

        obj
    }

    /// A function that does nothing.
    ///
    /// This can also serve as a no-op constructor.
    pub fn empty() -> Self {
        Self {
            function: Executable::Native(|_, _, _| Ok(Value::Undefined)),
            constructor: None,
        }
    }

    /// A function with AVM1 bytecode.
    pub fn bytecode(function: Gc<'gc, Avm1Function<'gc>>) -> Self {
        Self {
            function: Executable::Action(function),
            constructor: None,
        }
    }

    /// A function with a native executable.
    pub fn native(function: NativeFunction) -> Self {
        Self {
            function: Executable::Native(function),
            constructor: None,
        }
    }

    /// A function with a native executable, compatible with ASnative.
    pub fn table_native(native: TableNativeFunction, index: u16) -> Self {
        Self {
            function: Executable::TableNative { native, index },
            constructor: None,
        }
    }

    /// A native constructor.
    ///
    /// This differs from [`Self::native`] in two important ways:
    /// - When called through `new`, the return value will always become the result of the
    ///   operation. Native constructors should therefore generally return either `this`,
    ///   if the object was successfully constructed, or `undefined` if not.
    /// - When called as a normal function, `function` will be called instead of `constructor`;
    ///   if it is `None`, the return value will be `undefined`.
    pub fn constructor(constructor: NativeFunction, function: Option<NativeFunction>) -> Self {
        Self {
            function: Executable::Native(function.unwrap_or(|_, _, _| Ok(Value::Undefined))),
            constructor: Some(constructor),
        }
    }

    /// Execute the given code.
    ///
    /// This is fairly low-level; prefer using other call methods if possible.
    #[expect(clippy::too_many_arguments)]
    pub fn exec(
        self,
        name: ExecutionName<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Value<'gc>,
        depth: u8,
        args: &[Value<'gc>],
        reason: ExecutionReason,
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self.function {
            Executable::Native(nf) => {
                // TODO: Change NativeFunction to accept `this: Value`.
                let this = this.coerce_to_object_or_bare(activation)?;
                nf(activation, this, args)
            }
            Executable::TableNative { native, index } => {
                // TODO: Change TableNativeFunction to accept `this: Value`.
                let this = this.coerce_to_object_or_bare(activation)?;
                native(activation, this, args, index)
            }
            Executable::Action(af) => af.exec(name, activation, this, depth, args, reason, callee),
        }
    }

    /// Execute the given code as a constructor.
    ///
    /// This is fairly low-level; prefer using other call methods if possible.
    #[expect(clippy::too_many_arguments)]
    pub fn exec_constructor(
        self,
        name: ExecutionName<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Value<'gc>,
        depth: u8,
        args: &[Value<'gc>],
        reason: ExecutionReason,
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let constr = match self.constructor {
            Some(constr) => Executable::Native(constr),
            None => self.function,
        };
        match constr {
            Executable::Native(nf) => {
                // TODO: Change NativeFunction to accept `this: Value`.
                let this = this.coerce_to_object_or_bare(activation)?;
                nf(activation, this, args)
            }
            Executable::TableNative { native, index } => {
                // TODO: Change TableNativeFunction to accept `this: Value`.
                let this = this.coerce_to_object_or_bare(activation)?;
                native(activation, this, args, index)
            }
            Executable::Action(af) => af.exec(name, activation, this, depth, args, reason, callee),
        }
    }

    pub fn call(
        self,
        name: impl Into<ExecutionName<'gc>>,
        activation: &mut Activation<'_, 'gc>,
        callee: Object<'gc>,
        this: Value<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.exec(
            name.into(),
            activation,
            this,
            0,
            args,
            ExecutionReason::FunctionCall,
            callee,
        )
    }

    pub fn construct_on_existing(
        self,
        activation: &mut Activation<'_, 'gc>,
        callee: Object<'gc>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<(), Error<'gc>> {
        Self::define_constructor_props(activation, this, callee.into());

        // Always ignore the constructor's return value.
        let _ = self.exec_constructor(
            ExecutionName::Static("[ctor]"),
            activation,
            this.into(),
            1,
            args,
            ExecutionReason::ConstructorCall,
            callee,
        )?;

        Ok(())
    }

    pub fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        callee: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let prototype = callee.get(istr!("prototype"), activation)?;
        let this = Object::new(activation.strings(), Some(prototype));

        Self::define_constructor_props(activation, this, callee.into());

        // Propagate the return value only for native constructors.
        let propagate = self.constructor.is_some();
        let ret = self.exec_constructor(
            ExecutionName::Static("[ctor]"),
            activation,
            this.into(),
            1,
            args,
            ExecutionReason::ConstructorCall,
            callee,
        )?;
        Ok(if propagate { ret } else { this.into() })
    }

    fn define_constructor_props(
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
        callee: Value<'gc>,
    ) {
        this.define_value(
            activation.gc(),
            istr!("__constructor__"),
            callee,
            Attribute::DONT_ENUM,
        );
        if activation.swf_version() < 7 {
            this.define_value(
                activation.gc(),
                istr!("constructor"),
                callee,
                Attribute::DONT_ENUM,
            );
        }
    }
}
