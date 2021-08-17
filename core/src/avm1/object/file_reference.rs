use crate::add_field_accessors;
use crate::avm1::activation::Activation;
use crate::avm1::object::date_object::DateObject;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::backend::ui::FileDialogResult;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A FileReference
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct FileReferenceObject<'gc>(GcCell<'gc, FileReferenceData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct FileReferenceData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    creation_date: Option<DateObject<'gc>>,
    creator: Option<String>,
    modification_date: Option<DateObject<'gc>>,
    name: Option<String>,
    post_data: String,
    size: Option<u64>,
    file_type: Option<String>,
}

impl fmt::Debug for FileReferenceObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("FileReference")
            .field("creationDate", &this.creation_date)
            .field("creator", &this.creator)
            .field("modificationDate", &this.modification_date)
            .field("name", &this.name)
            .field("postData", &this.post_data)
            .field("size", &this.size)
            .field("type", &this.file_type)
            .finish()
    }
}

impl<'gc> FileReferenceObject<'gc> {
    add_field_accessors!(
        [creation_date, Option<DateObject<'gc>>, set => set_creation_date, get => creation_date],
        [modification_date, Option<DateObject<'gc>>, set => set_modification_date, get => modification_date],
        [size, Option<u64>, set => set_size, get => size],
    );

    pub fn name(self) -> Option<String> {
        self.0.read().name.clone()
    }

    pub fn set_name(&self, gc_context: MutationContext<'gc, '_>, v: Option<String>) {
        self.0.write(gc_context).name = v;
    }

    pub fn creator(self) -> Option<String> {
        self.0.read().creator.clone()
    }

    pub fn set_creator(&self, gc_context: MutationContext<'gc, '_>, v: Option<String>) {
        self.0.write(gc_context).creator = v;
    }

    pub fn post_data(self) -> String {
        self.0.read().post_data.clone()
    }

    pub fn set_post_data(&self, gc_context: MutationContext<'gc, '_>, v: String) {
        self.0.write(gc_context).post_data = v;
    }

    pub fn file_type(self) -> Option<String> {
        self.0.read().file_type.clone()
    }

    pub fn set_file_type(&self, gc_context: MutationContext<'gc, '_>, v: Option<String>) {
        self.0.write(gc_context).file_type = v;
    }

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        FileReferenceObject(GcCell::allocate(
            gc_context,
            FileReferenceData {
                base: ScriptObject::object(gc_context, proto),
                creation_date: None,
                creator: None,
                modification_date: None,
                name: None,
                post_data: "".to_string(),
                size: None,
                file_type: None,
            },
        ))
    }

    pub fn init_from_dialog_result(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        dialog_result: Box<dyn FileDialogResult>,
    ) {
        self.set_creation_date(
            activation.context.gc_context,
            Some(DateObject::with_date_time(
                activation.context.gc_context,
                Some(activation.context.avm1.prototypes().date),
                dialog_result.creation_time(),
            )),
        );

        self.set_file_type(activation.context.gc_context, dialog_result.file_type());

        self.set_modification_date(
            activation.context.gc_context,
            Some(DateObject::with_date_time(
                activation.context.gc_context,
                Some(activation.context.avm1.prototypes().date),
                dialog_result.modification_time(),
            )),
        );

        self.set_name(activation.context.gc_context, dialog_result.file_name());

        self.set_size(activation.context.gc_context, dialog_result.size());

        self.set_creator(activation.context.gc_context, dialog_result.creator());
    }
}

impl<'gc> TObject<'gc> for FileReferenceObject<'gc> {
    impl_custom_object!(base {
        set(proto: file_reference);
        bare_object(as_file_reference_object -> FileReferenceObject::empty_object);
    });
}
