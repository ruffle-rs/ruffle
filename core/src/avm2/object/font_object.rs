use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::{Activation, ClassObject, Error};
use crate::character::Character;
use crate::font::Font;
use gc_arena::Mutation;
use gc_arena::{Collect, GcCell, GcWeakCell};
use std::cell::{Ref, RefMut};
use std::fmt;

/// A class instance allocator that allocates Font objects.
pub fn font_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let font = if let Some((movie, id)) = activation
        .context
        .library
        .avm2_class_registry()
        .class_symbol(class)
    {
        if let Some(lib) = activation.context.library.library_for_movie(movie) {
            if let Some(Character::Font(font)) = lib.character_by_id(id) {
                Some(*font)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(FontObject(GcCell::new(
        activation.context.gc_context,
        FontObjectData { base, font },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FontObject<'gc>(pub GcCell<'gc, FontObjectData<'gc>>);

impl<'gc> FontObject<'gc> {
    pub fn for_font(mc: &Mutation<'gc>, class: ClassObject<'gc>, font: Font<'gc>) -> Object<'gc> {
        let base = ScriptObjectData::new(class);
        FontObject(GcCell::new(
            mc,
            FontObjectData {
                base,
                font: Some(font),
            },
        ))
        .into()
    }
}

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct FontObjectWeak<'gc>(pub GcWeakCell<'gc, FontObjectData<'gc>>);

impl<'gc> TObject<'gc> for FontObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_font(&self) -> Option<Font<'gc>> {
        self.0.read().font
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct FontObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    font: Option<Font<'gc>>,
}

impl fmt::Debug for FontObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FontObject")
    }
}
