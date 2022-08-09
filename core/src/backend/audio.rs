use crate::{
    avm1::SoundObject,
    avm2::SoundChannelObject,
    display_object::{self, DisplayObject, MovieClip, TDisplayObject},
};
use gc_arena::Collect;

pub mod decoders;
pub mod swf {
    pub use swf::{
        read, AudioCompression, CharacterId, Sound, SoundEnvelope, SoundEnvelopePoint, SoundEvent,
        SoundFormat, SoundInfo, SoundStreamHead,
    };
}

mod mixer;
pub use mixer::*;
use ruffle_types::backend::audio::{
    AudioBackend, SoundHandle, SoundInstanceHandle, SoundTransform,
};

#[derive(Collect)]
#[collect(no_drop)]
pub struct AudioManager<'gc> {
    /// The list of actively playing sounds.
    sounds: Vec<SoundInstance<'gc>>,

    /// The global sound transform applied to all sounds.
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

    /// The minimum audio sycning threshold in seconds.
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
    pub fn update_sounds(
        &mut self,
        audio: &mut dyn AudioBackend,
        gc_context: gc_arena::MutationContext<'gc, '_>,
        action_queue: &mut crate::context::ActionQueue<'gc>,
        root: DisplayObject<'gc>,
    ) {
        // Update the position of sounds, and remove any completed sounds.
        self.sounds.retain(|sound| {
            if let Some(pos) = audio.get_sound_position(sound.instance) {
                // Sounds still playing; update position.
                if let Some(avm1_object) = sound.avm1_object {
                    avm1_object.set_position(gc_context, pos.round() as u32);
                } else if let Some(avm2_object) = sound.avm2_object {
                    avm2_object.set_position(gc_context, pos);
                }
                true
            } else {
                // Sound ended.
                let duration = sound
                    .sound
                    .and_then(|sound| audio.get_sound_duration(sound))
                    .unwrap_or_default();
                if let Some(object) = sound.avm1_object {
                    object.set_position(gc_context, duration.round() as u32);

                    // Fire soundComplete event.
                    action_queue.queue_actions(
                        root,
                        crate::context::ActionType::Method {
                            object: object.into(),
                            name: "onSoundComplete",
                            args: vec![],
                        },
                        false,
                    );
                }

                if let Some(object) = sound.avm2_object {
                    object.set_position(gc_context, duration);

                    //TODO: AVM2 events are usually not queued, but we can't
                    //hold the update context in the audio manager yet.
                    action_queue.queue_actions(
                        root,
                        crate::context::ActionType::Event2 {
                            event_type: "soundComplete",
                            target: object.into(),
                        },
                        false,
                    )
                }

                false
            }
        });

        // Update sound transforms, if dirty.
        self.update_sound_transforms(audio);
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

    pub fn is_sound_playing(&mut self, sound: SoundInstanceHandle) -> bool {
        self.sounds.iter().any(|other| other.instance == sound)
    }

    pub fn is_sound_playing_with_handle(&mut self, sound: SoundHandle) -> bool {
        self.sounds.iter().any(|other| other.sound == Some(sound))
    }

    pub fn start_stream(
        &mut self,
        audio: &mut dyn AudioBackend,
        stream_handle: Option<SoundHandle>,
        movie_clip: MovieClip<'gc>,
        clip_frame: u16,
        data: ruffle_types::tag_utils::SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Option<SoundInstanceHandle> {
        if self.sounds.len() < Self::MAX_SOUNDS {
            let handle = audio
                .start_stream(stream_handle, clip_frame, data, stream_info)
                .ok()?;
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
                let frame_rate = clip.movie()?.frame_rate().to_f64();

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
    transform: display_object::SoundTransform,

    /// The AVM1 `Sound` object associated with this sound, if any.
    avm1_object: Option<SoundObject<'gc>>,

    /// The AVM2 `SoundChannel` object associated with this sound, if any.
    avm2_object: Option<SoundChannelObject<'gc>>,

    stream_start_frame: Option<u16>,
}
