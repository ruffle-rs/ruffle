//! AVM2 methods

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use gc_arena::{Collect, CollectionContext, Gc, MutationContext};
use std::fmt;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Index, Method as AbcMethod, MethodBody as AbcMethodBody};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct CollectWrapper<T>(T);

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
pub type NativeMethod<'gc> = fn(
    &mut Activation<'_, 'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    Option<Object<'gc>>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error>;

/// Represents a reference to an AVM2 method and body.
#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
pub struct BytecodeMethod<'gc> {
    /// The translation unit this function was defined in.
    pub txunit: TranslationUnit<'gc>,

    /// The underlying ABC file of the above translation unit.
    pub abc: CollectWrapper<Rc<AbcFile>>,

    /// The ABC method this function uses.
    pub abc_method: u32,

    /// The ABC method body this function uses.
    pub abc_method_body: Option<u32>,
}

impl<'gc> BytecodeMethod<'gc> {
    /// Construct an `BytecodeMethod` from an `AbcFile` and method index.
    ///
    /// The method body index will be determined by searching through the ABC
    /// for a matching method. If none exists, this function returns `None`.
    pub fn from_method_index(
        txunit: TranslationUnit<'gc>,
        abc_method: Index<AbcMethod>,
        mc: MutationContext<'gc, '_>,
    ) -> Option<Gc<'gc, Self>> {
        let abc = txunit.abc();

        if abc.methods.get(abc_method.0 as usize).is_some() {
            for (index, method_body) in abc.method_bodies.iter().enumerate() {
                if method_body.method.0 == abc_method.0 {
                    return Some(Gc::allocate(
                        mc,
                        Self {
                            txunit,
                            abc: CollectWrapper(txunit.abc()),
                            abc_method: abc_method.0,
                            abc_method_body: Some(index as u32),
                        },
                    ));
                }
            }
        }

        Some(Gc::allocate(
            mc,
            Self {
                txunit,
                abc: CollectWrapper(txunit.abc()),
                abc_method: abc_method.0,
                abc_method_body: None,
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
        &self.abc.0.methods.get(self.abc_method as usize).unwrap()
    }

    /// Get a reference to the ABC method body entry this refers to.
    ///
    /// Some methods do not have bodies; this returns `None` in that case.
    pub fn body(&self) -> Option<&AbcMethodBody> {
        if let Some(abc_method_body) = self.abc_method_body {
            self.abc.0.method_bodies.get(abc_method_body as usize)
        } else {
            None
        }
    }
}

/// An uninstantiated method that can either be natively implemented or sourced
/// from an ABC file.
#[derive(Clone)]
pub enum Method<'gc> {
    /// A native method.
    Native(NativeMethod<'gc>),

    /// An ABC-provided method entry.
    Entry(Gc<'gc, BytecodeMethod<'gc>>),
}

unsafe impl<'gc> Collect for Method<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Method::Native(_nf) => {}
            Method::Entry(entry) => entry.trace(cc),
        }
    }
}

impl<'gc> fmt::Debug for Method<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Native(_nf) => f
                .debug_tuple("Method::Native")
                .field(&"<native code>".to_string())
                .finish(),
            Method::Entry(entry) => f.debug_tuple("Method::Entry").field(entry).finish(),
        }
    }
}

impl<'gc> From<NativeMethod<'gc>> for Method<'gc> {
    fn from(nf: NativeMethod<'gc>) -> Self {
        Self::Native(nf)
    }
}

impl<'gc> From<Gc<'gc, BytecodeMethod<'gc>>> for Method<'gc> {
    fn from(bm: Gc<'gc, BytecodeMethod<'gc>>) -> Self {
        Self::Entry(bm)
    }
}

impl<'gc> Method<'gc> {
    /// Access the bytecode of this method.
    ///
    /// This function returns `Err` if there is no bytecode for this method.
    pub fn into_bytecode(self) -> Result<Gc<'gc, BytecodeMethod<'gc>>, Error> {
        match self {
            Method::Native(_) => {
                Err("Attempted to unwrap a native method as a user-defined one".into())
            }
            Method::Entry(bm) => Ok(bm),
        }
    }
}
