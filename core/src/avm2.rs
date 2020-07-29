//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::activation::Activation;
use crate::avm2::globals::SystemPrototypes;
use crate::avm2::object::{Object, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::script::Script;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::Value;
use crate::context::UpdateContext;
use crate::tag_utils::SwfSlice;
use gc_arena::{Collect, GcCell, MutationContext};
use std::rc::Rc;
use swf::avm2::read::Reader;

#[macro_export]
macro_rules! avm_debug {
    ($avm: expr, $($arg:tt)*) => (
        if $avm.show_debug_output() {
            log::debug!($($arg)*)
        }
    )
}

mod activation;
mod class;
mod function;
mod globals;
mod method;
mod names;
mod namespace_object;
mod object;
mod primitive_object;
mod property;
mod property_map;
mod return_value;
mod scope;
mod script;
mod script_object;
mod slot;
mod string;
mod r#trait;
mod value;

/// Boxed error alias.
///
/// As AVM2 is a far stricter VM than AVM1, this may eventually be replaced
/// with a proper Avm2Error enum.
type Error = Box<dyn std::error::Error>;

/// The state of an AVM2 interpreter.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm2<'gc> {
    /// Values currently present on the operand stack.
    stack: Vec<Value<'gc>>,

    /// Global scope object.
    globals: Object<'gc>,

    /// System prototypes.
    system_prototypes: SystemPrototypes<'gc>,

    #[cfg(feature = "avm_debug")]
    pub debug_output: bool,
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let (globals, system_prototypes) = globals::construct_global_scope(mc);

        Self {
            stack: Vec::new(),
            globals,
            system_prototypes,

            #[cfg(feature = "avm_debug")]
            debug_output: false,
        }
    }

    /// Return the current set of system prototypes.
    pub fn prototypes(&self) -> &SystemPrototypes<'gc> {
        &self.system_prototypes
    }

    /// Run a script's initializer method.
    pub fn run_script_initializer(
        script: GcCell<'gc, Script<'gc>>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let globals = context.avm2.globals;
        let mut init_activation = Activation::from_script(context.reborrow(), script, globals)?;

        init_activation.run_stack_frame_for_script(script)
    }

    /// Load an ABC file embedded in a `SwfSlice`.
    ///
    /// The `SwfSlice` must resolve to the contents of an ABC file.
    pub fn load_abc(
        abc: SwfSlice,
        _abc_name: &str,
        _lazy_init: bool,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut read = Reader::new(abc.as_ref());

        let abc_file = Rc::new(read.read()?);
        let tunit = TranslationUnit::from_abc(abc_file.clone(), context.gc_context);

        for i in (0..abc_file.scripts.len()).rev() {
            let script = tunit.load_script(i as u32, context.gc_context)?;
            let mut globals = context.avm2.globals();
            let scope = Scope::push_scope(None, globals, context.gc_context);
            let mut null_activation = Activation::from_nothing(context.reborrow());

            // TODO: Lazyinit means we shouldn't do this until traits are
            // actually mentioned...
            for trait_entry in script.read().traits()?.iter() {
                globals.install_foreign_trait(
                    &mut null_activation,
                    trait_entry.clone(),
                    Some(scope),
                    globals,
                )?;
            }

            drop(null_activation);

            Self::run_script_initializer(script, context)?;
        }

        Ok(())
    }

    pub fn globals(&self) -> Object<'gc> {
        self.globals
    }

    /// Push a value onto the operand stack.
    fn push(&mut self, value: impl Into<Value<'gc>>) {
        let value = value.into();
        avm_debug!(self, "Stack push {}: {:?}", self.stack.len(), value);
        self.stack.push(value);
    }

    /// Retrieve the top-most value on the operand stack.
    #[allow(clippy::let_and_return)]
    fn pop(&mut self) -> Value<'gc> {
        let value = self.stack.pop().unwrap_or_else(|| {
            log::warn!("Avm1::pop: Stack underflow");
            Value::Undefined
        });

        avm_debug!(self, "Stack pop {}: {:?}", self.stack.len(), value);

        value
    }

    fn pop_args(&mut self, arg_count: u32) -> Vec<Value<'gc>> {
        let mut args = Vec::with_capacity(arg_count as usize);
        args.resize(arg_count as usize, Value::Undefined);
        for arg in args.iter_mut().rev() {
            *arg = self.pop();
        }

        args
    }

    #[cfg(feature = "avm_debug")]
    #[inline]
    pub fn show_debug_output(&self) -> bool {
        self.debug_output
    }

    #[cfg(not(feature = "avm_debug"))]
    pub const fn show_debug_output(&self) -> bool {
        false
    }

    #[cfg(feature = "avm_debug")]
    pub fn set_show_debug_output(&mut self, visible: bool) {
        self.debug_output = visible;
    }

    #[cfg(not(feature = "avm_debug"))]
    pub const fn set_show_debug_output(&self, _visible: bool) {}
}
