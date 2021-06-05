use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, SoundHandle, SoundInstanceHandle, SoundTransform,
};
use ruffle_core::impl_audio_mixer_backend;
use std::convert::TryInto;

#[allow(dead_code)]
pub struct CpalAudioBackend {
    device: cpal::Device,
    output_config: cpal::StreamConfig,
    stream: Stream,
    mixer: AudioMixer,
}

type Error = Box<dyn std::error::Error>;

// Because of https://github.com/RustAudio/cpal/pull/348, we have to initialize cpal on a
// separate thread (see `new` below). Unfortunately `cpal::Stream` is marked `!Send`, but
// we know this should be safe (since we aren't accessing the stream at all after creation;
// we just want to keep it alive)
struct Stream(cpal::Stream);
unsafe impl Send for CpalAudioBackend {}

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
        let device = host
            .default_output_device()
            .ok_or("No audio devices available")?;

        // Create audio stream for device.
        let config = device.default_output_config()?;
        let sample_format = config.sample_format();
        let config = cpal::StreamConfig::from(config);
        let mixer = AudioMixer::new(config.channels.try_into()?, config.sample_rate.0);

        // Start the audio stream.
        let stream = {
            let mixer = mixer.proxy();
            let error_handler = move |err| log::error!("Audio stream error: {}", err);

            use cpal::SampleFormat;
            match sample_format {
                SampleFormat::F32 => device.build_output_stream(
                    &config,
                    move |buffer, _| mixer.mix::<f32>(buffer),
                    error_handler,
                ),
                SampleFormat::I16 => device.build_output_stream(
                    &config,
                    move |buffer, _| mixer.mix::<i16>(buffer),
                    error_handler,
                ),
                SampleFormat::U16 => device.build_output_stream(
                    &config,
                    move |buffer, _| mixer.mix::<u16>(buffer),
                    error_handler,
                ),
            }?
        };

        stream.play()?;

        Ok(Self {
            device,
            output_config: config,
            stream: Stream(stream),
            mixer,
        })
    }
}

impl AudioBackend for CpalAudioBackend {
    impl_audio_mixer_backend!(mixer);

    fn play(&mut self) {
        self.stream.0.play().expect("Error trying to resume CPAL audio stream. This feature may not be supported by your audio device.");
    }

    fn pause(&mut self) {
        self.stream.0.pause().expect("Error trying to pause CPAL audio stream. This feature may not be supported by your audio device.");
    }
}
