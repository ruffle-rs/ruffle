//! Object representation for Texture3D objects

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_render::backend::Texture;
use std::cell::{Ref, RefMut};
use std::rc::Rc;

use super::{ClassObject, Context3DObject};

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextureObject<'gc>(GcCell<'gc, TextureObjectData<'gc>>);

impl<'gc> TextureObject<'gc> {
    pub fn from_handle(
        activation: &mut Activation<'_, 'gc>,
        context3d: Context3DObject<'gc>,
        handle: Rc<dyn Texture>,
        class: ClassObject<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let base = ScriptObjectData::new(class);

        let mut this: Object<'gc> = TextureObject(GcCell::allocate(
            activation.context.gc_context,
            TextureObjectData {
                base,
                context3d,
                handle,
            },
        ))
        .into();
        this.install_instance_slots(activation);

        class.call_native_init(Some(this), &[], activation)?;

        Ok(this)
    }

    pub fn handle(&self) -> Rc<dyn Texture> {
        self.0.read().handle.clone()
    }

    pub fn context3d(&self) -> Context3DObject<'gc> {
        self.0.read().context3d
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct TextureObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    context3d: Context3DObject<'gc>,

    handle: Rc<dyn Texture>,
}

impl<'gc> TObject<'gc> for TextureObject<'gc> {
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

    fn as_texture(&self) -> Option<TextureObject<'gc>> {
        Some(*self)
    }
}

impl std::fmt::Debug for TextureObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Texture3D")
    }
}
