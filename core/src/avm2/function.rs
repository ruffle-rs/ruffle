//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::names::QName;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::script_object::ScriptObjectData;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, CollectionContext, Gc, GcCell};
use std::fmt;
use std::rc::Rc;
use swf::avm2::types::AbcFile;

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
pub type NativeFunction<'gc> = fn(
    &mut Avm2<'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    Object<'gc>,
    &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error>;

/// Represents an AVM2 function.
#[derive(Collect, Clone, Debug)]
#[collect(require_static)]
pub struct Avm2Function {
    /// The ABC file this function was defined in.
    pub abc: Rc<AbcFile>,

    /// The ABC method this function uses.
    pub abc_method: u32,

    /// The ABC method body this function uses.
    pub abc_method_body: u32,
}

/// Represents code that can be executed by some means.
#[derive(Clone)]
pub enum Executable<'gc> {
    Native(NativeFunction<'gc>),
    Action(Gc<'gc, Avm2Function>),
}

unsafe impl<'gc> Collect for Executable<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Action(a2f) => a2f.trace(cc),
            Self::Native(_nf) => {}
        }
    }
}

impl<'gc> fmt::Debug for Executable<'gc> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Action(a2f) => fmt.debug_tuple("Executable::Action").field(a2f).finish(),
            Self::Native(nf) => fmt
                .debug_tuple("Executable::Native")
                .field(&format!("{:p}", nf))
                .finish(),
        }
    }
}

/// An Object which can be called to execute it's function code.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct FunctionObject<'gc>(GcCell<'gc, FunctionObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct FunctionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Executable code
    exec: Executable<'gc>,
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    fn get_property(
        self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().base.get_property(name, avm, context)
    }

    fn set_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .base
            .set_property(name, value, avm, context)
    }

    fn has_property(self, name: &QName) -> bool {
        self.0.read().base.has_property(name)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn call(
        self,
        reciever: Object<'gc>,
        arguments: &[Value<'gc>],
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let exec = self.0.read().exec.clone();

        match exec {
            Executable::Native(nf) => nf(avm, context, reciever, arguments),
            Executable::Action(a2f) => {
                let activation = GcCell::allocate(
                    context.gc_context,
                    Activation::from_action(context, a2f, reciever, None)?,
                );

                avm.insert_stack_frame(activation);
                Ok(activation.into())
            }
        }
    }
}
