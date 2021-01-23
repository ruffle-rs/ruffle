use crate::{avm1::SoundObject, display_object::DisplayObject};
use downcast_rs::Downcast;
use gc_arena::{Collect, CollectionContext, MutationContext};
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

    /// Stops all active sound instances of a particular sound.
    /// Used by SWF `StartSound` tag with `SoundEvent::Stop`.
    fn stop_sounds_with_handle(&mut self, handle: SoundHandle);

    /// Returns whether a sound clip is playing.
    /// Used by SWF `StartSouynd` tag with `SoundEvent:Start`,
    /// which only plays a sound if that sound is not already playing.
    fn is_sound_playing_with_handle(&mut self, handle: SoundHandle) -> bool;

    /// Get the position of a sound instance in milliseconds.
    /// Returns `None` if ther sound is not/no longer playing
    fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<u32>;

    /// Get the duration of a sound in milliseconds.
    /// Returns `None` if sound is not registered.
    fn get_sound_duration(&self, sound: SoundHandle) -> Option<u32>;

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
    fn stop_sounds_with_handle(&mut self, _handle: SoundHandle) {}
    fn is_sound_playing_with_handle(&mut self, _handle: SoundHandle) -> bool {
        false
    }

    fn get_sound_position(&self, _instance: SoundInstanceHandle) -> Option<u32> {
        None
    }
    fn get_sound_duration(&self, _sound: SoundHandle) -> Option<u32> {
        None
    }
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        NullAudioBackend::new()
    }
}

pub struct AudioManager<'gc> {
    /// The list of actively playing sounds.
    sounds: Arena<SoundInstance<'gc>>,
}

impl<'gc> AudioManager<'gc> {
    pub const MAX_SOUNDS: usize = 32;

    pub fn new() -> Self {
        Self {
            sounds: Arena::with_capacity(Self::MAX_SOUNDS),
        }
    }

    pub fn update_sounds(
        &mut self,
        audio: &mut dyn AudioBackend,
        gc_context: MutationContext<'gc, '_>,
    ) {
        self.sounds.retain(move |_, sound| {
            if let Some(pos) = audio.get_sound_position(sound.instance) {
                if let Some(avm1_object) = sound.avm1_object {
                    avm1_object.set_position(gc_context, pos);
                }
                true
            } else {
                false
            }
        });
    }

    pub fn start_sound(
        &mut self,
        audio: &mut dyn AudioBackend,
        sound: SoundHandle,
        settings: &swf::SoundInfo,
        owner: Option<DisplayObject<'gc>>,
        avm1_object: Option<SoundObject<'gc>>,
    ) -> Option<SoundInstanceHandle> {
        if self.sounds.len() < Self::MAX_SOUNDS {
            let backend_instance = audio.start_sound(sound, settings).ok()?;
            let instance = SoundInstance {
                sound,
                instance: backend_instance,
                display_object: owner,
                avm1_object,
            };
            Some(self.sounds.insert(instance))
        } else {
            None
        }
    }

    pub fn stop_sound(&mut self, audio: &mut dyn AudioBackend, instance: SoundInstanceHandle) {
        if let Some(instance) = self.sounds.remove(instance) {
            audio.stop_sound(instance.instance);
        }
    }

    pub fn stop_sounds_with_handle(&mut self, audio: &mut dyn AudioBackend, sound: SoundHandle) {
        self.sounds.retain(move |_, other| {
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
        self.sounds.retain(move |_, sound| {
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
        self.sounds.iter().any(|(_, other)| other.sound == sound)
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
}

impl<'gc> Default for AudioManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<'gc> Collect for AudioManager<'gc> {
    fn trace(&self, cc: CollectionContext) {
        for (_, sound) in &self.sounds {
            sound.trace(cc);
        }
    }
}
struct SoundInstance<'gc> {
    instance: SoundInstanceHandle,
    sound: SoundHandle,
    display_object: Option<DisplayObject<'gc>>,
    avm1_object: Option<SoundObject<'gc>>,
}

unsafe impl<'gc> Collect for SoundInstance<'gc> {
    fn trace(&self, cc: CollectionContext) {
        self.display_object.trace(cc);
        self.avm1_object.trace(cc);
    }
}
