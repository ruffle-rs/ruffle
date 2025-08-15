//! AVM2 methods

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::error::{make_error_1014, verify_error, Error, Error1014Type};
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::verify::VerifiedMethodInfo;
use crate::avm2::Multiname;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use gc_arena::barrier::{unlock, Write};
use gc_arena::lock::OnceLock;
use gc_arena::{Collect, Gc};
use std::borrow::Cow;
use std::cell::Cell;
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
    Value<'gc>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>>;

/// Configuration of a single parameter of a method,
/// with the parameter's type resolved.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ResolvedParamConfig<'gc> {
    /// The type of the parameter.
    pub param_type: Option<Class<'gc>>,

    /// The default value for this parameter.
    pub default_value: Option<Value<'gc>>,
}

/// Configuration of a single parameter of a method.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ParamConfig<'gc> {
    /// The name of the type of the parameter.
    pub param_type_name: Option<Gc<'gc, Multiname<'gc>>>,

    /// The default value for this parameter.
    pub default_value: Option<Value<'gc>>,
}

impl<'gc> ParamConfig<'gc> {
    fn from_abc_param(
        config: &AbcMethodParam,
        txunit: TranslationUnit<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let param_type_name = txunit.pool_multiname_static_any(activation, config.kind)?;

        let default_value = if let Some(dv) = &config.default_value {
            Some(abc_default_value(txunit, dv, activation)?)
        } else {
            None
        };

        Ok(Self {
            param_type_name,
            default_value,
        })
    }

    pub fn optional(
        param_type_name: Option<Gc<'gc, Multiname<'gc>>>,
        default_value: impl Into<Value<'gc>>,
    ) -> Self {
        Self {
            param_type_name,
            default_value: Some(default_value.into()),
        }
    }
}

/// Represents a reference to an AVM2 method and body.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Method<'gc>(Gc<'gc, MethodData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
struct MethodData<'gc> {
    /// The translation unit this function was defined in.
    txunit: TranslationUnit<'gc>,

    /// The underlying ABC file of the above translation unit.
    #[collect(require_static)]
    abc: Rc<AbcFile>,

    /// The ABC method this function uses.
    abc_method: u32,

    /// The ABC method body this function uses.
    abc_method_body: Option<u32>,

    method_kind: MethodKind<'gc>,

    /// The parameter signature of this method.
    signature: Vec<ParamConfig<'gc>>,

    /// The return type of this method, or None if the method does not coerce
    /// its return value.
    return_type: Option<Gc<'gc, Multiname<'gc>>>,

    /// The resolved signature and return type.
    resolved_info: OnceLock<ResolvedMethodInfo<'gc>>,

    /// Whether this method should be run in "interpreter mode" (as opposed to
    /// "JIT mode"). Most methods run in "JIT mode", except for class
    /// initializer and script initializer methods, which always run in
    /// "interpreter mode". See `Activation.is_interpreter` for more information.
    is_interpreted: Cell<bool>,

    /// Whether or not this method was declared as a free-standing function.
    ///
    /// A free-standing function corresponds to the `Function` trait type, and
    /// is instantiated with the `newfunction` opcode.
    is_function: bool,

    /// Whether or not this method substitutes Undefined for missing arguments.
    ///
    /// This is true when the method is a free-standing function and none of the
    /// declared arguments have a type or a default value.
    is_unchecked: bool,
}

impl PartialEq for Method<'_> {
    fn eq(&self, other: &Self) -> bool {
        Gc::ptr_eq(self.0, other.0)
    }
}

impl core::fmt::Debug for Method<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Method")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

