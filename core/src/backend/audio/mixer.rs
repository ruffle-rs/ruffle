use super::decoders::{self, AdpcmDecoder, NellymoserDecoder, PcmDecoder, SeekableDecoder};
use super::{SoundHandle, SoundInstanceHandle, SoundTransform};
use crate::tag_utils::SwfSlice;
use generational_arena::Arena;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use swf::AudioCompression;

/// An audio mixer for a Flash movie.
///
/// `AudioMixer` manages the audio state for a Flash movie. This can be used by any backend that
/// can output a raw audio stream.
///
/// An `AudioBackend` can forward audio events to the `AudioMixer`, and it will track the state of
// all sounds and mix the audio into an output buffer audio stream.
pub struct AudioMixer {
    /// The currently registered sounds.
    sounds: Arena<Sound>,

    /// The list of actively playing sound instances.
    sound_instances: Arc<Mutex<Arena<SoundInstance>>>,

    /// The number of channels in the output stream. Must be 1 or 2.
    num_output_channels: u8,

    /// The sample rate of the output stream in Hz.
    output_sample_rate: u32,
}

/// An iterator for sound decoders that returns stereo samples.
type Signal = Box<dyn Send + dasp::signal::Signal<Frame = [i16; 2]>>;

type Error = Box<dyn std::error::Error>;

/// Contains the data and metadata for a sound in an SWF file.
///
/// A sound is defined by the `DefineSound` SWF tags and contains the audio data for the sound.
struct Sound {
    /// The format of the sound, including sample rate and compression format.
    format: swf::SoundFormat,

    /// The audio data of this sound.
    ///
    /// This will be compressed in the format indicated by `format.compression`.
    data: Arc<[u8]>,

    /// Number of samples in this audio.
    /// This does not include `skip_sample_frames`.
    num_sample_frames: u32,

    /// The number of sample frames to skip to reach the start of the audio.
    ///
    /// MP3 encoders introduce some silence at the beginning of the audio due to encoder delay.
    /// `skip_sample_frames` indicates how many sample frames to skip to bypass the delay.
    /// This is `0` unless `format.compression` is `AudioCompression::Mp3`.
    skip_sample_frames: u16,
}

/// An actively playing instance of a sound.
/// This sound can be either an event sound (`StartSound`) or
/// a stream sound (`SoundStreamBlock`).
/// The audio thread will iterate through all `SoundInstance`s
/// to fill the audio buffer.
struct SoundInstance {
    /// The handle the sound definition inside `sounds`.
    /// `None` if this is a stream sound.
    #[allow(dead_code)]
    handle: Option<SoundHandle>,

    /// The audio stream. Call `next()` to yield sample frames.
    signal: Signal,

    /// Flag indicating whether this sound is still playing.
    /// If this flag is false, the sound will be cleaned up during the
    /// next loop of the sound thread.
    active: bool,

    /// The transform for the left channel of this sound instance.
    left_transform: [f32; 2],

    /// The transform for the right channel of this sound instance.
    right_transform: [f32; 2],
}

impl AudioMixer {
    /// Creates a new `AudioMixer` with the given number of channels and sample rate.
    pub fn new(num_output_channels: u8, output_sample_rate: u32) -> Self {
        Self {
            sounds: Arena::new(),
            sound_instances: Arc::new(Mutex::new(Arena::new())),
            num_output_channels,
            output_sample_rate,
        }
    }

    /// Creates a proxy that may be sent to a different thread.
    pub fn proxy(&self) -> AudioMixerProxy {
        AudioMixerProxy {
            sound_instances: Arc::clone(&self.sound_instances),
            num_output_channels: self.num_output_channels,
        }
    }

    /// Mixes audio into the given `output_buffer`.
    ///
    /// All playing sound instances will be sampled and mixed to fill `output_buffer`.
    /// `output_buffer` is expected to be in 2-channel interleaved format.
    pub fn mix<'a, T>(&mut self, output_buffer: &mut [T])
    where
        T: 'a + dasp::Sample + Default,
        T::Signed: dasp::sample::conv::FromSample<i16>,
        T::Float: dasp::sample::conv::FromSample<f32>,
    {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        Self::mix_audio::<T>(
            &mut sound_instances,
            self.num_output_channels,
            output_buffer,
        )
    }

