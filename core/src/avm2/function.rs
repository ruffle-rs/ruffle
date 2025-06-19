use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::error::{make_mismatch_error, Error};
use crate::avm2::method::{Method, MethodKind, ParamConfig};
use crate::avm2::object::ClassObject;
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::TraitKind;
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::string::WString;
use gc_arena::{Collect, Gc};
use std::borrow::Cow;
use std::cell::Cell;
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
    ///
    /// This should never be `Value::Null` or `Value::Undefined`.
    bound_receiver: Option<Value<'gc>>,

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
        receiver: Option<Value<'gc>>,
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
        arguments: FunctionArgs<'_, 'gc>,
        activation: &mut Activation<'_, 'gc>,
        callee: Value<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let receiver = if let Some(receiver) = self.bound_receiver {
            receiver
        } else if matches!(unbound_receiver, Value::Null | Value::Undefined) {
            self.scope
                .get(0)
                .expect("No global scope for function call")
                .values()
        } else {
            unbound_receiver
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
        display_function(&mut output, self.as_method(), self.bound_class());
        output
    }

    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        self.method.signature()
    }

    pub fn is_variadic(&self) -> bool {
        self.method.is_variadic()
    }

    pub fn return_type(&self) -> Option<Gc<'gc, Multiname<'gc>>> {
        self.method.return_type()
    }
}

#[derive(Clone, Copy)]
pub enum FunctionArgs<'a, 'gc> {
    AsCellArgSlice { arguments: &'a [Cell<Value<'gc>>] },
    AsArgSlice { arguments: &'a [Value<'gc>] },
}

impl<'a, 'gc> FunctionArgs<'a, 'gc> {
    pub fn to_slice(self) -> Cow<'a, [Value<'gc>]> {
        match self {
            FunctionArgs::AsCellArgSlice { arguments } => {
                Cow::Owned(arguments.iter().map(|o| o.get()).collect::<Vec<_>>())
            }
            FunctionArgs::AsArgSlice { arguments } => Cow::Borrowed(arguments),
        }
    }

    pub fn get_at(&self, index: usize) -> Value<'gc> {
        match self {
            FunctionArgs::AsCellArgSlice { arguments } => arguments[index].get(),
            FunctionArgs::AsArgSlice { arguments } => arguments[index],
        }
    }

    pub fn len(&self) -> usize {
        match self {
            FunctionArgs::AsCellArgSlice { arguments } => arguments.len(),
            FunctionArgs::AsArgSlice { arguments } => arguments.len(),
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
///
/// It is the caller's responsibility to ensure that the `receiver` passed
/// to this method is not Value::Null or Value::Undefined.
#[allow(clippy::too_many_arguments)]
pub fn exec<'gc>(
    method: Method<'gc>,
    scope: ScopeChain<'gc>,
    receiver: Value<'gc>,
    bound_superclass: Option<ClassObject<'gc>>,
    bound_class: Option<Class<'gc>>,
    arguments: FunctionArgs<'_, 'gc>,
    activation: &mut Activation<'_, 'gc>,
    callee: Value<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let mc = activation.gc();

    let ret = match method.method_kind() {
        MethodKind::Native { native_method, .. } => {
            let arguments = &arguments.to_slice();

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

            method.resolve_info(&mut activation)?;

            let signature = method.resolved_param_config();

            // Check for too many arguments
            if arguments.len() > signature.len() && !method.is_variadic() && !method.is_unchecked()
            {
                return Err(Error::avm_error(make_mismatch_error(
                    &mut activation,
                    method,
                    arguments.len(),
                    bound_class,
                )?));
            }

            let arguments =
                activation.resolve_parameters(method, arguments, signature, bound_class)?;

            #[cfg(feature = "tracy_avm")]
            let _span = {
                let mut name = WString::new();
                display_function(&mut name, method, bound_class);
                let span = tracy_client::Client::running()
                    .expect("tracy_client should be running")
                    .span_alloc(None, &name.to_utf8_lossy(), "rust", 0, 0);
                span.emit_color(0x2c4980);
                span
            };

            activation.context.avm2.push_call(mc, method, bound_class);

            native_method(&mut activation, receiver, &arguments)
        }
        MethodKind::Bytecode { .. } => {
            // We must initialize the stack frame here so the lifetime works out
            let stack = activation.context.avm2.stack;
            let stack_frame = stack.get_stack_frame(method);

            // This used to be a one step called Activation::from_method,
            // but avoiding moving an Activation around helps perf
            let mut activation = Activation::from_nothing(activation.context);
            if let Err(e) = activation.init_from_method(
                method,
                scope,
                receiver,
                arguments,
                stack_frame,
                bound_superclass,
                bound_class,
                callee,
            ) {
                // If an error is thrown during verification or argument coercion,
                // we still need to call cleanup to dispose of the stack frame
                activation.cleanup();
                return Err(e);
            }

            #[cfg(feature = "tracy_avm")]
            let _span = {
                let mut name = WString::new();
                display_function(&mut name, method, bound_class);
                let option = tracy_client::Client::running();
                let span = option.expect("tracy_client should be running").span_alloc(
                    None,
                    &name.to_utf8_lossy(),
                    method.owner_movie().url(),
                    line!(),
                    0,
                );
                span.emit_color(0x425fa1);
                span
            };

            activation.context.avm2.push_call(mc, method, bound_class);

            let result = activation.run_actions(method);

            activation.cleanup();

            result
        }
    };
    activation.context.avm2.pop_call(mc);
    ret
}

impl fmt::Debug for BoundMethod<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BoundMethod")
            .field("method", &self.method)
            .field("scope", &self.scope)
            .field("receiver", &self.bound_receiver)
            .finish()
    }
}

pub fn display_function<'gc>(
    output: &mut WString,
    method: Method<'gc>,
    bound_class: Option<Class<'gc>>,
) {
    if let Some(bound_class) = bound_class {
        let name = bound_class.name().to_qualified_name_no_mc();
        output.push_str(&name);
    }

    // NOTE: The name of a bytecode method refers to the name of the trait that contains the method,
    // rather than the name of the method itself.
    if let Some(bound_class) = bound_class {
        if bound_class.instance_init() == Some(method) {
            if bound_class.is_c_class() {
                // If the associated class is a c_class, its initializer
                // method is a class initializer.
                output.push_utf8("cinit");
            }
            // We purposely do nothing for instance initializers
        } else {
            let mut method_trait = None;

            for t in bound_class.traits() {
                if t.as_method().is_some_and(|tm| tm == method) {
                    method_trait = Some(t);
                    break;
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
    } else if method.is_function() && !method.method_name().is_empty() {
        output.push_utf8("Function/");
        output.push_utf8(&method.method_name());
    } else {
        output.push_utf8("MethodInfo-");
        output.push_utf8(&method.abc_method_index().to_string());
    }

    output.push_utf8("()");
}
