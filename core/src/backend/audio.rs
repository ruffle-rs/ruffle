use crate::{
    avm1::SoundObject,
    avm2::{Avm2, EventObject as Avm2EventObject, SoundChannelObject},
    buffer::Substream,
    context::UpdateContext,
    display_object::{self, DisplayObject, MovieClip, TDisplayObject},
};
use downcast_rs::Downcast;
use gc_arena::Collect;
use slotmap::{new_key_type, Key, SlotMap};

#[cfg(feature = "audio")]
pub mod decoders;
pub mod swf {
    pub use swf::{
        read, AudioCompression, CharacterId, Sound, SoundEnvelope, SoundEnvelopePoint, SoundEvent,
        SoundFormat, SoundInfo, SoundStreamHead,
    };
}

#[cfg(feature = "audio")]
mod mixer;
#[cfg(feature = "audio")]
pub use mixer::*;

#[cfg(not(feature = "audio"))]
mod decoders {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Too many sounds are playing")]
        TooManySounds,
    }
}

use crate::swf::{CharacterId, SoundInfo};
use thiserror::Error;
use web_time::Duration;

new_key_type! {
    pub struct SoundHandle;
    pub struct SoundInstanceHandle;
}

pub type DecodeError = decoders::Error;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum SoundStreamWrapping {
    /// Sound is being streamed from an SWF.
    ///
    /// MP3 chunks within SWFs are wrapped in a header that lists the number of
    /// samples in the chunk.
    Swf,

    /// Sound is being streamed from an unwrapped bitstream.
    ///
    /// Container formats that expose unwrapped bitstreams - that is, without
    /// needing additional unwrapping - may also use `Unwrapped`.
    Unwrapped,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SoundStreamInfo {
    pub wrapping: SoundStreamWrapping,
    pub stream_format: swf::SoundFormat,
    pub num_samples_per_block: u16,
    pub latency_seek: i16,
}

impl From<swf::SoundStreamHead> for SoundStreamInfo {
    fn from(swfhead: swf::SoundStreamHead) -> SoundStreamInfo {
        SoundStreamInfo {
            wrapping: SoundStreamWrapping::Swf,
            stream_format: swfhead.stream_format,
            num_samples_per_block: swfhead.num_samples_per_block,
            latency_seek: swfhead.latency_seek,
        }
    }
}

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("MP3 sound is too short")]
    ShortMp3,
}

pub trait AudioBackend: Downcast {
    fn play(&mut self);
    fn pause(&mut self);

    /// Registers an sound embedded in an SWF.
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, RegisterError>;

    /// Registers MP3 audio from an external source.
    fn register_mp3(&mut self, data: &[u8]) -> Result<SoundHandle, DecodeError>;