    /// Instantiate a seekable decoder for audio data with the given format.
    ///
    /// A seekable decoder is used for:
    ///  * "Event" sounds on the timeline with custom start/loop settings
    ///  * ActionScript sounds that may have a custom start and loop setting
    fn make_seekable_decoder(
        format: &swf::SoundFormat,
        data: Cursor<ArcAsRef>,
    ) -> Result<Box<dyn Send + SeekableDecoder>, Error> {
        let decoder: Box<dyn Send + SeekableDecoder> = match format.compression {
            AudioCompression::Uncompressed => Box::new(PcmDecoder::new(
                data,
                format.is_stereo,
                format.sample_rate,
                format.is_16_bit,
            )),
            AudioCompression::Adpcm => Box::new(AdpcmDecoder::new(
                data,
                format.is_stereo,
                format.sample_rate,
            )?),
            #[cfg(feature = "minimp3")]
            AudioCompression::Mp3 => Box::new(decoders::Mp3Decoder::new(
                if format.is_stereo { 2 } else { 1 },
                format.sample_rate.into(),
                data,
            )),
            #[cfg(all(feature = "symphonia", not(feature = "minimp3")))]
            AudioCompression::Mp3 => Box::new(decoders::Mp3Decoder::new_seekable(
                if format.is_stereo { 2 } else { 1 },
                format.sample_rate.into(),
                data,
            )),
            AudioCompression::Nellymoser => {
                Box::new(NellymoserDecoder::new(data, format.sample_rate.into()))
            }
            _ => {
                let msg = format!(
                    "start_stream: Unhandled audio compression {:?}",
                    format.compression
                );
                log::error!("{}", msg);
                return Err(msg.into());
            }
        };
        Ok(decoder)
    }

    /// Transforms a `Signal` into a new `Signal` that matches the output sample rate.
    fn make_resampler<S: Send + dasp::signal::Signal<Frame = [i16; 2]>>(
        &self,
        format: &swf::SoundFormat,
        mut signal: S,
    ) -> dasp::signal::interpolate::Converter<
        S,
        impl dasp::interpolate::Interpolator<Frame = [i16; 2]>,
    > {
        // TODO: Allow interpolator to be user-configurable?
        let left = signal.next();
        let right = signal.next();
        let interpolator = dasp::interpolate::linear::Linear::new(left, right);
        dasp::signal::interpolate::Converter::from_hz_to_hz(
            signal,
            interpolator,
            format.sample_rate.into(),
            self.output_sample_rate.into(),
        )
    }

    /// Creates a `Signal` for an "event" that decodes and resamples the audio stream to the
    /// output format.
    ///
    /// This also applies the custom envelope, start/end, and looping parameters from `settings`.
    fn make_signal_from_event_sound(
        &self,
        sound: &Sound,
        settings: &swf::SoundInfo,
        data: Cursor<ArcAsRef>,
    ) -> Result<Box<dyn Send + dasp::signal::Signal<Frame = [i16; 2]>>, Error> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = Self::make_seekable_decoder(&sound.format, data)?;

