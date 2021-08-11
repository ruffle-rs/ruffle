//! Object representation for sounds

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::backend::audio::SoundHandle;
use crate::{
    impl_avm2_custom_object, impl_avm2_custom_object_instance, impl_avm2_custom_object_properties,
};
use gc_arena::{Collect, GcCell, MutationContext};

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
                &QName::new(Namespace::public(), "prototype"),
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

        //TODO: Do we need to call the default initializer of sounds?

        Ok(sound_object)
    }
}

impl<'gc> TObject<'gc> for SoundObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);
    impl_avm2_custom_object_instance!(base);

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
