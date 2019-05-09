use crate::backend::audio::AudioBackend;
//use generational_arena::Arena;
use swf::SoundStreamInfo;

pub struct Audio {
    backend: Box<AudioBackend>,
}

pub type AudioStreamHandle = generational_arena::Index;
pub type SoundHandle = generational_arena::Index;

type Error = Box<std::error::Error>;

impl Audio {
    pub fn new(backend: Box<AudioBackend>) -> Audio {
        Audio { backend }
    }

    pub fn register_sound(&mut self, sound: &swf::Sound) -> Result<SoundHandle, Error> {
        self.backend.register_sound(sound)
    }

    pub fn register_stream(&mut self, stream_info: &SoundStreamInfo) -> AudioStreamHandle {
        self.backend.register_stream(stream_info)
    }

    pub fn play_sound(&mut self, sound: SoundHandle) {
        self.backend.play_sound(sound)
    }

    pub fn preload_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]) {
        self.backend.preload_stream_samples(handle, samples)
    }

    pub fn preload_stream_finalize(&mut self, handle: AudioStreamHandle) {
        self.backend.preload_stream_finalize(handle)
    }

    pub fn start_stream(&mut self, handle: AudioStreamHandle) -> bool {
        self.backend.start_stream(handle)
    }

    pub fn queue_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]) {
        self.backend.queue_stream_samples(handle, samples)
    }

    pub fn stop_all_sounds(&mut self) {
        // TODO(Herschel)
    }
}
