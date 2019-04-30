use crate::backend::audio::AudioBackend;
//use generational_arena::Arena;
use swf::SoundStreamInfo;

pub struct Audio {
    backend: Box<AudioBackend>,
}

pub type AudioStreamHandle = generational_arena::Index;

impl Audio {
    pub fn new(backend: Box<AudioBackend>) -> Audio {
        Audio { backend }
    }

    pub fn register_stream(&mut self, stream_info: &SoundStreamInfo) -> AudioStreamHandle {
        self.backend.register_stream(stream_info)
    }

    pub fn queue_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]) {
        self.backend.queue_stream_samples(handle, samples)
    }
}

struct AudioStream {
    stream_info: SoundStreamInfo,
}
