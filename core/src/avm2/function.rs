use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, ParamConfig};
use crate::avm2::object::{ClassObject, Object};
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::TraitKind;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use crate::string::WString;
use gc_arena::{Collect, Gc};
use std::fmt;

/// Represents a bound method.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BoundMethod<'gc> {
    /// The method code to execute from a given ABC file.
    method: Method<'gc>,

    /// The scope this method was defined in.
    scope: ScopeChain<'gc>,

    /// The receiver that this function is always called with.
    ///
    /// If `None`, then the receiver provided by the caller is used. A
    /// `Some` value indicates a bound executable.
    bound_receiver: Option<Object<'gc>>,

    /// The superclass of the bound class for this method.
    ///
    /// The `bound_superclass` is the superclass of the class that defined
    /// this method. If `None`, then there is no defining class and `super`
    /// operations should be invalid.
    bound_superclass: Option<ClassObject<'gc>>,

    bound_class: Option<Class<'gc>>,
}

impl<'gc> BoundMethod<'gc> {
    pub fn from_method(
        method: Method<'gc>,
        scope: ScopeChain<'gc>,
        receiver: Option<Object<'gc>>,
        superclass: Option<ClassObject<'gc>>,
        class: Option<Class<'gc>>,
    ) -> Self {
        Self {
            method,
            scope,
            bound_receiver: receiver,
            bound_superclass: superclass,
            bound_class: class,
        }
    }

    pub fn exec(
        &self,
        unbound_receiver: Value<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        callee: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let receiver = if let Some(receiver) = self.bound_receiver {
            receiver
        } else if matches!(unbound_receiver, Value::Null | Value::Undefined) {
            self.scope
                .get(0)
                .expect("No global scope for function call")
                .values()
        } else {
            unbound_receiver.coerce_to_object(activation)?
        };

        exec(
            self.method,
            self.scope,
            receiver,
            self.bound_superclass,
            self.bound_class,
            arguments,
            activation,
            callee,
        )
    }

    pub fn bound_class(&self) -> Option<Class<'gc>> {
        self.bound_class
    }

    pub fn as_method(&self) -> Method<'gc> {
        self.method
    }

    pub fn debug_full_name(&self) -> WString {
        let mut output = WString::new();
        display_function(&mut output, &self.as_method(), self.bound_class());
        output
    }

    pub fn num_parameters(&self) -> usize {
        match self.method {
            Method::Native(method) => method.signature.len(),
            Method::Bytecode(method) => method.signature.len(),
        }
    }

    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        match &self.method {
            Method::Native(method) => &method.signature,
            Method::Bytecode(method) => method.signature(),
        }
    }

    pub fn is_variadic(&self) -> bool {
        match self.method {
            Method::Native(method) => method.is_variadic,
            Method::Bytecode(method) => method.is_variadic(),
        }
    }

    pub fn return_type(&self) -> &Multiname<'gc> {
        match &self.method {
            Method::Native(method) => &method.return_type,
            Method::Bytecode(method) => &method.return_type,
        }
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
#[allow(clippy::too_many_arguments)]
pub fn exec<'gc>(
    method: Method<'gc>,
    scope: ScopeChain<'gc>,
    receiver: Object<'gc>,
    bound_superclass: Option<ClassObject<'gc>>,
    bound_class: Option<Class<'gc>>,
    mut arguments: &[Value<'gc>],
    activation: &mut Activation<'_, 'gc>,
    callee: Object<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let ret = match method {
        Method::Native(bm) => {
            let caller_domain = activation.caller_domain();
            let caller_movie = activation.caller_movie();
            let mut activation = Activation::from_builtin(
                activation.context,
                bound_superclass,
                bound_class,
                scope,
                caller_domain,
                caller_movie,
            );

            if arguments.len() > bm.signature.len() && !bm.is_variadic {
                return Err(format!(
                    "Attempted to call {:?} with {} arguments (more than {} is prohibited)",
                    bm.name,
                    arguments.len(),
                    bm.signature.len()
                )
                .into());
            }

            if bm.resolved_signature.read().is_none() {
                bm.resolve_signature(&mut activation)?;
            }

            let resolved_signature = bm.resolved_signature.read();
            let resolved_signature = resolved_signature.as_ref().unwrap();

            let arguments = activation.resolve_parameters(
                method,
                arguments,
                resolved_signature,
                bound_class,
            )?;

            #[cfg(feature = "tracy_avm")]
            let _span = {
                let mut name = WString::new();
                display_function(&mut name, &method, bound_class);
                let span = tracy_client::Client::running()
                    .expect("tracy_client should be running")
                    .span_alloc(None, &name.to_utf8_lossy(), "rust", 0, 0);
                span.emit_color(0x2c4980);
                span
            };

            activation
                .context
                .avm2
                .push_call(activation.context.gc_context, method, bound_class);
            (bm.method)(&mut activation, receiver, &arguments)
        }
        Method::Bytecode(bm) => {
            if bm.is_unchecked() {
                let max_args = bm.signature().len();
                if arguments.len() > max_args && !bm.is_variadic() {
                    arguments = &arguments[..max_args];
                }
            }

            // This used to be a one step called Activation::from_method,
            // but avoiding moving an Activation around helps perf
            let mut activation = Activation::from_nothing(activation.context);
            activation.init_from_method(
                bm,
                scope,
                receiver,
                arguments,
                bound_superclass,
                bound_class,
                callee,
            )?;

            #[cfg(feature = "tracy_avm")]
            let _span = {
                let mut name = WString::new();
                display_function(&mut name, &method, bound_class);
                let option = tracy_client::Client::running();
                let span = option.expect("tracy_client should be running").span_alloc(
                    None,
                    &name.to_utf8_lossy(),
                    bm.owner_movie().url(),
                    line!(),
                    0,
                );
                span.emit_color(0x425fa1);
                span
            };

            activation
                .context
                .avm2
                .push_call(activation.context.gc_context, method, bound_class);
            activation.run_actions(bm)
        }
    };
    activation
        .context
        .avm2
        .pop_call(activation.context.gc_context);
    ret
}

