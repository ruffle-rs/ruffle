use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use generational_arena::Arena;
use ruffle_core::backend::audio::decoders::{
    self, AdpcmDecoder, Mp3Decoder, PcmDecoder, SeekableDecoder,
};
use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioStreamHandle, SoundHandle, SoundInstanceHandle,
};
use ruffle_core::tag_utils::SwfSlice;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use swf::AudioCompression;

#[allow(dead_code)]
pub struct CpalAudioBackend {
    device: cpal::Device,
    output_format: cpal::Format,
    audio_thread_handle: std::thread::JoinHandle<()>,

    sounds: Arena<Sound>,
    sound_instances: Arc<Mutex<Arena<SoundInstance>>>,
}

type Signal = Box<dyn Send + sample::signal::Signal<Frame = [i16; 2]>>;

type Error = Box<dyn std::error::Error>;

/// Contains the data and metadata for a sound in an SWF file.
/// A `Sound` is defined by the `DefineSound` SWF tags.
struct Sound {
    format: swf::SoundFormat,
    data: Arc<Vec<u8>>,
    /// Number of samples in this audio.
    /// This does not include the skip_sample_frames.
    num_sample_frames: u32,

    /// Number of samples to skip encoder delay.
    skip_sample_frames: u16,
}

/// An actively playing instance of a sound.
/// This sound can be either an event sound (`StartSound`) or
/// a stream sound (`SoundStreamBlock`).
/// The audio thread will iterate through all `SoundInstance`s
/// to fill the audio buffer.
#[allow(dead_code)]
struct SoundInstance {
    /// The handle the sound definition inside `sounds`.
    /// `None` if this is a stream sound.
    handle: Option<SoundHandle>,

    /// The audio stream. Call `next()` to yield sample frames.
    signal: Signal,

    /// The character ID of the movie clip that contains this stream.
    /// `None` if this sound is an event sound (`StartSound`).
    clip_id: Option<swf::CharacterId>,

    /// Flag indicating whether this sound is still playing.
    /// If this flag is false, the sound will be cleaned up during the
    /// next loop of the sound thread.
    active: bool,
}

impl CpalAudioBackend {
    pub fn new() -> Result<Self, Error> {
        // Initialize cpal on a separate thread to issues on Windows with cpal + winit:
        // https://github.com/RustAudio/cpal/pull/348
        // TODO: Revert back to doing this on the same thread when the above is fixed.
        let init_thread = std::thread::spawn(move || -> Result<Self, String> {
            Self::init().map_err(|e| e.to_string())
        });

        match init_thread.join() {
            Ok(Ok(audio)) => Ok(audio),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err("Panic when initializing audio".into()),
        }
    }

    fn init() -> Result<Self, Error> {
        // Create CPAL audio device.
        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device = host
            .default_output_device()
            .ok_or("No audio devices available")?;

        // Create audio stream for device.
        let mut supported_formats = device
            .supported_output_formats()
            .map_err(|_| "No supported audio format")?;
        let format = supported_formats
            .next()
            .ok_or("No supported audio formats")?
            .with_max_sample_rate();
        let stream_id = event_loop
            .build_output_stream(&device, &format)
            .map_err(|_| "Unable to create audio stream")?;

        let output_format = format.clone();
        // Start the stream.
        event_loop
            .play_stream(stream_id)
            .map_err(|_| "Unable to start audio stream")?;

        let sound_instances: Arc<Mutex<Arena<SoundInstance>>> = Arc::new(Mutex::new(Arena::new()));

        // Start the audio thread.
        let audio_thread_handle = {
            let sound_instances = Arc::clone(&sound_instances);
            std::thread::spawn(move || {
                event_loop.run(move |stream_id, stream_result| {
                    use cpal::{StreamData, UnknownTypeOutputBuffer};

                    let stream_data = match stream_result {
                        Ok(data) => data,
                        Err(err) => {
                            eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                            return;
                        }
                    };

                    let mut sound_instances = sound_instances.lock().unwrap();
                    match stream_data {
                        StreamData::Output {
                            buffer: UnknownTypeOutputBuffer::U16(buffer),
                        } => {
                            Self::mix_audio(&mut sound_instances, &output_format, buffer);
                        }
                        StreamData::Output {
                            buffer: UnknownTypeOutputBuffer::I16(buffer),
                        } => {
                            Self::mix_audio(&mut sound_instances, &output_format, buffer);
                        }
                        StreamData::Output {
                            buffer: UnknownTypeOutputBuffer::F32(buffer),
                        } => {
                            Self::mix_audio(&mut sound_instances, &output_format, buffer);
                        }
                        _ => (),
                    }
                });
            })
        };

        Ok(Self {
            device,
            output_format: format,
            audio_thread_handle,
            sounds: Arena::new(),
            sound_instances,
        })
    }

