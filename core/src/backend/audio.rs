use generational_arena::{Arena, Index};

pub mod swf {
    pub use swf::{AudioCompression, SoundStreamInfo};
}

pub type AudioStreamHandle = Index;

pub trait AudioBackend {
    fn register_stream(&mut self, stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle;
    fn queue_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]);
}

pub struct NullAudioBackend {
    streams: Arena<()>,
}

impl NullAudioBackend {
    pub fn new() -> NullAudioBackend {
        NullAudioBackend {
            streams: Arena::new(),
        }
    }
}

impl AudioBackend for NullAudioBackend {
    fn register_stream(&mut self, _stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle {
        self.streams.insert(())
    }

    fn queue_stream_samples(&mut self, _handle: AudioStreamHandle, _samples: &[u8]) {
        // Noop
    }
}
