use super::{AudioBackend, AudioStreamHandle};
use generational_arena::Arena;

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