impl<'gc> fmt::Debug for BoundMethod<'gc> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.method {
            Method::Bytecode(be) => fmt
                .debug_struct("BoundMethod")
                .field("method", &Gc::as_ptr(be))
                .field("scope", &self.scope)
                .field("receiver", &self.bound_receiver)
                .finish(),
            Method::Native(bm) => fmt
                .debug_struct("BoundMethod")
                .field("method", &bm)
                .field("scope", &self.scope)
                .field("bound_receiver", &self.bound_receiver)
                .finish(),
        }
    }
}

pub fn display_function<'gc>(
    output: &mut WString,
    method: &Method<'gc>,
    bound_class: Option<Class<'gc>>,
) {
    if let Some(bound_class) = bound_class {
        let name = bound_class.name().to_qualified_name_no_mc();
        output.push_str(&name);
    }

    match method {
        Method::Native(method) => {
            output.push_char('/');
            output.push_utf8(method.name)
        }
        Method::Bytecode(method) => {
            // NOTE: The name of a bytecode method refers to the name of the trait that contains the method,
            // rather than the name of the method itself.
            if let Some(bound_class) = bound_class {
                if bound_class
                    .instance_init()
                    .into_bytecode()
                    .map(|b| Gc::ptr_eq(b, *method))
                    .unwrap_or(false)
                {
                    if bound_class.is_c_class() {
                        // If the associated class is a c_class, its initializer
                        // method is a class initializer.
                        output.push_utf8("cinit");
                    }
                    // We purposely do nothing for instance initializers
                } else {
                    let mut method_trait = None;

                    let traits = bound_class.traits();
                    for t in &*traits {
                        if let Some(b) = t.as_method().and_then(|m| m.into_bytecode().ok()) {
                            if Gc::ptr_eq(b, *method) {
                                method_trait = Some(t);
                                break;
                            }
                        }
                    }

                    if let Some(method_trait) = method_trait {
                        output.push_char('/');
                        match method_trait.kind() {
                            TraitKind::Setter { .. } => output.push_utf8("set "),
                            TraitKind::Getter { .. } => output.push_utf8("get "),
                            _ => (),
                        }
                        if method_trait.name().namespace().is_namespace() {
                            output.push_str(&method_trait.name().to_qualified_name_no_mc());
                        } else {
                            output.push_str(&method_trait.name().local_name());
                        }
                    } else if !method.method_name().is_empty() {
                        // Last resort if we can't find a name anywhere else.
                        // SWF's with debug information will provide a method name attached
                        // to the method definition, so we can use that.
                        output.push_char('/');
                        output.push_utf8(&method.method_name());
                    }
                    // TODO: What happens if we can't find the trait?
                }
            } else if method.is_function && !method.method_name().is_empty() {
                output.push_utf8("Function/");
                output.push_utf8(&method.method_name());
            } else {
                output.push_utf8("MethodInfo-");
                output.push_utf8(&method.abc_method.to_string());
            }
        }
    }
    output.push_utf8("()");
}
