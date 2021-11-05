//! Object representation for TextFormat

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::html::TextFormat;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates TextFormat objects.
pub fn textformat_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(TextFormatObject(GcCell::allocate(
        activation.context.gc_context,
        TextFormatObjectData {
            base,
            text_format: Default::default(),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct TextFormatObject<'gc>(GcCell<'gc, TextFormatObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct TextFormatObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    text_format: TextFormat,
}

impl<'gc> TextFormatObject<'gc> {
    pub fn from_text_format(
        activation: &mut Activation<'_, 'gc, '_>,
        text_format: TextFormat,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().textformat;
        let proto = activation.avm2().prototypes().textformat;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut this: Object<'gc> = Self(GcCell::allocate(
            activation.context.gc_context,
            TextFormatObjectData { base, text_format },
        ))
        .into();
        this.install_instance_traits(activation, class)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for TextFormatObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(Self(GcCell::allocate(
            activation.context.gc_context,
            TextFormatObjectData {
                base,
                text_format: Default::default(),
            },
        ))
        .into())
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    /// Unwrap this object as a text format.
    fn as_text_format(&self) -> Option<Ref<TextFormat>> {
        Some(Ref::map(self.0.read(), |d| &d.text_format))
    }

    /// Unwrap this object as a mutable text format.
    fn as_text_format_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<TextFormat>> {
        Some(RefMut::map(self.0.write(mc), |d| &mut d.text_format))
    }
}
