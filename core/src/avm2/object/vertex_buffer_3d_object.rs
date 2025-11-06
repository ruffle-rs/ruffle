//! Object representation for VertexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, TObject};
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::backend::VertexBuffer;
use std::rc::Rc;

use super::Context3DObject;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct VertexBuffer3DObject<'gc>(pub Gc<'gc, VertexBuffer3DObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct VertexBuffer3DObjectWeak<'gc>(pub GcWeak<'gc, VertexBuffer3DObjectData<'gc>>);

impl<'gc> VertexBuffer3DObject<'gc> {
    pub fn from_handle(
        activation: &mut Activation<'_, 'gc>,
        context3d: Context3DObject<'gc>,
        handle: Rc<dyn VertexBuffer>,
        data32_per_vertex: u8,
    ) -> Object<'gc> {
        let class = activation.avm2().classes().vertexbuffer3d;

        VertexBuffer3DObject(Gc::new(
            activation.gc(),
            VertexBuffer3DObjectData {
                base: ScriptObjectData::new(class),
                context3d,
                handle,
                data32_per_vertex,
            },
        ))
        .into()
    }

    pub fn handle(self) -> Rc<dyn VertexBuffer> {
        self.0.handle.clone()
    }

    pub fn context3d(self) -> Context3DObject<'gc> {
        self.0.context3d
    }

    pub fn data32_per_vertex(self) -> u8 {
        self.0.data32_per_vertex
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct VertexBuffer3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    context3d: Context3DObject<'gc>,

    #[collect(require_static)]
    handle: Rc<dyn VertexBuffer>,

    /// The 'data32PerVertex' value that this object was created with.
    /// This is the number of 32-bit values associated with each vertex,
    /// and is at most 64
    data32_per_vertex: u8,
}

impl<'gc> TObject<'gc> for VertexBuffer3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl std::fmt::Debug for VertexBuffer3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VertexBuffer3D")
    }
}
