use generational_arena::{Arena, Index};

pub mod decoders;
pub mod swf {
    pub use swf::{read, AudioCompression, CharacterId, Sound, SoundFormat, SoundStreamHead};
}

pub type AudioStreamHandle = Index;
pub type SoundHandle = Index;

type Error = Box<std::error::Error>;

pub trait AudioBackend {
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error>;
    fn preload_sound_stream_head(
        &mut self,
        _clip_id: swf::CharacterId,
        _stream_info: &swf::SoundStreamHead,
    ) {
    }
    fn preload_sound_stream_block(&mut self, _clip_id: swf::CharacterId, _audio_data: &[u8]) {}
    fn preload_sound_stream_end(&mut self, _clip_id: swf::CharacterId) {}
    fn play_sound(&mut self, sound: SoundHandle);
    fn start_stream(
        &mut self,
        clip_id: crate::prelude::CharacterId,
        clip_data: crate::tag_utils::SwfSlice,
        handle: &swf::SoundStreamHead,
    ) -> AudioStreamHandle;
    // TODO: Eventually remove this/move it to library.
    fn is_loading_complete(&self) -> bool {
        true
    }
    fn tick(&mut self) {}
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
    fn register_sound(&mut self, _sound: &swf::Sound) -> Result<SoundHandle, Error> {
        Ok(self.sounds.insert(()))
    }

    fn play_sound(&mut self, _sound: SoundHandle) {}

    fn start_stream(
        &mut self,
        _clip_id: crate::prelude::CharacterId,
        _clip_data: crate::tag_utils::SwfSlice,
        _handle: &swf::SoundStreamHead,
    ) -> AudioStreamHandle {
        self.streams.insert(())
    }
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        NullAudioBackend::new()
    }
}
