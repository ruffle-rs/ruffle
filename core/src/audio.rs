use crate::backend::audio::AudioBackend;

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

    pub fn play_sound(&mut self, sound: SoundHandle) {
        self.backend.play_sound(sound)
    }

    pub fn preload_sound_stream_head(
        &mut self,
        clip_id: swf::CharacterId,
        stream_info: &swf::SoundStreamHead,
    ) {
        self.backend.preload_sound_stream_head(clip_id, stream_info)
    }

    pub fn preload_sound_stream_block(&mut self, clip_id: swf::CharacterId, audio_data: &[u8]) {
        self.backend.preload_sound_stream_block(clip_id, audio_data);
    }

    pub fn preload_sound_stream_end(&mut self, clip_id: swf::CharacterId) {
        self.backend.preload_sound_stream_end(clip_id);
    }

    pub fn start_stream(
        &mut self,
        clip_id: crate::prelude::CharacterId,
        clip_data: crate::tag_utils::SwfSlice,
        handle: &swf::SoundStreamHead,
    ) -> AudioStreamHandle {
        self.backend.start_stream(clip_id, clip_data, handle)
    }

    pub fn stop_all_sounds(&mut self) {
        // TODO(Herschel)
    }

    pub fn is_loading_complete(&self) -> bool {
        self.backend.is_loading_complete()
    }

    pub fn prime_audio(&mut self) {
        self.backend.prime_audio();
    }
}
