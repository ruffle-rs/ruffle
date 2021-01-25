use crate::{
    avm1::SoundObject,
    display_object::{
        self, DisplayObject, SoundTransform as DisplayObjectSoundTransform, TDisplayObject,
    },
};
use downcast_rs::Downcast;
use gc_arena::{Collect, CollectionContext};
use generational_arena::{Arena, Index};

pub mod decoders;
pub mod swf {
    pub use swf::{
        read, AudioCompression, CharacterId, Sound, SoundEnvelope, SoundEnvelopePoint, SoundEvent,
        SoundFormat, SoundInfo, SoundStreamHead,
    };
}

pub type AudioStreamHandle = Index;
pub type SoundHandle = Index;
pub type SoundInstanceHandle = Index;

type Error = Box<dyn std::error::Error>;

pub trait AudioBackend: Downcast {
    fn play(&mut self);
    fn pause(&mut self);
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error>;
    fn preload_sound_stream_head(
        &mut self,
        _clip_id: swf::CharacterId,
        _clip_frame: u16,
        _stream_info: &swf::SoundStreamHead,
    ) {
    }
    fn preload_sound_stream_block(
        &mut self,
        _clip_id: swf::CharacterId,
        _clip_frame: u16,
        _audio_data: &[u8],
    ) {
    }
    fn preload_sound_stream_end(&mut self, _clip_id: swf::CharacterId) {}

    /// Starts playing a sound instance that is not tied to a MovieClip timeline.
    /// In Flash, this is known as an "Event" sound.
    fn start_sound(
        &mut self,
        sound: SoundHandle,
        settings: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error>;

    fn start_stream(
        &mut self,
        clip_id: crate::prelude::CharacterId,
        clip_frame: u16,
        clip_data: crate::tag_utils::SwfSlice,
        handle: &swf::SoundStreamHead,
    ) -> Result<AudioStreamHandle, Error>;

    /// Stops a playing sound instance.
    /// No-op if the sound is not playing.
    fn stop_sound(&mut self, sound: SoundInstanceHandle);

    /// Stops a playing stream souund.
    /// Should be called whenever a MovieClip timeline stops playing or seeks to a new frame.
    fn stop_stream(&mut self, stream: AudioStreamHandle);

    /// Good ol' stopAllSounds() :-)
    fn stop_all_sounds(&mut self);

    /// Get the position of a sound instance in milliseconds.
    /// Returns `None` if ther sound is not/no longer playing
    fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<u32>;

    /// Get the duration of a sound in milliseconds.
    /// Returns `None` if sound is not registered.
    fn get_sound_duration(&self, sound: SoundHandle) -> Option<u32>;

    /// Set the volume transform for a sound instance.
    fn set_sound_transform(&mut self, instance: SoundInstanceHandle, transform: SoundTransform);

    // TODO: Eventually remove this/move it to library.
    fn is_loading_complete(&self) -> bool {
        true
    }
    fn tick(&mut self) {}

    /// Inform the audio backend of the current stage frame rate.
    ///
    /// This is only necessary if your particular audio backend needs to know
    /// what the stage frame rate is. Otherwise, you are free to avoid
    /// implementing it.
    fn set_frame_rate(&mut self, _frame_rate: f64) {}
}

impl_downcast!(AudioBackend);

/// Audio backend that ignores all audio.
pub struct NullAudioBackend {
    sounds: Arena<()>,
    streams: Arena<()>,
}

impl NullAudioBackend {
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            streams: Arena::new(),
            sounds: Arena::new(),
        }
    }
}

impl AudioBackend for NullAudioBackend {
    fn play(&mut self) {}
    fn pause(&mut self) {}
    fn register_sound(&mut self, _sound: &swf::Sound) -> Result<SoundHandle, Error> {
        Ok(self.sounds.insert(()))
    }

