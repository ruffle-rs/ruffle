use generational_arena::Arena;
use js_sys::Uint8Array;
use ruffle_core::backend::audio::{swf, AudioBackend, AudioStreamHandle, SoundHandle};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::AudioContext;

thread_local! {
    //pub static SOUNDS: RefCell<Vec<>>>> = RefCell::new(vec![]);
}

pub struct WebAudioBackend {
    context: AudioContext,
    sounds: Arena<Sound>,
    streams: Arena<AudioStream>,
}

struct Sound {
    object: js_sys::Object,
}

struct AudioStream {
    info: swf::SoundStreamHead,
    compressed_data: Vec<u8>,
    sample_data: [Vec<f32>; 2],
    object: js_sys::Object,
}

type Error = Box<std::error::Error>;

impl WebAudioBackend {
    pub fn new() -> Result<Self, Error> {
        let context = AudioContext::new().map_err(|_| "Unable to create AudioContext")?;
        Ok(Self {
            context,
            sounds: Arena::new(),
            streams: Arena::new(),
        })
    }
}

impl AudioBackend for WebAudioBackend {
    fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, Error> {
        let object = js_sys::Object::new();
        let sound = Sound {
            object: object.clone(),
        };
        let value = wasm_bindgen::JsValue::from(object);
        let handle = self.sounds.insert(sound);

        // Firefox doesn't seem to support <11025Hz sample rates.
        let (sample_multiplier, sample_rate) = if swf_sound.format.sample_rate < 11025 {
            (2, 11025)
        } else {
            (1, swf_sound.format.sample_rate)
        };

        log::info!(
            "Compression: {:?} SR: {} {} {}",
            swf_sound.format.compression,
            swf_sound.format.sample_rate,
            sample_rate,
            sample_multiplier
        );

        use byteorder::{LittleEndian, ReadBytesExt};
        match swf_sound.format.compression {
            swf::AudioCompression::Uncompressed => {
                let num_channels: usize = if swf_sound.format.is_stereo { 2 } else { 1 };
                let num_frames = swf_sound.data.len() * sample_multiplier / num_channels;
                let audio_buffer = self
                    .context
                    .create_buffer(num_channels as u32, num_frames as u32, sample_rate.into())
                    .unwrap();
                let mut out = Vec::with_capacity(num_channels);
                for _ in 0..num_channels {
                    out.push(Vec::with_capacity(num_frames));
                }
                let mut data = &swf_sound.data[..];
                while !data.is_empty() {
                    for channel in &mut out {
                        if sample_rate != swf_sound.format.sample_rate {
                            let sample = f32::from(data.read_i16::<LittleEndian>()?) / 32768.0;
                            for _ in 0..sample_multiplier {
                                channel.push(sample);
                            }
                        }
                    }
                }
                for (i, channel) in out.iter_mut().enumerate() {
                    audio_buffer
                        .copy_to_channel(&mut channel[..], i as i32)
                        .unwrap();
                }
                js_sys::Reflect::set(&value, &"buffer".into(), &audio_buffer).unwrap();
            }
            swf::AudioCompression::Adpcm => {
                let num_channels: usize = if swf_sound.format.is_stereo { 2 } else { 1 };
                let audio_buffer = self
                    .context
                    .create_buffer(
                        num_channels as u32,
                        swf_sound.num_samples * sample_multiplier as u32,
                        sample_rate.into(),
                    )
                    .unwrap();
                let mut out = Vec::with_capacity(num_channels);
                let data = &swf_sound.data[..];
                let mut decoder = ruffle_core::backend::audio::AdpcmDecoder::new(
                    data,
                    swf_sound.format.is_stereo,
                )?;
                for _ in 0..num_channels {
                    out.push(Vec::with_capacity(swf_sound.num_samples as usize));
                }
                while let Ok((left, right)) = decoder.next_sample() {
                    for _ in 0..sample_multiplier {
                        out[0].push(f32::from(left) / 32768.0);
                        if swf_sound.format.is_stereo {
                            out[1].push(f32::from(right) / 32768.0);
                        }
                    }
                }
                for (i, channel) in out.iter_mut().enumerate() {
                    audio_buffer
                        .copy_to_channel(&mut channel[..], i as i32)
                        .unwrap();
                }
                js_sys::Reflect::set(&value, &"buffer".into(), &audio_buffer).unwrap();
            }
            swf::AudioCompression::Mp3 => {
                let data_array = unsafe { Uint8Array::view(&swf_sound.data[2..]) };
                let array_buffer = data_array.buffer().slice_with_end(
                    data_array.byte_offset(),
                    data_array.byte_offset() + data_array.byte_length(),
                );
                let closure = Closure::wrap(Box::new(move |buffer: wasm_bindgen::JsValue| {
                    js_sys::Reflect::set(&value, &"buffer".into(), &buffer).unwrap();
                })
                    as Box<dyn FnMut(wasm_bindgen::JsValue)>);
                self.context
                    .decode_audio_data(&array_buffer)
                    .unwrap()
                    .then(&closure);
                closure.forget();
            }
            _ => unimplemented!(),
        }
        Ok(handle)
    }

    fn register_stream(&mut self, stream_info: &swf::SoundStreamHead) -> AudioStreamHandle {
        let stream = AudioStream {
            info: stream_info.clone(),
            sample_data: [vec![], vec![]],
            compressed_data: vec![],
            object: js_sys::Object::new(),
        };
        self.streams.insert(stream)
    }

