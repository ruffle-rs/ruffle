//! Object representation for IndexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::backend::IndexBuffer;
use std::cell::{Ref, RefMut};
use std::rc::Rc;

use super::Context3DObject;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct IndexBuffer3DObject<'gc>(GcCell<'gc, IndexBuffer3DObjectData<'gc>>);

impl<'gc> IndexBuffer3DObject<'gc> {
    pub fn from_handle(
        activation: &mut Activation<'_, 'gc, '_>,
        context3d: Context3DObject<'gc>,
        handle: Rc<dyn IndexBuffer>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().indexbuffer3d;
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = IndexBuffer3DObject(GcCell::allocate(
            activation.context.gc_context,
            IndexBuffer3DObjectData {
                base,
                context3d,
                handle,
                count: 0,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    pub fn count(&self) -> usize {
        self.0.read().count
    }

    pub fn set_count(&self, val: usize, mc: MutationContext<'gc, '_>) {
        self.0.write(mc).count = val;
    }

    pub fn handle(&self) -> Rc<dyn IndexBuffer> {
        self.0.read().handle.clone()
    }

    pub fn context3d(&self) -> Context3DObject<'gc> {
        self.0.read().context3d
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct IndexBuffer3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    handle: Rc<dyn IndexBuffer>,

    count: usize,

    context3d: Context3DObject<'gc>,
}

impl<'gc> TObject<'gc> for IndexBuffer3DObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
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
