//! Code relating to executable functions + calling conventions.

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::super_object::SuperObject;
use crate::avm1::property::{Attribute, Attribute::*};
use crate::avm1::scope::Scope;
use crate::avm1::value::Value;
use crate::avm1::{Object, ObjectPtr, ScriptObject, TObject, UpdateContext};
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::tag_utils::SwfSlice;
use enumset::EnumSet;
use gc_arena::{Collect, CollectionContext, Gc, GcCell, MutationContext};
use std::borrow::Cow;
use std::fmt;
use swf::avm1::types::FunctionParam;

/// Represents a function defined in Ruffle's code.
///
/// Parameters are as follows:
///
///  * The AVM1 runtime
///  * The action context
///  * The current `this` object
///  * The arguments this function was called with
///
/// Native functions are allowed to return a value or `None`. `None` indicates
/// that the given value will not be returned on the stack and instead will
/// resolve on the AVM stack, as if you had called a non-native function. If
/// your function yields `None`, you must ensure that the top-most activation
/// in the AVM1 runtime will return with the value of this function.
pub type NativeFunction<'gc> = fn(
    &mut Activation<'_, 'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    Object<'gc>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>>;

/// Indicates the reason for an execution
#[derive(Debug, Clone)]
pub enum ExecutionReason {
    /// This execution is a "normal" function call, from either user-code or builtins.
    FunctionCall,

    /// This execution is a "special" function call, such as a getter or setter.
    Special,
}

/// Represents a function defined in the AVM1 runtime, either through
/// `DefineFunction` or `DefineFunction2`.
#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct Avm1Function<'gc> {
    /// The file format version of the SWF that generated this function.
    swf_version: u8,

    /// A reference to the underlying SWF data.
    data: SwfSlice,
    /// The name of the function, if not anonymous.
    name: Option<String>,

    /// The number of registers to allocate for this function's private register
    /// set. Any register beyond this ID will be served from the global one.
    register_count: u8,

    preload_parent: bool,
    preload_root: bool,
    suppress_super: bool,
    preload_super: bool,
    suppress_arguments: bool,
    preload_arguments: bool,
    suppress_this: bool,
    preload_this: bool,
    preload_global: bool,

    /// The names of the function parameters and their register mappings.
    /// r0 indicates that no register shall be written and the parameter stored
    /// as a Variable instead.
    params: Vec<(Option<u8>, String)>,

    /// The scope the function was born into.
    scope: GcCell<'gc, Scope<'gc>>,

    /// The constant pool the function executes with.
    constant_pool: GcCell<'gc, Vec<String>>,

    /// The base movie clip that the function was defined on.
    /// This is the movie clip that contains the bytecode.
    base_clip: DisplayObject<'gc>,
}

impl<'gc> Avm1Function<'gc> {
    /// Construct a function from a DefineFunction action.
    ///
    /// Parameters not specified in DefineFunction are filled with reasonable
    /// defaults.
    pub fn from_df1(
        swf_version: u8,
        actions: SwfSlice,
        name: &str,
        params: &[&str],
        scope: GcCell<'gc, Scope<'gc>>,
        constant_pool: GcCell<'gc, Vec<String>>,
        base_clip: DisplayObject<'gc>,
    ) -> Self {
        let name = match name {
            "" => None,
            name => Some(name.to_string()),
        };

        Avm1Function {
            swf_version,
            data: actions,
            name,
            register_count: 0,
            preload_parent: false,
            preload_root: false,
            suppress_super: false,
            preload_super: false,
            suppress_arguments: false,
            preload_arguments: false,
            suppress_this: false,
            preload_this: false,
            preload_global: false,
            params: params.iter().map(|&s| (None, s.to_string())).collect(),
            scope,
            constant_pool,
            base_clip,
        }
    }