impl<'gc> Method<'gc> {
    /// Construct a `Method` from an `AbcFile` and method index.
    pub fn from_method_index(
        txunit: TranslationUnit<'gc>,
        abc_method: Index<AbcMethod>,
        is_function: bool,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let method_index = abc_method.0 as usize;
        let abc = txunit.abc();

        let Some(method) = abc.methods.get(method_index) else {
            return Err(Error::avm_error(verify_error(
                activation,
                "Error #1027: Method_info exceeds method_count.",
                1027,
            )?));
        };

        let mut signature = Vec::new();
        for param in &method.params {
            signature.push(ParamConfig::from_abc_param(param, txunit, activation)?);
        }

        let return_type = txunit.pool_multiname_static_any(activation, method.return_type)?;

        let abc_method_body = method.body.map(|b| b.0);

        let mut all_params_unchecked = true;
        for param in &signature {
            if param.param_type_name.is_some() || param.default_value.is_some() {
                all_params_unchecked = false;
            }
        }

        let mut native_info = None;
        if txunit.domain().is_playerglobals_domain(activation.avm2()) {
            if let Some(native_method) = activation.avm2().native_method_table[method_index] {
                let fast_call = activation
                    .avm2()
                    .native_fast_call_list
                    .contains(&method_index);

                native_info = Some((native_method, fast_call));
            }
        };

        let method_kind = if let Some((native_method, fast_call)) = native_info {
            MethodKind::Native {
                native_method,
                fast_call,
            }
        } else {
            MethodKind::Bytecode {
                verified_info: OnceLock::new(),
            }
        };

        Ok(Self(Gc::new(
            activation.gc(),
            MethodData {
                txunit,
                abc: txunit.abc(),
                abc_method: abc_method.0,
                abc_method_body,
                method_kind,
                signature,
                return_type,
                resolved_info: OnceLock::new(),
                is_interpreted: Cell::new(false),
                is_function,
                is_unchecked: is_function && all_params_unchecked,
            },
        )))
    }

    /// Get the underlying ABC file.
    pub fn abc(&self) -> Rc<AbcFile> {
        self.0.txunit.abc()
    }

    /// Get the underlying translation unit this method was defined in.
    pub fn translation_unit(&self) -> TranslationUnit<'gc> {
        self.0.txunit
    }

    pub fn abc_method_index(&self) -> u32 {
        self.0.abc_method
    }

    /// Get a reference to the ABC method entry this refers to.
    pub fn method(&self) -> &AbcMethod {
        &self.0.abc.methods[self.0.abc_method as usize]
    }

    /// Get a reference to the SwfMovie this method came from.
    pub fn owner_movie(&self) -> Arc<SwfMovie> {
        self.0.txunit.movie()
    }

    /// Get a reference to the ABC method body entry this refers to.
    ///
    /// Some methods do not have bodies; this returns `None` in that case.
    pub fn body(&self) -> Option<&AbcMethodBody> {
        if let Some(abc_method_body) = self.0.abc_method_body {
            self.0.abc.method_bodies.get(abc_method_body as usize)
        } else {
            None
        }
    }

    #[inline]
    pub fn verify(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        // TODO: avmplus seems to eagerly verify some methods

        match &self.0.method_kind {
            MethodKind::Bytecode { verified_info } if verified_info.get().is_none() => {
                let info = crate::avm2::verify::verify_method(activation, self)?;

                Gc::write(activation.gc(), self.0);

                // SAFETY: We just triggered a write barrier on the Gc.
                let verified_info = unsafe { Write::assume(verified_info) };
                let _ = verified_info.unlock().set(info);

                Ok(())
            }
            MethodKind::Bytecode { .. } | MethodKind::Native { .. } => Ok(()),
        }
    }

    /// Get the list of method params for this method.
    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        &self.0.signature
    }

    pub fn resolved_param_config(&self) -> &[ResolvedParamConfig<'gc>] {
        &self.0.resolved_info.get().unwrap().param_config
    }

    pub fn resolved_return_type(&self) -> Option<Class<'gc>> {
        self.0.resolved_info.get().unwrap().return_type
    }

    pub fn get_verified_info(&self) -> &VerifiedMethodInfo<'gc> {
        match &self.0.method_kind {
            MethodKind::Bytecode { verified_info } => verified_info.get().unwrap(),
            _ => panic!("get_verified_info should be called on a bytecode method"),
        }
    }

    /// Resolve the classes used in this method's signature and return type.
    #[inline(never)]
    pub fn resolve_info(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        if self.0.resolved_info.get().is_some() {
            return Ok(());
        }

        let param_config = resolve_param_config(activation, self.signature())?;
        let return_type = resolve_return_type(activation, self.return_type())?;

        let resolved_info = ResolvedMethodInfo {
            param_config,
            return_type,
        };

        let _ = unlock!(
            Gc::write(activation.gc(), self.0),
            MethodData,
            resolved_info
        )
        .set(resolved_info);

        Ok(())
    }

    /// Get the name of this method.
    pub fn method_name(&self) -> Cow<'_, str> {
        let name_index = self.method().name.0 as usize;
        if name_index == 0 {
            return Cow::Borrowed("");
        }

        self.0
            .abc
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

    /// Check if this method needs `arguments`.
    pub fn needs_arguments_object(&self) -> bool {
        self.method().flags.contains(AbcMethodFlags::NEED_ARGUMENTS)
    }

    pub fn method_kind(&self) -> &MethodKind<'gc> {
        &self.0.method_kind
    }

    pub fn return_type(&self) -> Option<Gc<'gc, Multiname<'gc>>> {
        self.0.return_type
    }

    /// Whether this method should be run in "interpreter mode" (as opposed to
    /// "JIT mode").
    pub fn is_interpreted(self) -> bool {
        self.0.is_interpreted.get()
    }

    /// Mark this method as one that should be run in "interpreter mode" (as
    /// opposed to "JIT mode").
    pub fn mark_as_interpreted(self) {
        self.0.is_interpreted.set(true);
    }

    pub fn is_function(self) -> bool {
        self.0.is_function
    }

    /// Determine if a given method is unchecked.
    ///
    /// A method is unchecked if both of the following are true:
    ///
    ///  * The method was declared as a free-standing function
    ///  * The function's parameters have no declared types or default values
    pub fn is_unchecked(&self) -> bool {
        self.0.is_unchecked
    }
}

