use generational_arena::{Arena, Index};

pub mod swf {
    pub use swf::{AudioCompression, Sound, SoundFormat, SoundStreamInfo};
}

pub type AudioStreamHandle = Index;
pub type SoundHandle = Index;

type Error = Box<std::error::Error>;

pub trait AudioBackend {
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error>;
    fn register_stream(&mut self, stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle;
    fn play_sound(&mut self, sound: SoundHandle);
    fn queue_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]);
}

pub struct NullAudioBackend {
    sounds: Arena<()>,
    streams: Arena<()>,
}

impl NullAudioBackend {
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            streams: Arena::new(),
            sounds: Arena::new(),
        }
    }
}

impl AudioBackend for NullAudioBackend {
    fn register_sound(&mut self, sound: &swf::Sound) -> Result<SoundHandle, Error> {
        Ok(self.sounds.insert(()))
    }

    fn play_sound(&mut self, sound: SoundHandle) {}

    fn register_stream(&mut self, _stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle {
        self.streams.insert(())
    }

    fn queue_stream_samples(&mut self, _handle: AudioStreamHandle, _samples: &[u8]) {
        // Noop
    }
}
