//! Object representation for `ShaderData`

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use core::fmt;
use gc_arena::{Collect, GcCell, GcWeakCell, MutationContext};
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates ShaderData objects.
pub fn shader_data_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(ShaderDataObject(GcCell::allocate(
        activation.context.gc_context,
        ShaderDataObjectData { base, shader: None },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ShaderDataObject<'gc>(pub GcCell<'gc, ShaderDataObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ShaderDataObjectWeak<'gc>(pub GcWeakCell<'gc, ShaderDataObjectData<'gc>>);

impl fmt::Debug for ShaderDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShaderDataObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

impl<'gc> ShaderDataObject<'gc> {
    pub fn pixel_bender_shader(&self) -> Ref<'_, Option<PixelBenderShaderHandle>> {
        Ref::map(self.0.read(), |read| &read.shader)
    }

    pub fn set_pixel_bender_shader(
        &self,
        shader: PixelBenderShaderHandle,
        mc: MutationContext<'gc, '_>,
    ) {
        self.0.write(mc).shader = Some(shader);
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct ShaderDataObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    #[collect(require_static)]
    shader: Option<PixelBenderShaderHandle>,
}

impl<'gc> TObject<'gc> for ShaderDataObject<'gc> {
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

    fn as_shader_data(&self) -> Option<ShaderDataObject<'gc>> {
        Some(*self)
    }
}