    fn start_sound(
        &mut self,
        _sound: SoundHandle,
        _sound_info: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error> {
        Ok(SoundInstanceHandle::from_raw_parts(0, 0))
    }

    fn start_stream(
        &mut self,
        _clip_id: crate::prelude::CharacterId,
        _stream_start_frame: u16,
        _clip_data: crate::tag_utils::SwfSlice,
        _handle: &swf::SoundStreamHead,
    ) -> Result<AudioStreamHandle, Error> {
        Ok(self.streams.insert(()))
    }

    fn stop_sound(&mut self, _sound: SoundInstanceHandle) {}

    fn stop_stream(&mut self, stream: AudioStreamHandle) {
        self.streams.remove(stream);
    }
    fn stop_all_sounds(&mut self) {}
    fn get_sound_position(&self, _instance: SoundInstanceHandle) -> Option<u32> {
        None
    }
    fn get_sound_duration(&self, _sound: SoundHandle) -> Option<u32> {
        None
    }

    fn set_sound_transform(&mut self, _instance: SoundInstanceHandle, _transform: SoundTransform) {}
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        NullAudioBackend::new()
    }
}

pub struct AudioManager<'gc> {
    /// The list of actively playing sounds.
    sounds: Vec<SoundInstance<'gc>>,

    /// The global sound transform applied to all sounds.
    global_sound_transform: DisplayObjectSoundTransform,
}

impl<'gc> AudioManager<'gc> {
    pub const MAX_SOUNDS: usize = 32;

    pub fn new() -> Self {
        Self {
            sounds: Vec::with_capacity(Self::MAX_SOUNDS),
            global_sound_transform: Default::default(),
        }
    }

    pub fn update_sounds(
        &mut self,
        audio: &mut dyn AudioBackend,
        gc_context: gc_arena::MutationContext<'gc, '_>,
    ) -> Vec<SoundInstance<'gc>> {
        let mut removed = vec![];
        self.sounds.retain(|sound| {
            if let Some(pos) = audio.get_sound_position(sound.instance) {
                // Sounds still playing; update position.
                if let Some(avm1_object) = sound.avm1_object {
                    avm1_object.set_position(gc_context, pos);
                }
                true
            } else {
                // Sound ended; fire end event.
                removed.push(sound.clone());
                false
            }
        });
        removed
    }

    /// Update the sound transforms for all sounds.
    /// This should be called whenever a sound transform changes on a display object.
    pub fn update_sound_transforms(&mut self, audio: &mut dyn AudioBackend) {
        // This updates the sound transform for all sounds, even though the transform has
        // only changed on a single display object. There are only a small amount
        // of sounds playing at any time, so this shouldn't be a big deal.
        for sound in &self.sounds {
            let transform = self.transform_for_sound(sound);
            audio.set_sound_transform(sound.instance, transform);
        }
    }

    pub fn start_sound(
        &mut self,
        audio: &mut dyn AudioBackend,
        sound: SoundHandle,
        settings: &swf::SoundInfo,
        display_object: Option<DisplayObject<'gc>>,
        avm1_object: Option<SoundObject<'gc>>,
    ) -> Option<SoundInstanceHandle> {
        if self.sounds.len() < Self::MAX_SOUNDS {
            let handle = audio.start_sound(sound, settings).ok()?;
            let instance = SoundInstance {
                sound,
                instance: handle,
                display_object,
                avm1_object,
            };
            audio.set_sound_transform(handle, self.transform_for_sound(&instance));
            self.sounds.push(instance);
            Some(handle)
        } else {
            None
        }
    }

    pub fn stop_sound(&mut self, audio: &mut dyn AudioBackend, instance: SoundInstanceHandle) {
        if let Some(i) = self
            .sounds
            .iter()
            .position(|other| other.instance == instance)
        {
            let instance = &self.sounds[i];
            audio.stop_sound(instance.instance);
            self.sounds.swap_remove(i);
        }
    }

    pub fn stop_sounds_with_handle(&mut self, audio: &mut dyn AudioBackend, sound: SoundHandle) {
        self.sounds.retain(move |other| {
            if other.sound == sound {
                audio.stop_sound(other.instance);
                false
            } else {
                true
            }
        });
    }

    pub fn stop_sounds_with_display_object(
        &mut self,
        audio: &mut dyn AudioBackend,
        display_object: DisplayObject<'gc>,
    ) {
        self.sounds.retain(move |sound| {
            if let Some(other) = sound.display_object {
                if DisplayObject::ptr_eq(other, display_object) {
                    audio.stop_sound(sound.instance);
                    return false;
                }
            }
            true
        });
    }

    pub fn stop_all_sounds(&mut self, audio: &mut dyn AudioBackend) {
        self.sounds.clear();
        audio.stop_all_sounds();
    }

    pub fn is_sound_playing_with_handle(&mut self, sound: SoundHandle) -> bool {
        self.sounds.iter().any(|other| other.sound == sound)
    }

    pub fn start_stream(
        &mut self,
        audio: &mut dyn AudioBackend,
        clip_id: swf::CharacterId,

        frame: u16,
        data: crate::tag_utils::SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Option<AudioStreamHandle> {
        audio.start_stream(clip_id, frame, data, stream_info).ok()
    }

    pub fn stop_stream(&mut self, audio: &mut dyn AudioBackend, handle: AudioStreamHandle) {
        audio.stop_stream(handle)
    }

    pub fn global_sound_transform(&self) -> &DisplayObjectSoundTransform {
        &self.global_sound_transform
    }

    pub fn set_global_sound_transform(
        &mut self,
        audio: &mut dyn AudioBackend,
        sound_transform: DisplayObjectSoundTransform,
    ) {
        self.global_sound_transform = sound_transform;
        self.update_sound_transforms(audio);
    }

    fn transform_for_sound(&self, sound: &SoundInstance<'gc>) -> SoundTransform {
        let mut transform = DisplayObjectSoundTransform::default();
        let mut parent = sound.display_object;
        while let Some(display_object) = parent {
            transform.concat(&display_object.sound_transform());
            parent = display_object.parent();
        }
        transform.concat(&self.global_sound_transform);
        SoundTransform::from_display_object_transform(&transform)
    }
}

