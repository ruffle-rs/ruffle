//! Object representation for sounds

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::backend::audio::SoundHandle;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Sound objects.
pub fn sound_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(SoundObject(GcCell::allocate(
        activation.context.gc_context,
        SoundObjectData { base, sound: None },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct SoundObject<'gc>(GcCell<'gc, SoundObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct SoundObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The sound this object holds.
    #[collect(require_static)]
    sound: Option<SoundHandle>,
}

impl<'gc> SoundObject<'gc> {
    /// Convert a bare sound into it's object representation.
    ///
    /// In AS3, library sounds are accessed through subclasses of `Sound`. As a
    /// result, this needs to take the subclass so that the returned object is
    /// an instance of the correct class.
    pub fn from_sound(
        activation: &mut Activation<'_, 'gc, '_>,
        class: Object<'gc>,
        sound: SoundHandle,
    ) -> Result<Object<'gc>, Error> {
        let proto = class
            .get_property(
                class,
                &QName::new(Namespace::public(), "prototype").into(),
                activation,
            )?
            .coerce_to_object(activation)?;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut sound_object: Object<'gc> = SoundObject(GcCell::allocate(
            activation.context.gc_context,
            SoundObjectData {
                base,
                sound: Some(sound),
            },
        ))
        .into();
        sound_object.install_instance_traits(activation, class)?;

        class.call_native_init(Some(sound_object), &[], activation, Some(class))?;

        Ok(sound_object)
    }
}

impl<'gc> TObject<'gc> for SoundObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Object::from(*self).into())
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(SoundObject(GcCell::allocate(
            activation.context.gc_context,
            SoundObjectData { base, sound: None },
        ))
        .into())
    }

    fn as_sound(self) -> Option<SoundHandle> {
        self.0.read().sound
    }

    /// Associate the object with a particular sound handle.
    ///
    /// This does nothing if the object is not a sound.
    fn set_sound(self, mc: MutationContext<'gc, '_>, sound: SoundHandle) {
        self.0.write(mc).sound = Some(sound);
    }
}
