use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, DecodeError, RegisterError, SoundHandle, SoundInstanceHandle,
    SoundTransform,
};
use ruffle_core::impl_audio_mixer_backend;

pub struct CpalAudioBackend {
    #[allow(dead_code)]
    device: cpal::Device,
    #[allow(dead_code)]
    config: cpal::StreamConfig,
    stream: cpal::Stream,
    mixer: AudioMixer,
}

type Error = Box<dyn std::error::Error>;

impl CpalAudioBackend {
    pub fn new() -> Result<Self, Error> {
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

            match sample_format {
                cpal::SampleFormat::F32 => device.build_output_stream(
                    &config,
                    move |buffer, _| mixer.mix::<f32>(buffer),
                    error_handler,
                ),
                cpal::SampleFormat::I16 => device.build_output_stream(
                    &config,
                    move |buffer, _| mixer.mix::<i16>(buffer),
                    error_handler,
                ),
                cpal::SampleFormat::U16 => device.build_output_stream(
                    &config,
                    move |buffer, _| mixer.mix::<u16>(buffer),
                    error_handler,
                ),
            }?
        };

        stream.play()?;

        Ok(Self {
            device,
            config,
            stream,
            mixer,
        })
    }
}

impl AudioBackend for CpalAudioBackend {
    impl_audio_mixer_backend!(mixer);

    fn play(&mut self) {
        self.stream.play().expect("Error trying to resume CPAL audio stream. This feature may not be supported by your audio device.");
    }

    fn pause(&mut self) {
        self.stream.pause().expect("Error trying to pause CPAL audio stream. This feature may not be supported by your audio device.");
    }
}