        // Wrap the decoder in the event sound signal (controls looping/envelope)
        let signal = EventSoundSignal::new_with_settings(
            decoder,
            settings,
            sound.num_sample_frames,
            sound.skip_sample_frames,
        );
        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = self.make_resampler(&sound.format, signal);
        if let Some(envelope) = &settings.envelope {
            use dasp::Signal;
            let envelope_signal = EnvelopeSignal::new(&envelope[..], self.output_sample_rate);
            Ok(Box::new(signal.mul_amp(envelope_signal)))
        } else {
            Ok(Box::new(signal))
        }
    }

    /// Creates a `Signal` for a simple "event" sound that decodes and resamples the audio stream
    /// to the output format.
    ///
    /// This is used for cases where there is no custom envelope or looping on the sound instance.
    /// Otherwise, `AudioMixer::make_signal_from_event_sound` should be used.
    fn make_signal_from_simple_event_sound<R: 'static + std::io::Read + Send>(
        &self,
        format: &swf::SoundFormat,
        data_stream: R,
    ) -> Result<Box<dyn Send + dasp::signal::Signal<Frame = [i16; 2]>>, Error> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = decoders::make_decoder(format, data_stream)?;

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = dasp::signal::from_iter(decoder);
        let signal = self.make_resampler(format, signal);
        Ok(Box::new(signal))
    }

    /// Creates a `Signal` that decodes and resamples a timeline "stream" sound.
    fn make_signal_from_stream<'a>(
        &self,
        format: &swf::SoundFormat,
        data_stream: SwfSlice,
    ) -> Result<Box<dyn 'a + Send + dasp::signal::Signal<Frame = [i16; 2]>>, Error> {
        // Instantiate a decoder for the compression that the sound data uses.
        let clip_stream_decoder = decoders::make_stream_decoder(format, data_stream)?;

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = dasp::signal::from_iter(clip_stream_decoder);
        let signal = Box::new(self.make_resampler(format, signal));
        Ok(signal)
    }

    /// Callback to the audio thread.
    /// Refill the output buffer by stepping through all active sounds
    /// and mixing in their output.
    fn mix_audio<'a, T>(
        sound_instances: &mut Arena<SoundInstance>,
        num_channels: u8,
        mut output_buffer: &mut [T],
    ) where
        T: 'a + Default + dasp::Sample,
        T::Signed: dasp::sample::conv::FromSample<i16>,
        T::Float: dasp::sample::conv::FromSample<f32>,
    {
        use dasp::{
            frame::{Frame, Stereo},
            Sample,
        };
        use std::ops::DerefMut;

        // For each sample, mix the samples from all active sound instances.
        for buf_frame in output_buffer
            .deref_mut()
            .chunks_exact_mut(num_channels.into())
        {
            let mut output_frame = Stereo::<T::Signed>::EQUILIBRIUM;
            for (_, sound) in sound_instances.iter_mut() {
                if sound.active && !sound.signal.is_exhausted() {
                    let sound_frame = sound.signal.next();
                    let [left_0, left_1] = sound_frame.mul_amp(sound.left_transform);
                    let [right_0, right_1] = sound_frame.mul_amp(sound.right_transform);
                    let sound_frame: Stereo<T::Signed> = [
                        Sample::add_amp(left_0, left_1).to_sample(),
                        Sample::add_amp(right_0, right_1).to_sample(),
                    ];
                    output_frame = output_frame.add_amp(sound_frame);
                } else {
                    sound.active = false;
                }
            }

            for (buf_sample, output_sample) in buf_frame.iter_mut().zip(output_frame.iter()) {
                *buf_sample = output_sample.to_sample();
            }
        }

        // Remove all dead sounds.
        sound_instances.retain(|_, sound| sound.active);
    }

    /// Registers a sound with the audio mixer.
    pub fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error> {
        // Slice off latency seek for MP3 data.
        let (skip_sample_frames, data) = if swf_sound.format.compression == AudioCompression::Mp3 {
            let skip_sample_frames = u16::from_le_bytes([swf_sound.data[0], swf_sound.data[1]]);
            (skip_sample_frames, &swf_sound.data[2..])
        } else {
            (0, swf_sound.data)
        };

        let sound = Sound {
            format: swf_sound.format.clone(),
            data: Arc::from(data),
            num_sample_frames: swf_sound.num_samples,
            skip_sample_frames,
        };
        Ok(self.sounds.insert(sound))
    }

    /// Starts a timeline audio stream.
    pub fn start_stream(
        &mut self,
        _stream_handle: Option<SoundHandle>,
        _clip_frame: u16,
        clip_data: SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, Error> {
        let format = &stream_info.stream_format;

        // The audio data for stream sounds is distributed among the frames of a
        // movie clip. The stream tag reader will parse through the SWF and
        // feed the decoder audio data on the fly.
        let signal = self.make_signal_from_stream(format, clip_data)?;

        let mut sound_instances = self.sound_instances.lock().unwrap();
        let handle = sound_instances.insert(SoundInstance {
            handle: None,
            signal,
            active: true,
            left_transform: [1.0, 0.0],
            right_transform: [0.0, 1.0],
        });
        Ok(handle)
    }

    /// Starts a sound.
    ///
    /// The sound must have been registered using `AudioMixer::register_sound`.
    pub fn start_sound(
        &mut self,
        sound_handle: SoundHandle,
        settings: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error> {
        let sound = &self.sounds[sound_handle];
        let data = Cursor::new(ArcAsRef(Arc::clone(&sound.data)));
        // Create a signal that decodes and resamples the sound.
        let signal = if sound.skip_sample_frames == 0
            && settings.in_sample.is_none()
            && settings.out_sample.is_none()
            && settings.num_loops <= 1
            && settings.envelope.is_none()
        {
            // For simple event sounds, just use the same signal as streams.
            self.make_signal_from_simple_event_sound(&sound.format, data)?
        } else {
            // For event sounds with envelopes/other properties, wrap it in `EventSoundSignal`.
            self.make_signal_from_event_sound(sound, settings, data)?
        };

        // Add sound instance to active list.
        let mut sound_instances = self.sound_instances.lock().unwrap();
        let handle = sound_instances.insert(SoundInstance {
            handle: Some(sound_handle),
            signal,
            active: true,
            left_transform: [1.0, 0.0],
            right_transform: [0.0, 1.0],
        });
        Ok(handle)
    }

    /// Stops a playing sound instance.
    pub fn stop_sound(&mut self, sound: SoundInstanceHandle) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.remove(sound);
    }

    pub fn stop_all_sounds(&mut self) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        // This is a workaround for a bug in generational-arena:
        // Arena::clear does not properly bump the generational index, allowing for stale references
        // to continue to work (this caused #1315). Arena::remove will force a generation bump.
        // See https://github.com/fitzgen/generational-arena/issues/30
        if let Some((i, _)) = sound_instances.iter().next() {
            sound_instances.remove(i);
        }
        sound_instances.clear();
    }

    /// Returns the position of a playing sound in milliseconds.
    ///
    ////// Returns `None` if the sound is no longer playing.
    pub fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<u32> {
        let sound_instances = self.sound_instances.lock().unwrap();
        // TODO: Return actual position
        sound_instances.get(instance).map(|_| 0)
    }

    /// Returns the duration of a registered sound in milliseconds.
    ///
    /// Returns `None` if the sound is not registered or invalid.
    pub fn get_sound_duration(&self, sound: SoundHandle) -> Option<f64> {
        if let Some(sound) = self.sounds.get(sound) {
            // AS duration does not subtract `skip_sample_frames`.
            let num_sample_frames: f64 = sound.num_sample_frames.into();
            let sample_rate: f64 = sound.format.sample_rate.into();
            let ms = num_sample_frames * 1000.0 / sample_rate;
            Some(ms)
        } else {
            None
        }
    }

    pub fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
        self.sounds.get(sound).map(|s| s.data.len() as u32)
    }

    pub fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat> {
        self.sounds.get(sound).map(|s| &s.format)
    }

    /// Sets the sound transform for the given playing sound.
    pub fn set_sound_transform(
        &mut self,
        instance: SoundInstanceHandle,
        transform: SoundTransform,
    ) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        if let Some(instance) = sound_instances.get_mut(instance) {
            instance.left_transform = [transform.left_to_left, transform.right_to_left];
            instance.right_transform = [transform.left_to_right, transform.right_to_right];
        }
    }
}

