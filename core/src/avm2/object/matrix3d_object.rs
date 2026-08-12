use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;

/// A class instance allocator that allocates Matrix3D objects.
pub fn matrix_3d_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(Matrix3DObject(Gc::new(activation.gc(), Matrix3DObjectData { base })).into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Matrix3DObject<'gc>(pub Gc<'gc, Matrix3DObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct Matrix3DObjectWeak<'gc>(pub GcWeak<'gc, Matrix3DObjectData<'gc>>);

impl fmt::Debug for Matrix3DObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matrix3DObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct Matrix3DObjectData<'gc> {
    /// Base script object.
    base: ScriptObjectData<'gc>,
}

impl<'gc> TObject<'gc> for Matrix3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
