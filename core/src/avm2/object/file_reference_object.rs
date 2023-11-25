use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Activation, Error};
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc};
use gc_arena::{GcWeak, Mutation};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::fmt;

pub fn file_reference_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class).into();

    Ok(FileReferenceObject(Gc::new(
        activation.context.gc(),
        FileReferenceObjectData {
            base,
        },
    ))
    .into())
}


#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FileReferenceObject<'gc>(pub Gc<'gc, FileReferenceObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct FileReferenceObjectWeak<'gc>(pub GcWeak<'gc, FileReferenceObjectData<'gc>>);

impl<'gc> TObject<'gc> for FileReferenceObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        self.0.base.borrow()
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        unlock!(Gc::write(mc, self.0), FileReferenceObjectData, base).borrow_mut()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_file_reference(&self) -> Option<FileReferenceObject<'gc>> {
        Some(*self)
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct FileReferenceObjectData<'gc> {
    /// Base script object
    base: RefLock<ScriptObjectData<'gc>>,
}

impl fmt::Debug for FileReferenceObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileReferenceObject")
    }
}
