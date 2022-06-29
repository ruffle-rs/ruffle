//! Object representation for sounds

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::backend::audio::SoundInstanceHandle;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates SoundChannel objects.
pub fn soundchannel_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::new(class);

    Ok(SoundChannelObject(GcCell::allocate(
        activation.context.gc_context,
        SoundChannelObjectData {
            base,
            sound: None,
            position: 0.0,
        },
    ))
    .into())
}

#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct SoundChannelObject<'gc>(GcCell<'gc, SoundChannelObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct SoundChannelObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The sound this object holds.
    #[collect(require_static)]
    sound: Option<SoundInstanceHandle>,

    /// Position of the last playing sound in milliseconds.
    position: f64,
}

impl<'gc> SoundChannelObject<'gc> {
    /// Convert a bare sound instance into it's object representation.
    pub fn from_sound_instance(
        activation: &mut Activation<'_, 'gc, '_>,
        sound: SoundInstanceHandle,
    ) -> Result<Self, Error> {
        let class = activation.avm2().classes().soundchannel;
        let base = ScriptObjectData::new(class);

        let mut sound_object = SoundChannelObject(GcCell::allocate(
            activation.context.gc_context,
            SoundChannelObjectData {
                base,
                sound: Some(sound),
                position: 0.0,
            },
        ));
        sound_object.install_instance_slots(activation);

        class.call_native_init(Some(sound_object.into()), &[], activation)?;

        Ok(sound_object)
    }

    /// Return the backend handle to the currently playing sound instance.
    pub fn instance(self) -> Option<SoundInstanceHandle> {
        self.0.read().sound
    }

    /// Return the position of the playing sound in seconds.
    pub fn position(self) -> f64 {
        self.0.read().position
    }

    /// Set the position of the playing sound in seconds.
    pub fn set_position(self, mc: MutationContext<'gc, '_>, value: f64) {
        self.0.write(mc).position = value;
    }
}

impl<'gc> TObject<'gc> for SoundChannelObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Object::from(*self).into())
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn as_sound_channel(self) -> Option<SoundChannelObject<'gc>> {
        Some(self)
    }

    fn set_sound_instance(self, mc: MutationContext<'gc, '_>, sound: SoundInstanceHandle) {
        self.0.write(mc).sound = Some(sound);
    }
}