impl<'gc> Default for AudioManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<'gc> Collect for AudioManager<'gc> {
    fn trace(&self, cc: CollectionContext) {
        for sound in &self.sounds {
            sound.trace(cc);
        }
    }
}
#[derive(Clone)]
pub struct SoundInstance<'gc> {
    instance: SoundInstanceHandle,
    sound: SoundHandle,
    display_object: Option<DisplayObject<'gc>>,
    pub avm1_object: Option<SoundObject<'gc>>,
}

unsafe impl<'gc> Collect for SoundInstance<'gc> {
    fn trace(&self, cc: CollectionContext) {
        self.display_object.trace(cc);
        self.avm1_object.trace(cc);
    }
}

/// A sound transform for a playing sound, for use by audio backends.
/// This differs from `display_object::SoundTranform` by being
/// already converted to `f32` and having `volume` baked in.
#[derive(Debug, PartialEq, Clone)]
pub struct SoundTransform {
    pub left_to_left: f32,
    pub left_to_right: f32,
    pub right_to_left: f32,
    pub right_to_right: f32,
}

impl SoundTransform {
    /// Converts from a `display_object::SoundTransform` to a `backend::audio::SoundTransform`.
    fn from_display_object_transform(other: &DisplayObjectSoundTransform) -> Self {
        const SCALE: f32 = (display_object::SoundTransform::MAX_VOLUME
            * display_object::SoundTransform::MAX_VOLUME) as f32;
        Self {
            left_to_left: other.left_to_left as f32 * other.volume as f32 / SCALE,
            left_to_right: other.left_to_right as f32 * other.volume as f32 / SCALE,
            right_to_left: other.right_to_left as f32 * other.volume as f32 / SCALE,
            right_to_right: other.right_to_right as f32 * other.volume as f32 / SCALE,
        }
    }
}

impl Default for SoundTransform {
    fn default() -> Self {
        Self {
            left_to_left: 1.0,
            left_to_right: 0.0,
            right_to_left: 0.0,
            right_to_right: 1.0,
        }
    }
}