    /// Instantiate a seeabkle decoder for the compression that the sound data uses.
    fn make_seekable_decoder(
        format: &swf::SoundFormat,
        data: Cursor<VecAsRef>,
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
            )),
            AudioCompression::Mp3 => Box::new(Mp3Decoder::new(
                if format.is_stereo { 2 } else { 1 },
                format.sample_rate.into(),
                data,
            )),
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

    /// Resamples a stream.
    /// TODO: Allow interpolator to be user-configurable?
    fn make_resampler<S: Send + sample::signal::Signal<Frame = [i16; 2]>>(
        &self,
        format: &swf::SoundFormat,
        mut signal: S,
    ) -> sample::interpolate::Converter<S, impl sample::interpolate::Interpolator<Frame = [i16; 2]>>
    {
        let interpolator = sample::interpolate::Linear::from_source(&mut signal);
        sample::interpolate::Converter::from_hz_to_hz(
            signal,
            interpolator,
            format.sample_rate.into(),
            self.output_format.sample_rate.0.into(),
        )
    }

    /// Creates a `sample::signal::Signal` that decodes and resamples the audio stream
    /// to the output format.
    fn make_signal_from_event_sound(
        &self,
        sound: &Sound,
        settings: &swf::SoundInfo,
        data: Cursor<VecAsRef>,
    ) -> Result<Box<dyn Send + sample::signal::Signal<Frame = [i16; 2]>>, Error> {
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
        Ok(Box::new(signal))
    }

    /// Creates a `sample::signal::Signal` that decodes and resamples a "stream" sound.
    fn make_signal_from_stream<'a>(
        &self,
        format: &swf::SoundFormat,
        data_stream: SwfSlice,
    ) -> Result<Box<dyn 'a + Send + sample::signal::Signal<Frame = [i16; 2]>>, Error> {
        // Instantiate a decoder for the compression that the sound data uses.
        let clip_stream_decoder = decoders::make_stream_decoder(format, data_stream)?;

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = sample::signal::from_iter(clip_stream_decoder);
        let signal = Box::new(self.make_resampler(format, signal));
        Ok(Box::new(signal))
    }

    /// Creates a `sample::signal::Signal` that decodes and resamples the audio stream
    /// to the output format.
    fn make_signal_from_simple_event_sound<'a, R: 'a + std::io::Read + Send>(
        &self,
        format: &swf::SoundFormat,
        data_stream: R,
    ) -> Result<Box<dyn 'a + Send + sample::signal::Signal<Frame = [i16; 2]>>, Error> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = decoders::make_decoder(format, data_stream)?;

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = sample::signal::from_iter(decoder);
        let signal = self.make_resampler(format, signal);
        Ok(Box::new(signal))
    }

    /// Callback to the audio thread.
    /// Refill the output buffer by stepping through all active sounds
    /// and mixing in their output.
    fn mix_audio<'a, T>(
        sound_instances: &mut Arena<SoundInstance>,
        output_format: &cpal::Format,
        mut output_buffer: cpal::OutputBuffer<'a, T>,
    ) where
        T: 'a + cpal::Sample + Default + sample::Sample,
        T::Signed: sample::conv::FromSample<i16>,
    {
        use sample::{
            frame::{Frame, Stereo},
            Sample,
        };
        use std::ops::DerefMut;

        // For each sample, mix the samples from all active sound instances.
        for buf_frame in output_buffer
            .deref_mut()
            .chunks_exact_mut(output_format.channels.into())
        {
            let mut output_frame = Stereo::<T::Signed>::equilibrium();
            for (_, sound) in sound_instances.iter_mut() {
                if sound.active && !sound.signal.is_exhausted() {
                    let sound_frame = sound.signal.next();
                    let sound_frame: Stereo<T::Signed> = sound_frame.map(Sample::to_sample);
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
}

impl AudioBackend for CpalAudioBackend {
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error> {
        // Slice off latency seek for MP3 data.
        let (skip_sample_frames, data) = if swf_sound.format.compression == AudioCompression::Mp3 {
            let skip_sample_frames =
                u16::from(swf_sound.data[0]) | (u16::from(swf_sound.data[1]) << 8);
            (skip_sample_frames, &swf_sound.data[2..])
        } else {
            (0, &swf_sound.data[..])
        };

        let sound = Sound {
            format: swf_sound.format.clone(),
            data: Arc::new(data.to_vec()),
            num_sample_frames: swf_sound.num_samples,
            skip_sample_frames,
        };
        Ok(self.sounds.insert(sound))
    }

    fn start_stream(
        &mut self,
        clip_id: swf::CharacterId,
        _clip_frame: u16,
        clip_data: SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Result<AudioStreamHandle, Error> {
        let format = &stream_info.stream_format;

        // The audio data for stream sounds is distributed among the frames of a
        // movie clip. The stream tag reader will parse through the SWF and
        // feed the decoder audio data on the fly.
        let signal = self.make_signal_from_stream(format, clip_data)?;

        let mut sound_instances = self.sound_instances.lock().unwrap();
        let handle = sound_instances.insert(SoundInstance {
            handle: None,
            clip_id: Some(clip_id),
            signal,
            active: true,
        });
        Ok(handle)
    }

    fn stop_stream(&mut self, stream: AudioStreamHandle) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.remove(stream);
    }

    fn start_sound(
        &mut self,
        sound_handle: SoundHandle,
        settings: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error> {
        let sound = &self.sounds[sound_handle];
        let data = Cursor::new(VecAsRef(Arc::clone(&sound.data)));
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
            self.make_signal_from_event_sound(&sound, settings, data)?
        };

        // Add sound instance to active list.
        let mut sound_instances = self.sound_instances.lock().unwrap();
        let handle = sound_instances.insert(SoundInstance {
            handle: Some(sound_handle),
            clip_id: None,
            signal,
            active: true,
        });
        Ok(handle)
    }

    fn stop_sound(&mut self, sound: SoundInstanceHandle) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.remove(sound);
    }

    fn stop_all_sounds(&mut self) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.clear();
    }

    fn stop_sounds_with_handle(&mut self, handle: SoundHandle) {
        let mut sound_instances = self.sound_instances.lock().unwrap();
        let handle = Some(handle);
        sound_instances.retain(|_, instance| instance.handle != handle);
    }

    fn get_sound_duration(&self, sound: SoundHandle) -> Option<u32> {
        if let Some(sound) = self.sounds.get(sound) {
            // AS duration does not subtract skip_sample_frames.
            let num_sample_frames = u64::from(sound.num_sample_frames);
            let ms = num_sample_frames * 1000 / u64::from(sound.format.sample_rate);
            Some(ms as u32)
        } else {
            None
        }
    }

    fn is_sound_playing_with_handle(&mut self, handle: SoundHandle) -> bool {
        let sound_instances = self.sound_instances.lock().unwrap();
        let handle = Some(handle);
        sound_instances
            .iter()
            .any(|(_, instance)| instance.handle == handle && instance.active)
    }

    fn tick(&mut self) {}
}

/// A dummy wrapper struct to implement `AsRef<[u8]>` for `Arc<Vec<u8>`.
/// Not having this trait causes problems when trying to use `Cursor<Vec<u8>>`.
struct VecAsRef(Arc<Vec<u8>>);

impl AsRef<[u8]> for VecAsRef {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Default for VecAsRef {
    fn default() -> Self {
        VecAsRef(Arc::new(vec![]))
    }
}

/// A signal for event sound instances using sound settings (looping, start/end point, envelope).
struct EventSoundSignal {
    decoder: Box<dyn SeekableDecoder + Send>,
    num_loops: u16,
    envelope_signal: Option<EnvelopeSignal>,
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
        let skip_sample_frames = u32::from(skip_sample_frames);
        let sample_divisor = 44100 / u32::from(decoder.sample_rate());
        let start_sample_frame =
            settings.in_sample.unwrap_or(0) / sample_divisor + skip_sample_frames;
        let end_sample_frame = settings
            .out_sample
            .map(|n| n / sample_divisor)
            .unwrap_or(num_sample_frames)
            + skip_sample_frames;

        let envelope_signal = if let Some(envelope) = &settings.envelope {
            Some(EnvelopeSignal::new(envelope.clone()))
        } else {
            None
        };

        let mut signal = Self {
            decoder,
            num_loops: settings.num_loops,
            envelope_signal,
            start_sample_frame,
            end_sample_frame: Some(end_sample_frame),
            cur_sample_frame: start_sample_frame,
            is_exhausted: false,
        };
        signal.next_loop();
        signal
    }
}

impl EventSoundSignal {
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

impl sample::signal::Signal for EventSoundSignal {
    type Frame = [i16; 2];

    fn next(&mut self) -> Self::Frame {
        // Loop the sound if necessary, and get the next frame.
        if !self.is_exhausted {
            let frame = if let Some(frame) = self.decoder.next() {
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
            };
            if let Some(envelope) = &mut self.envelope_signal {
                use sample::frame::Frame;
                frame.mul_amp(envelope.next())
            } else {
                frame
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
    fn new(envelope: swf::SoundEnvelope) -> Self {
        // TODO: This maybe can be done more clever using the `sample` crate.
        let mut envelope = envelope.into_iter();
        let first_point = envelope.next().unwrap_or_else(|| swf::SoundEnvelopePoint {
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
impl sample::signal::Signal for EnvelopeSignal {
    type Frame = [f32; 2];

    fn next(&mut self) -> Self::Frame {
        // Calculate interpolated volume.
        let out = if self.prev_point.sample < self.next_point.sample {
            let a = f64::from(self.cur_sample - self.prev_point.sample);
            let b = f64::from(self.next_point.sample - self.prev_point.sample);
            let lerp = a / b;
            let interpolator = sample::interpolate::Linear::new(
                [self.prev_point.left_volume, self.prev_point.right_volume],
                [self.next_point.left_volume, self.next_point.right_volume],
            );
            use sample::interpolate::Interpolator;
            interpolator.interpolate(lerp)
        } else {
            [self.next_point.left_volume, self.next_point.right_volume]
        };

        // Update envelope endpoints.
        self.cur_sample = self.cur_sample.saturating_add(1);
        while self.cur_sample > self.next_point.sample {
            self.prev_point = self.next_point.clone();
            self.next_point =
                self.envelope
                    .next()
                    .clone()
                    .unwrap_or_else(|| swf::SoundEnvelopePoint {
                        sample: std::u32::MAX,
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
