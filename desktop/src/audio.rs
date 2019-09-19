use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use generational_arena::Arena;
use ruffle_core::backend::audio::decoders::{
    stream_tag_reader, AdpcmDecoder, Decoder, Mp3Decoder, PcmDecoder,
};
use ruffle_core::backend::audio::{swf, AudioBackend, AudioStreamHandle, SoundHandle};
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

    /// Creates a `sample::signal::Signal` that decodes and resamples the audio stream
    /// to the output format.
    fn make_signal_from_stream<'a, R: 'a + std::io::Read + Send>(
        &self,
        format: &swf::SoundFormat,
        data_stream: R,
    ) -> Box<dyn 'a + Send + sample::signal::Signal<Frame = [i16; 2]>> {
        // Instantiate a decoder for the compression that the sound data uses.
        use swf::AudioCompression;
        let decoder: Box<dyn 'a + Send + Decoder> = match format.compression {
            AudioCompression::Uncompressed => Box::new(PcmDecoder::new(
                data_stream,
                format.is_stereo,
                format.sample_rate,
                format.is_16_bit,
            )),
            AudioCompression::Adpcm => Box::new(
                AdpcmDecoder::new(data_stream, format.is_stereo, format.sample_rate).unwrap(),
            ),
            AudioCompression::Mp3 => Box::new(Mp3Decoder::new(
                if format.is_stereo { 2 } else { 1 },
                format.sample_rate.into(),
                data_stream,
            )),
            _ => {
                log::error!(
                    "start_stream: Unhandled audio compression {:?}",
                    format.compression
                );
                unimplemented!()
            }
        };

        // Convert the `Decoder` to a `Signal`, and resample it the the output
        // sample rate.
        let mut signal = sample::signal::from_iter(decoder);
        let interpolator = sample::interpolate::Linear::from_source(&mut signal);
        Box::new(sample::interpolate::Converter::from_hz_to_hz(
            signal,
            interpolator,
            format.sample_rate.into(),
            self.output_format.sample_rate.0.into(),
        ))
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
        clip_data: ruffle_core::tag_utils::SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> AudioStreamHandle {
        let format = &stream_info.stream_format;

        // The audio data for stream sounds is distributed among the frames of a
        // movie clip. The stream tag reader will parse through the SWF and
        // feed the decoder audio data on the fly.
        let clip_stream_reader = stream_tag_reader(clip_data);
        // Create a signal that decodes and resamples the stream.
        let signal = self.make_signal_from_stream(format, clip_stream_reader);

        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.insert(SoundInstance {
            handle: None,
            clip_id: Some(clip_id),
            signal,
            active: true,
        })
    }

    fn play_sound(&mut self, sound_handle: SoundHandle) {
        let sound = &self.sounds[sound_handle];
        let data = VecAsRef(Arc::clone(&sound.data));
        // Create a signal that decodes and resamples the sound.
        let signal = self.make_signal_from_stream(&sound.format, Cursor::new(data));

        // Add sound instance to active list.
        let mut sound_instances = self.sound_instances.lock().unwrap();
        sound_instances.insert(SoundInstance {
            handle: Some(sound_handle),
            clip_id: None,
            signal,
            active: true,
        });
    }

    fn tick(&mut self) {}
}

// Unfortunately `Arc<Vec<u8>>` does not implement `AsRef<[u8]>`.
// This causes problem when trying to use `Cursor<Vec<u8>>`.
// Use a dummy wrapper struct to implement this trait.
struct VecAsRef(Arc<Vec<u8>>);

impl AsRef<[u8]> for VecAsRef {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
