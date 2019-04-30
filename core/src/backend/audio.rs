pub mod null;
pub mod web;

pub type AudioStreamHandle = generational_arena::Index;

pub trait AudioBackend {
    fn register_stream(&mut self, stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle;
    fn queue_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]);
}