/// Represents info for either a bytecode or native method
#[derive(Collect)]
#[collect(no_drop)]
pub enum MethodKind<'gc> {
    Bytecode {
        verified_info: OnceLock<VerifiedMethodInfo<'gc>>,
    },
    Native {
        #[collect(require_static)]
        native_method: NativeMethodImpl,
        fast_call: bool,
    },
}

/// The resolved parameters and return type of a method.
#[derive(Collect)]
#[collect(no_drop)]
struct ResolvedMethodInfo<'gc> {
    param_config: Vec<ResolvedParamConfig<'gc>>,
    return_type: Option<Class<'gc>>,
}

fn resolve_param_config<'gc>(
    activation: &mut Activation<'_, 'gc>,
    param_config: &[ParamConfig<'gc>],
) -> Result<Vec<ResolvedParamConfig<'gc>>, Error<'gc>> {
    let mut resolved_param_config = Vec::new();

    for param in param_config {
        let resolved_class = if let Some(param_type_name) = param.param_type_name {
            if param_type_name.has_lazy_component() {
                return Err(make_error_1014(
                    activation,
                    Error1014Type::VerifyError,
                    AvmString::new_utf8(activation.gc(), "[]"),
                ));
            }

            Some(
                activation
                    .domain()
                    .get_class(activation.context, &param_type_name)
                    .ok_or_else(|| {
                        make_error_1014(
                            activation,
                            Error1014Type::VerifyError,
                            param_type_name.to_qualified_name(activation.gc()),
                        )
                    })?,
            )
        } else {
            None
        };

        resolved_param_config.push(ResolvedParamConfig {
            param_type: resolved_class,
            default_value: param.default_value,
        });
    }

    Ok(resolved_param_config)
}

fn resolve_return_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    return_type: Option<Gc<'gc, Multiname<'gc>>>,
) -> Result<Option<Class<'gc>>, Error<'gc>> {
    if let Some(return_type) = return_type {
        if return_type.has_lazy_component() {
            return Err(make_error_1014(
                activation,
                Error1014Type::VerifyError,
                AvmString::new_utf8(activation.gc(), "[]"),
            ));
        }

        Ok(Some(
            activation
                .domain()
                .get_class(activation.context, &return_type)
                .ok_or_else(|| {
                    make_error_1014(
                        activation,
                        Error1014Type::VerifyError,
                        return_type.to_qualified_name(activation.gc()),
                    )
                })?,
        ))
    } else {
        Ok(None)
    }
}
