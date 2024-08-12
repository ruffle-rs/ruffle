//! AVM2 methods

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::object::{ClassObject, Object};
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::verify::{resolve_param_config, VerifiedMethodInfo};
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcCell, Mutation};
use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use swf::avm2::types::{
    AbcFile, Index, Method as AbcMethod, MethodBody as AbcMethodBody,
    MethodFlags as AbcMethodFlags, MethodParam as AbcMethodParam,
};

/// Represents a function defined in Ruffle's code.
///
/// Parameters are as follows:
///
///  * The AVM2 runtime
///  * The current `this` object
///  * The arguments this function was called with
///
/// Native functions are allowed to return a Value or an Error.
pub type NativeMethodImpl = for<'gc> fn(
    &mut Activation<'_, 'gc>,
    Object<'gc>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>>;

/// Configuration of a single parameter of a method,
/// with the parameter's type resolved.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ResolvedParamConfig<'gc> {
    /// The name of the parameter.
    pub param_name: AvmString<'gc>,

    /// The type of the parameter.
    pub param_type: Option<Class<'gc>>,

    /// The default value for this parameter.
    pub default_value: Option<Value<'gc>>,
}

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
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let param_name = if let Some(name) = &config.name {
            txunit
                .pool_string(name.0, &mut activation.borrow_gc())?
                .into()
        } else {
            AvmString::from("<Unnamed Parameter>")
        };
        let param_type_name = txunit
            .pool_multiname_static_any(config.kind, activation.context)?
            .deref()
            .clone();

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
#[derive(Collect)]
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

    pub verified_info: GcCell<'gc, Option<VerifiedMethodInfo<'gc>>>,

    /// The parameter signature of this method.
    pub signature: Vec<ParamConfig<'gc>>,

    /// The return type of this method.
    pub return_type: Multiname<'gc>,

    /// The associated activation class. Initialized lazily, and only
    /// if the method requires it.
    activation_class: Lock<Option<ClassObject<'gc>>>,

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
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let abc = txunit.abc();
        let mut signature = Vec::new();
        let mut return_type = Multiname::any(activation.gc());

        if abc.methods.get(abc_method.0 as usize).is_some() {
            let method = &abc.methods[abc_method.0 as usize];
            for param in &method.params {
                signature.push(ParamConfig::from_abc_param(param, txunit, activation)?);
            }

            return_type = txunit
                .pool_multiname_static_any(method.return_type, activation.context)?
                .deref()
                .clone();

            for (index, method_body) in abc.method_bodies.iter().enumerate() {
                if method_body.method.0 == abc_method.0 {
                    return Ok(Self {
                        txunit,
                        abc: txunit.abc(),
                        abc_method: abc_method.0,
                        abc_method_body: Some(index as u32),
                        verified_info: GcCell::new(activation.context.gc_context, None),
                        signature,
                        return_type,
                        is_function,
                        activation_class: Lock::new(None),
                    });
                }
            }
        }

        Ok(Self {
            txunit,
            abc: txunit.abc(),
            abc_method: abc_method.0,
            abc_method_body: None,
            verified_info: GcCell::new(activation.context.gc_context, None),
            signature,
            return_type,
            is_function,
            activation_class: Lock::new(None),
        })
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

    /// Get a reference to the SwfMovie this method came from.
    pub fn owner_movie(&self) -> Arc<SwfMovie> {
        self.txunit.movie()
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

    #[inline(never)]
    pub fn verify(&self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        // TODO: avmplus seems to eaglerly verify some methods

        *self.verified_info.write(activation.context.gc_context) =
            Some(crate::avm2::verify::verify_method(activation, self)?);

        Ok(())
    }

    /// Get the list of method params for this method.
    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        &self.signature
    }

    pub fn resolved_return_type(&self) -> Option<Class<'gc>> {
        let verified_info = self.verified_info.read();

        verified_info.as_ref().unwrap().return_type
    }

    /// Get the name of this method.
    pub fn method_name(&self) -> Cow<'_, str> {
        let name_index = self.method().name.0 as usize;
        if name_index == 0 {
            return Cow::Borrowed("");
        }

        self.abc
            .constant_pool
            .strings
            .get(name_index - 1)
            .map(|s| String::from_utf8_lossy(s))
            .unwrap_or(Cow::Borrowed(""))
    }

    /// Determine if a given method is variadic.
    ///
    /// Variadic methods shove excess parameters into a final register.
    pub fn is_variadic(&self) -> bool {
        self.method()
            .flags
            .intersects(AbcMethodFlags::NEED_ARGUMENTS | AbcMethodFlags::NEED_REST)
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
            if !param.param_type_name.is_any_name() || param.default_value.is_some() {
                return false;
            }
        }

        !self.method().flags.contains(AbcMethodFlags::NEED_REST)
    }

    /// Initialize and return the activation class object, if the method requires it.
    pub fn get_or_init_activation_class(
        this: Gc<'gc, Self>,
        mc: &Mutation<'gc>,
        init: impl FnOnce() -> Result<ClassObject<'gc>, Error<'gc>>,
    ) -> Result<Option<ClassObject<'gc>>, Error<'gc>> {
        Ok(if let Some(cached) = this.activation_class.get() {
            Some(cached)
        } else if this
            .method()
            .flags
            .contains(AbcMethodFlags::NEED_ACTIVATION)
        {
            let cls = Some(init()?);
            unlock!(Gc::write(mc, this), Self, activation_class).set(cls);
            cls
        } else {
            None
        })
    }
}

