//! AVM1 object type to represent Sound objects.

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::display_object::DisplayObject;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A SounObject that is tied to a sound from the AudioBackend.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct SoundObject<'gc>(GcCell<'gc, SoundObjectData<'gc>>);

pub struct SoundObjectData<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The sound that is attached to this object.
    sound: Option<SoundHandle>,

    /// The instance of the last played sound on this object.
    sound_instance: Option<SoundInstanceHandle>,

    /// Sounds in AVM1 are tied to a speicifc movie clip.
    owner: Option<DisplayObject<'gc>>,

    /// Position of the last playing sound in milliseconds.
    position: u32,

    /// Duration of the currently attached sound in milliseconds.
    duration: u32,
}

unsafe impl<'gc> Collect for SoundObjectData<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.owner.trace(cc);
    }
}

impl fmt::Debug for SoundObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("SoundObject")
            .field("sound", &this.sound)
            .field("sound_instance", &this.sound_instance)
            .field("owner", &this.owner)
            .finish()
    }
}

impl<'gc> SoundObject<'gc> {
    pub fn empty_sound(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> SoundObject<'gc> {
        SoundObject(GcCell::allocate(
            gc_context,
            SoundObjectData {
                base: ScriptObject::object(gc_context, proto),
                sound: None,
                sound_instance: None,
                owner: None,
                position: 0,
                duration: 0,
            },
        ))
    }

    pub fn duration(self) -> u32 {
        self.0.read().duration
    }

    pub fn set_duration(self, gc_context: MutationContext<'gc, '_>, duration: u32) {
        self.0.write(gc_context).duration = duration;
    }

    pub fn sound(self) -> Option<SoundHandle> {
        self.0.read().sound
    }

    pub fn set_sound(self, gc_context: MutationContext<'gc, '_>, sound: Option<SoundHandle>) {
        self.0.write(gc_context).sound = sound;
    }

    pub fn sound_instance(self) -> Option<SoundInstanceHandle> {
        self.0.read().sound_instance
    }

    pub fn set_sound_instance(
        self,
        gc_context: MutationContext<'gc, '_>,
        sound_instance: Option<SoundInstanceHandle>,
    ) {
        self.0.write(gc_context).sound_instance = sound_instance;
    }

    pub fn owner(self) -> Option<DisplayObject<'gc>> {
        self.0.read().owner
    }

    pub fn set_owner(
        self,
        gc_context: MutationContext<'gc, '_>,
        owner: Option<DisplayObject<'gc>>,
    ) {
        self.0.write(gc_context).owner = owner;
    }

    pub fn position(self) -> u32 {
        self.0.read().position
    }

    pub fn set_position(self, gc_context: MutationContext<'gc, '_>, position: u32) {
        self.0.write(gc_context).position = position;
    }
}

impl<'gc> TObject<'gc> for SoundObject<'gc> {
    impl_custom_object!(base);

    #[allow(clippy::new_ret_no_self)]
    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, '_, 'gc, '_>,

        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(SoundObject::empty_sound(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.sound),
        )
        .into())
    }

    fn as_sound_object(&self) -> Option<SoundObject<'gc>> {
        Some(*self)
    }
}
