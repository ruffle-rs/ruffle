use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::matrix3d::Matrix3D;
use std::cell::{Ref, RefCell, RefMut};

/// A class instance allocator that allocates Matrix3D objects.
pub fn matrix_3d_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(Matrix3DObject(Gc::new(
        activation.gc(),
        Matrix3DObjectData {
            base,
            matrix: RefCell::new(Matrix3D {
                raw_data: [
                    1.0, 0.0, 0.0, 0.0, //
                    0.0, 1.0, 0.0, 0.0, //
                    0.0, 0.0, 1.0, 0.0, //
                    0.0, 0.0, 0.0, 1.0,
                ],
            }),
        },
    ))
    .into())
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

    matrix: RefCell<Matrix3D>,
}

impl<'gc> Matrix3DObject<'gc> {
    pub fn matrix_ref(&self) -> Ref<'_, Matrix3D> {
        self.0.matrix.borrow()
    }

    pub fn matrix_mut(&self) -> RefMut<'_, Matrix3D> {
        self.0.matrix.borrow_mut()
    }

    pub fn matrix(self) -> Matrix3D {
        *self.matrix_ref()
    }

    pub fn replace_matrix(self, matrix: Matrix3D) -> Matrix3D {
        self.0.matrix.replace(matrix)
    }
}

impl<'gc> TObject<'gc> for Matrix3DObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
