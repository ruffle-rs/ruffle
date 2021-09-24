//! Object representation for sounds

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
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
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(SoundChannelObject(GcCell::allocate(
        activation.context.gc_context,
        SoundChannelObjectData { base, sound: None },
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
}

impl<'gc> SoundChannelObject<'gc> {
    /// Convert a bare sound instance into it's object representation.
    pub fn from_sound_instance(
        activation: &mut Activation<'_, 'gc, '_>,
        sound: SoundInstanceHandle,
    ) -> Result<Object<'gc>, Error> {
        let class = activation.avm2().classes().soundchannel;
        let proto = class
            .get_property(
                class.into(),
                &QName::new(Namespace::public(), "prototype").into(),
                activation,
            )?
            .coerce_to_object(activation)?;
        let base = ScriptObjectData::base_new(Some(proto), Some(class));

        let mut sound_object: Object<'gc> = SoundChannelObject(GcCell::allocate(
            activation.context.gc_context,
            SoundChannelObjectData {
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

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let base = ScriptObjectData::base_new(Some((*self).into()), None);

        Ok(SoundChannelObject(GcCell::allocate(
            activation.context.gc_context,
            SoundChannelObjectData { base, sound: None },
        ))
        .into())
    }

    fn as_sound_instance(self) -> Option<SoundInstanceHandle> {
        self.0.read().sound
    }

    fn set_sound_instance(self, mc: MutationContext<'gc, '_>, sound: SoundInstanceHandle) {
        self.0.write(mc).sound = Some(sound);
    }
}