/// An uninstantiated method
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct NativeMethod<'gc> {
    /// The function to call to execute the method.
    #[collect(require_static)]
    pub method: NativeMethodImpl,

    /// The name of the method.
    pub name: &'static str,

    /// The parameter signature of the method.
    pub signature: Vec<ParamConfig<'gc>>,

    /// The resolved parameter signature of the method.
    pub resolved_signature: GcCell<'gc, Option<Vec<ResolvedParamConfig<'gc>>>>,

    /// The return type of this method.
    pub return_type: Multiname<'gc>,

    /// Whether or not this method accepts parameters beyond those
    /// mentioned in the parameter list.
    pub is_variadic: bool,
}

impl<'gc> NativeMethod<'gc> {
    pub fn resolve_signature(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        *self.resolved_signature.write(activation.context.gc_context) =
            Some(resolve_param_config(activation, &self.signature)?);

        Ok(())
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
#[derive(Copy, Clone, Collect)]
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
        return_type: Multiname<'gc>,
        is_variadic: bool,
        mc: &Mutation<'gc>,
    ) -> Self {
        Self::Native(Gc::new(
            mc,
            NativeMethod {
                method,
                name,
                signature,
                resolved_signature: GcCell::new(mc, None),
                return_type,
                is_variadic,
            },
        ))
    }

    /// Define a builtin with no parameter constraints.
    pub fn from_builtin(method: NativeMethodImpl, name: &'static str, mc: &Mutation<'gc>) -> Self {
        Self::Native(Gc::new(
            mc,
            NativeMethod {
                method,
                name,
                signature: Vec::new(),
                resolved_signature: GcCell::new(mc, None),
                // FIXME - take in the real return type. This is needed for 'describeType'
                return_type: Multiname::any(mc),
                is_variadic: true,
            },
        ))
    }

    /// Access the bytecode of this method.
    ///
    /// This function returns `Err` if there is no bytecode for this method.
    pub fn into_bytecode(self) -> Result<Gc<'gc, BytecodeMethod<'gc>>, Error<'gc>> {
        match self {
            Method::Native { .. } => {
                Err("Attempted to unwrap a native method as a user-defined one".into())
            }
            Method::Bytecode(bm) => Ok(bm),
        }
    }

    pub fn return_type(&self) -> Multiname<'gc> {
        match self {
            Method::Native(nm) => nm.return_type.clone(),
            Method::Bytecode(bm) => bm.return_type.clone(),
        }
    }

    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        match self {
            Method::Native(nm) => &nm.signature,
            Method::Bytecode(bm) => bm.signature(),
        }
    }

    pub fn is_variadic(&self) -> bool {
        match self {
            Method::Native(nm) => nm.is_variadic,
            Method::Bytecode(bm) => bm.is_variadic(),
        }
    }

    /// Check if this method needs `arguments`.
    pub fn needs_arguments_object(&self) -> bool {
        match self {
            Method::Native { .. } => false,
            Method::Bytecode(bm) => bm.method().flags.contains(AbcMethodFlags::NEED_ARGUMENTS),
        }
    }
}
