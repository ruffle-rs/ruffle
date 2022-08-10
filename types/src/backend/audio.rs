use downcast_rs::{impl_downcast, Downcast};
use gc_arena::Collect;
use generational_arena::{Arena, Index};

pub mod swf {
    pub use swf::{
        read, AudioCompression, CharacterId, Sound, SoundEnvelope, SoundEnvelopePoint, SoundEvent,
        SoundFormat, SoundInfo, SoundStreamHead,
    };
}

use crate::display_object;
use instant::Duration;

pub type SoundHandle = Index;
pub type SoundInstanceHandle = Index;

type Error = Box<dyn std::error::Error>;

pub trait AudioBackend: Downcast {
    fn play(&mut self);
    fn pause(&mut self);
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error>;

    /// Plays a sound.
    fn start_sound(
        &mut self,
        sound: SoundHandle,
        settings: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error>;

    /// Starts playing a "stream" sound, which is an audio stream that is distributed
    /// among the frames of a Flash MovieClip.
    /// On the web backend, `stream_handle` should be the handle for the preloaded stream.
    /// Other backends can pass `None`.
    fn start_stream(
        &mut self,
        stream_handle: Option<SoundHandle>,
        clip_frame: u16,
        clip_data: crate::tag_utils::SwfSlice,
        handle: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, Error>;

    /// Stops a playing sound instance.
    /// No-op if the sound is not playing.
    fn stop_sound(&mut self, sound: SoundInstanceHandle);

    /// Good ol' stopAllSounds() :-)
    fn stop_all_sounds(&mut self);

    /// Get the position of a sound instance in milliseconds.
    /// Returns `None` if ther sound is not/no longer playing
    fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<f64>;

    /// Get the duration of a sound in milliseconds.
    /// Returns `None` if sound is not registered.
    fn get_sound_duration(&self, sound: SoundHandle) -> Option<f64>;

    /// Get the size of the data stored within a given sound.
    ///
    /// This is specifically measured in compressed bytes.
    fn get_sound_size(&self, sound: SoundHandle) -> Option<u32>;

    /// Get the sound format that a given sound was added with.
    fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat>;

    /// Set the volume transform for a sound instance.
    fn set_sound_transform(&mut self, instance: SoundInstanceHandle, transform: SoundTransform);

    // TODO: Eventually remove this/move it to library.
    #[inline]
    fn is_loading_complete(&self) -> bool {
        true
    }

    /// Allows the audio backend to update.
    ///
    /// Runs once per event loop iteration.
    #[inline]
    fn tick(&mut self) {}

    /// Inform the audio backend of the current stage frame rate.
    ///
    /// This is only necessary if your particular audio backend needs to know
    /// what the stage frame rate is. Otherwise, you are free to avoid
    /// implementing it.
    #[inline]
    fn set_frame_rate(&mut self, _frame_rate: f64) {}

    /// The approximate interval that this backend updates a sound's position value. `None` if the
    /// value is unknown.
    ///
    /// This determines the time threshold for syncing embedded audio streams to the animation.
    #[inline]
    fn position_resolution(&self) -> Option<Duration> {
        None
    }
}

impl_downcast!(AudioBackend);

/// Information about a sound provided to `NullAudioBackend`.
struct NullSound {
    /// The duration of the sound in milliseconds.
    duration: f64,

    /// The compressed size of the sound data, excluding MP3 latency seek data.
    size: u32,

    /// The stated format of the sound data.
    format: swf::SoundFormat,
}

/// Audio backend that ignores all audio.
pub struct NullAudioBackend {
    sounds: Arena<NullSound>,
}

impl NullAudioBackend {
    #[inline]
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            sounds: Arena::new(),
        }
    }
}

impl AudioBackend for NullAudioBackend {
    #[inline]
    fn play(&mut self) {}

    #[inline]
    fn pause(&mut self) {}

    #[inline]
    fn register_sound(&mut self, sound: &swf::Sound) -> Result<SoundHandle, Error> {
        // Slice off latency seek for MP3 data.
        let data = if sound.format.compression == swf::AudioCompression::Mp3 {
            sound.data.get(2..).ok_or("MP3 sound is too short")?
        } else {
            sound.data
        };

        // AS duration does not subtract `skip_sample_frames`.
        let num_sample_frames: f64 = sound.num_samples.into();
        let sample_rate: f64 = sound.format.sample_rate.into();
        let duration = num_sample_frames * 1000.0 / sample_rate;

        Ok(self.sounds.insert(NullSound {
            duration,
            size: data.len() as u32,
            format: sound.format.clone(),
        }))
    }

    #[inline]
    fn start_sound(
        &mut self,
        _sound: SoundHandle,
        _sound_info: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error> {
        Ok(SoundInstanceHandle::from_raw_parts(0, 0))
    }

    #[inline]
    fn start_stream(
        &mut self,
        _stream_handle: Option<SoundHandle>,
        _clip_frame: u16,
        _clip_data: crate::tag_utils::SwfSlice,
        _handle: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, Error> {
        Ok(SoundInstanceHandle::from_raw_parts(0, 0))
    }

    #[inline]
    fn stop_sound(&mut self, _sound: SoundInstanceHandle) {}

    #[inline]
    fn stop_all_sounds(&mut self) {}

    #[inline]
    fn get_sound_position(&self, _instance: SoundInstanceHandle) -> Option<f64> {
        Some(0.0)
    }

    #[inline]
    fn get_sound_duration(&self, sound: SoundHandle) -> Option<f64> {
        if let Some(sound) = self.sounds.get(sound) {
            Some(sound.duration)
        } else {
            None
        }
    }

    #[inline]
    fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
        if let Some(sound) = self.sounds.get(sound) {
            Some(sound.size)
        } else {
            None
        }
    }

    #[inline]
    fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat> {
        self.sounds.get(sound).map(|s| &s.format)
    }

    #[inline]
    fn set_sound_transform(&mut self, _instance: SoundInstanceHandle, _transform: SoundTransform) {}
}

impl Default for NullAudioBackend {
    #[inline]
    fn default() -> Self {
        NullAudioBackend::new()
    }
}

/// A sound transform for a playing sound, for use by audio backends.
/// This differs from `display_object::SoundTransform` by being
/// already converted to `f32` and having `volume` baked in.
#[derive(Debug, PartialEq, Clone, Collect)]
#[collect(require_static)]
pub struct SoundTransform {
    pub left_to_left: f32,
    pub left_to_right: f32,
    pub right_to_left: f32,
    pub right_to_right: f32,
}

impl From<display_object::SoundTransform> for SoundTransform {
    /// Converts from a `display_object::SoundTransform` to a `backend::audio::SoundTransform`.
    #[inline]
    fn from(other: display_object::SoundTransform) -> Self {
        const SCALE: f32 = display_object::SoundTransform::MAX_VOLUME.pow(2) as f32;

        // The volume multiplication wraps around at `u32::MAX`.
        Self {
            left_to_left: other.left_to_left.wrapping_mul(other.volume) as f32 / SCALE,
            left_to_right: other.left_to_right.wrapping_mul(other.volume) as f32 / SCALE,
            right_to_left: other.right_to_left.wrapping_mul(other.volume) as f32 / SCALE,
            right_to_right: other.right_to_right.wrapping_mul(other.volume) as f32 / SCALE,
        }
    }
}

impl Default for SoundTransform {
    #[inline]
    fn default() -> Self {
        Self {
            left_to_left: 1.0,
            left_to_right: 0.0,
            right_to_left: 0.0,
            right_to_right: 1.0,
        }
    }
}
