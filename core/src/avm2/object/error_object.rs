//! Object representation for Error objects

use crate::avm2::activation::Activation;
use crate::avm2::call_stack::CallStack;
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::slots::error as error_slots;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::{AvmString, WStr, WString};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use std::fmt::Debug;

/// A class instance allocator that allocates Error objects.
pub fn error_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let error_object = ErrorObject::new(activation, class);

    Ok(error_object.into())
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
            .field("class", &self.base().class_name())
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ErrorObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    call_stack: CallStack<'gc>,
}

impl<'gc> ErrorObject<'gc> {
    /// Allocates a new `ErrorObject` for the given class, without running the
    /// class initializer.
    pub fn new(activation: &mut Activation<'_, 'gc>, class: ClassObject<'gc>) -> Self {
        let base = ScriptObjectData::new(class);

        // Stack trace is always collected for debugging purposes.
        let call_stack = activation.avm2().capture_call_stack();

        ErrorObject(Gc::new(
            activation.gc(),
            ErrorObjectData { base, call_stack },
        ))
    }

    /// Allocates and constructs a new `ErrorObject` for the given class, using
    /// the provided error message and id.
    pub fn from_info(
        activation: &mut Activation<'_, 'gc>,
        class: ClassObject<'gc>,
        message: AvmString<'gc>,
        error_id: u32,
    ) -> Self {
        let allocated_object = Self::new(activation, class);

        let arguments = &[message.into(), error_id.into()];

        class
            .call_init(
                allocated_object.into(),
                FunctionArgs::from_slice(arguments),
                activation,
            )
            .expect("Error constructors are infallible");

        allocated_object
    }

    pub fn display(self) -> WString {
        let name = match self.base().get_slot(error_slots::NAME) {
            Value::String(string) => string.as_wstr(),
            Value::Null => WStr::from_units(b"null"),
            _ => unreachable!("String-typed slot must be String or Null"),
        };

        let message = match self.base().get_slot(error_slots::MESSAGE) {
            Value::String(string) => string.as_wstr(),
            Value::Null => WStr::from_units(b"null"),
            _ => unreachable!("String-typed slot must be String or Null"),
        };
        if message.is_empty() {
            return name.to_owned();
        }

        let mut output = WString::new();
        output.push_str(name);
        output.push_utf8(": ");
        output.push_str(message);
        output
    }

    pub fn display_full(self) -> WString {
        let mut output = WString::new();
        output.push_str(&self.display());
        self.call_stack().display(&mut output);
        output
    }

    pub fn call_stack(&self) -> &CallStack<'gc> {
        &self.0.call_stack
    }
}

impl<'gc> TObject<'gc> for ErrorObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
