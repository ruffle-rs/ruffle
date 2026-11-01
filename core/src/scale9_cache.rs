use ruffle_render::backend::ShapeHandle;
use ruffle_render::matrix::Matrix;
use ruffle_render::shape_utils::Scale9;
use std::cell::RefCell;

/// The grid, the matrix into the grid's space, and a bucket of the tessellation scale.
pub(crate) type Scale9Key = (Scale9, Matrix, i32);

/// The most recent 9-sliced tessellation of a shape. Slicing bakes the scale into the
/// vertices, so without this a gridded object would re-tessellate every frame. Boxed so an
/// ungridded object pays only a pointer.
#[derive(Clone, Debug, Default)]
pub(crate) struct Scale9Cache(RefCell<Option<Box<(Scale9Key, ShapeHandle)>>>);

impl Scale9Cache {
    pub(crate) fn get_or_register(
        &self,
        key: Scale9Key,
        build: impl FnOnce() -> ShapeHandle,
    ) -> ShapeHandle {
        if let Some(entry) = self.0.borrow().as_deref()
            && entry.0 == key
        {
            return entry.1.clone();
        }
        let handle = build();
        *self.0.borrow_mut() = Some(Box::new((key, handle.clone())));
        handle
    }

    pub(crate) fn clear(&self) {
        self.0.take();
    }
}
