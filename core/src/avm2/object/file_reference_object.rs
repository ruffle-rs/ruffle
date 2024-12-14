use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::{Activation, Error};
use crate::backend::ui::FileDialogResult;
use gc_arena::GcWeak;
use gc_arena::{Collect, Gc};
use std::cell::{Cell, Ref, RefCell};
use std::fmt;

pub fn file_reference_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(FileReferenceObject(Gc::new(
        activation.context.gc(),
        FileReferenceObjectData {
            base,
            reference: RefCell::new(FileReference::None),
            loaded: Cell::new(false),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FileReferenceObject<'gc>(pub Gc<'gc, FileReferenceObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct FileReferenceObjectWeak<'gc>(pub GcWeak<'gc, FileReferenceObjectData<'gc>>);

impl<'gc> TObject<'gc> for FileReferenceObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_file_reference(&self) -> Option<FileReferenceObject<'gc>> {
        Some(*self)
    }
}

impl FileReferenceObject<'_> {
    pub fn init_from_dialog_result(&self, result: Box<dyn FileDialogResult>) -> FileReference {
        self.0
            .reference
            .replace(FileReference::FileDialogResult(result))
    }

    pub fn file_reference(&self) -> Ref<'_, FileReference> {
        self.0.reference.borrow()
    }

    pub fn set_loaded(&self, value: bool) {
        self.0.loaded.set(value)
    }

    pub fn loaded(&self) -> bool {
        self.0.loaded.get()
    }
}

pub enum FileReference {
    None,
    FileDialogResult(Box<dyn FileDialogResult>),
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FileReferenceObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    reference: RefCell<FileReference>,

    loaded: Cell<bool>,
}

const _: () = assert!(std::mem::offset_of!(FileReferenceObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<FileReferenceObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl fmt::Debug for FileReferenceObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileReferenceObject")
    }
}
