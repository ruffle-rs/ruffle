//! Object representation for `ShaderData`

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::Error;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_render::pixel_bender::PixelBenderShaderHandle;
use std::cell::Cell;

/// A class instance allocator that allocates ShaderData objects.
pub fn shader_data_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(ShaderDataObject(Gc::new(
        activation.gc(),
        ShaderDataObjectData {
            base: ScriptObjectData::new(class),
            shader: Cell::new(None),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ShaderDataObject<'gc>(pub Gc<'gc, ShaderDataObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ShaderDataObjectWeak<'gc>(pub GcWeak<'gc, ShaderDataObjectData<'gc>>);

impl fmt::Debug for ShaderDataObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShaderDataObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

impl ShaderDataObject<'_> {
    pub fn pixel_bender_shader(&self) -> Option<PixelBenderShaderHandle> {
        let shader = &self.0.shader;
        let guard = scopeguard::guard(shader.take(), |stolen| shader.set(stolen));
        guard.clone()
    }

    pub fn set_pixel_bender_shader(&self, shader: PixelBenderShaderHandle) {
        self.0.shader.set(Some(shader));
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ShaderDataObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    shader: Cell<Option<PixelBenderShaderHandle>>,
}

const _: () = assert!(std::mem::offset_of!(ShaderDataObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<ShaderDataObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> TObject<'gc> for ShaderDataObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_shader_data(&self) -> Option<ShaderDataObject<'gc>> {
        Some(*self)
    }
}
