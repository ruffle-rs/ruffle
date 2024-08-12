//! Object representation for IndexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_render::backend::IndexBuffer;
use std::cell::{Cell, RefCell, RefMut};

use super::Context3DObject;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct IndexBuffer3DObject<'gc>(pub Gc<'gc, IndexBuffer3DObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct IndexBuffer3DObjectWeak<'gc>(pub GcWeak<'gc, IndexBuffer3DObjectData<'gc>>);

impl<'gc> IndexBuffer3DObject<'gc> {
    pub fn from_handle(
        activation: &mut Activation<'_, 'gc>,
        context3d: Context3DObject<'gc>,
        handle: Box<dyn IndexBuffer>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().indexbuffer3d;

        let this: Object<'gc> = IndexBuffer3DObject(Gc::new(
            activation.gc(),
            IndexBuffer3DObjectData {
                base: ScriptObjectData::new(class),
                context3d,
                handle: RefCell::new(handle),
                count: Cell::new(0),
            },
        ))
        .into();
        this.install_instance_slots(activation.gc());

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn count(&self) -> usize {
        self.0.count.get()
    }

    pub fn set_count(&self, val: usize) {
        self.0.count.set(val);
    }

    pub fn handle(&self) -> RefMut<'_, dyn IndexBuffer> {
        RefMut::map(self.0.handle.borrow_mut(), |h| h.as_mut())
    }

    pub fn context3d(&self) -> Context3DObject<'gc> {
        self.0.context3d
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct IndexBuffer3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    handle: RefCell<Box<dyn IndexBuffer>>,

    count: Cell<usize>,

    context3d: Context3DObject<'gc>,
}

const _: () = assert!(std::mem::offset_of!(IndexBuffer3DObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<IndexBuffer3DObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> TObject<'gc> for IndexBuffer3DObject<'gc> {
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

    fn as_index_buffer(&self) -> Option<IndexBuffer3DObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for IndexBuffer3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexBuffer3D")
    }
}
