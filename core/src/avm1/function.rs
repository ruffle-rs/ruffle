//! Code relating to executable functions + calling conventions.

use gc_arena::GcCell;
use swf::avm1::types::FunctionParam;
use crate::tag_utils::SwfSlice;
use crate::avm1::{Avm1, ActionContext};
use crate::avm1::object::Object;
use crate::avm1::value::Value;
use crate::avm1::scope::Scope;

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
    scope: GcCell<'gc, Scope<'gc>>
}

impl<'gc> Avm1Function<'gc> {
    pub fn new(swf_version: u8, actions: SwfSlice, name: &str, params: &[&str], scope: GcCell<'gc, Scope<'gc>>) -> Self {
        let name = match name {
            "" => None,
            name => Some(name.to_string())
        };

        Avm1Function {
            swf_version: swf_version,
            data: actions,
            name: name,
            params: params.into_iter().map(|s| s.to_string()).collect(),
            scope: scope
        }
    }

    pub fn swf_version(&self) -> u8 {
        self.swf_version
    }

    pub fn data(&self) -> SwfSlice {
        self.data.clone()
    }

    pub fn scope(&self) -> GcCell<'gc, Scope<'gc>> {
        self.scope.clone()
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
    supress_super: bool,
    preload_super: bool,
    supress_arguments: bool,
    preload_arguments: bool,
    supress_this: bool,
    preload_this: bool,

    /// The names of the function parameters and their register mappings.
    /// r0 indicates that no register shall be written and the parameter stored
    /// as a Variable instead.
    params: Vec<(Option<u8>, String)>,

    /// The scope the function was born into.
    scope: GcCell<'gc, Scope<'gc>>
}

impl<'gc> Avm1Function2<'gc> {
    pub fn new(swf_version: u8,
        actions: SwfSlice,
        name: &str,
        register_count: u8,
        preload_parent: bool,
        preload_root: bool,
        supress_super: bool,
        preload_super: bool,
        supress_arguments: bool,
        preload_arguments: bool,
        supress_this: bool,
        preload_this: bool,
        params: &Vec<FunctionParam>,
        scope: GcCell<'gc, Scope<'gc>>) -> Self {
        
        let name = match name {
            "" => None,
            name => Some(name.to_string())
        };

        let mut owned_params = Vec::new();
        for FunctionParam{name: s, register_index: r} in params.into_iter() {
            owned_params.push((r.clone(), s.to_string()))
        }

        Avm1Function2 {
            swf_version: swf_version,
            data: actions,
            name: name,
            register_count: register_count,
            preload_parent: preload_parent,
            preload_root: preload_root,
            supress_super: supress_super,
            preload_super: preload_super,
            supress_arguments: supress_arguments,
            preload_arguments: preload_arguments,
            supress_this: supress_this,
            preload_this: preload_this,
            params: owned_params,
            scope: scope
        }
    }

    pub fn swf_version(&self) -> u8 {
        self.swf_version
    }

    pub fn data(&self) -> SwfSlice {
        self.data.clone()
    }

    pub fn scope(&self) -> GcCell<'gc, Scope<'gc>> {
        self.scope.clone()
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
    Action2(Avm1Function2<'gc>)
}

impl<'gc> Executable<'gc> {
    /// Execute the given code.
    /// 
    /// Execution is not guaranteed to have completed when this function
    /// returns. If on-stack execution is possible, then this function returns
    /// a return value you must push onto the stack. Otherwise, you must
    /// create a new stack frame and execute the action data yourself.
    pub fn exec(&self, avm: &mut Avm1<'gc>, ac: &mut ActionContext<'_, 'gc, '_>, this: GcCell<'gc, Object<'gc>>, args: &[Value<'gc>]) -> Option<Value<'gc>> {
        match self {
            Executable::Native(nf) => Some(nf(avm, ac, this, args)),
            Executable::Action(af) => {
                let mut arguments = Object::object(ac.gc_context);

                for i in 0..args.len() {
                    arguments.force_set(&format!("{}", i), args.get(i).unwrap().clone());
                }

                arguments.force_set("length", Value::Number(args.len() as f64));

                avm.insert_stack_frame_for_function(af, this, GcCell::allocate(ac.gc_context, arguments), ac);

                for i in 0..args.len() {
                    if let Some(argname) = af.params.get(i) {
                        avm.current_stack_frame_mut().unwrap().define(argname, args.get(i).unwrap().clone(), ac.gc_context);
                    }
                }

                None
            },
            Executable::Action2(af) => {
                let mut arguments = Object::object(ac.gc_context);
                if !af.supress_arguments {
                    for i in 0..args.len() {
                        arguments.set(&format!("{}", i), args.get(i).unwrap().clone())
                    }

                    arguments.set("length", Value::Number(args.len() as f64));
                }

                avm.insert_stack_frame_for_function2(af, this, GcCell::allocate(ac.gc_context, arguments), ac);

                for i in 0..args.len() {
                    match (args.get(i), af.params.get(i)) {
                        (Some(arg), Some((Some(argreg), argname))) => avm.set_current_register(*argreg, arg.clone(), ac),
                        (Some(arg), Some((None, argname))) => avm.current_stack_frame_mut().unwrap().define(argname, arg.clone(), ac.gc_context),
                        (Some(arg), _) => {},
                        _ => panic!("Missing argument value")
                    }
                }

                None
            }
        }
    }
}