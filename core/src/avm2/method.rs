//! AVM2 methods

use crate::avm2::activation::Activation;
use crate::avm2::names::Multiname;
use crate::avm2::object::Object;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, CollectionContext, Gc, MutationContext};
use std::fmt;
use std::rc::Rc;
use swf::avm2::types::{
    AbcFile, Index, Method as AbcMethod, MethodBody as AbcMethodBody, MethodParam as AbcMethodParam,
};

/// Represents a function defined in Ruffle's code.
///
/// Parameters are as follows:
///
///  * The AVM2 runtime
///  * The action context
///  * The current `this` object
///  * The arguments this function was called with
///
/// Native functions are allowed to return a value or `None`. `None` indicates
/// that the given value will not be returned on the stack and instead will
/// resolve on the AVM stack, as if you had called a non-native function. If
/// your function yields `None`, you must ensure that the top-most activation
/// in the AVM1 runtime will return with the value of this function.
pub type NativeMethodImpl = for<'gc> fn(
    &mut Activation<'_, 'gc, '_>,
    Option<Object<'gc>>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error>;

/// Configuration of a single parameter of a method.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ParamConfig<'gc> {
    /// The name of the parameter.
    pub param_name: AvmString<'gc>,

    /// The name of the type of the parameter.
    pub param_type_name: Multiname<'gc>,

    /// The default value for this parameter.
    pub default_value: Option<Value<'gc>>,
}

impl<'gc> ParamConfig<'gc> {
    fn from_abc_param(
        config: &AbcMethodParam,
        txunit: TranslationUnit<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let param_name = if let Some(name) = &config.name {
            txunit.pool_string(name.0, activation.context.gc_context)?
        } else {
            "<Unnamed Parameter>".into()
        };
        let param_type_name = if config.kind.0 == 0 {
            Multiname::any()
        } else {
            Multiname::from_abc_multiname_static(
                txunit,
                config.kind,
                activation.context.gc_context,
            )?
        };
        let default_value = if let Some(dv) = &config.default_value {
            Some(abc_default_value(txunit, dv, activation)?)
        } else {
            None
        };

        Ok(Self {
            param_name,
            param_type_name,
            default_value,
        })
    }

    pub fn of_type(name: impl Into<AvmString<'gc>>, param_type_name: Multiname<'gc>) -> Self {
        Self {
            param_name: name.into(),
            param_type_name,
            default_value: None,
        }
    }

    pub fn optional(
        name: impl Into<AvmString<'gc>>,
        param_type_name: Multiname<'gc>,
        default_value: impl Into<Value<'gc>>,
    ) -> Self {
        Self {
            param_name: name.into(),
            param_type_name,
            default_value: Some(default_value.into()),
        }
    }
}

/// Represents a reference to an AVM2 method and body.
#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
pub struct BytecodeMethod<'gc> {
    /// The translation unit this function was defined in.
    pub txunit: TranslationUnit<'gc>,

    /// The underlying ABC file of the above translation unit.
    #[collect(require_static)]
    pub abc: Rc<AbcFile>,

    /// The ABC method this function uses.
    pub abc_method: u32,

    /// The ABC method body this function uses.
    pub abc_method_body: Option<u32>,

    /// The parameter signature of this method.
    pub signature: Vec<ParamConfig<'gc>>,

    /// The return type of this method.
    pub return_type: Multiname<'gc>,

    /// Whether or not this method was declared as a free-standing function.
    ///
    /// A free-standing function corresponds to the `Function` trait type, and
    /// is instantiated with the `newfunction` opcode.
    pub is_function: bool,
}

impl<'gc> BytecodeMethod<'gc> {
    /// Construct an `BytecodeMethod` from an `AbcFile` and method index.
    pub fn from_method_index(
        txunit: TranslationUnit<'gc>,
        abc_method: Index<AbcMethod>,
        is_function: bool,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Gc<'gc, Self>, Error> {
        let abc = txunit.abc();
        let mut signature = Vec::new();

        if abc.methods.get(abc_method.0 as usize).is_some() {
            let method = &abc.methods[abc_method.0 as usize];
            for param in &method.params {
                signature.push(ParamConfig::from_abc_param(param, txunit, activation)?);
            }

            let return_type = if method.return_type.0 == 0 {
                Multiname::any()
            } else {
                Multiname::from_abc_multiname_static(
                    txunit,
                    method.return_type,
                    activation.context.gc_context,
                )?
            };

            for (index, method_body) in abc.method_bodies.iter().enumerate() {
                if method_body.method.0 == abc_method.0 {
                    return Ok(Gc::allocate(
                        activation.context.gc_context,
                        Self {
                            txunit,
                            abc: txunit.abc(),
                            abc_method: abc_method.0,
                            abc_method_body: Some(index as u32),
                            signature,
                            return_type,
                            is_function,
                        },
                    ));
                }
            }
        }

        Ok(Gc::allocate(
            activation.context.gc_context,
            Self {
                txunit,
                abc: txunit.abc(),
                abc_method: abc_method.0,
                abc_method_body: None,
                signature,
                return_type: Multiname::any(),
                is_function,
            },
        ))
    }

