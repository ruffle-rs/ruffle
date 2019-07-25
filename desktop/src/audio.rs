use generational_arena::Arena;
use ruffle_core::backend::audio::decoders::{stream_tag_reader, AdpcmDecoder, Decoder, Mp3Decoder};
use ruffle_core::backend::audio::{swf, AudioBackend, AudioStreamHandle, SoundHandle};
use std::io::Cursor;
use std::sync::Arc;

pub struct RodioAudioBackend {
    sounds: Arena<Sound>,
    active_sounds: Arena<rodio::Sink>,
    streams: Arena<AudioStream>,
    device: rodio::Device,
}

#[allow(dead_code)]
struct AudioStream {
    clip_id: swf::CharacterId,
    info: swf::SoundStreamHead,
    sink: rodio::Sink,
}

#[allow(dead_code)]
struct Sound {
    format: swf::SoundFormat,
    data: Arc<Vec<u8>>,
}

impl RodioAudioBackend {
    pub fn new() -> Result<Self, Box<std::error::Error>> {
        Ok(Self {
            sounds: Arena::new(),
            streams: Arena::new(),
            active_sounds: Arena::new(),
            device: rodio::default_output_device().ok_or("Unable to create output device")?,
        })
    }
}

impl AudioBackend for RodioAudioBackend {
    fn register_sound(
        &mut self,
        swf_sound: &swf::Sound,
    ) -> Result<SoundHandle, Box<std::error::Error>> {
        let sound = Sound {
            format: swf_sound.format.clone(),
            data: Arc::new(swf_sound.data.clone()),
        };
        Ok(self.sounds.insert(sound))
    }

    fn start_stream(
        &mut self,
        clip_id: swf::CharacterId,
        clip_data: ruffle_core::tag_utils::SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> AudioStreamHandle {
        let sink = rodio::Sink::new(&self.device);

        let format = &stream_info.stream_format;
        let decoder = Mp3Decoder::new(
            if format.is_stereo { 2 } else { 1 },
            format.sample_rate.into(),
            stream_tag_reader(clip_data),
        );

        let stream = AudioStream {
            clip_id,
            info: stream_info.clone(),
            sink,
        };
        stream.sink.append(DecoderSource(Box::new(decoder)));
        self.streams.insert(stream)
    }

    fn play_sound(&mut self, sound: SoundHandle) {
        let sound = &self.sounds[sound];
        use swf::AudioCompression;

        match sound.format.compression {
            AudioCompression::Uncompressed => {
                let mut data = Vec::with_capacity(sound.data.len() / 2);
                let mut i = 0;
                while i < sound.data.len() {
                    let val = i16::from(sound.data[i]) | (i16::from(sound.data[i + 1]) << 8);
                    data.push(val);
                    i += 2;
                }
                let buffer = rodio::buffer::SamplesBuffer::new(
                    if sound.format.is_stereo { 2 } else { 1 },
                    sound.format.sample_rate.into(),
                    data,
                );
                let sink = rodio::Sink::new(&self.device);
                sink.append(buffer);
                self.active_sounds.insert(sink);
            }
            AudioCompression::Adpcm => {
                let decoder = AdpcmDecoder::new(
                    Cursor::new(sound.data.to_vec()),
                    sound.format.is_stereo,
                    sound.format.sample_rate,
                )
                .unwrap();
                let sink = rodio::Sink::new(&self.device);
                sink.append(DecoderSource(Box::new(decoder)));
                self.active_sounds.insert(sink);
            }
            AudioCompression::Mp3 => {
                let decoder = Mp3Decoder::new(
                    if sound.format.is_stereo { 2 } else { 1 },
                    sound.format.sample_rate.into(),
                    Cursor::new(sound.data.to_vec()),
                );
                let sink = rodio::Sink::new(&self.device);
                sink.append(DecoderSource(Box::new(decoder)));
                self.active_sounds.insert(sink);
            }
            _ => unimplemented!(),
        }
    }

    fn tick(&mut self) {
        self.active_sounds.retain(|_, sink| !sink.empty());
    }
}

struct DecoderSource(Box<Decoder + Send>);

impl Iterator for DecoderSource {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        self.0.next()
    }
}
impl rodio::Source for DecoderSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.0.num_channels().into()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.0.sample_rate().into()
    }

    #[inline]
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
