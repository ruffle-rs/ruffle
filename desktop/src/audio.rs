use generational_arena::Arena;
use rodio::{source::Source, Sample, Sink};
use ruffle_core::backend::audio::{swf, AudioBackend, AudioStreamHandle, SoundHandle};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

pub struct RodioAudioBackend {
    sounds: Arena<Sound>,
    active_sounds: Arena<Sink>,
    streams: Arena<AudioStream>,
    device: rodio::Device,
}

struct AudioStream {
    info: swf::SoundStreamInfo,
    sink: rodio::Sink,
    data: Arc<Mutex<Cursor<Vec<u8>>>>,
}

struct Sound {
    format: swf::SoundFormat,
    data: Vec<u8>,
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
            data: swf_sound.data.clone(),
        };
        Ok(self.sounds.insert(sound))
    }

    fn register_stream(&mut self, stream_info: &swf::SoundStreamInfo) -> AudioStreamHandle {
        let sink = Sink::new(&self.device);
        let data = Arc::new(Mutex::new(Cursor::new(vec![])));

        let format = &stream_info.stream_format;
        let decoder = Mp3Decoder::new(
            if format.is_stereo { 2 } else { 1 },
            format.sample_rate as u32,
            ThreadRead(Arc::clone(&data)),
        )
        .unwrap();
        let stream = AudioStream {
            info: stream_info.clone(),
            sink,
            data,
        };
        stream.sink.append(decoder);
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
                    let val = sound.data[i] as i16 | ((sound.data[i + 1] as i16) << 8);
                    data.push(val);
                }
                let buffer = rodio::buffer::SamplesBuffer::new(
                    if sound.format.is_stereo { 2 } else { 1 },
                    sound.format.sample_rate.into(),
                    data,
                );
                let sink = Sink::new(&self.device);
                sink.append(buffer);
                self.active_sounds.insert(sink);
            }
            AudioCompression::Mp3 => {
                let decoder = Mp3EventDecoder::new(Cursor::new(sound.data.clone())).unwrap();
                let sink = Sink::new(&self.device);
                sink.append(decoder);
                self.active_sounds.insert(sink);
            }
            _ => unimplemented!(),
        }
    }

    fn queue_stream_samples(&mut self, handle: AudioStreamHandle, mut samples: &[u8]) {
        if let Some(stream) = self.streams.get_mut(handle) {
            let mut stream_channels = 0;
            let mut n = 0;
            let mut stream_sample_rate = 0;

            let tag_samples = (samples[0] as u16) | ((samples[1] as u16) << 8);
            samples = &samples[4..];

            let mut buffer = stream.data.lock().unwrap();
            buffer.get_mut().extend_from_slice(&samples);
        }
    }

    fn tick(&mut self) {
        self.active_sounds.retain(|_, sink| !sink.empty());
    }
}

use std::io::{self, Read, Seek};
use std::time::Duration;

use minimp3::{Decoder, Frame};

pub struct ThreadRead(Arc<Mutex<Cursor<Vec<u8>>>>);

impl Read for ThreadRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buffer = self.0.lock().unwrap();
        let result = buffer.read(buf);
        let len_remaining = buffer.get_ref().len() - buffer.position() as usize;
        let tmp = buffer.get_ref()[buffer.position() as usize..].to_vec();
        buffer.get_mut().resize(len_remaining, 0);
        *buffer.get_mut() = tmp;
        buffer.set_position(0);
        result
    }
}

impl Seek for ThreadRead {
    fn seek(&mut self, pos: std::io::SeekFrom) -> io::Result<u64> {
        self.0.lock().unwrap().seek(pos)
    }
}

pub struct Mp3Decoder {
    decoder: Decoder<ThreadRead>,
    sample_rate: u32,
    num_channels: u16,
    current_frame: Frame,
    current_frame_offset: usize,
    playing: bool,
}

impl Mp3Decoder {
    pub fn new(num_channels: u16, sample_rate: u32, data: ThreadRead) -> Result<Self, ()> {
        let decoder = Decoder::new(data);
        let current_frame = Frame {
            data: vec![],
            sample_rate: sample_rate as _,
            channels: num_channels as _,
            layer: 3,
            bitrate: 160,
        };

        Ok(Mp3Decoder {
            decoder,
            num_channels,
            sample_rate,
            current_frame,
            current_frame_offset: 0,
            playing: false,
        })
    }
}

impl Source for Mp3Decoder {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None //Some(self.current_frame.data.len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.num_channels
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for Mp3Decoder {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if !self.playing {
            let buffer = self.decoder.reader().0.lock().unwrap();
            if buffer.get_ref().len() < 44100 / 60 {
                return Some(0);
            }
            self.playing = true;
        }

        if self.current_frame_offset == self.current_frame.data.len() {
            match self.decoder.next_frame() {
                Ok(frame) => self.current_frame = frame,
                _ => return Some(0),
            }
            self.current_frame_offset = 0;
        }

        let v = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;

        Some(v)
    }
}

pub struct Mp3EventDecoder<R>
where
    R: Read + Seek,
{
    decoder: Decoder<R>,
    current_frame: Frame,
    current_frame_offset: usize,
}

impl<R> Mp3EventDecoder<R>
where
    R: Read + Seek,
{
    pub fn new(data: R) -> Result<Self, ()> {
        let mut decoder = Decoder::new(data);
        let current_frame = decoder.next_frame().map_err(|_| ())?;

        Ok(Mp3EventDecoder {
            decoder,
            current_frame,
            current_frame_offset: 0,
        })
    }
}

impl<R> Source for Mp3EventDecoder<R>
where
    R: Read + Seek,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.current_frame.data.len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.current_frame.channels as _
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.current_frame.sample_rate as _
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<R> Iterator for Mp3EventDecoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.current_frame.data.len() {
            self.current_frame_offset = 0;
            match self.decoder.next_frame() {
                Ok(frame) => self.current_frame = frame,
                _ => return None,
            }
        }

        let v = self.current_frame.data[self.current_frame_offset];
        self.current_frame_offset += 1;

        return Some(v);
    }
}
