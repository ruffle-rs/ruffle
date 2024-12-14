use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, DecodeError, RegisterError, SoundHandle, SoundInstanceHandle,
    SoundStreamInfo, SoundTransform,
};
use ruffle_core::impl_audio_mixer_backend;

pub struct TestAudioBackend {
    mixer: AudioMixer,
    buffer: Vec<f32>,
}

impl Default for TestAudioBackend {
    fn default() -> Self {
        Self {
            mixer: AudioMixer::new(Self::NUM_CHANNELS, Self::SAMPLE_RATE),
            buffer: vec![],
        }
    }
}

impl TestAudioBackend {
    const NUM_CHANNELS: u8 = 2;
    const SAMPLE_RATE: u32 = 44100;
}

impl AudioBackend for TestAudioBackend {
    impl_audio_mixer_backend!(mixer);
    fn play(&mut self) {}
    fn pause(&mut self) {}

    fn set_frame_rate(&mut self, frame_rate: f64) {
        let new_buffer_size =
            ((Self::NUM_CHANNELS as u32 * Self::SAMPLE_RATE) as f64 / frame_rate).round() as usize;
        self.buffer.resize(new_buffer_size, 0.0);
    }
    fn tick(&mut self) {
        debug_assert!(!self.buffer.is_empty());
        self.mixer.mix::<f32>(self.buffer.as_mut());
    }
}