/// A thread-safe proxy to the main `AudioMixer`, allowing for mixing audio from a different thread.
///
/// An `AudioMixerProxy` can be created via `AudioMixer::proxy`. The proxy can be sent to another thread
/// to perform audio mixing on a different thread.
pub struct AudioMixerProxy {
    /// The list of actively playing sound instances.
    sound_instances: Arc<Mutex<Arena<SoundInstance>>>,

    /// The number of channels in the output stream. Must be 1 or 2.
    num_output_channels: u8,
}

impl AudioMixerProxy {
    /// Mixes audio into the given `output_buffer`.
    ///
    /// All playing sound instances will be sampled and mixed to fill `output_buffer`.
    /// `output_buffer` is expected to be in 2-channel interleaved format.
    pub fn mix<'a, T>(&self, output_buffer: &mut [T])
    where
        T: 'a + dasp::Sample + Default,
        T::Signed: dasp::sample::conv::FromSample<i16>,
        T::Float: dasp::sample::conv::FromSample<f32>,
    {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        AudioMixer::mix_audio::<T>(
            &mut sound_instances,
            self.num_output_channels,
            output_buffer,
        )
    }
}

/// A dummy wrapper struct to implement `AsRef<[u8]>` for `Arc<Vec<u8>>`.
/// Not having this trait causes problems when trying to use `Cursor<Vec<u8>>`.
struct ArcAsRef(Arc<[u8]>);

