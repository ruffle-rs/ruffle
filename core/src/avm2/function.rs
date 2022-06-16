//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::method::{BytecodeMethod, Method, NativeMethod};
use crate::avm2::object::{ClassObject, Object};
use crate::avm2::scope::ScopeChain;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, Gc};
use std::fmt;

/// Represents code written in AVM2 bytecode that can be executed by some
/// means.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BytecodeExecutable<'gc> {
    /// The method code to execute from a given ABC file.
    method: Gc<'gc, BytecodeMethod<'gc>>,

    /// The scope this method was defined in.
    scope: ScopeChain<'gc>,

    /// The receiver that this function is always called with.
    ///
    /// If `None`, then the receiver provided by the caller is used. A
    /// `Some` value indicates a bound executable.
    receiver: Option<Object<'gc>>,

    /// The bound superclass for this method.
    ///
    /// The `superclass` is the class that defined this method. If `None`,
    /// then there is no defining superclass and `super` operations should fall
    /// back to the `receiver`.
    bound_superclass: Option<ClassObject<'gc>>,
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct NativeExecutable<'gc> {
    /// The method associated with the executable.
    method: Gc<'gc, NativeMethod<'gc>>,

    /// The scope this method was defined in.
    scope: ScopeChain<'gc>,

    /// The bound reciever for this method.
    bound_receiver: Option<Object<'gc>>,

    /// The bound superclass for this method.
    ///
    /// The `superclass` is the class that defined this method. If `None`,
    /// then there is no defining superclass and `super` operations should fall
    /// back to the `receiver`.
    bound_superclass: Option<ClassObject<'gc>>,
}

/// Represents code that can be executed by some means.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum Executable<'gc> {
    /// Code defined in Ruffle's binary.
    Native(NativeExecutable<'gc>),

    /// Code defined in a loaded ABC file.
    Action(BytecodeExecutable<'gc>),
}

impl<'gc> Executable<'gc> {
    /// Convert a method into an executable.
    pub fn from_method(
        method: Method<'gc>,
        scope: ScopeChain<'gc>,
        receiver: Option<Object<'gc>>,
        superclass: Option<ClassObject<'gc>>,
    ) -> Self {
        match method {
            Method::Native(method) => Self::Native(NativeExecutable {
                method,
                scope,
                bound_receiver: receiver,
                bound_superclass: superclass,
            }),
            Method::Bytecode(method) => Self::Action(BytecodeExecutable {
                method,
                scope,
                receiver,
                bound_superclass: superclass,
            }),
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
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error> {
        match self {
            Executable::Native(bm) => {
                let method = bm.method.method;
                let receiver = bm.bound_receiver.or(unbound_receiver);
                let caller_domain = activation.caller_domain();
                let subclass_object = bm.bound_superclass;
                let mut activation = Activation::from_builtin(
                    activation.context.reborrow(),
                    receiver,
                    subclass_object,
                    bm.scope,
                    caller_domain,
                )?;

                if arguments.len() > bm.method.signature.len() && !bm.method.is_variadic {
                    return Err(format!(
                        "Attempted to call {:?} with {} arguments (more than {} is prohibited)",
                        bm.method.name,
                        arguments.len(),
                        bm.method.signature.len()
                    )
                    .into());
                }

                let arguments = activation.resolve_parameters(
                    &bm.method.name,
                    arguments,
                    &bm.method.signature,
                )?;

                method(&mut activation, receiver, &arguments)
            }
            Executable::Action(bm) => {
                if bm.method.is_unchecked() {
                    let max_args = bm.method.signature().len();
                    if arguments.len() > max_args && !bm.method.is_variadic() {
                        arguments = &arguments[..max_args];
                    }
                }

                let receiver = bm.receiver.or(unbound_receiver);
                let subclass_object = bm.bound_superclass;

                let mut activation = Activation::from_method(
                    activation.context.reborrow(),
                    bm.method,
                    bm.scope,
                    receiver,
                    arguments,
                    subclass_object,
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
            Self::Native(bm) => fmt
                .debug_struct("Executable::Native")
                .field("method", &bm.method)
                .field("bound_receiver", &bm.bound_receiver)
                .finish(),
        }
    }
}
