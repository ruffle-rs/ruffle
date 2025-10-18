use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::{Activation, Error};
use crate::backend::ui::FileDialogResult;
use crate::context::UpdateContext;
use crate::utils::HasPrefixField;
use gc_arena::{Collect, DynamicRoot, Gc, GcWeak, Rootable};
use std::cell::{Cell, Ref, RefCell};
use std::fmt;

pub fn file_reference_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(FileReferenceObject(Gc::new(
        activation.gc(),
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

#[derive(Clone)]
pub struct FileReferenceObjectHandle(DynamicRoot<Rootable![FileReferenceObjectData<'_>]>);

impl FileReferenceObjectHandle {
    pub fn stash<'gc>(context: &UpdateContext<'gc>, this: FileReferenceObject<'gc>) -> Self {
        Self(context.dynamic_root.stash(context.gc(), this.0))
    }

    pub fn fetch<'gc>(&self, context: &UpdateContext<'gc>) -> FileReferenceObject<'gc> {
        FileReferenceObject(context.dynamic_root.fetch(&self.0))
    }
}

impl<'gc> TObject<'gc> for FileReferenceObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl FileReferenceObject<'_> {
    pub fn init_from_dialog_result(self, result: Box<dyn FileDialogResult>) -> FileReference {
        self.0
            .reference
            .replace(FileReference::FileDialogResult(result))
    }

    pub fn file_reference(&self) -> Ref<'_, FileReference> {
        self.0.reference.borrow()
    }

    pub fn set_loaded(self, value: bool) {
        self.0.loaded.set(value)
    }

    pub fn loaded(self) -> bool {
        self.0.loaded.get()
    }
}

pub enum FileReference {
    None,
    FileDialogResult(Box<dyn FileDialogResult>),
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FileReferenceObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    reference: RefCell<FileReference>,

    loaded: Cell<bool>,
}

impl fmt::Debug for FileReferenceObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileReferenceObject")
    }
}
