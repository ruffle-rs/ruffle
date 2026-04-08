use ruffle_core::backend::audio::{AudioMixer};
use ctru::services::ndsp::Ndsp;

const SAMPLE_RATE: u32 = 22050;
const NUM_CHANNELS: u8 = 24;

pub struct NdspAudioBackend {
    ndsp: Ndsp,
    mixer: AudioMixer,
}

impl NdspAudioBackend {
    pub fn new() -> ctru::error::Result<Self> {
        Self {
            ndsp: Ndsp::new()?,
            mixer: AudioMixer::new(NUM_CHANNELS, SAMPLE_RATE)
        }
    }
}

impl AudioBackend for NdspAudioBackend {
    impl_audio_mixer_backend!(mixer);

    fn play(&mut self) {
        self.ndsp.channel(0).expect("Failed to get NDSP Channel 0").set_paused(false);
    }

    fn play(&mut self) {
        self.ndsp.channel(0).expect("Failed to get NDSP Channel 0").set_paused(true);
    }
}