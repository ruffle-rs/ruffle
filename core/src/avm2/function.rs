//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::method::{BytecodeMethod, Method, NativeMethod, ParamConfig};
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

    /// The receiver that this function is always called with.
    ///
    /// If `None`, then the receiver provided by the caller is used. A
    /// `Some` value indicates a bound executable.
    receiver: Option<Object<'gc>>,
}

/// Represents code that can be executed by some means.
#[derive(Clone)]
pub enum Executable<'gc> {
    /// Code defined in Ruffle's binary.
    Native {
        /// The function to call to execute the method.
        method: NativeMethod,

        /// The name of the method.
        name: &'static str,

        /// The bound reciever for this method.
        bound_receiver: Option<Object<'gc>>,

        /// The parameter signature of the method.
        signature: Gc<'gc, Vec<ParamConfig<'gc>>>,

        /// Whether or not this method accepts parameters beyond those
        /// mentioned in the parameter list.
        is_variadic: bool,
    },

    /// Code defined in a loaded ABC file.
    Action(Gc<'gc, BytecodeExecutable<'gc>>),
}

unsafe impl<'gc> Collect for Executable<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Action(be) => be.trace(cc),
            Self::Native {
                bound_receiver,
                signature,
                ..
            } => {
                bound_receiver.trace(cc);
                signature.trace(cc);
            }
        }
    }
}

impl<'gc> Executable<'gc> {
    /// Convert a method into an executable.
    pub fn from_method(
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        receiver: Option<Object<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        match method {
            Method::Native {
                method,
                name,
                signature,
                is_variadic,
            } => Self::Native {
                method,
                name,
                bound_receiver: receiver,
                signature,
                is_variadic,
            },
            Method::Entry(method) => Self::Action(Gc::allocate(
                mc,
                BytecodeExecutable {
                    method,
                    scope,
                    receiver,
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
    /// It is a panicking logic error to attempt to execute user code while any
    /// reachable object is currently under a GcCell write lock.
    ///
    /// Passed-in arguments will be conformed to the set of method parameters
    /// declared on the function.
    pub fn exec(
        &self,
        unbound_receiver: Option<Object<'gc>>,
        mut arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        base_constr: Option<Object<'gc>>,
        callee: Object<'gc>,
        error_on_too_many_arguments: bool,
    ) -> Result<Value<'gc>, Error> {
        match self {
            Executable::Native {
                method,
                name,
                bound_receiver,
                signature,
                is_variadic,
            } => {
                let receiver = bound_receiver.or(unbound_receiver);
                let scope = activation.scope();
                let mut activation = Activation::from_builtin(
                    activation.context.reborrow(),
                    scope,
                    receiver,
                    base_constr,
                )?;

                if arguments.len() > signature.len() && !is_variadic {
                    return Err(format!(
                        "Attempted to call {:?} with {} arguments (more than {} is prohibited)",
                        name,
                        arguments.len(),
                        signature.len()
                    )
                    .into());
                }

                let arguments = activation.resolve_parameters(name, arguments, &signature)?;

                method(&mut activation, receiver, &arguments)
            }
            Executable::Action(bm) => {
                if !error_on_too_many_arguments {
                    let max_args = bm.method.signature().len();
                    if arguments.len() > max_args && !bm.method.is_variadic() {
                        arguments = &arguments[..max_args];
                    }
                }

                let receiver = bm.receiver.or(unbound_receiver);
                let mut activation = Activation::from_method(
                    activation.context.reborrow(),
                    bm.method,
                    bm.scope,
                    receiver,
                    arguments,
                    base_constr,
                    callee,
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
                .field("receiver", &be.receiver)
                .finish(),
            Self::Native {
                method,
                name,
                bound_receiver,
                signature,
                is_variadic,
            } => fmt
                .debug_struct("Executable::Native")
                .field("method", &format!("{:p}", method))
                .field("name", name)
                .field("bound_receiver", bound_receiver)
                .field("signature", signature)
                .field("is_variadic", is_variadic)
                .finish(),
        }
    }
}
