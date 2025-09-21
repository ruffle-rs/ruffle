//! Object representation for TextFormat

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, TObject};
use crate::avm2::Error;
use crate::html::{TextDisplay, TextFormat};
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak};
use std::cell::{Ref, RefCell, RefMut};

/// A class instance allocator that allocates TextFormat objects.
pub fn textformat_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    Ok(TextFormatObject(Gc::new(
        activation.gc(),
        TextFormatObjectData {
            base: ScriptObjectData::new(class),
            text_format: RefCell::new(TextFormat {
                display: Some(TextDisplay::Block),
                ..Default::default()
            }),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextFormatObject<'gc>(pub Gc<'gc, TextFormatObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct TextFormatObjectWeak<'gc>(pub GcWeak<'gc, TextFormatObjectData<'gc>>);

impl fmt::Debug for TextFormatObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextFormatObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextFormatObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    text_format: RefCell<TextFormat>,
}

impl<'gc> TextFormatObject<'gc> {
    pub fn from_text_format(
        activation: &mut Activation<'_, 'gc>,
        text_format: TextFormat,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let class = activation.avm2().classes().textformat;

        let this: Object<'gc> = Self(Gc::new(
            activation.gc(),
            TextFormatObjectData {
                base: ScriptObjectData::new(class),
                text_format: RefCell::new(text_format),
            },
        ))
        .into();

        Ok(this)
    }

    pub fn text_format(self) -> Ref<'gc, TextFormat> {
        Gc::as_ref(self.0).text_format.borrow()
    }

    pub fn text_format_mut(self) -> RefMut<'gc, TextFormat> {
        Gc::as_ref(self.0).text_format.borrow_mut()
    }
}

impl<'gc> TObject<'gc> for TextFormatObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}