    /// Plays a sound.
    fn start_sound(
        &mut self,
        sound: SoundHandle,
        settings: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, DecodeError>;

    /// Starts playing a "stream" sound, which is an audio stream that is distributed
    /// among the frames of a Flash MovieClip.
    fn start_stream(
        &mut self,
        clip_data: crate::tag_utils::SwfSlice,
        handle: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, DecodeError>;

    /// Starts playing a "stream" sound, which is an audio stream that is
    /// distributed among multiple chunks of a larger data stream such as a
    /// Flash MovieClip or video container format.
    ///
    /// The `handle` parameter should refer to an actual SWF `SoundStreamHead`
    /// tag for SWF-wrapped audio. Other container formats will need to have
    /// their data converted to an equivalent SWF tag.
    ///
    /// MP3 data within the substream is expected to have sample count and
    /// seek offset data at the head of each chunk (SWF19 p.184). This is used
    /// to allow MP3 data to be buffered across multiple chunks or frames as
    /// necessary for playback.
    ///
    /// The sound instance constructed by this function explicitly supports
    /// streaming additional data to it by appending chunks to the underlying
    /// `Substream`, if the sound is still playing.
    fn start_substream(
        &mut self,
        stream_data: Substream,
        stream_info: &SoundStreamInfo,
    ) -> Result<SoundInstanceHandle, DecodeError>;

    /// Stops a playing sound instance.
    /// No-op if the sound is not playing.
    fn stop_sound(&mut self, sound: SoundInstanceHandle);

    /// Good ol' stopAllSounds() :-)
    fn stop_all_sounds(&mut self);

    /// Get the position of a sound instance in milliseconds.
    /// Returns `None` if the sound is not/no longer playing
    fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<f64>;

    /// Get the duration of a sound in milliseconds.
    /// Returns `None` if the sound is not registered.
    fn get_sound_duration(&self, sound: SoundHandle) -> Option<f64>;

    /// Get the size of the data stored within a given sound.
    ///
    /// This is specifically measured in compressed bytes.
    fn get_sound_size(&self, sound: SoundHandle) -> Option<u32>;

    /// Get the sound format that a given sound was added with.
    fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat>;

    /// Set the volume transform for a sound instance.
    fn set_sound_transform(&mut self, instance: SoundInstanceHandle, transform: SoundTransform);

    fn get_sound_peak(&mut self, instance: SoundInstanceHandle) -> Option<[f32; 2]>;

    /// Allows the audio backend to update.
    ///
    /// Runs once per event loop iteration.
    fn tick(&mut self) {}

    /// Inform the audio backend of the current stage frame rate.
    ///
    /// This is only necessary if your particular audio backend needs to know
    /// what the stage frame rate is. Otherwise, you are free to avoid
    /// implementing it.
    fn set_frame_rate(&mut self, _frame_rate: f64) {}

    /// The approximate interval that this backend updates a sound's position value. `None` if the
    /// value is unknown.
    ///
    /// This determines the time threshold for syncing embedded audio streams to the animation.
    fn position_resolution(&self) -> Option<Duration> {
        None
    }

    /// Returns the master volume of the audio backend.
    fn volume(&self) -> f32;

    /// Sets the master volume of the audio backend.
    fn set_volume(&mut self, volume: f32);

    /// Returns the last whole window of output samples.
    fn get_sample_history(&self) -> [[f32; 2]; 1024];

    /// Determine if a sound is still playing.
    fn is_sound_playing(&self, instance: SoundInstanceHandle) -> bool {
        self.get_sound_position(instance).is_some()
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
    sounds: SlotMap<SoundHandle, NullSound>,
    volume: f32,
}

impl NullAudioBackend {
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            sounds: SlotMap::with_key(),
            volume: 1.0,
        }
    }
}

impl AudioBackend for NullAudioBackend {
    fn play(&mut self) {}
    fn pause(&mut self) {}
    fn register_sound(&mut self, sound: &swf::Sound) -> Result<SoundHandle, RegisterError> {
        // Slice off latency seek for MP3 data.
        let data = if sound.format.compression == swf::AudioCompression::Mp3 {
            sound.data.get(2..).ok_or(RegisterError::ShortMp3)?
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

    fn register_mp3(&mut self, _data: &[u8]) -> Result<SoundHandle, DecodeError> {
        Ok(self.sounds.insert(NullSound {
            size: 0,
            duration: 0.0,
            format: swf::SoundFormat {
                compression: swf::AudioCompression::Mp3,
                sample_rate: 44100,
                is_stereo: true,
                is_16_bit: true,
            },
        }))
    }

    fn start_sound(
        &mut self,
        _sound: SoundHandle,
        _sound_info: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        Ok(SoundInstanceHandle::null())
    }

    fn start_stream(
        &mut self,
        _clip_data: crate::tag_utils::SwfSlice,
        _handle: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        Ok(SoundInstanceHandle::null())
    }

    fn start_substream(
        &mut self,
        _stream_data: Substream,
        _handle: &SoundStreamInfo,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        Ok(SoundInstanceHandle::null())
    }

    fn stop_sound(&mut self, _sound: SoundInstanceHandle) {}

    fn stop_all_sounds(&mut self) {}
    fn get_sound_position(&self, _instance: SoundInstanceHandle) -> Option<f64> {
        Some(0.0)
    }
    fn get_sound_duration(&self, sound: SoundHandle) -> Option<f64> {
        if let Some(sound) = self.sounds.get(sound) {
            Some(sound.duration)
        } else {
            None
        }
    }
    fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
        if let Some(sound) = self.sounds.get(sound) {
            Some(sound.size)
        } else {
            None
        }
    }

    fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat> {
        self.sounds.get(sound).map(|s| &s.format)
    }

    fn set_sound_transform(&mut self, _instance: SoundInstanceHandle, _transform: SoundTransform) {}

    fn get_sound_peak(&mut self, _instance: SoundInstanceHandle) -> Option<[f32; 2]> {
        None
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    fn get_sample_history(&self) -> [[f32; 2]; 1024] {
        [[0.0f32; 2]; 1024]
    }
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        NullAudioBackend::new()
    }
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct AudioManager<'gc> {
    /// The list of actively playing sounds.
    sounds: Vec<SoundInstance<'gc>>,

    /// The global sound transform applied to all sounds.
    #[collect(require_static)]
    global_sound_transform: display_object::SoundTransform,

    /// The number of seconds that a timeline audio stream should buffer before playing.
    ///
    /// This is returned by `_soundbuftime` in AVM1 and `SoundMixer.bufferTime` in AVM2.
    /// Currently unused by Ruffle.
    /// [ActionScript 3.0: SoundMixer.bufferTime](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/media/SoundMixer.html#bufferTime)
    stream_buffer_time: i32,

    /// Whether a sound transform has been changed.
    transforms_dirty: bool,
}

impl<'gc> AudioManager<'gc> {
    /// The maximum number of sound instances that can play at once.
    pub const MAX_SOUNDS: usize = 32;

