use crate::avm2::activation::Activation;
use crate::avm2::call_stack::StackTrace;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::{AvmString, WString};
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
            id: 0,
            message: AvmString::default(),
            name: "Error".into(),
            stack_trace: if cfg!(feature = "avm_debug") {
                activation.id.to_stack_trace()
            } else {
                StackTrace::default()
            },
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
    name: AvmString<'gc>,
    message: AvmString<'gc>,
    id: i32,
    stack_trace: StackTrace<'gc>,
}

impl<'gc> ErrorObject<'gc> {
    pub fn name(&self) -> AvmString<'gc> {
        self.0.read().name
    }

    pub fn set_name(&self, mc: MutationContext<'gc, '_>, name: AvmString<'gc>) {
        self.0.write(mc).name = name;
    }

    pub fn message(&self) -> AvmString<'gc> {
        self.0.read().message
    }

    pub fn set_message(&self, mc: MutationContext<'gc, '_>, message: AvmString<'gc>) {
        self.0.write(mc).message = message;
    }

    pub fn id(&self) -> i32 {
        self.0.read().id
    }

    pub fn set_id(&self, mc: MutationContext<'gc, '_>, id: i32) {
        self.0.write(mc).id = id;
    }

    pub fn display(&self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let read = self.0.read();
        if read.message.is_empty() {
            return read.name;
        }
        let mut output = WString::new();
        output.push_str(&read.name);
        output.push_utf8(": ");
        output.push_str(&read.message);
        AvmString::new(mc, output)
    }

    pub fn display_full(&self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let mut output = WString::new();
        output.push_str(&self.display(mc));
        self.stack_trace().display(mc, &mut output);
        AvmString::new(mc, output)
    }

    pub fn stack_trace(&self) -> Ref<StackTrace<'gc>> {
        Ref::map(self.0.read(), |r| &r.stack_trace)
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
        let this: Object<'gc> = Object::ErrorObject(*self);

        Ok(this.into())
    }

    fn to_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.display(mc).into())
    }

    fn as_error_object(self) -> Option<Self> {
        Some(self)
    }
}
