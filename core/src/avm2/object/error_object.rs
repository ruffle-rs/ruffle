//! Object representation for Error objects

use crate::avm2::activation::Activation;
use crate::avm2::call_stack::CallStack;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::WString;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use tracing::{enabled, Level};

/// A class instance allocator that allocates Error objects.
pub fn error_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(ErrorObject(GcCell::new(
        activation.context.gc_context,
        ErrorObjectData {
            base,
            call_stack: (enabled!(Level::INFO) || cfg!(feature = "avm_debug"))
                .then(|| activation.avm2().call_stack().read().clone())
                .unwrap_or_default(),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ErrorObject<'gc>(pub GcCell<'gc, ErrorObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ErrorObjectWeak<'gc>(pub GcWeakCell<'gc, ErrorObjectData<'gc>>);

impl fmt::Debug for ErrorObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorObject")
            .field("class", &self.debug_class_name())
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ErrorObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    call_stack: CallStack<'gc>,
}

impl<'gc> ErrorObject<'gc> {
    pub fn display(&self) -> Result<WString, Error<'gc>> {
        // FIXME - we should have a safer way of accessing properties without
        // an `Activation`. For now, we just access the 'name' and 'message' fields
        // by hardcoded slot id. Our `Error` class definition should fully match
        // Flash Player, and we have lots of test coverage around error, so
        // there should be very little risk to doing this.
        let name = match self.base().get_slot(0)? {
            Value::String(string) => string,
            Value::Null => "null".into(),
            Value::Undefined => "undefined".into(),
            name => {
                return Err(Error::RustError(
                    format!("Error.name {name:?} is not a string on error object {self:?}",).into(),
                ))
            }
        };
        let message = match self.base().get_slot(1)? {
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

    pub fn call_stack(&self) -> Ref<CallStack<'gc>> {
        Ref::map(self.0.read(), |r| &r.call_stack)
    }

    fn debug_class_name(&self) -> Box<dyn Debug + 'gc> {
        self.0
            .try_read()
            .map(|obj| {
                obj.base
                    .instance_class()
                    .map(|cls| cls.debug_name())
                    .unwrap_or_else(|| Box::new("None"))
            })
            .unwrap_or_else(|err| Box::new(err))
    }
}

impl<'gc> TObject<'gc> for ErrorObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_error_object(&self) -> Option<ErrorObject<'gc>> {
        Some(*self)
    }
}
