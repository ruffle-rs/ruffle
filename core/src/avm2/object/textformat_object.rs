//! Object representation for TextFormat

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::html::{TextDisplay, TextFormat};
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
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

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextFormatObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    text_format: RefCell<TextFormat>,
}

const _: () = assert!(std::mem::offset_of!(TextFormatObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<TextFormatObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

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
}

impl<'gc> TObject<'gc> for TextFormatObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    /// Unwrap this object as a text format.
    fn as_text_format(&self) -> Option<Ref<TextFormat>> {
        Some(self.0.text_format.borrow())
    }

    /// Unwrap this object as a mutable text format.
    fn as_text_format_mut(&self) -> Option<RefMut<TextFormat>> {
        Some(self.0.text_format.borrow_mut())
    }
}