impl AsRef<[u8]> for ArcAsRef {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Default for ArcAsRef {
    fn default() -> Self {
        ArcAsRef(Arc::new([]))
    }
}

/// A signal for event sound instances with custom envelopes, start/end point, or loop settings.
struct EventSoundSignal {
    decoder: Box<dyn SeekableDecoder + Send>,
    num_loops: u16,
    start_sample_frame: u32,
    end_sample_frame: Option<u32>,
    cur_sample_frame: u32,
    is_exhausted: bool,
}

impl EventSoundSignal {
    fn new_with_settings(
        decoder: Box<dyn SeekableDecoder + Send>,
        settings: &swf::SoundInfo,
        num_sample_frames: u32,
        skip_sample_frames: u16,
    ) -> Self {
        let skip_sample_frames: u32 = skip_sample_frames.into();
        let sample_divisor = 44100 / u32::from(decoder.sample_rate());
        let start_sample_frame =
            settings.in_sample.unwrap_or(0) / sample_divisor + skip_sample_frames;
        let end_sample_frame = settings
            .out_sample
            .map(|n| n / sample_divisor)
            .unwrap_or(num_sample_frames)
            + skip_sample_frames;

        let mut signal = Self {
            decoder,
            num_loops: settings.num_loops,
            start_sample_frame,
            end_sample_frame: Some(end_sample_frame),
            cur_sample_frame: start_sample_frame,
            is_exhausted: false,
        };
        signal.next_loop();
        signal
    }

    /// Resets the decoder to the start point of the loop.
    fn next_loop(&mut self) {
        if self.num_loops > 0 {
            self.num_loops -= 1;
            self.decoder.seek_to_sample_frame(self.start_sample_frame);
            self.cur_sample_frame = self.start_sample_frame;
        } else {
            self.is_exhausted = true;
        }
    }
}

impl dasp::signal::Signal for EventSoundSignal {
    type Frame = [i16; 2];

    fn next(&mut self) -> Self::Frame {
        // Loop the sound if necessary, and get the next frame.
        if !self.is_exhausted {
            if let Some(frame) = self.decoder.next() {
                self.cur_sample_frame += 1;
                if let Some(end) = self.end_sample_frame {
                    if self.cur_sample_frame > end {
                        self.next_loop();
                    }
                }
                frame
            } else {
                self.next_loop();
                self.next()
            }
        } else {
            [0, 0]
        }
    }

    fn is_exhausted(&self) -> bool {
        self.is_exhausted
    }
}

/// A signal that represents the sound envelope for an event sound.
/// The sound signal gets multiplied by the envelope for volume/panning effects.
struct EnvelopeSignal {
    /// Iterator through the envelope points specified in the SWWF file.
    envelope: std::vec::IntoIter<swf::SoundEnvelopePoint>,

    /// The starting envelope point.
    prev_point: swf::SoundEnvelopePoint,

    /// The ending envelope point.
    next_point: swf::SoundEnvelopePoint,