    /// The default timeline stream buffer time in seconds.
    pub const DEFAULT_STREAM_BUFFER_TIME: i32 = 5;

    /// The threshold in seconds where an audio stream is considered too out-of-sync and will be stopped.
    pub const STREAM_RESTART_THRESHOLD: f64 = 1.0;

    /// The minimum audio syncing threshold in seconds.
    ///
    /// The player will adjust animation speed to stay within this many seconds of the audio track.
    pub const STREAM_DEFAULT_SYNC_THRESHOLD: f64 = 0.2;

    pub fn new() -> Self {
        Self {
            sounds: Vec::with_capacity(Self::MAX_SOUNDS),
            global_sound_transform: Default::default(),
            stream_buffer_time: Self::DEFAULT_STREAM_BUFFER_TIME,
            transforms_dirty: false,
        }
    }

    /// Update state of active sounds. Should be called once per frame.
    pub fn update_sounds(context: &mut UpdateContext<'gc>) {
        // We can't use 'context' to construct an event inside the
        // 'retain()' closure, so we queue the events up here, and fire
        // them after running 'retain()'
        let mut event_targets = Vec::new();

        // Update the position of sounds, and remove any completed sounds.
        context.audio_manager.sounds.retain(|sound| {
            if let Some(pos) = context.audio.get_sound_position(sound.instance) {
                // Sounds still playing; update position for AVM1 sounds.
                // AVM2 sounds do not update position and instead grab the position on demand.
                if let Some(avm1_object) = sound.avm1_object {
                    avm1_object.set_position(context.gc_context, pos.round() as u32);
                }
                true
            } else {
                // Sound ended.
                let duration = sound
                    .sound
                    .and_then(|sound| context.audio.get_sound_duration(sound))
                    .unwrap_or_default();
                if let Some(object) = sound.avm1_object {
                    object.set_position(context.gc_context, duration.round() as u32);

                    // Fire soundComplete event.
                    if let Some(root) = context.stage.root_clip() {
                        context.action_queue.queue_action(
                            root,
                            crate::context::ActionType::Method {
                                object: object.into(),
                                name: "onSoundComplete",
                                args: vec![],
                            },
                            false,
                        );
                    }
                }

                if let Some(object) = sound.avm2_object {
                    event_targets.push(object);
                }

                false
            }
        });

        for target in event_targets {
            let event = Avm2EventObject::bare_default_event(context, "soundComplete");
            Avm2::dispatch_event(context, event, target.into());
        }

        // Update sound transforms, if dirty.
        context.audio_manager.update_sound_transforms(context.audio);
    }

