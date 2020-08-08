//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::method::{BytecodeMethod, Method, NativeMethod};
use crate::avm2::object::Object;
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, CollectionContext, Gc, GcCell, MutationContext};
use std::fmt;

/// Represents code written in AVM2 bytecode that can be executed by some
/// means.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BytecodeExecutable<'gc> {
    /// The method code to execute from a given ABC file.
    method: Gc<'gc, BytecodeMethod<'gc>>,

    /// The scope stack to pull variables from.
    scope: Option<GcCell<'gc, Scope<'gc>>>,

    /// The reciever that this function is always called with.
    ///
    /// If `None`, then the reciever provided by the caller is used. A
    /// `Some` value indicates a bound executable.
    reciever: Option<Object<'gc>>,
}

/// Represents code that can be executed by some means.
#[derive(Copy, Clone)]
pub enum Executable<'gc> {
    /// Code defined in Ruffle's binary.
    ///
    /// The second parameter stores the bound reciever for this function.
    Native(NativeMethod<'gc>, Option<Object<'gc>>),

    /// Code defined in a loaded ABC file.
    Action(Gc<'gc, BytecodeExecutable<'gc>>),
}

unsafe impl<'gc> Collect for Executable<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Action(be) => be.trace(cc),
            Self::Native(_nf, reciever) => reciever.trace(cc),
        }
    }
}

impl<'gc> Executable<'gc> {
    /// Convert a method into an executable.
    pub fn from_method(
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        reciever: Option<Object<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        match method {
            Method::Native(nf) => Self::Native(nf, reciever),
            Method::Entry(method) => Self::Action(Gc::allocate(
                mc,
                BytecodeExecutable {
                    method,
                    scope,
                    reciever,
                },
            )),
        }
    }

    /// Execute a method.
    ///
    /// The function will either be called directly if it is a Rust builtin, or
    /// executed on the same AVM2 instance as the activation passed in here.
    /// The value returned in either case will be provided here.
    ///
    /// It is a panicing logic error to attempt to execute user code while any
    /// reachable object is currently under a GcCell write lock.
    pub fn exec(
        &self,
        unbound_reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        match self {
            Executable::Native(nf, reciever) => {
                nf(activation, reciever.or(unbound_reciever), arguments)
            }
            Executable::Action(bm) => {
                let reciever = bm.reciever.or(unbound_reciever);
                let mut activation = Activation::from_method(
                    activation.context.reborrow(),
                    bm.method,
                    bm.scope,
                    reciever,
                    arguments,
                    base_proto,
                )?;

                activation.run_actions(bm.method)
            }
        }
    }
}

impl<'gc> fmt::Debug for Executable<'gc> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Action(be) => fmt
                .debug_struct("Executable::Action")
                .field("method", &be.method)
                .field("scope", &be.scope)
                .field("reciever", &be.reciever)
                .finish(),
            Self::Native(nf, reciever) => fmt
                .debug_tuple("Executable::Native")
                .field(&format!("{:p}", nf))
                .field(reciever)
                .finish(),
        }
    }
}