    /// The current sample index.
    cur_sample: u32,
}

impl EnvelopeSignal {
    fn new(envelope: &[swf::SoundEnvelopePoint], output_sample_rate: u32) -> Self {
        // Envelope samples are always in 44.1KHz.
        const ENVELOPE_SAMPLE_RATE: u32 = 44100;

        // Scale the envelope points from 44.1KHz to the output rate.
        let scale = f64::from(output_sample_rate) / f64::from(ENVELOPE_SAMPLE_RATE);
        let mut envelope = envelope
            .iter()
            .map(|pt| swf::SoundEnvelopePoint {
                sample: (f64::from(pt.sample) * scale) as u32,
                ..*pt
            })
            .collect::<swf::SoundEnvelope>()
            .into_iter();
        let first_point = envelope.next().unwrap_or(swf::SoundEnvelopePoint {
            sample: 0,
            left_volume: 1.0,
            right_volume: 1.0,
        });
        Self {
            // The initial volume is the first point's volume.
            prev_point: swf::SoundEnvelopePoint {
                sample: 0,
                left_volume: first_point.left_volume,
                right_volume: first_point.right_volume,
            },
            next_point: first_point,
            cur_sample: 0,
            envelope,
        }
    }
}

impl dasp::signal::Signal for EnvelopeSignal {
    type Frame = [f32; 2];

    fn next(&mut self) -> Self::Frame {
        // Calculate interpolated volume.
        let out = if self.prev_point.sample < self.next_point.sample {
            let a: f64 = (self.cur_sample - self.prev_point.sample).into();
            let b: f64 = (self.next_point.sample - self.prev_point.sample).into();
            let lerp = a / b;
            let interpolator = dasp::interpolate::linear::Linear::new(
                [self.prev_point.left_volume, self.prev_point.right_volume],
                [self.next_point.left_volume, self.next_point.right_volume],
            );
            use dasp::interpolate::Interpolator;
            interpolator.interpolate(lerp)
        } else {
            [self.next_point.left_volume, self.next_point.right_volume]
        };

        // Update envelope endpoints.
        self.cur_sample = self.cur_sample.saturating_add(1);
        while self.cur_sample > self.next_point.sample {
            self.prev_point = self.next_point.clone();
            self.next_point = self
                .envelope
                .next()
                .clone()
                .unwrap_or(swf::SoundEnvelopePoint {
                    sample: u32::MAX,
                    left_volume: self.prev_point.left_volume,
                    right_volume: self.prev_point.right_volume,
                });

            if self.prev_point.sample > self.next_point.sample {
                self.next_point.sample = self.prev_point.sample;
                log::error!("Invalid sound envelope; sample indices are out of order");
            }
        }

        out
    }

    fn is_exhausted(&self) -> bool {
        false
    }
}

#[macro_export]
macro_rules! impl_audio_mixer_backend {
    ($mixer:ident) => {
        #[inline]
        fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error> {
            self.$mixer.register_sound(swf_sound)
        }

        #[inline]
        fn start_stream(
            &mut self,
            stream_handle: Option<SoundHandle>,
            clip_frame: u16,
            clip_data: $crate::tag_utils::SwfSlice,
            stream_info: &swf::SoundStreamHead,
        ) -> Result<SoundInstanceHandle, Error> {
            self.$mixer
                .start_stream(stream_handle, clip_frame, clip_data, stream_info)
        }

        #[inline]
        fn start_sound(
            &mut self,
            sound_handle: SoundHandle,
            settings: &swf::SoundInfo,
        ) -> Result<SoundInstanceHandle, Error> {
            self.$mixer.start_sound(sound_handle, settings)
        }

        #[inline]
        fn stop_sound(&mut self, sound: SoundInstanceHandle) {
            self.$mixer.stop_sound(sound)
        }

        #[inline]
        fn stop_all_sounds(&mut self) {
            self.$mixer.stop_all_sounds()
        }

        #[inline]
        fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<u32> {
            self.$mixer.get_sound_position(instance)
        }

        #[inline]
        fn get_sound_duration(&self, sound: SoundHandle) -> Option<f64> {
            self.$mixer.get_sound_duration(sound)
        }

        #[inline]
        fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
            self.$mixer.get_sound_size(sound)
        }

        #[inline]
        fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat> {
            self.$mixer.get_sound_format(sound)
        }

        #[inline]
        fn set_sound_transform(
            &mut self,
            instance: SoundInstanceHandle,
            transform: SoundTransform,
        ) {
            self.$mixer.set_sound_transform(instance, transform)
        }
    };
}
