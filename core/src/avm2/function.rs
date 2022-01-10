//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::method::{
    BytecodeMethod, Method, MethodKind, MethodMetadata, MethodPosition, NativeMethod,
};
use crate::avm2::object::{ClassObject, Object};
use crate::avm2::scope::ScopeChain;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::WString;
use gc_arena::{Collect, Gc, MutationContext};
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

    /// The metadata of this method. A value of None indicates that this is an
    /// anonymous function.
    meta: Option<MethodMetadata<'gc>>,
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

    /// The metadata of this method. A value of None indicates that this is an
    /// anonymous function.
    meta: Option<MethodMetadata<'gc>>,
}

/// Represents code that can be executed by some means.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub enum Executable<'gc> {
    /// Code defined in Ruffle's binary.
    Native(Gc<'gc, NativeExecutable<'gc>>),

    /// Code defined in a loaded ABC file.
    Action(Gc<'gc, BytecodeExecutable<'gc>>),
}

impl<'gc> Executable<'gc> {
    /// Convert a method into an executable.
    pub fn from_method(
        method: Method<'gc>,
        scope: ScopeChain<'gc>,
        receiver: Option<Object<'gc>>,
        superclass: Option<ClassObject<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        match method {
            Method::Native(method, meta) => Self::Native(Gc::allocate(
                mc,
                NativeExecutable {
                    method,
                    scope,
                    bound_receiver: receiver,
                    bound_superclass: superclass,
                    meta,
                },
            )),
            Method::Bytecode(method, meta) => Self::Action(Gc::allocate(
                mc,
                BytecodeExecutable {
                    method,
                    scope,
                    receiver,
                    bound_superclass: superclass,
                    meta,
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
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error> {
        let ret = match self {
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
                    bm.method.name,
                    arguments,
                    &bm.method.signature,
                )?;
                activation.context.avm2.push_call(*self)?;
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
                activation.context.avm2.push_call(*self)?;
                activation.run_actions(bm.method)
            }
        };
        activation.context.avm2.pop_call();
        ret
    }

    pub fn bound_superclass(&self) -> Option<ClassObject<'gc>> {
        match self {
            Executable::Native(ne) => ne.bound_superclass,
            Executable::Action(be) => be.bound_superclass,
        }
    }

    pub fn meta(&self) -> Option<MethodMetadata<'gc>> {
        match self {
            Executable::Native(ne) => ne.meta,
            Executable::Action(be) => be.meta,
        }
    }

    pub fn write_full_name(&self, mc: MutationContext<'gc, '_>, output: &mut WString) {
        if let Some(superclass) = self.bound_superclass() {
            let class_def = superclass.inner_class_definition();
            let name = class_def.read().name().to_qualified_name(mc);
            output.push_str(&name);
        }
        if let Some(meta) = self.meta() {
            if meta.position() == MethodPosition::ClassTrait {
                output.push_char('$');
            }

            let prefix = match meta.kind() {
                MethodKind::Setter => "/set ",
                MethodKind::Getter => "/get ",
                MethodKind::Regular => "/",
                MethodKind::Initializer => "",
            };
            output.push_utf8(prefix);
            output.push_str(&meta.name());
        } else {
            match self {
                Executable::Native(ne) => output.push_utf8(ne.method.name),
                Executable::Action(be) => {
                    let name = be.method.method_name();
                    if name.is_empty() {
                        output.push_utf8("MethodInfo-");
                        output.push_utf8(&be.method.abc_method.to_string());
                    } else {
                        output.push_utf8(name)
                    }
                }
            }
        }

        output.push_utf8("()");
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
