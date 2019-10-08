//! Code relating to executable functions + calling conventions.

use crate::avm1::activation::Activation;
use crate::avm1::object::{Attribute, Object};
use crate::avm1::scope::Scope;
use crate::avm1::value::Value;
use crate::avm1::{ActionContext, Avm1};
use crate::tag_utils::SwfSlice;
use gc_arena::GcCell;
use swf::avm1::types::FunctionParam;

pub type NativeFunction<'gc> = fn(
    &mut Avm1<'gc>,
    &mut ActionContext<'_, 'gc, '_>,
    GcCell<'gc, Object<'gc>>,
    &[Value<'gc>],
) -> Value<'gc>;

/// Represents a function defined in the AVM1 runtime.
#[derive(Clone)]
pub struct Avm1Function<'gc> {
    /// The file format version of the SWF that generated this function.
    swf_version: u8,

    /// A reference to the underlying SWF data.
    data: SwfSlice,
    /// The name of the function, if not anonymous.
    name: Option<String>,

    /// The names of the function parameters.
    params: Vec<String>,

    /// The scope the function was born into.
    scope: GcCell<'gc, Scope<'gc>>,
}

impl<'gc> Avm1Function<'gc> {
    pub fn new(
        swf_version: u8,
        actions: SwfSlice,
        name: &str,
        params: &[&str],
        scope: GcCell<'gc, Scope<'gc>>,
    ) -> Self {
        let name = match name {
            "" => None,
            name => Some(name.to_string()),
        };

        Avm1Function {
            swf_version,
            data: actions,
            name,
            params: params.iter().map(|s| s.to_string()).collect(),
            scope,
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
}

unsafe impl<'gc> gc_arena::Collect for Avm1Function<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.scope.trace(cc);
    }
}

/// Represents a function defined in the AVM1 runtime's `ActionDefineFunction2`
/// opcode.
#[derive(Clone)]
pub struct Avm1Function2<'gc> {
    /// The file format version of the SWF that generated this function.
    swf_version: u8,

    /// A reference to the underlying SWF data.
    data: SwfSlice,
    /// The name of the function, if not anonymous.
    name: Option<String>,

    /// The number of registers to allocate for this function's private register
    /// set.
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
}

impl<'gc> Avm1Function2<'gc> {
    pub fn new(
        swf_version: u8,
        actions: SwfSlice,
        swf_function: &swf::avm1::types::Function,
        scope: GcCell<'gc, Scope<'gc>>,
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
            owned_params.push((*r, s.to_string()))
        }

        Avm1Function2 {
            swf_version,
            data: actions,
            name,
            register_count: swf_function.params.capacity() as u8,
            preload_parent: swf_function.preload_parent,
            preload_root: swf_function.preload_root,
            suppress_super: swf_function.suppress_super,
            preload_super: swf_function.preload_super,
            suppress_arguments: swf_function.suppress_super,
            preload_arguments: swf_function.preload_arguments,
            suppress_this: swf_function.suppress_this,
            preload_this: swf_function.preload_this,
            preload_global: swf_function.preload_global,
            params: owned_params,
            scope,
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

    /// ActionScript data defined by a previous `DefineFunction` action.
    Action(Avm1Function<'gc>),

    /// ActionScript data defined by a previous `DefineFunction2` action.
    Action2(Avm1Function2<'gc>),
}

impl<'gc> Executable<'gc> {
    /// Execute the given code.
    ///
    /// Execution is not guaranteed to have completed when this function
    /// returns. If on-stack execution is possible, then this function returns
    /// a return value you must push onto the stack. Otherwise, you must
    /// create a new stack frame and execute the action data yourself.
    pub fn exec(
        &self,
        avm: &mut Avm1<'gc>,
        ac: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Option<Value<'gc>> {
        match self {
            Executable::Native(nf) => Some(nf(avm, ac, this, args)),
            Executable::Action(af) => {
                let mut arguments = Object::object(ac.gc_context);

                for i in 0..args.len() {
                    arguments.force_set(
                        &format!("{}", i),
                        args.get(i).unwrap().clone(),
                        Attribute::DontDelete,
                    );
                }

                arguments.force_set(
                    "length",
                    Value::Number(args.len() as f64),
                    Attribute::DontDelete | Attribute::DontEnum,
                );
                let argcell = GcCell::allocate(ac.gc_context, arguments);
                let child_scope = GcCell::allocate(
                    ac.gc_context,
                    Scope::new_local_scope(af.scope(), ac.gc_context),
                );

                for i in 0..args.len() {
                    if let Some(argname) = af.params.get(i) {
                        child_scope.write(ac.gc_context).define(
                            argname,
                            args.get(i).unwrap().clone(),
                            ac.gc_context,
                        );
                    }
                }

                let frame = Activation::from_function(
                    af.swf_version(),
                    af.data(),
                    child_scope,
                    this,
                    Some(argcell),
                );
                avm.insert_stack_frame(frame);

                None
            }
            Executable::Action2(af) => {
                let child_scope = GcCell::allocate(
                    ac.gc_context,
                    Scope::new_local_scope(af.scope(), ac.gc_context),
                );
                let mut arguments = Object::object(ac.gc_context);
                if !af.suppress_arguments {
                    for i in 0..args.len() {
                        arguments.force_set(
                            &format!("{}", i),
                            args.get(i).unwrap().clone(),
                            Attribute::DontDelete,
                        )
                    }

                    arguments.force_set(
                        "length",
                        Value::Number(args.len() as f64),
                        Attribute::DontDelete | Attribute::DontEnum,
                    );
                }

                let argcell = GcCell::allocate(ac.gc_context, arguments);
                let mut frame = Activation::from_function(
                    af.swf_version(),
                    af.data(),
                    child_scope,
                    this,
                    Some(argcell),
                );
                let mut preload_r = 1;

                if af.preload_this {
                    //TODO: What happens if you specify both suppress and
                    //preload for this?
                    frame.set_local_register(preload_r, Value::Object(this), ac.gc_context);
                    preload_r += 1;
                }

                if af.preload_arguments {
                    //TODO: What happens if you specify both suppress and
                    //preload for arguments?
                    frame.set_local_register(preload_r, Value::Object(argcell), ac.gc_context);
                    preload_r += 1;
                }

                if af.preload_super {
                    //TODO: super not implemented
                    log::warn!("Cannot preload super into register because it's not implemented");
                    //TODO: What happens if you specify both suppress and
                    //preload for super?
                    preload_r += 1;
                }

                if af.preload_root {
                    frame.set_local_register(preload_r, avm.root_object(ac), ac.gc_context);
                    preload_r += 1;
                }

                if af.preload_parent {
                    //TODO: _parent not implemented
                    log::warn!("Cannot preload parent into register because it's not implemented");
                    preload_r += 1;
                }

                if af.preload_global {
                    frame.set_local_register(preload_r, avm.global_object(ac), ac.gc_context);
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
                frame.allocate_local_registers(af.register_count(), ac.gc_context);
                avm.insert_stack_frame(frame);

                None
            }
        }
    }
}
