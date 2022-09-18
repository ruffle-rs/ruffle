//! Object representation for VertexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::backend::VertexBuffer;
use std::cell::{Ref, RefMut};
use std::rc::Rc;

use super::Context3DObject;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct VertexBuffer3DObject<'gc>(GcCell<'gc, VertexBuffer3DObjectData<'gc>>);

impl<'gc> VertexBuffer3DObject<'gc> {
    pub fn from_handle(
        activation: &mut Activation<'_, 'gc, '_>,
        context3d: Context3DObject<'gc>,
        handle: Rc<dyn VertexBuffer>,
        data_per_vertex: usize,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().vertexbuffer3d;
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = VertexBuffer3DObject(GcCell::allocate(
            activation.context.gc_context,
            VertexBuffer3DObjectData {
                base,
                context3d,
                handle,
                data_per_vertex,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    pub fn handle(&self) -> Rc<dyn VertexBuffer> {
        self.0.read().handle.clone()
    }

    pub fn context3d(&self) -> Context3DObject<'gc> {
        self.0.read().context3d
    }

    pub fn data_per_vertex(&self) -> usize {
        self.0.read().data_per_vertex
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct VertexBuffer3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    context3d: Context3DObject<'gc>,

    handle: Rc<dyn VertexBuffer>,

    data_per_vertex: usize,
}

impl<'gc> TObject<'gc> for VertexBuffer3DObject<'gc> {
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

    fn as_vertex_buffer(&self) -> Option<VertexBuffer3DObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for VertexBuffer3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VertexBuffer3D")
    }
}
