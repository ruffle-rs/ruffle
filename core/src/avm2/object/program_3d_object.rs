//! Object representation for VertexBuffer3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use ruffle_gc_extra::lock::RefLock;
use ruffle_gc_extra::{unlock, GcExt as _};
use ruffle_render::backend::ShaderModule;
use std::cell::{Ref, RefMut};
use std::rc::Rc;

use super::Context3DObject;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Program3DObject<'gc>(Gc<'gc, Program3DObjectData<'gc>>);

impl<'gc> Program3DObject<'gc> {
    pub fn from_context(
        activation: &mut Activation<'_, 'gc>,
        context3d: Context3DObject<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().program3d;
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = Program3DObject(Gc::allocate(
            activation.context.gc_context,
            Program3DObjectData {
                base: RefLock::new(base),
                context3d,
                vertex_shader_handle: GcCell::allocate(activation.context.gc_context, None),
                fragment_shader_handle: GcCell::allocate(activation.context.gc_context, None),
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    pub fn vertex_shader_handle(&self) -> GcCell<'gc, Option<Rc<dyn ShaderModule>>> {
        self.0.vertex_shader_handle
    }

    pub fn fragment_shader_handle(&self) -> GcCell<'gc, Option<Rc<dyn ShaderModule>>> {
        self.0.fragment_shader_handle
    }

    pub fn context3d(&self) -> Context3DObject<'gc> {
        self.0.context3d
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct Program3DObjectData<'gc> {
    /// Base script object
    base: RefLock<ScriptObjectData<'gc>>,

    context3d: Context3DObject<'gc>,

    vertex_shader_handle: GcCell<'gc, Option<Rc<dyn ShaderModule>>>,

    fragment_shader_handle: GcCell<'gc, Option<Rc<dyn ShaderModule>>>,
}

impl<'gc> TObject<'gc> for Program3DObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        self.0.base.borrow()
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        unlock!(Gc::write(mc, self.0), Program3DObjectData, base).borrow_mut()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_program_3d(&self) -> Option<Program3DObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for Program3DObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program3D")
    }
}