    /// Starts a sound and optionally associates it with a Display Object.
    /// Sounds associated with DOs are an AVM1/Timeline concept and should not be called from AVM2 scripts.
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
                sound: Some(sound),
                instance: handle,
                display_object,
                transform: display_object::SoundTransform::default(),
                avm1_object,
                avm2_object: None,
                stream_start_frame: None,
            };
            audio.set_sound_transform(handle, self.transform_for_sound(&instance));
            self.sounds.push(instance);
            Some(handle)
        } else {
            None
        }
    }

    pub fn attach_avm2_sound_channel(
        &mut self,
        instance: SoundInstanceHandle,
        avm2_object: SoundChannelObject<'gc>,
    ) {
        if let Some(i) = self
            .sounds
            .iter()
            .position(|other| other.instance == instance)
        {
            let instance = &mut self.sounds[i];
            instance.avm2_object = Some(avm2_object);
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
            if other.sound == Some(sound) {
                audio.stop_sound(other.instance);
                false
            } else {
                true
            }
        });
    }

    /// Stops any sound associated with the given Display Object.
    /// Sounds associated with DOs are an AVM1/Timeline concept and should not be called from AVM2 scripts.
    pub fn stop_sounds_with_display_object(
        &mut self,
        audio: &mut dyn AudioBackend,
        display_object: DisplayObject<'gc>,
    ) {
        self.sounds.retain(move |sound| {
            if let Some(other) = sound.display_object {
                // [NA] This may or may not be correct, Flash is very inconsistent here
                // If you make 50 movies with the same path and attach sounds to one of them,
                // they all stop when you stop one of the movies.
                // However, it also stops (some of?) them when they've changed their name too...

                // Path can be expensive, so do the cheap checks first
                if DisplayObject::ptr_eq(other, display_object)
                    || (other.name() == display_object.name()
                        && other.path() == display_object.path())
                {
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

    pub fn is_sound_playing(&self, sound: SoundInstanceHandle) -> bool {
        self.sounds.iter().any(|other| other.instance == sound)
    }

    pub fn is_sound_playing_with_handle(&self, sound: SoundHandle) -> bool {
        self.sounds.iter().any(|other| other.sound == Some(sound))
    }

    pub fn start_stream(
        &mut self,
        audio: &mut dyn AudioBackend,
        movie_clip: MovieClip<'gc>,
        clip_frame: u16,
        data: crate::tag_utils::SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Option<SoundInstanceHandle> {
        if self.sounds.len() < Self::MAX_SOUNDS {
            let handle = audio.start_stream(data, stream_info).ok()?;
            let instance = SoundInstance {
                sound: None,
                instance: handle,
                display_object: Some(movie_clip.into()),
                transform: display_object::SoundTransform::default(),
                avm1_object: None,
                avm2_object: None,
                stream_start_frame: Some(clip_frame),
            };
            audio.set_sound_transform(handle, self.transform_for_sound(&instance));
            self.sounds.push(instance);
            Some(handle)
        } else {
            None
        }
    }

    pub fn start_substream(
        &mut self,
        audio: &mut dyn AudioBackend,
        stream_data: Substream,
        movie_clip: MovieClip<'gc>,
        stream_info: &SoundStreamInfo,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        if self.sounds.len() < Self::MAX_SOUNDS {
            let handle = audio.start_substream(stream_data, stream_info)?;
            let instance = SoundInstance {
                sound: None,
                instance: handle,
                display_object: Some(movie_clip.into()),
                transform: display_object::SoundTransform::default(),
                avm1_object: None,
                avm2_object: None,
                stream_start_frame: None,
            };
            audio.set_sound_transform(handle, self.transform_for_sound(&instance));
            self.sounds.push(instance);
            Ok(handle)
        } else {
            Err(DecodeError::TooManySounds)
        }
    }

    /// Returns the difference in seconds between the primary audio stream's time and the player's time.
    pub fn audio_skew_time(&mut self, audio: &mut dyn AudioBackend, offset_ms: f64) -> f64 {
        // Consider the first playing "stream" sound to be the primary audio track.
        // Needs research: It's not clear how Flash handles the case of multiple stream sounds.
        let (i, skew) = self
            .sounds
            .iter()
            .enumerate()
            .find_map(|(i, instance)| {
                let start_frame = instance.stream_start_frame?;
                let clip = instance
                    .display_object
                    .and_then(|clip| clip.as_movie_clip())?;
                let stream_pos = audio.get_sound_position(instance.instance)?;
                let frame_rate = clip.movie().frame_rate().to_f64();

                // Calculate the difference in time between the owning movie clip and its audio track.
                // If the difference is beyond some threshold, inform the player to adjust playback speed.
                let timeline_pos = f64::from(clip.current_frame().saturating_sub(start_frame))
                    / frame_rate
                    + offset_ms / 1000.0;

                Some((i, stream_pos / 1000.0 - timeline_pos))
            })
            .unwrap_or_default();

        // Calculate the syncing threshold based on the audio backend's frequency in updating sound position.
        let sync_threshold = audio
            .position_resolution()
            .map(|duration| duration.as_secs_f64())
            .unwrap_or(Self::STREAM_DEFAULT_SYNC_THRESHOLD);

        if skew.abs() >= Self::STREAM_RESTART_THRESHOLD {
            // Way out of sync, let's stop the entire stream.
            // The movie clip will probably restart it naturally on the next frame.
            let instance = &self.sounds[i];
            audio.stop_sound(instance.instance);
            self.sounds.swap_remove(i);
            0.0
        } else if skew.abs() >= sync_threshold {
            // Slightly out of sync, adjust player speed to compensate.
            skew
        } else {
            // More or less in sync, no adjustment.
            0.0
        }
    }

    pub fn global_sound_transform(&self) -> &display_object::SoundTransform {
        &self.global_sound_transform
    }

    pub fn set_global_sound_transform(&mut self, sound_transform: display_object::SoundTransform) {
        self.global_sound_transform = sound_transform;
        self.transforms_dirty = true;
    }

    /// Get the local sound transform of a single sound instance.
    pub fn local_sound_transform(
        &self,
        instance: SoundInstanceHandle,
    ) -> Option<&display_object::SoundTransform> {
        if let Some(i) = self
            .sounds
            .iter()
            .position(|other| other.instance == instance)
        {
            let instance = &self.sounds[i];
            Some(&instance.transform)
        } else {
            None
        }
    }

    /// Set the local sound transform of a single sound instance.
    pub fn set_local_sound_transform(
        &mut self,
        instance: SoundInstanceHandle,
        sound_transform: display_object::SoundTransform,
    ) {
        if let Some(i) = self
            .sounds
            .iter()
            .position(|other| other.instance == instance)
        {
            let instance = &mut self.sounds[i];

            instance.transform = sound_transform;
            self.transforms_dirty = true;
        }
    }

    /// Returns the number of seconds that a timeline audio stream should buffer before playing.
    ///
    /// Currently unused by Ruffle.
    pub fn stream_buffer_time(&self) -> i32 {
        self.stream_buffer_time
    }

    /// Sets the number of seconds that a timeline audio stream should buffer before playing.
    ///
    /// Currently unused by Ruffle.
    pub fn set_stream_buffer_time(&mut self, stream_buffer_time: i32) {
        self.stream_buffer_time = stream_buffer_time;
    }

    pub fn set_sound_transforms_dirty(&mut self) {
        self.transforms_dirty = true;
    }

    fn transform_for_sound(&self, sound: &SoundInstance<'gc>) -> SoundTransform {
        let mut transform = sound.transform.clone();
        let mut parent = sound.display_object;
        while let Some(display_object) = parent {
            transform.concat(display_object.base().sound_transform());
            parent = display_object.parent();
        }
        transform.concat(&self.global_sound_transform);
        transform.into()
    }

    /// Update the sound transforms for all sounds.
    /// This should be called whenever a sound transform changes on a display object.
    fn update_sound_transforms(&mut self, audio: &mut dyn AudioBackend) {
        // This updates the sound transform for all sounds, even though the transform has
        // only changed on a single display object. There are only a small amount
        // of sounds playing at any time, so this shouldn't be a big deal.
        if self.transforms_dirty {
            for sound in &self.sounds {
                let transform = self.transform_for_sound(sound);
                audio.set_sound_transform(sound.instance, transform);
            }
            self.transforms_dirty = false;
        }
    }

    pub fn perform_sound_event(
        display_object: DisplayObject<'gc>,
        context: &mut UpdateContext<'gc>,
        character_id: CharacterId,
        sound_info: &SoundInfo,
    ) {
        if let Some(handle) = context
            .library
            .library_for_movie_mut(display_object.movie())
            .get_sound(character_id)
        {
            use swf::SoundEvent;
            // The sound event type is controlled by the "Sync" setting in the Flash IDE.
            match sound_info.event {
                // "Event" sounds always play, independent of the timeline.
                SoundEvent::Event => {
                    let _ = context.start_sound(handle, sound_info, Some(display_object), None);
                }

                // "Start" sounds only play if an instance of the same sound is not already playing.
                SoundEvent::Start => {
                    if !context.is_sound_playing_with_handle(handle) {
                        let _ = context.start_sound(handle, sound_info, Some(display_object), None);
                    }
                }

                // "Stop" stops any active instances of a given sound.
                SoundEvent::Stop => context.stop_sounds_with_handle(handle),
            }
        }
    }
}

impl<'gc> Default for AudioManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct SoundInstance<'gc> {
    /// The handle to the sound instance in the audio backend.
    #[collect(require_static)]
    instance: SoundInstanceHandle,

    /// The handle to the sound definition in the audio backend.
    /// This will be `None` for stream sounds.
    #[collect(require_static)]
    sound: Option<SoundHandle>,

    /// The display object that this sound is playing in, if any.
    /// Used for volume mixing and `Sound.stop()`.
    display_object: Option<DisplayObject<'gc>>,

    /// The local sound transform of this sound.
    ///
    /// Only AVM2 sounds have a local sound transform. In AVM1, sound instances
    /// instead get the sound transform of the display object they're
    /// associated with.
    #[collect(require_static)]
    transform: display_object::SoundTransform,

    /// The AVM1 `Sound` object associated with this sound, if any.
    avm1_object: Option<SoundObject<'gc>>,

    /// The AVM2 `SoundChannel` object associated with this sound, if any.
    avm2_object: Option<SoundChannelObject<'gc>>,

    stream_start_frame: Option<u16>,
}

/// A sound transform for a playing sound, for use by audio backends.
/// This differs from `display_object::SoundTransform` by being
/// already converted to `f32` and having `volume` baked in.
#[derive(Debug, PartialEq, Clone)]
pub struct SoundTransform {
    pub left_to_left: f32,
    pub left_to_right: f32,
    pub right_to_left: f32,
    pub right_to_right: f32,
}

impl From<display_object::SoundTransform> for SoundTransform {
    /// Converts from a `display_object::SoundTransform` to a `backend::audio::SoundTransform`.
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
    fn default() -> Self {
        Self {
            left_to_left: 1.0,
            left_to_right: 0.0,
            right_to_left: 0.0,
            right_to_right: 1.0,
        }
    }
}