    /// Construct a function from a DefineFunction2 action.
    pub fn from_df2(
        swf_version: u8,
        actions: SwfSlice,
        swf_function: &swf::avm1::types::Function,
        scope: GcCell<'gc, Scope<'gc>>,
        constant_pool: GcCell<'gc, Vec<String>>,
        base_clip: DisplayObject<'gc>,
    ) -> Self {
        let name = match swf_function.name {
            "" => None,
            name => Some(name.to_string()),
        };

        let mut owned_params = Vec::new();
        for FunctionParam {
            name: s,
            register_index: r,
        } in &swf_function.params
        {
            owned_params.push((*r, (*s).to_string()))
        }

        Avm1Function {
            swf_version,
            data: actions,
            name,
            register_count: swf_function.register_count,
            preload_parent: swf_function.preload_parent,
            preload_root: swf_function.preload_root,
            suppress_super: swf_function.suppress_super,
            preload_super: swf_function.preload_super,
            suppress_arguments: swf_function.suppress_arguments,
            preload_arguments: swf_function.preload_arguments,
            suppress_this: swf_function.suppress_this,
            preload_this: swf_function.preload_this,
            preload_global: swf_function.preload_global,
            params: owned_params,
            scope,
            constant_pool,
            base_clip,
        }
    }

    pub fn swf_version(&self) -> u8 {
        self.swf_version
    }

    pub fn data(&self) -> SwfSlice {
        self.data.clone()
    }

    pub fn scope(&self) -> GcCell<'gc, Scope<'gc>> {
        self.scope
    }

    pub fn register_count(&self) -> u8 {
        self.register_count
    }
}

/// Represents a function that can be defined in the Ruffle runtime or by the
/// AVM1 bytecode itself.
#[derive(Clone)]
pub enum Executable<'gc> {
    /// A function provided by the Ruffle runtime and implemented in Rust.
    Native(NativeFunction<'gc>),

    /// ActionScript data defined by a previous `DefineFunction` or
    /// `DefineFunction2` action.
    Action(Gc<'gc, Avm1Function<'gc>>),
}

unsafe impl<'gc> Collect for Executable<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Native(_) => {}
            Self::Action(af) => af.trace(cc),
        }
    }
}

impl fmt::Debug for Executable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Executable::Native(nf) => f
                .debug_tuple("Executable::Native")
                .field(&format!("{:p}", nf))
                .finish(),
            Executable::Action(af) => f.debug_tuple("Executable::Action").field(&af).finish(),
        }
    }
}

