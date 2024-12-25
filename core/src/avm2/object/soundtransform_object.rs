use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::Error;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use std::cell::Cell;

/// A class instance allocator that allocates SoundTransform objects.
pub fn sound_transform_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(SoundTransformObject(Gc::new(
        activation.gc(),
        SoundTransformObjectData {
            base,
            left_to_left: Cell::new(0.0),
            left_to_right: Cell::new(0.0),
            right_to_left: Cell::new(0.0),
            right_to_right: Cell::new(0.0),
            volume: Cell::new(0.0),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct SoundTransformObject<'gc>(pub Gc<'gc, SoundTransformObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct SoundTransformObjectWeak<'gc>(pub GcWeak<'gc, SoundTransformObjectData<'gc>>);

impl fmt::Debug for SoundTransformObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoundTransformObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct SoundTransformObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    left_to_left: Cell<f64>,
    left_to_right: Cell<f64>,
    right_to_left: Cell<f64>,
    right_to_right: Cell<f64>,

    volume: Cell<f64>,
}

const _: () = assert!(std::mem::offset_of!(SoundTransformObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<SoundTransformObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl SoundTransformObject<'_> {
    pub fn left_to_left(self) -> f64 {
        self.0.left_to_left.get()
    }

    pub fn set_left_to_left(self, value: f64) {
        self.0.left_to_left.set(value);
    }

    pub fn left_to_right(self) -> f64 {
        self.0.left_to_right.get()
    }

    pub fn set_left_to_right(self, value: f64) {
        self.0.left_to_right.set(value);
    }

    pub fn right_to_left(self) -> f64 {
        self.0.right_to_left.get()
    }

    pub fn set_right_to_left(self, value: f64) {
        self.0.right_to_left.set(value);
    }

    pub fn right_to_right(self) -> f64 {
        self.0.right_to_right.get()
    }

    pub fn set_right_to_right(self, value: f64) {
        self.0.right_to_right.set(value);
    }

    pub fn volume(self) -> f64 {
        self.0.volume.get()
    }

    pub fn set_volume(self, value: f64) {
        self.0.volume.set(value);
    }
}

impl<'gc> TObject<'gc> for SoundTransformObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_sound_transform(&self) -> Option<SoundTransformObject<'gc>> {
        Some(*self)
    }
}
