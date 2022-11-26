//! AVM1 object type to represent Sound objects.

use crate::avm1::{Object, ScriptObject, TObject};
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::display_object::DisplayObject;
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

/// A SoundObject that is tied to a sound from the AudioBackend.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct SoundObject<'gc>(GcCell<'gc, SoundObjectData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct SoundObjectData<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The sound that is attached to this object.
    #[collect(require_static)]
    sound: Option<SoundHandle>,

    /// The instance of the last played sound on this object.
    #[collect(require_static)]
    sound_instance: Option<SoundInstanceHandle>,

    /// Sounds in AVM1 are tied to a specific movie clip.
    owner: Option<DisplayObject<'gc>>,

    /// Position of the last playing sound in milliseconds.
    position: u32,

    /// Duration of the currently attached sound in milliseconds.
    duration: Option<u32>,

    /// Whether this sound is an external streaming MP3.
    /// This will be true if `Sound.loadSound` was called with `isStreaming` of `true`.
    /// A streaming sound can only have a single active instance.
    is_streaming: bool,
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
        proto: Object<'gc>,
    ) -> SoundObject<'gc> {
        SoundObject(GcCell::allocate(
            gc_context,
            SoundObjectData {
                base: ScriptObject::new(gc_context, Some(proto)),
                sound: None,
                sound_instance: None,
                owner: None,
                position: 0,
                duration: None,
                is_streaming: false,
            },
        ))
    }

    pub fn duration(self) -> Option<u32> {
        self.0.read().duration
    }

    pub fn set_duration(self, gc_context: MutationContext<'gc, '_>, duration: Option<u32>) {
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

    pub fn is_streaming(self) -> bool {
        self.0.read().is_streaming
    }

    pub fn set_is_streaming(self, gc_context: MutationContext<'gc, '_>, is_streaming: bool) {
        self.0.write(gc_context).is_streaming = is_streaming;
    }
}

impl<'gc> TObject<'gc> for SoundObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_sound_object -> SoundObject::empty_sound);
    });
}
