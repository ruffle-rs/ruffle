use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::object::element_format_object::ElementFormatObject;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::string::AvmString;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use ruffle_common::utils::HasPrefixField;

pub fn content_element_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(ContentElementObject(Gc::new(
        activation.gc(),
        ContentElementObjectData {
            base: ScriptObjectData::new(class),
            element_format: Lock::new(None),
            text: Lock::new(None),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct ContentElementObject<'gc>(pub Gc<'gc, ContentElementObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct ContentElementObjectWeak<'gc>(pub GcWeak<'gc, ContentElementObjectData<'gc>>);

impl fmt::Debug for ContentElementObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContentElementObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ContentElementObjectData<'gc> {
    base: ScriptObjectData<'gc>,
    element_format: Lock<Option<ElementFormatObject<'gc>>>,
    text: Lock<Option<AvmString<'gc>>>,
}

impl<'gc> ContentElementObject<'gc> {
    pub fn element_format(self) -> Option<ElementFormatObject<'gc>> {
        self.0.element_format.get()
    }

    pub fn set_element_format(self, value: Option<ElementFormatObject<'gc>>, mc: &Mutation<'gc>) {
        unlock!(
            Gc::write(mc, self.0),
            ContentElementObjectData,
            element_format
        )
        .set(value);
    }

    pub fn text(self) -> Option<AvmString<'gc>> {
        self.0.text.get()
    }

    pub fn set_text(self, value: Option<AvmString<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), ContentElementObjectData, text).set(value);
    }
}

impl<'gc> TObject<'gc> for ContentElementObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
