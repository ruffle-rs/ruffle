//! Object representation for IndexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, TObject};
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
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
    ) -> Object<'gc> {
        let class = activation.avm2().classes().indexbuffer3d;

        IndexBuffer3DObject(Gc::new(
            activation.gc(),
            IndexBuffer3DObjectData {
                base: ScriptObjectData::new(class),
                context3d,
                handle: RefCell::new(handle),
                count: Cell::new(0),
            },
        ))
        .into()
    }

    pub fn count(self) -> usize {
        self.0.count.get()
    }

    pub fn set_count(self, val: usize) {
        self.0.count.set(val);
    }

    pub fn handle(&self) -> RefMut<'_, dyn IndexBuffer> {
        RefMut::map(self.0.handle.borrow_mut(), |h| h.as_mut())
    }

    pub fn context3d(self) -> Context3DObject<'gc> {
        self.0.context3d
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct IndexBuffer3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    handle: RefCell<Box<dyn IndexBuffer>>,

    count: Cell<usize>,

    context3d: Context3DObject<'gc>,
}

impl<'gc> TObject<'gc> for IndexBuffer3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl std::fmt::Debug for IndexBuffer3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IndexBuffer3D")
    }
}
