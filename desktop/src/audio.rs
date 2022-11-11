use anyhow::{anyhow, Context, Error};
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

impl CpalAudioBackend {
    pub fn new() -> Result<Self, Error> {
        // Create CPAL audio device.
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("No audio devices available"))?;

        // Create audio stream for device.
        let config = device
            .default_output_config()
            .context("Failed to get default output config")?;
        let sample_format = config.sample_format();
        let config = cpal::StreamConfig::from(config);
        let mixer = AudioMixer::new(config.channels as u8, config.sample_rate.0);

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
                    move |buffer: &mut [u16], _| {
                        // Since I couldn't easily make `mixer` work with `u16` samples,
                        // we fill the buffer as if it was `&[i16]`, and then rotate
                        // the sample values to make 32768 the equilibrium.
                        mixer.mix::<i16>(bytemuck::cast_slice_mut(buffer));
                        for s in buffer.iter_mut() {
                            *s = (*s).wrapping_add(32768);
                        }
                    },
                    error_handler,
                ),
            }?
        };

        stream.play().context("Couldn't play the audio stream")?;

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
