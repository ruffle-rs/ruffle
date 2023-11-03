use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, FunctionObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Activation, Error};
use crate::context::UpdateContext;
use crate::net_connection::ResponderCallback;
use flash_lso::types::Value as AMFValue;
use gc_arena::Mutation;
use gc_arena::{Collect, GcCell, GcWeakCell};
use std::cell::{Ref, RefMut};
use std::fmt;

/// A class instance allocator that allocates Responder objects.
pub fn responder_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(ResponderObject(GcCell::new(
        activation.context.gc(),
        ResponderObjectData {
            base,
            result: Default::default(),
            status: Default::default(),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ResponderObject<'gc>(pub GcCell<'gc, ResponderObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ResponderObjectWeak<'gc>(pub GcWeakCell<'gc, ResponderObjectData<'gc>>);

impl<'gc> TObject<'gc> for ResponderObject<'gc> {
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
        Ok(Value::Object((*self).into()))
    }

    fn as_responder(self) -> Option<ResponderObject<'gc>> {
        Some(self)
    }
}

impl<'gc> ResponderObject<'gc> {
    pub fn result(&self) -> Option<FunctionObject<'gc>> {
        self.0.read().result
    }

    pub fn status(&self) -> Option<FunctionObject<'gc>> {
        self.0.read().status
    }

    pub fn set_callbacks(
        &self,
        gc_context: &Mutation<'gc>,
        result: Option<FunctionObject<'gc>>,
        status: Option<FunctionObject<'gc>>,
    ) {
        self.0.write(gc_context).result = result;
        self.0.write(gc_context).status = status;
    }

    pub fn send_callback(
        &self,
        context: &mut UpdateContext<'_, 'gc>,
        callback: ResponderCallback,
        message: &AMFValue,
    ) -> Result<(), Error<'gc>> {
        let function = match callback {
            ResponderCallback::Result => self.0.read().result,
            ResponderCallback::Status => self.0.read().status,
        };

        if let Some(function) = function {
            let mut activation = Activation::from_nothing(context.reborrow());
            let value = crate::avm2::amf::deserialize_value(&mut activation, message)?;
            function.call((*self).into(), &[value], &mut activation)?;
        }

        Ok(())
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct ResponderObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Method to call with any result
    result: Option<FunctionObject<'gc>>,

    /// Method to call with status info (likely errors)
    status: Option<FunctionObject<'gc>>,
}

impl fmt::Debug for ResponderObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResponderObject")
    }
}
