//! Object representation for Error objects

use crate::avm2::activation::Activation;
use crate::avm2::call_stack::CallStack;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::QName;
use crate::string::WString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Error objects.
pub fn error_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::new(class);

    Ok(ErrorObject(GcCell::allocate(
        activation.context.gc_context,
        ErrorObjectData {
            base,
            call_stack: activation.avm2().call_stack().read().clone(),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ErrorObject<'gc>(GcCell<'gc, ErrorObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ErrorObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    call_stack: CallStack<'gc>,
}

impl<'gc> ErrorObject<'gc> {
    pub fn display(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<AvmString<'gc>, Error> {
        let name = self
            .get_property(&QName::dynamic_name("name").into(), activation)?
            .coerce_to_string(activation)?;
        let message = self
            .get_property(&QName::dynamic_name("message").into(), activation)?
            .coerce_to_string(activation)?;
        if message.is_empty() {
            return Ok(name);
        }
        let mut output = WString::new();
        output.push_str(&name);
        output.push_utf8(": ");
        output.push_str(&message);
        Ok(AvmString::new(activation.context.gc_context, output))
    }

    pub fn display_full(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<AvmString<'gc>, Error> {
        let mut output = WString::new();
        output.push_str(&self.display(activation)?);
        self.call_stack().display(&mut output);
        Ok(AvmString::new(activation.context.gc_context, output))
    }

    pub fn call_stack(&self) -> Ref<CallStack<'gc>> {
        Ref::map(self.0.read(), |r| &r.call_stack)
    }
}

impl<'gc> TObject<'gc> for ErrorObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn to_string(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.display(activation)?.into())
    }

    fn as_error_object(&self) -> Option<ErrorObject<'gc>> {
        Some(*self)
    }
}
