//! Object representation for Error objects

use crate::avm2::activation::Activation;
use crate::avm2::call_stack::CallStack;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::WString;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::fmt::Debug;
use tracing::{enabled, Level};

/// A class instance allocator that allocates Error objects.
pub fn error_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let call_stack = (enabled!(Level::INFO) || cfg!(feature = "avm_debug"))
        .then(|| activation.avm2().call_stack().borrow().clone())
        .unwrap_or_default();

    Ok(ErrorObject(Gc::new(
        activation.context.gc_context,
        ErrorObjectData { base, call_stack },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ErrorObject<'gc>(pub Gc<'gc, ErrorObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ErrorObjectWeak<'gc>(pub GcWeak<'gc, ErrorObjectData<'gc>>);

impl fmt::Debug for ErrorObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorObject")
            .field("class", &self.debug_class_name())
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ErrorObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    call_stack: CallStack<'gc>,
}

const _: () = assert!(std::mem::offset_of!(ErrorObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<ErrorObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> ErrorObject<'gc> {
    pub fn display(&self) -> Result<WString, Error<'gc>> {
        // FIXME - we should have a safer way of accessing properties without
        // an `Activation`. For now, we just access the 'name' and 'message' fields
        // by hardcoded slot id. Our `Error` class definition should fully match
        // Flash Player, and we have lots of test coverage around error, so
        // there should be very little risk to doing this.
        let name = match self.base().get_slot(0) {
            Value::String(string) => string,
            Value::Null => "null".into(),
            Value::Undefined => "undefined".into(),
            name => {
                return Err(Error::RustError(
                    format!("Error.name {name:?} is not a string on error object {self:?}",).into(),
                ))
            }
        };
        let message = match self.base().get_slot(1) {
            Value::String(string) => string,
            Value::Null => "null".into(),
            Value::Undefined => "undefined".into(),
            message => {
                return Err(Error::RustError(
                    format!("Error.message {message:?} is not a string on error object {self:?}")
                        .into(),
                ))
            }
        };
        if message.is_empty() {
            return Ok(name.as_wstr().to_owned());
        }

        let mut output = WString::new();
        output.push_str(&name);
        output.push_utf8(": ");
        output.push_str(&message);
        Ok(output)
    }

    pub fn display_full(&self) -> Result<WString, Error<'gc>> {
        let mut output = WString::new();
        output.push_str(&self.display()?);
        self.call_stack().display(&mut output);
        Ok(output)
    }

    pub fn call_stack(&self) -> &CallStack<'gc> {
        &self.0.call_stack
    }

    fn debug_class_name(&self) -> Box<dyn Debug + 'gc> {
        self.base().instance_class().debug_name()
    }
}

impl<'gc> TObject<'gc> for ErrorObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_error_object(&self) -> Option<ErrorObject<'gc>> {
        Some(*self)
    }
}