    /// Get the underlying ABC file.
    pub fn abc(&self) -> Rc<AbcFile> {
        self.txunit.abc()
    }

    /// Get the underlying translation unit this method was defined in.
    pub fn translation_unit(&self) -> TranslationUnit<'gc> {
        self.txunit
    }

    /// Get a reference to the ABC method entry this refers to.
    pub fn method(&self) -> &AbcMethod {
        self.abc.methods.get(self.abc_method as usize).unwrap()
    }

    /// Get a reference to the ABC method body entry this refers to.
    ///
    /// Some methods do not have bodies; this returns `None` in that case.
    pub fn body(&self) -> Option<&AbcMethodBody> {
        if let Some(abc_method_body) = self.abc_method_body {
            self.abc.method_bodies.get(abc_method_body as usize)
        } else {
            None
        }
    }

    /// Get the list of method params for this method.
    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        &self.signature
    }

    /// Get the name of this method.
    pub fn method_name(&self) -> &str {
        let name_index = self.method().name.0 as usize;
        if name_index == 0 {
            return "";
        }

        self.abc
            .constant_pool
            .strings
            .get(name_index - 1)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Determine if a given method is variadic.
    ///
    /// Variadic methods shove excess parameters into a final register.
    pub fn is_variadic(&self) -> bool {
        self.method().needs_arguments_object || self.method().needs_rest
    }

    /// Determine if a given method is unchecked.
    ///
    /// A method is unchecked if all of the following are true:
    ///
    ///  * The method was declared as a free-standing function
    ///  * The function does not use rest-parameters
    ///  * The function's parameters have no declared types or default values
    pub fn is_unchecked(&self) -> bool {
        if !self.is_function {
            return false;
        }

        for param in self.signature() {
            if !param.param_type_name.is_any() || param.default_value.is_some() {
                return false;
            }
        }

        !self.method().needs_rest
    }
}

/// An uninstantiated method
#[derive(Clone)]
pub struct NativeMethod<'gc> {
    /// The function to call to execute the method.
    pub method: NativeMethodImpl,

    /// The name of the method.
    pub name: &'static str,

    /// The parameter signature of the method.
    pub signature: Vec<ParamConfig<'gc>>,

    /// Whether or not this method accepts parameters beyond those
    /// mentioned in the parameter list.
    pub is_variadic: bool,
}

unsafe impl<'gc> Collect for NativeMethod<'gc> {
    fn trace(&self, cc: CollectionContext) {
        self.signature.trace(cc);
    }
}

impl<'gc> fmt::Debug for NativeMethod<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeMethod")
            .field("method", &format!("{:p}", &self.method))
            .field("name", &self.name)
            .field("signature", &self.signature)
            .field("is_variadic", &self.is_variadic)
            .finish()
    }
}

/// An uninstantiated method that can either be natively implemented or sourced
/// from an ABC file.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Method<'gc> {
    /// A native method.
    Native(Gc<'gc, NativeMethod<'gc>>),

    /// An ABC-provided method entry.
    Bytecode(Gc<'gc, BytecodeMethod<'gc>>),
}

impl<'gc> From<Gc<'gc, BytecodeMethod<'gc>>> for Method<'gc> {
    fn from(bm: Gc<'gc, BytecodeMethod<'gc>>) -> Self {
        Self::Bytecode(bm)
    }
}

impl<'gc> Method<'gc> {
    /// Define a builtin method with a particular param configuration.
    pub fn from_builtin_and_params(
        method: NativeMethodImpl,
        name: &'static str,
        signature: Vec<ParamConfig<'gc>>,
        is_variadic: bool,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        Self::Native(Gc::allocate(
            mc,
            NativeMethod {
                method,
                name,
                signature,
                is_variadic,
            },
        ))
    }

    /// Define a builtin with no parameter constraints.
    pub fn from_builtin(
        method: NativeMethodImpl,
        name: &'static str,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        Self::Native(Gc::allocate(
            mc,
            NativeMethod {
                method,
                name,
                signature: Vec::new(),
                is_variadic: true,
            },
        ))
    }

    /// Access the bytecode of this method.
    ///
    /// This function returns `Err` if there is no bytecode for this method.
    pub fn into_bytecode(self) -> Result<Gc<'gc, BytecodeMethod<'gc>>, Error> {
        match self {
            Method::Native { .. } => {
                Err("Attempted to unwrap a native method as a user-defined one".into())
            }
            Method::Bytecode(bm) => Ok(bm),
        }
    }

    /// Check if this method needs `arguments`.
    pub fn needs_arguments_object(&self) -> bool {
        match self {
            Method::Native { .. } => false,
            Method::Bytecode(bm) => bm.method().needs_arguments_object,
        }
    }
}