impl<'gc> Executable<'gc> {
    /// Execute the given code.
    ///
    /// Execution is not guaranteed to have completed when this function
    /// returns. If on-stack execution is possible, then this function returns
    /// a return value you must push onto the stack. Otherwise, you must
    /// create a new stack frame and execute the action data yourself.
    #[allow(clippy::too_many_arguments)]
    pub fn exec(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc>,
        ac: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
        reason: ExecutionReason,
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self {
            Executable::Native(nf) => nf(activation, ac, this, args),
            Executable::Action(af) => {
                let child_scope = GcCell::allocate(
                    ac.gc_context,
                    Scope::new_local_scope(af.scope(), ac.gc_context),
                );
                let arguments =
                    ScriptObject::array(ac.gc_context, Some(activation.avm.prototypes().array));
                arguments.define_value(ac.gc_context, "callee", callee.into(), DontEnum.into());

                if !af.suppress_arguments {
                    for i in 0..args.len() {
                        arguments.set_array_element(i, args.get(i).unwrap().clone(), ac.gc_context);
                    }
                }

                let argcell = arguments.into();
                let super_object: Option<Object<'gc>> = if !af.suppress_super {
                    Some(
                        SuperObject::from_this_and_base_proto(
                            this,
                            base_proto.unwrap_or(this),
                            activation,
                            ac,
                        )?
                        .into(),
                    )
                } else {
                    None
                };

                let effective_ver = if activation.current_swf_version() > 5 {
                    af.swf_version()
                } else {
                    this.as_display_object()
                        .map(|dn| dn.swf_version())
                        .unwrap_or(ac.player_version)
                };

                let name = if activation.avm.show_debug_output() {
                    let mut result = match &af.name {
                        None => name.to_string(),
                        Some(name) => name.to_string(),
                    };

                    result.push('(');
                    for i in 0..args.len() {
                        result.push_str(args.get(i).unwrap().type_of());
                        if i < args.len() - 1 {
                            result.push_str(", ");
                        }
                    }
                    result.push(')');

                    Cow::Owned(result)
                } else {
                    Cow::Borrowed("[Anonymous]")
                };

                let mut frame = Activation::from_action(
                    activation.avm,
                    activation
                        .id
                        .function(name, reason, activation.avm.max_recursion_depth())?,
                    effective_ver,
                    child_scope,
                    af.constant_pool,
                    af.base_clip,
                    this,
                    Some(argcell),
                );

                frame.allocate_local_registers(af.register_count(), ac.gc_context);

                let mut preload_r = 1;

                if af.preload_this {
                    //TODO: What happens if you specify both suppress and
                    //preload for this?
                    frame.set_local_register(preload_r, this, ac.gc_context);
                    preload_r += 1;
                }

                if af.preload_arguments {
                    //TODO: What happens if you specify both suppress and
                    //preload for arguments?
                    frame.set_local_register(preload_r, argcell, ac.gc_context);
                    preload_r += 1;
                }

                if let Some(super_object) = super_object {
                    if af.preload_super {
                        frame.set_local_register(preload_r, super_object, ac.gc_context);
                        //TODO: What happens if you specify both suppress and
                        //preload for super?
                        preload_r += 1;
                    } else {
                        frame.define("super", super_object, ac.gc_context);
                    }
                }

                if af.preload_root {
                    frame.set_local_register(
                        preload_r,
                        af.base_clip.root().object(),
                        ac.gc_context,
                    );
                    preload_r += 1;
                }

                if af.preload_parent {
                    // If _parent is undefined (because this is a root timeline), it actually does not get pushed,
                    // and _global ends up incorrectly taking _parent's register.
                    // See test for more info.
                    if let Some(parent) = af.base_clip.parent() {
                        frame.set_local_register(preload_r, parent.object(), ac.gc_context);
                        preload_r += 1;
                    }
                }

                if af.preload_global {
                    let global = frame.avm.global_object(ac);
                    frame.set_local_register(preload_r, global, ac.gc_context);
                }

                //TODO: What happens if the argument registers clash with the
                //preloaded registers? What gets done last?
                for i in 0..args.len() {
                    match (args.get(i), af.params.get(i)) {
                        (Some(arg), Some((Some(argreg), _argname))) => {
                            frame.set_local_register(*argreg, arg.clone(), ac.gc_context)
                        }
                        (Some(arg), Some((None, argname))) => {
                            frame.define(argname, arg.clone(), ac.gc_context)
                        }
                        _ => {}
                    }
                }

                Ok(frame.run_actions(ac, af.data.clone())?.value())
            }
        }
    }
}

impl<'gc> From<NativeFunction<'gc>> for Executable<'gc> {
    fn from(nf: NativeFunction<'gc>) -> Self {
        Executable::Native(nf)
    }
}

impl<'gc> From<Gc<'gc, Avm1Function<'gc>>> for Executable<'gc> {
    fn from(af: Gc<'gc, Avm1Function<'gc>>) -> Self {
        Executable::Action(af)
    }
}

pub const TYPE_OF_FUNCTION: &str = "function";

/// Represents an `Object` that holds executable code.
#[derive(Debug, Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FunctionObject<'gc> {
    /// The script object base.
    ///
    /// TODO: Can we move the object's data into our own struct?
    base: ScriptObject<'gc>,

    data: GcCell<'gc, FunctionObjectData<'gc>>,
}

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
struct FunctionObjectData<'gc> {
    /// The code that will be invoked when this object is called.
    function: Option<Executable<'gc>>,

    /// The value to be returned by `toString` and `valueOf`.
    primitive: Value<'gc>,
}

impl<'gc> FunctionObject<'gc> {
    /// Construct a function sans prototype.
    pub fn bare_function(
        gc_context: MutationContext<'gc, '_>,
        function: impl Into<Executable<'gc>>,
        fn_proto: Option<Object<'gc>>,
    ) -> Self {
        let base = ScriptObject::object(gc_context, fn_proto);

        FunctionObject {
            base,
            data: GcCell::allocate(
                gc_context,
                FunctionObjectData {
                    function: Some(function.into()),
                    primitive: "[type Function]".into(),
                },
            ),
        }
    }

