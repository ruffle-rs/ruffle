//! Object representation for Texture3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_render::backend::{Context3DTextureFormat, Texture};
use std::rc::Rc;

use super::{ClassObject, Context3DObject};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextureObject<'gc>(pub Gc<'gc, TextureObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct TextureObjectWeak<'gc>(pub GcWeak<'gc, TextureObjectData<'gc>>);

impl<'gc> TextureObject<'gc> {
    pub fn from_handle(
        activation: &mut Activation<'_, 'gc>,
        context3d: Context3DObject<'gc>,
        handle: Rc<dyn Texture>,
        original_format: Context3DTextureFormat,
        class: ClassObject<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let this: Object<'gc> = TextureObject(Gc::new(
            activation.gc(),
            TextureObjectData {
                base: ScriptObjectData::new(class),
                context3d,
                original_format,
                handle,
            },
        ))
        .into();
        this.install_instance_slots(activation.gc());

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    pub fn original_format(&self) -> Context3DTextureFormat {
        self.0.original_format
    }

    pub fn handle(&self) -> Rc<dyn Texture> {
        self.0.handle.clone()
    }

    pub fn context3d(&self) -> Context3DObject<'gc> {
        self.0.context3d
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextureObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    context3d: Context3DObject<'gc>,

    #[collect(require_static)]
    original_format: Context3DTextureFormat,

    #[collect(require_static)]
    handle: Rc<dyn Texture>,
}

const _: () = assert!(std::mem::offset_of!(TextureObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<TextureObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> TObject<'gc> for TextureObject<'gc> {
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

    fn as_texture(&self) -> Option<TextureObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for TextureObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Texture3D")
    }
}
