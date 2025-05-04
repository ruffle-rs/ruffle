//! Object representation for SharedObjects

use crate::avm2::activation::Activation;
use crate::avm2::error::argument_error;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, ScriptObject, TObject};
use crate::avm2::Error;
use crate::utils::HasPrefixField;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak};
use std::fmt::Debug;

/// SharedObjects cannot be constructed by AS.
pub fn shared_object_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let class_name = class.inner_class_definition().name().local_name();

    Err(Error::AvmError(argument_error(
        activation,
        &format!("Error #2012: {class_name}$ class cannot be instantiated."),
        2012,
    )?))
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct SharedObjectObject<'gc>(pub Gc<'gc, SharedObjectObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct SharedObjectObjectWeak<'gc>(pub GcWeak<'gc, SharedObjectObjectData<'gc>>);

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct SharedObjectObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The SharedObject data that this SharedObjectObject holds.
    data: Lock<Object<'gc>>,

    /// The name of this SharedObject.
    name: String,
}

impl<'gc> SharedObjectObject<'gc> {
    pub fn from_data_and_name(
        activation: &mut Activation<'_, 'gc>,
        data: Object<'gc>,
        name: String,
    ) -> Self {
        let class = activation.avm2().classes().sharedobject;
        let base = ScriptObjectData::new(class);

        SharedObjectObject(Gc::new(
            activation.gc(),
            SharedObjectObjectData {
                base,
                data: Lock::new(data),
                name,
            },
        ))
    }

    pub fn data(&self) -> Object<'gc> {
        self.0.data.get()
    }

    pub fn reset_data(&self, activation: &mut Activation<'_, 'gc>) {
        let empty_data = ScriptObject::new_object(activation);

        unlock!(
            Gc::write(activation.gc(), self.0),
            SharedObjectObjectData,
            data
        )
        .set(empty_data);
    }

    pub fn name(&self) -> &String {
        &self.0.name
    }
}

impl<'gc> TObject<'gc> for SharedObjectObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_shared_object(&self) -> Option<SharedObjectObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for SharedObjectObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedObject")
    }
}
