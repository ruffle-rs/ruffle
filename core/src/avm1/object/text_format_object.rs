use crate::avm1::{Activation, Object, ScriptObject, TObject};
use crate::html::TextFormat;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::fmt;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct TextFormatObject<'gc>(GcCell<'gc, TextFormatData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct TextFormatData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    text_format: TextFormat,
}

impl fmt::Debug for TextFormatObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("TextFormatObject")
            .field("text_format", &this.text_format)
            .finish()
    }
}

impl<'gc> TextFormatObject<'gc> {
    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        Self(GcCell::allocate(
            gc_context,
            TextFormatData {
                base: ScriptObject::object(gc_context, proto),
                text_format: TextFormat::default(),
            },
        ))
    }

    pub fn new(activation: &mut Activation<'_, 'gc, '_>, text_format: TextFormat) -> Self {
        Self(GcCell::allocate(
            activation.context.gc_context,
            TextFormatData {
                base: ScriptObject::object(
                    activation.context.gc_context,
                    Some(activation.context.avm1.prototypes.text_format),
                ),
                text_format,
            },
        ))
    }

    pub fn text_format(&self) -> Ref<TextFormat> {
        Ref::map(self.0.read(), |o| &o.text_format)
    }

    pub fn text_format_mut(&self, gc_context: MutationContext<'gc, '_>) -> RefMut<TextFormat> {
        RefMut::map(self.0.write(gc_context), |o| &mut o.text_format)
    }

    pub fn set_text_format(&self, gc_context: MutationContext<'gc, '_>, text_format: TextFormat) {
        self.0.write(gc_context).text_format = text_format
    }
}

impl<'gc> TObject<'gc> for TextFormatObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_text_format_object -> TextFormatObject::empty_object);
    });
}