    fn play_sound(&mut self, sound: SoundHandle) {
        if let Some(sound) = self.sounds.get(sound) {
            let object = js_sys::Reflect::get(&sound.object, &"buffer".into()).unwrap();
            if object.is_undefined() {
                return;
            }
            let buffer: &web_sys::AudioBuffer = object.dyn_ref().unwrap();
            let buffer_node = self.context.create_buffer_source().unwrap();
            buffer_node.set_buffer(Some(buffer));
            buffer_node
                .connect_with_audio_node(&self.context.destination())
                .unwrap();

            buffer_node.start().unwrap();
        }
    }

    fn queue_stream_samples(&mut self, _handle: AudioStreamHandle, _samples: &[u8]) {}

    fn preload_stream_samples(&mut self, handle: AudioStreamHandle, samples: &[u8]) {
        use swf::AudioCompression;
        if let Some(stream) = self.streams.get_mut(handle) {
            let format = &stream.info.stream_format;
            let num_channels = if format.is_stereo { 2 } else { 1 };
            let frame_size = num_channels * if format.is_16_bit { 2 } else { 1 };
            let _num_frames = samples.len() / frame_size;
            let mut i = 0;
            match format.compression {
                AudioCompression::Uncompressed | AudioCompression::UncompressedUnknownEndian => {
                    if format.is_16_bit {
                        while i < samples.len() {
                            for c in 0..num_channels {
                                let sample = (u16::from(samples[i])
                                    | (u16::from(samples[i + 1]) << 8))
                                    as i16;
                                stream.sample_data[c].push((f32::from(sample)) / 32768.0);
                                i += 2;
                            }
                        }
                    } else {
                        while i < samples.len() {
                            for c in 0..num_channels {
                                stream.sample_data[c].push((f32::from(samples[i]) - 127.0) / 128.0);
                                i += 1;
                            }
                        }
                    }
                }
                AudioCompression::Mp3 => {
                    stream.compressed_data.extend_from_slice(&samples[4..]);
                }
                AudioCompression::Adpcm => {
                    let mut decoder =
                        ruffle_core::backend::audio::AdpcmDecoder::new(samples, format.is_stereo)
                            .unwrap();
                    while let Ok((left, right)) = decoder.next_sample() {
                        stream.sample_data[0].push(f32::from(left) / 32768.0);
                        if format.is_stereo {
                            stream.sample_data[1].push(f32::from(right) / 32768.0);
                        }
                    }
                }
                _ => (),
            }
        }
    }

    fn preload_stream_finalize(&mut self, handle: AudioStreamHandle) {
        if let Some(stream) = self.streams.get_mut(handle) {
            let format = &stream.info.stream_format;

            let num_channels = if format.is_stereo { 2 } else { 1 };

            use swf::AudioCompression;
            match format.compression {
                AudioCompression::UncompressedUnknownEndian
                | AudioCompression::Uncompressed
                | AudioCompression::Adpcm => {
                    if stream.sample_data[0].is_empty() {
                        return;
                    }

                    let frame_size = num_channels * if format.is_16_bit { 2 } else { 1 };
                    let num_frames = stream.sample_data[0].len() / frame_size;
                    let audio_buffer = self
                        .context
                        .create_buffer(
                            num_channels as u32,
                            num_frames as u32,
                            format.sample_rate.into(),
                        )
                        .unwrap();
                    for i in 0..num_channels {
                        audio_buffer
                            .copy_to_channel(&mut stream.sample_data[i][..], i as i32)
                            .unwrap();
                    }
                    js_sys::Reflect::set(&stream.object, &"buffer".into(), &audio_buffer).unwrap();
                }
                AudioCompression::Mp3 => {
                    if stream.compressed_data.is_empty() {
                        return;
                    }

                    let data_array = unsafe { Uint8Array::view(&stream.compressed_data[..]) };
                    let array_buffer = data_array.buffer().slice_with_end(
                        data_array.byte_offset(),
                        data_array.byte_offset() + data_array.byte_length(),
                    );
                    let object = stream.object.clone();
                    let closure = Closure::wrap(Box::new(move |buffer: wasm_bindgen::JsValue| {
                        js_sys::Reflect::set(&object, &"buffer".into(), &buffer).unwrap();
                    })
                        as Box<dyn FnMut(wasm_bindgen::JsValue)>);
                    self.context
                        .decode_audio_data(&array_buffer)
                        .unwrap()
                        .then(&closure);
                    closure.forget();
                }
                _ => log::info!("Unsupported sound format"),
            }
        }
    }

    fn start_stream(&mut self, handle: AudioStreamHandle) -> bool {
        if let Some(stream) = self.streams.get_mut(handle) {
            let object = js_sys::Reflect::get(&stream.object, &"buffer".into()).unwrap();
            if object.is_undefined() {
                return false;
            }

            let buffer: &web_sys::AudioBuffer = object.dyn_ref().unwrap();
            log::info!("Playing stream: {:?} {}", handle, buffer.length());

            let buffer_node = self.context.create_buffer_source().unwrap();
            buffer_node.set_buffer(Some(buffer));
            buffer_node
                .connect_with_audio_node(&self.context.destination())
                .unwrap();

            buffer_node.start().unwrap();
        }
        true
    }
}
