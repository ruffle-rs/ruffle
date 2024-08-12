//! Object representation for sounds

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::backend::audio::SoundInstanceHandle;
use crate::context::UpdateContext;
use crate::display_object::SoundTransform;
use core::fmt;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::cell::{Cell, RefCell};

/// A class instance allocator that allocates SoundChannel objects.
pub fn sound_channel_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(SoundChannelObject(Gc::new(
        activation.context.gc_context,
        SoundChannelObjectData {
            base,
            sound_channel_data: RefCell::new(SoundChannelData::NotLoaded {
                sound_transform: None,
                should_stop: false,
            }),
            position: Cell::new(0.0),
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct SoundChannelObject<'gc>(pub Gc<'gc, SoundChannelObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct SoundChannelObjectWeak<'gc>(pub GcWeak<'gc, SoundChannelObjectData<'gc>>);

impl fmt::Debug for SoundChannelObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoundChannelObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct SoundChannelObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The sound this object holds.
    sound_channel_data: RefCell<SoundChannelData>,

    /// Position of the last playing sound in milliseconds.
    position: Cell<f64>,
}

const _: () = assert!(std::mem::offset_of!(SoundChannelObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<SoundChannelObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

pub enum SoundChannelData {
    NotLoaded {
        sound_transform: Option<SoundTransform>,
        should_stop: bool,
    },
    Loaded {
        sound_instance: SoundInstanceHandle,
    },
}

impl<'gc> SoundChannelObject<'gc> {
    /// Convert a bare sound instance into it's object representation.
    pub fn empty(activation: &mut Activation<'_, 'gc>) -> Result<Self, Error<'gc>> {
        let class = activation.avm2().classes().soundchannel;
        let base = ScriptObjectData::new(class);

        let sound_object = SoundChannelObject(Gc::new(
            activation.context.gc_context,
            SoundChannelObjectData {
                base,
                sound_channel_data: RefCell::new(SoundChannelData::NotLoaded {
                    sound_transform: None,
                    should_stop: false,
                }),
                position: Cell::new(0.0),
            },
        ));
        sound_object.install_instance_slots(activation.context.gc_context);

        class.call_native_init(Value::Object(sound_object.into()), &[], activation)?;

        Ok(sound_object)
    }

    /// Return the position of the playing sound in seconds.
    pub fn position(self, context: &mut UpdateContext<'gc>) -> f64 {
        // The position is cached on read. This means that if the position isn't read until after
        // the sound has played, the position will be 0 (#9952).
        let sound_channel_data = self.0.sound_channel_data.borrow();
        if let SoundChannelData::Loaded { sound_instance } = &*sound_channel_data {
            if let Some(pos) = context.audio.get_sound_position(*sound_instance) {
                self.0.position.set(pos);
            }
        }

        self.0.position.get()
    }

    pub fn instance(self) -> Option<SoundInstanceHandle> {
        let sound_channel_data = self.0.sound_channel_data.borrow();
        match &*sound_channel_data {
            SoundChannelData::NotLoaded { .. } => None,
            SoundChannelData::Loaded { sound_instance } => Some(*sound_instance),
        }
    }

    pub fn set_sound_instance(
        self,
        activation: &mut Activation<'_, 'gc>,
        instance: SoundInstanceHandle,
    ) {
        let mut sound_channel_data = self.0.sound_channel_data.borrow_mut();
        match &mut *sound_channel_data {
            SoundChannelData::NotLoaded {
                sound_transform,
                should_stop,
            } => {
                if let Some(sound_transform) = sound_transform {
                    activation
                        .context
                        .set_local_sound_transform(instance, sound_transform.clone());
                }

                if *should_stop {
                    activation.context.stop_sound(instance);
                }
                *sound_channel_data = SoundChannelData::Loaded {
                    sound_instance: instance,
                }
            }
            SoundChannelData::Loaded { sound_instance } => {
                panic!(
                    "Tried to replace loaded sound instance {sound_instance:?} with {instance:?}"
                )
            }
        }
    }

    pub fn sound_transform(self, activation: &mut Activation<'_, 'gc>) -> Option<SoundTransform> {
        let sound_channel_data = self.0.sound_channel_data.borrow();
        match &*sound_channel_data {
            SoundChannelData::NotLoaded {
                sound_transform, ..
            } => sound_transform.clone(),
            SoundChannelData::Loaded { sound_instance } => activation
                .context
                .local_sound_transform(*sound_instance)
                .cloned(),
        }
    }

    pub fn set_sound_transform(
        self,
        activation: &mut Activation<'_, 'gc>,
        new_sound_transform: SoundTransform,
    ) {
        let mut sound_channel_data = self.0.sound_channel_data.borrow_mut();
        match &mut *sound_channel_data {
            SoundChannelData::NotLoaded {
                sound_transform, ..
            } => {
                *sound_transform = Some(new_sound_transform);
            }
            SoundChannelData::Loaded { sound_instance } => {
                activation
                    .context
                    .set_local_sound_transform(*sound_instance, new_sound_transform);
            }
        }
    }

    pub fn stop(self, activation: &mut Activation<'_, 'gc>) {
        let mut sound_channel_data = self.0.sound_channel_data.borrow_mut();
        match &mut *sound_channel_data {
            SoundChannelData::NotLoaded {
                sound_transform: _,
                should_stop,
            } => {
                *should_stop = true;
            }
            SoundChannelData::Loaded { sound_instance } => {
                activation.context.stop_sound(*sound_instance);
            }
        }
    }
}

impl<'gc> TObject<'gc> for SoundChannelObject<'gc> {
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
        Ok(Object::from(*self).into())
    }

    fn as_sound_channel(self) -> Option<SoundChannelObject<'gc>> {
        Some(self)
    }
}
