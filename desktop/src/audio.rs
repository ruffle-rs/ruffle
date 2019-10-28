use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use generational_arena::Arena;
use ruffle_core::backend::audio::decoders::{
    self, AdpcmDecoder, Mp3Decoder, PcmDecoder, SeekableDecoder,
};
use ruffle_core::backend::audio::{swf, AudioBackend, AudioStreamHandle, SoundHandle};
use ruffle_core::tag_utils::SwfSlice;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct CpalAudioBackend {
    device: cpal::Device,
    output_format: cpal::Format,
    audio_thread_handle: std::thread::JoinHandle<()>,

    sounds: Arena<Sound>,
    sound_instances: Arc<Mutex<Arena<SoundInstance>>>,
}

type Signal = Box<dyn Send + sample::signal::Signal<Frame = [i16; 2]>>;

/// Contains the data and metadata for a sound in an SWF file.
/// A `Sound` is defined by the `DefineSound` SWF tags.
struct Sound {
    format: swf::SoundFormat,
    data: Arc<Vec<u8>>,
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
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
    ) -> Box<dyn Send + SeekableDecoder> {
        use swf::AudioCompression;
        match format.compression {
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
                log::error!(
                    "start_stream: Unhandled audio compression {:?}",
                    format.compression
                );
                unimplemented!()
            }
        }
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
        format: &swf::SoundFormat,
        settings: &swf::SoundInfo,
        data: Cursor<VecAsRef>,
    ) -> Box<dyn Send + sample::signal::Signal<Frame = [i16; 2]>> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = Self::make_seekable_decoder(format, data);

        // Wrap the decoder in the event sound signal (controls looping/envelope)
        let signal = EventSoundSignal::new_with_settings(decoder, settings);
        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = self.make_resampler(format, signal);
        Box::new(signal)
    }

    /// Creates a `sample::signal::Signal` that decodes and resamples a "stream" sound.
    fn make_signal_from_stream<'a>(
        &self,
        format: &swf::SoundFormat,
        data_stream: SwfSlice,
        swf_version: u8,
    ) -> Box<dyn 'a + Send + sample::signal::Signal<Frame = [i16; 2]>> {
        // Instantiate a decoder for the compression that the sound data uses.
        let clip_stream_decoder = decoders::make_stream_decoder(format, data_stream, swf_version);

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = sample::signal::from_iter(clip_stream_decoder);
        let signal = Box::new(self.make_resampler(format, signal));
        Box::new(signal)
    }

    /// Creates a `sample::signal::Signal` that decodes and resamples the audio stream
    /// to the output format.
    fn make_signal_from_simple_event_sound<'a, R: 'a + std::io::Read + Send>(
        &self,
        format: &swf::SoundFormat,
        data_stream: R,
    ) -> Box<dyn 'a + Send + sample::signal::Signal<Frame = [i16; 2]>> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = decoders::make_decoder(format, data_stream);

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let signal = sample::signal::from_iter(decoder);
        let signal = self.make_resampler(format, signal);
        Box::new(signal)
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
    fn register_sound(
        &mut self,
        swf_sound: &swf::Sound,
    ) -> Result<SoundHandle, Box<dyn std::error::Error>> {
        let sound = Sound {
            format: swf_sound.format.clone(),
            data: Arc::new(swf_sound.data.clone()),
        };
        Ok(self.sounds.insert(sound))
    }

    fn start_stream(
        &mut self,
        clip_id: swf::CharacterId,
        clip_data: SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> AudioStreamHandle {
        let format = &stream_info.stream_format;

        // The audio data for stream sounds is distributed among the frames of a
        // movie clip. The stream tag reader will parse through the SWF and
        // feed the decoder audio data on the fly.
        // TODO: Use actual SWF version here (would only matter for SWF <3...)
        let signal = self.make_signal_from_stream(format, clip_data, 8);

        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.insert(SoundInstance {
            handle: None,
            clip_id: Some(clip_id),
            signal,
            active: true,
        })
    }

    fn start_sound(&mut self, sound_handle: SoundHandle, settings: &swf::SoundInfo) {
        let sound = &self.sounds[sound_handle];
        let data = Cursor::new(VecAsRef(Arc::clone(&sound.data)));
        // Create a signal that decodes and resamples the sound.
        let signal = if settings.in_sample.is_none()
            && settings.out_sample.is_none()
            && settings.num_loops <= 1
            && settings.envelope.is_none()
        {
            // For simple event sounds, just use the same signal as streams.
            self.make_signal_from_simple_event_sound(&sound.format, data)
        } else {
            // For event sounds with envelopes/other properties, wrap it in `EventSoundSignal`.
            self.make_signal_from_event_sound(&sound.format, settings, data)
        };

        // Add sound instance to active list.
        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.insert(SoundInstance {
            handle: Some(sound_handle),
            clip_id: None,
            signal,
            active: true,
        });
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
    start_sample_frame: u32,
    end_sample_frame: Option<u32>,
    cur_sample_frame: u32,
    is_exhausted: bool,
}

impl EventSoundSignal {
    fn new_with_settings(
        decoder: Box<dyn SeekableDecoder + Send>,
        settings: &swf::SoundInfo,
    ) -> Self {
        let sample_divisor = 44100 / u32::from(decoder.sample_rate());
        let start_sample_frame = settings.in_sample.unwrap_or(0) / sample_divisor;
        let mut signal = Self {
            decoder,
            num_loops: settings.num_loops,
            start_sample_frame,
            end_sample_frame: settings.out_sample.map(|n| n / sample_divisor),
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
