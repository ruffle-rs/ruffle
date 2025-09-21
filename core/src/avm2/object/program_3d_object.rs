//! Object representation for VertexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, TObject};
use crate::utils::HasPrefixField;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_render::backend::ShaderModule;
use std::cell::RefCell;
use std::rc::Rc;

use super::Context3DObject;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Program3DObject<'gc>(pub Gc<'gc, Program3DObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct Program3DObjectWeak<'gc>(pub GcWeak<'gc, Program3DObjectData<'gc>>);

impl<'gc> Program3DObject<'gc> {
    pub fn from_context(
        activation: &mut Activation<'_, 'gc>,
        context3d: Context3DObject<'gc>,
    ) -> Object<'gc> {
        let class = activation.avm2().classes().program3d;
        let base = ScriptObjectData::new(class);

        Program3DObject(Gc::new(
            activation.gc(),
            Program3DObjectData {
                base,
                context3d,
                shader_module_handle: RefCell::new(None),
            },
        ))
        .into()
    }

    pub fn shader_module_handle(&self) -> &RefCell<Option<Rc<dyn ShaderModule>>> {
        &self.0.shader_module_handle
    }

    pub fn context3d(self) -> Context3DObject<'gc> {
        self.0.context3d
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Program3DObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    context3d: Context3DObject<'gc>,

    shader_module_handle: RefCell<Option<Rc<dyn ShaderModule>>>,
}

impl<'gc> TObject<'gc> for Program3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl std::fmt::Debug for Program3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program3D")
    }
}