    /// Construct a function from an executable and associated protos.
    ///
    /// Since prototypes need to link back to themselves, this function builds
    /// both objects itself and returns the function to you, fully allocated.
    ///
    /// `fn_proto` refers to the implicit proto of the function object, and the
    /// `prototype` refers to the explicit prototype of the function. If
    /// provided, the function and it's prototype will be linked to each other.
    pub fn function(
        context: MutationContext<'gc, '_>,
        function: impl Into<Executable<'gc>>,
        fn_proto: Option<Object<'gc>>,
        prototype: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let function = Self::bare_function(context, function, fn_proto).into();

        if let Some(p) = prototype {
            p.define_value(
                context,
                "constructor",
                Value::Object(function),
                DontEnum.into(),
            );
            function.define_value(context, "prototype", p.into(), EnumSet::empty());
        }

        function
    }
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.base.get_local(name, activation, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        self.base.set(name, value, activation, context)
    }

    fn call(
        &self,
        name: &str,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        if let Some(exec) = self.as_executable() {
            exec.exec(
                name,
                activation,
                context,
                this,
                base_proto,
                args,
                ExecutionReason::FunctionCall,
                (*self).into(),
            )
        } else {
            Ok(Value::Undefined)
        }
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.base.call_setter(name, value, activation, context)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        prototype: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error<'gc>> {
        let base = ScriptObject::object(context.gc_context, Some(prototype));
        let fn_object = FunctionObject {
            base,
            data: GcCell::allocate(
                context.gc_context,
                FunctionObjectData {
                    function: None,
                    primitive: "[type Function]".into(),
                },
            ),
        };

        Ok(fn_object.into())
    }

    fn delete(
        &self,
        activation: &mut Activation<'_, 'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
    ) -> bool {
        self.base.delete(activation, gc_context, name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.base.proto()
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>) {
        self.base.set_proto(gc_context, prototype);
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base.define_value(gc_context, name, value, attributes)
    }

    fn set_attributes(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: EnumSet<Attribute>,
        clear_attributes: EnumSet<Attribute>,
    ) {
        self.base
            .set_attributes(gc_context, name, set_attributes, clear_attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base
            .add_property_with_case(activation, gc_context, name, get, set, attributes)
    }

    fn set_watcher(
        &self,
        activation: &mut Activation<'_, 'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.base
            .set_watcher(activation, gc_context, name, callback, user_data);
    }

    fn remove_watcher(
        &self,
        activation: &mut Activation<'_, 'gc>,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
    ) -> bool {
        self.base.remove_watcher(activation, gc_context, name)
    }

    fn has_property(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base.has_property(activation, context, name)
    }

    fn has_own_property(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base.has_own_property(activation, context, name)
    }

    fn has_own_virtual(
        &self,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        name: &str,
    ) -> bool {
        self.base.has_own_virtual(activation, context, name)
    }

    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc>, name: &str) -> bool {
        self.base.is_property_enumerable(activation, name)
    }

    fn get_keys(&self, activation: &mut Activation<'_, 'gc>) -> Vec<String> {
        self.base.get_keys(activation)
    }

    fn as_string(&self) -> Cow<str> {
        Cow::Borrowed("[type Function]")
    }

    fn type_of(&self) -> &'static str {
        TYPE_OF_FUNCTION
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.base.interfaces()
    }

    /// Set the interface list for this object. (Only useful for prototypes.)
    fn set_interfaces(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        iface_list: Vec<Object<'gc>>,
    ) {
        self.base.set_interfaces(gc_context, iface_list)
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base)
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.data.read().function.clone()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.base.as_ptr()
    }

    fn length(&self) -> usize {
        self.base.length()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, new_length: usize) {
        self.base.set_length(gc_context, new_length)
    }

    fn array(&self) -> Vec<Value<'gc>> {
        self.base.array()
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        self.base.array_element(index)
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.base.set_array_element(index, value, gc_context)
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        self.base.delete_array_element(index, gc_context)
    }
}
