use fnv::FnvHashMap;
use generational_arena::Arena;
use ruffle_core::backend::audio::{
    decoders::{AdpcmDecoder, NellymoserDecoder},
    swf::{self, AudioCompression},
    AudioBackend, PreloadStreamHandle, SoundHandle, SoundInstanceHandle, SoundTransform,
};
use ruffle_web_common::JsResult;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::{AudioContext, GainNode};

pub struct WebAudioBackend {
    context: AudioContext,
    sounds: Arena<Sound>,
    left_samples: Vec<f32>,
    right_samples: Vec<f32>,
    frame_rate: f64,
    min_sample_rate: u16,
    preload_stream_data: FnvHashMap<PreloadStreamHandle, StreamData>,
    next_stream_id: u32,
}

thread_local! {
    static SOUND_INSTANCES: RefCell<Arena<SoundInstance>> = RefCell::new(Arena::new());
    static NUM_SOUNDS_LOADING: Cell<u32> = Cell::new(0);
}

#[derive(Clone)]
struct StreamData {
    format: swf::SoundFormat,
    audio_data: Vec<u8>,
    num_sample_frames: u32,
    samples_per_block: u32,
    skip_sample_frames: u16,
    adpcm_block_offsets: Vec<usize>,

    /// List of stream segments. Contains the frame they start on and the starting sample.
    /// Guaranteed to be in frame order.
    stream_segments: Vec<(u16, u32)>,

    /// The last frame we received a `StreamSoundBlock` from.
    last_clip_frame: u16,
}

type AudioBufferPtr = Rc<RefCell<web_sys::AudioBuffer>>;

// A sound can be either as a JS AudioBuffer and as a on--the-fly decoded stream using a ScriptProcessorNode.
#[allow(dead_code)]
enum SoundSource {
    // Pre-decoded audio buffer.
    AudioBuffer(AudioBufferPtr),

    // Decode the audio data on the fly from a byte stream.
    Decoder(Vec<u8>),
}

#[allow(dead_code)]
struct Sound {
    format: swf::SoundFormat,
    source: SoundSource,

    /// Number of samples in this audio.
    /// This may be shorter than the actual length of the audio data to allow for seamless looping.
    /// For example, MP3 encoder adds gaps from encoder delay.
    num_sample_frames: u32,

    /// Number of samples to skip encoder delay.
    skip_sample_frames: u16,

    /// If this is a stream sound, the frame numbers and sample counts for each segment of the stream.
    stream_segments: Vec<(u16, u32)>,

    /// The length of the sound data as encoded in the SWF.
    size: u32,
}

type Decoder = Box<dyn Iterator<Item = [i16; 2]>>;

/// An actively playing instance of a sound.
/// This sound can be either an event sound (`StartSound`) or
/// a stream sound (`SoundStreamBlock`).
struct SoundInstance {
    /// Handle to the sound clip.
    #[allow(dead_code)]
    handle: Option<SoundHandle>,

    /// Format of the sound.
    format: swf::SoundFormat,

    /// On web, sounds can be played via different methods:
    /// either decoded on the fly with Decoder, or pre-decoded
    /// and played with and AudioBufferSourceNode.
    instance_type: SoundInstanceType,
}

/// The Drop impl ensures that the sound is stopped and remove from the audio context,
/// and any event listeners are removed.
impl Drop for SoundInstance {
    fn drop(&mut self) {
        if let SoundInstanceType::AudioBuffer(instance) = &self.instance_type {
            let _ = instance.buffer_source_node.set_onended(None);
            let _ = instance.node.disconnect();
        }
    }
}

#[allow(dead_code)]
enum SoundInstanceType {
    Decoder(Decoder),
    AudioBuffer(AudioBufferInstance),
}

/// A sound instance that is playing from an AudioBuffersource node.
struct AudioBufferInstance {
    /// The node that is connected to the output.
    node: web_sys::AudioNode,

    /// The buffer node containing the audio data.
    /// This is often the same as `node`, but will be different
    /// if there is a custom envelope on this sound.
    sound_transform_nodes: SoundTransformNodes,

    /// The audio node with envelopes applied.
    envelope_node: web_sys::AudioNode,

    /// Whether the output of `envelope_node` is mono or stereo.
    envelope_is_stereo: bool,

    /// The buffer node containing the audio data.
    /// This is often the same as `envelope_node`, but will be different
    /// if there is a custom envelope on this sound.
    buffer_source_node: web_sys::AudioBufferSourceNode,
}

impl AudioBufferInstance {
    #[allow(clippy::float_cmp)]
    fn set_transform(&mut self, context: &AudioContext, transform: &SoundTransform) {
        let is_full_transform = transform.left_to_right != 0.0
            || transform.right_to_left != 0.0
            || transform.left_to_left != transform.right_to_right;

        // Lazily instantiate gain nodes, depending on the type of transform.
        match &self.sound_transform_nodes {
            SoundTransformNodes::None => {
                if is_full_transform {
                    let _ = self.create_full_transform(context);
                } else if transform.left_to_left != 1.0 || transform.right_to_right != 1.0 {
                    let _ = self.create_volume_transform(context);
                }
            }
            SoundTransformNodes::Volume { .. } => {
                if is_full_transform {
                    let _ = self.create_full_transform(context);
                }
            }
            SoundTransformNodes::Transform { .. } => (),
        }

        match &self.sound_transform_nodes {
            SoundTransformNodes::None => (),
            SoundTransformNodes::Volume { gain } => {
                // Assumes right_to_right is matching.
                gain.gain().set_value(transform.left_to_left);
            }
            SoundTransformNodes::Transform {
                left_to_left_gain,
                left_to_right_gain,
                right_to_left_gain,
                right_to_right_gain,
            } => {
                left_to_left_gain.gain().set_value(transform.left_to_left);
                left_to_right_gain.gain().set_value(transform.left_to_right);
                right_to_left_gain.gain().set_value(transform.right_to_left);
                right_to_right_gain
                    .gain()
                    .set_value(transform.right_to_right);
            }
        }
    }

    /// Adds a gain node to this sound instance, allowing the volume to be adjusted.
    fn create_volume_transform(
        &mut self,
        context: &AudioContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create the gain node to control the volume.
        let gain = context.create_gain().into_js_result()?;

        // Wire up the nodes.
        // Note that for mono tracks, we want to use channel 0 (left) for both the left and right.
        self.node.disconnect().warn_on_error();
        self.envelope_node.disconnect().warn_on_error();
        self.envelope_node
            .connect_with_audio_node(&gain)
            .into_js_result()?;

        gain.connect_with_audio_node(&context.destination())
            .warn_on_error();

        self.node = gain.clone().into();
        self.sound_transform_nodes = SoundTransformNodes::Volume { gain };
        Ok(())
    }

    /// Adds a bunch of gain nodes to this sound instance, allowing a SoundTransform
    /// to be applied to it.
    fn create_full_transform(
        &mut self,
        context: &AudioContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Split the left and right channels.
        let splitter = context
            .create_channel_splitter_with_number_of_outputs(2)
            .into_js_result()?;

        // Create the envelope gain nodes for the left and right channels.
        let left_to_left_gain = context.create_gain().into_js_result()?;
        let left_to_right_gain = context.create_gain().into_js_result()?;
        let right_to_left_gain = context.create_gain().into_js_result()?;
        let right_to_right_gain = context.create_gain().into_js_result()?;

        let merger: web_sys::AudioNode = context
            .create_channel_merger_with_number_of_inputs(2)
            .into_js_result()?
            .into();

        // Wire up the nodes.
        // Note that for mono tracks, we want to use channel 0 (left) for both the left and right.
        self.node.disconnect().warn_on_error();
        self.envelope_node.disconnect().warn_on_error();
        self.envelope_node
            .connect_with_audio_node(&splitter)
            .into_js_result()?;
        splitter
            .connect_with_audio_node_and_output(&left_to_left_gain, 0)
            .into_js_result()?;
        splitter
            .connect_with_audio_node_and_output(&left_to_right_gain, 0)
            .into_js_result()?;
        splitter
            .connect_with_audio_node_and_output(
                &right_to_left_gain,
                if self.envelope_is_stereo { 1 } else { 0 },
            )
            .into_js_result()?;
        splitter
            .connect_with_audio_node_and_output(
                &right_to_right_gain,
                if self.envelope_is_stereo { 1 } else { 0 },
            )
            .into_js_result()?;

        left_to_left_gain
            .connect_with_audio_node_and_output_and_input(&merger, 0, 0)
            .into_js_result()?;
        left_to_right_gain
            .connect_with_audio_node_and_output_and_input(&merger, 0, 1)
            .into_js_result()?;
        right_to_left_gain
            .connect_with_audio_node_and_output_and_input(&merger, 0, 0)
            .into_js_result()?;
        right_to_right_gain
            .connect_with_audio_node_and_output_and_input(&merger, 0, 1)
            .into_js_result()?;

        merger
            .connect_with_audio_node(&context.destination())
            .warn_on_error();

        self.node = merger;
        self.envelope_is_stereo = true;
        self.sound_transform_nodes = SoundTransformNodes::Transform {
            left_to_left_gain,
            left_to_right_gain,
            right_to_left_gain,
            right_to_right_gain,
        };
        Ok(())
    }
}

/// The gain nodes controlling the sound transform for this sound.
/// Because most sounds will be untransformed, we lazily instantiate
/// this only when necessary to play a transformed sound.
enum SoundTransformNodes {
    /// No transform is applied to this sound.
    None,

    /// This sound has volume applied to it.
    Volume { gain: GainNode },

    /// This sound has a full transform applied to it.
    Transform {
        left_to_left_gain: GainNode,
        left_to_right_gain: GainNode,
        right_to_left_gain: GainNode,
        right_to_right_gain: GainNode,
    },
}

type Error = Box<dyn std::error::Error>;

impl WebAudioBackend {
    pub fn new() -> Result<Self, Error> {
        let context = AudioContext::new().map_err(|_| "Unable to create AudioContext")?;

        // Deduce the minimum sample rate for this browser.
        let mut min_sample_rate = 44100;
        while min_sample_rate > 5512
            && context
                .create_buffer(1, 1, (min_sample_rate >> 1) as f32)
                .is_ok()
        {
            min_sample_rate >>= 1;
        }
        log::info!("Minimum audio buffer sample rate: {}", min_sample_rate);

        Ok(Self {
            context,
            sounds: Arena::new(),
            preload_stream_data: FnvHashMap::default(),
            next_stream_id: 0,
            left_samples: vec![],
            right_samples: vec![],
            frame_rate: 1.0,
            min_sample_rate,
        })
    }

    /// Returns the JavaScript AudioContext.
    pub fn audio_context(&self) -> &AudioContext {
        &self.context
    }

    fn start_sound_internal(
        &mut self,
        handle: SoundHandle,
        settings: Option<&swf::SoundInfo>,
    ) -> Result<SoundInstanceHandle, Error> {
        let sound = self.sounds.get(handle).unwrap();
        let handle = match &sound.source {
            SoundSource::AudioBuffer(audio_buffer) => {
                let audio_buffer = audio_buffer.borrow();
                let node = self.context.create_buffer_source().unwrap();
                node.set_buffer(Some(&*audio_buffer));

                let buffer_source_node = node.clone();

                let sound_sample_rate: f64 = sound.format.sample_rate.into();
                let mut is_stereo = sound.format.is_stereo;
                let node: web_sys::AudioNode = match settings {
                    Some(settings)
                        if sound.skip_sample_frames > 0
                            || settings.num_loops > 1
                            || settings.in_sample.is_some()
                            || settings.out_sample.is_some()
                            || settings.envelope.is_some() =>
                    {
                        // Event sound with non-default parameters.
                        // Note that start/end values are in 44.1kHZ samples regardless of the sound's sample rate.
                        let start_sample_frame = f64::from(settings.in_sample.unwrap_or(0))
                            / 44100.0
                            + f64::from(sound.skip_sample_frames) / sound_sample_rate;
                        node.set_loop(settings.num_loops > 1);
                        node.set_loop_start(start_sample_frame);
                        node.start_with_when_and_grain_offset(0.0, start_sample_frame)
                            .warn_on_error();

                        let current_time = self.context.current_time();

                        // The length of the sound in the swf, or by the script playing it, doesn't
                        // always line up with the actual length of the sound.
                        // Always set a custom end point to make sure we're correct.
                        let end_sample_frame = if let Some(out_sample) = settings.out_sample {
                            f64::from(out_sample) / 44100.0
                        } else {
                            f64::from(sound.num_sample_frames + u32::from(sound.skip_sample_frames))
                                / sound_sample_rate
                        };
                        // `AudioSourceBufferNode.loop` is a bool, so we have to stop the loop at the proper time.
                        // `start_with_when_and_grain_offset_and_grain_duration` unfortunately doesn't work
                        // as you might expect with loops, so we use `stop_with_when` to stop the loop.
                        let total_len =
                            (end_sample_frame - start_sample_frame) * f64::from(settings.num_loops);
                        node.set_loop_end(end_sample_frame);
                        node.stop_with_when(current_time + total_len)
                            .warn_on_error();

                        // For envelopes, we rig the node up to some splitter/gain nodes.
                        if let Some(envelope) = &settings.envelope {
                            is_stereo = true;
                            self.create_sound_envelope(
                                node.into(),
                                envelope,
                                sound.format.is_stereo,
                                current_time,
                            )
                            .unwrap()
                        } else {
                            node.into()
                        }
                    }
                    _ => {
                        // Default event sound or stream.
                        node.start().warn_on_error();
                        node.into()
                    }
                };

                node.connect_with_audio_node(&self.context.destination())
                    .warn_on_error();

                // Create the sound instance and add it to the active instances list.
                let instance = SoundInstance {
                    handle: Some(handle),
                    format: sound.format.clone(),
                    instance_type: SoundInstanceType::AudioBuffer(AudioBufferInstance {
                        envelope_node: node.clone(),
                        envelope_is_stereo: is_stereo,
                        node,
                        buffer_source_node: buffer_source_node.clone(),
                        sound_transform_nodes: SoundTransformNodes::None,
                    }),
                };
                let instance_handle = SOUND_INSTANCES.with(|instances| {
                    let mut instances = instances.borrow_mut();
                    instances.insert(instance)
                });

                // Create the listener to remove the sound when it ends.
                let ended_handler = move || {
                    SOUND_INSTANCES.with(|instances| {
                        let mut instances = instances.borrow_mut();
                        instances.remove(instance_handle)
                    });
                };
                let closure = Closure::once_into_js(Box::new(ended_handler) as Box<dyn FnMut()>);
                // Note that we add the ended event to the AudioBufferSourceNode; an audio envelope adds more nodes
                // in the graph, but these nodes don't fire the ended event.
                let _ = buffer_source_node.set_onended(Some(closure.as_ref().unchecked_ref()));

                instance_handle
            }
            SoundSource::Decoder(audio_data) => {
                let decoder: Decoder = match sound.format.compression {
                    AudioCompression::Adpcm => Box::new(AdpcmDecoder::new(
                        std::io::Cursor::new(audio_data.to_vec()),
                        sound.format.is_stereo,
                        sound.format.sample_rate,
                    )),
                    AudioCompression::Nellymoser => Box::new(NellymoserDecoder::new(
                        std::io::Cursor::new(audio_data.to_vec()),
                        sound.format.sample_rate.into(),
                    )),
                    compression => {
                        return Err(format!("Unimplemented codec: {:?}", compression).into())
                    }
                };

                let decoder: Decoder =
                    if sound.format.sample_rate != self.context.sample_rate() as u16 {
                        Box::new(resample(
                            decoder,
                            sound.format.sample_rate,
                            self.context.sample_rate() as u16,
                        ))
                    } else {
                        decoder
                    };

                let instance = SoundInstance {
                    handle: Some(handle),
                    format: sound.format.clone(),
                    instance_type: SoundInstanceType::Decoder(decoder),
                };
                SOUND_INSTANCES.with(|instances| {
                    let mut instances = instances.borrow_mut();
                    let instance_handle = instances.insert(instance);
                    let script_processor_node = self.context.create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(4096, 0, if sound.format.is_stereo { 2 } else { 1 }).unwrap();
                    let script_node = script_processor_node.clone();
                    let closure = Closure::wrap(Box::new(move |event| {
                            SOUND_INSTANCES.with(|instances| {
                                let mut instances = instances.borrow_mut();
                                let instance = instances.get_mut(instance_handle).unwrap();
                                let complete = WebAudioBackend::update_script_processor(instance, event);
                                if complete {
                                    instances.remove(instance_handle);
                                    script_node.disconnect().unwrap();
                                }
                            })
                        }) as Box<dyn FnMut(web_sys::AudioProcessingEvent)>);
                        script_processor_node.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
                        // TODO: This will leak memory per playing sound. Remember and properly drop the closure.
                        closure.forget();

                    instance_handle
                })
            }
        };
        Ok(handle)
    }

    /// Wires up the envelope for Flash event sounds using `ChannelSplitter`, `Gain`, and `ChannelMerger` nodes.
    fn create_sound_envelope(
        &self,
        node: web_sys::AudioNode,
        envelope: &[swf::SoundEnvelopePoint],
        is_stereo: bool,
        start_time: f64,
    ) -> Result<web_sys::AudioNode, Box<dyn std::error::Error>> {
        // Split the left and right channels.
        let splitter = self
            .context
            .create_channel_splitter_with_number_of_outputs(2)
            .into_js_result()?;

        // Create the envelope gain nodes for the left and right channels.
        let left_gain = self.context.create_gain().into_js_result()?;
        let right_gain = self.context.create_gain().into_js_result()?;

        // Initial volume is clamped to first envelope point.
        if let Some(point) = envelope.get(0) {
            left_gain
                .gain()
                .set_value_at_time(point.left_volume, 0.0)
                .warn_on_error();
            right_gain
                .gain()
                .set_value_at_time(point.right_volume, 0.0)
                .warn_on_error();
        }

        // Add volume lerps for envelope points.
        for point in envelope {
            left_gain
                .gain()
                .linear_ramp_to_value_at_time(
                    point.left_volume,
                    start_time + f64::from(point.sample) / 44100.0,
                )
                .warn_on_error();
            right_gain
                .gain()
                .linear_ramp_to_value_at_time(
                    point.right_volume,
                    start_time + f64::from(point.sample) / 44100.0,
                )
                .warn_on_error();
        }

        // Merge the channels back together.
        let merger: web_sys::AudioNode = self
            .context
            .create_channel_merger_with_number_of_inputs(2)
            .into_js_result()?
            .into();

        // Wire up the nodes.
        node.connect_with_audio_node(&splitter).into_js_result()?;
        splitter
            .connect_with_audio_node_and_output(&left_gain, 0)
            .into_js_result()?;
        // Note that for mono tracks, we want to use channel 0 (left) for both the left and right.
        splitter
            .connect_with_audio_node_and_output(&right_gain, if is_stereo { 1 } else { 0 })
            .into_js_result()?;
        left_gain
            .connect_with_audio_node_and_output_and_input(&merger, 0, 0)
            .into_js_result()?;
        right_gain
            .connect_with_audio_node_and_output_and_input(&merger, 0, 1)
            .into_js_result()?;

        Ok(merger)
    }

    fn decompress_to_audio_buffer(
        &mut self,
        format: &swf::SoundFormat,
        audio_data: &[u8],
        num_sample_frames: u32,
        adpcm_block_offsets: Option<&[usize]>,
    ) -> Result<AudioBufferPtr, Error> {
        if format.compression == AudioCompression::Mp3 {
            return Ok(self.decompress_mp3_to_audio_buffer(format, audio_data, num_sample_frames));
        }

        self.left_samples.clear();
        self.right_samples.clear();

        match format.compression {
            AudioCompression::Uncompressed | AudioCompression::UncompressedUnknownEndian => {
                use byteorder::{LittleEndian, ReadBytesExt};
                let mut audio_data = audio_data;

                let read_sample = |audio_data: &mut &[u8]| {
                    if format.is_16_bit {
                        f32::from(audio_data.read_i16::<LittleEndian>().unwrap_or(0)) / 32767.0
                    } else {
                        f32::from(audio_data.read_u8().unwrap_or(0)) / 128.0 - 1.0
                    }
                };
                while !audio_data.is_empty() {
                    self.left_samples.push(read_sample(&mut audio_data));
                    if format.is_stereo {
                        self.right_samples.push(read_sample(&mut audio_data));
                    }
                }
            }
            AudioCompression::Adpcm => {
                // For stream sounds, the ADPCM header is included in each block,
                // so we must recreate the decoder for each block.
                // Event sounds don't have this issue.
                let full = [0, audio_data.len()];
                let adpcm_block_offsets = adpcm_block_offsets.unwrap_or(&full);
                for block in adpcm_block_offsets.windows(2) {
                    let start = block[0];
                    let end = block[1];
                    let decoder = AdpcmDecoder::new(
                        &audio_data[start..end],
                        format.is_stereo,
                        format.sample_rate,
                    );
                    if format.is_stereo {
                        for frame in decoder {
                            let (l, r) = (frame[0], frame[1]);
                            self.left_samples.push(f32::from(l) / 32767.0);
                            self.right_samples.push(f32::from(r) / 32767.0);
                        }
                    } else {
                        self.left_samples
                            .extend(decoder.map(|n| f32::from(n[0]) / 32767.0));
                    }
                }
            }
            AudioCompression::Nellymoser => {
                let decoder = NellymoserDecoder::new(audio_data, format.sample_rate.into());
                for frame in decoder {
                    let (l, r) = (frame[0], frame[1]);
                    self.left_samples.push(f32::from(l) / 32767.0);
                    self.right_samples.push(f32::from(r) / 32767.0);
                }
            }
            compression => return Err(format!("Unimplemented codec: {:?}", compression).into()),
        }

        // This sucks. Firefox and Safari don't like low sample rates,
        // so manually multiply the samples.
        let sample_rate = if format.sample_rate < self.min_sample_rate {
            let sample_multiplier = self.min_sample_rate / format.sample_rate;
            let mut samples = Vec::with_capacity(self.left_samples.len() * 2);
            for sample in &self.left_samples {
                for _ in 0..sample_multiplier {
                    samples.push(*sample);
                }
            }
            self.left_samples = samples;

            if format.is_stereo {
                let mut samples = Vec::with_capacity(self.right_samples.len() * 2);
                for sample in &self.right_samples {
                    for _ in 0..sample_multiplier {
                        samples.push(*sample);
                    }
                }
                self.right_samples = samples;
            }

            self.min_sample_rate
        } else {
            format.sample_rate
        };

        let num_sample_frames = self.left_samples.len() as u32;
        let audio_buffer = self
            .context
            .create_buffer(
                if format.is_stereo { 2 } else { 1 },
                num_sample_frames,
                f32::from(sample_rate),
            )
            .unwrap();

        copy_to_audio_buffer(
            &audio_buffer,
            Some(&self.left_samples),
            if format.is_stereo {
                Some(&self.right_samples)
            } else {
                None
            },
        );

        Ok(Rc::new(RefCell::new(audio_buffer)))
    }

    fn decompress_mp3_to_audio_buffer(
        &mut self,
        format: &swf::SoundFormat,
        audio_data: &[u8],
        _num_sample_frames: u32,
    ) -> AudioBufferPtr {
        // We use the Web decodeAudioData API to decode MP3 data.
        // TODO: Is it possible we finish loading before the MP3 is decoding?
        let audio_buffer = self
            .context
            .create_buffer(1, 1, self.context.sample_rate())
            .unwrap();
        let audio_buffer = Rc::new(RefCell::new(audio_buffer));

        // Clone the audio data into an ArrayBuffer
        // SAFETY: (compare with the docs for `Uint8Array::view`)
        // - We don't resize WASMs backing buffer before the view is cloned
        // - We don't mutate `data_array`
        // - Since we clone the buffer, its lifetime is correctly disconnected from `audio_data`
        let array_buffer = {
            let data_array = unsafe { js_sys::Uint8Array::view(audio_data) };
            data_array.buffer().slice_with_end(
                data_array.byte_offset(),
                data_array.byte_offset() + data_array.byte_length(),
            )
        };

        NUM_SOUNDS_LOADING.with(|n| n.set(n.get() + 1));

        let _num_channels = if format.is_stereo { 2 } else { 1 };
        let buffer_ptr = Rc::clone(&audio_buffer);
        let success_closure = Closure::wrap(Box::new(move |buffer: web_sys::AudioBuffer| {
            *buffer_ptr.borrow_mut() = buffer;
            NUM_SOUNDS_LOADING.with(|n| n.set(n.get() - 1));
        }) as Box<dyn FnMut(web_sys::AudioBuffer)>);
        let error_closure = Closure::wrap(Box::new(move || {
            log::info!("Error decoding MP3 audio");
            NUM_SOUNDS_LOADING.with(|n| n.set(n.get() - 1));
        }) as Box<dyn FnMut()>);
        let _ = self
            .context
            .decode_audio_data_with_success_callback_and_error_callback(
                &array_buffer,
                success_closure.as_ref().unchecked_ref(),
                error_closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        // TODO: This will leak memory (once per decompressed MP3).
        // Not a huge deal as there are probably not many MP3s in an SWF.
        success_closure.forget();
        error_closure.forget();

        audio_buffer
    }

    fn update_script_processor(
        instance: &mut SoundInstance,
        event: web_sys::AudioProcessingEvent,
    ) -> bool {
        let mut complete = false;
        let mut left_samples = vec![];
        let mut right_samples = vec![];
        if let SoundInstanceType::Decoder(ref mut decoder) = &mut instance.instance_type {
            let output_buffer = event.output_buffer().unwrap();
            let num_frames = output_buffer.length() as usize;

            for _ in 0..num_frames {
                if let Some(frame) = decoder.next() {
                    let (l, r) = (frame[0], frame[1]);
                    left_samples.push(f32::from(l) / 32767.0);
                    if instance.format.is_stereo {
                        right_samples.push(f32::from(r) / 32767.0);
                    }
                } else {
                    complete = true;
                    break;
                }
            }
            copy_to_audio_buffer(
                &output_buffer,
                Some(&left_samples),
                if instance.format.is_stereo {
                    Some(&right_samples)
                } else {
                    None
                },
            );
        }

        complete
    }
}

impl AudioBackend for WebAudioBackend {
    fn set_frame_rate(&mut self, frame_rate: f64) {
        self.frame_rate = frame_rate
    }

    fn register_sound(&mut self, sound: &swf::Sound) -> Result<SoundHandle, Error> {
        // Slice off latency seek for MP3 data.
        let (skip_sample_frames, data) = if sound.format.compression == AudioCompression::Mp3 {
            let skip_sample_frames = u16::from_le_bytes([sound.data[0], sound.data[1]]);
            (skip_sample_frames, &sound.data[2..])
        } else {
            (0, sound.data)
        };

        let sound = Sound {
            format: sound.format.clone(),
            source: SoundSource::AudioBuffer(self.decompress_to_audio_buffer(
                &sound.format,
                data,
                sound.num_samples,
                None,
            )?),
            num_sample_frames: sound.num_samples,
            skip_sample_frames,
            stream_segments: vec![],
            size: data.len() as u32,
        };
        Ok(self.sounds.insert(sound))
    }

    fn preload_sound_stream_head(
        &mut self,
        stream_info: &swf::SoundStreamHead,
    ) -> Option<PreloadStreamHandle> {
        let stream_id = self.next_stream_id;
        self.next_stream_id = self.next_stream_id.wrapping_add(1);
        self.preload_stream_data
            .entry(stream_id)
            .or_insert_with(|| StreamData {
                format: stream_info.stream_format.clone(),
                audio_data: vec![],
                num_sample_frames: 0,
                samples_per_block: stream_info.num_samples_per_block.into(),
                skip_sample_frames: stream_info.latency_seek as u16,
                adpcm_block_offsets: vec![],
                stream_segments: vec![],
                last_clip_frame: 0,
            });
        Some(stream_id)
    }

    fn preload_sound_stream_block(
        &mut self,
        stream_id: PreloadStreamHandle,
        clip_frame: u16,
        audio_data: &[u8],
    ) {
        if let Some(stream) = self.preload_stream_data.get_mut(&stream_id) {
            // Handle gaps in streaming audio. Store the offsets for each stream segment.
            if stream.audio_data.is_empty() || stream.last_clip_frame + 1 != clip_frame {
                let sample_mult = 44100 / stream.format.sample_rate;
                let start_sample = stream.num_sample_frames * u32::from(sample_mult);
                stream.stream_segments.push((clip_frame, start_sample));
            }
            stream.last_clip_frame = clip_frame;

            match stream.format.compression {
                AudioCompression::Uncompressed | AudioCompression::UncompressedUnknownEndian => {
                    let frame_len = if stream.format.is_stereo { 2 } else { 1 }
                        * if stream.format.is_16_bit { 2 } else { 1 };
                    stream.num_sample_frames += (audio_data.len() as u32) / frame_len;
                    stream.audio_data.extend_from_slice(audio_data);
                }
                AudioCompression::Mp3 => {
                    // Sometimes you may get blocks with zero samples; this may be because
                    // previous blocks had more samples than necessary, or because the stream
                    // is stopping (silence).
                    if audio_data.len() >= 4 {
                        let num_sample_frames: u32 =
                            u16::from_le_bytes([audio_data[0], audio_data[1]]).into();
                        stream.num_sample_frames += num_sample_frames;
                        // MP3 streaming data:
                        // First two bytes = number of samples
                        // Second two bytes = 'latency seek' (amount to skip when seeking to this frame)
                        stream.audio_data.extend_from_slice(&audio_data[4..]);
                    }
                }
                AudioCompression::Adpcm => {
                    // For ADPCM data, we must keep track of where each block starts,
                    // so that we read the header in each block.
                    stream.num_sample_frames += stream.samples_per_block;
                    stream.adpcm_block_offsets.push(stream.audio_data.len());
                    stream.audio_data.extend_from_slice(audio_data);
                }
                AudioCompression::Nellymoser => {
                    stream.num_sample_frames += stream.samples_per_block;
                    stream.audio_data.extend_from_slice(audio_data);
                }
                _ => {
                    // TODO: This is a guess and will vary slightly from block to block!
                    stream.num_sample_frames += stream.samples_per_block;
                }
            }
        }
    }

    fn preload_sound_stream_end(&mut self, stream_id: PreloadStreamHandle) -> Option<SoundHandle> {
        let stream_data = self.preload_stream_data.remove(&stream_id);

        if let Some(mut stream) = stream_data {
            if !stream.audio_data.is_empty() {
                if let Ok(audio_buffer) = self.decompress_to_audio_buffer(
                    &stream.format,
                    &stream.audio_data[..],
                    stream.num_sample_frames,
                    if stream.format.compression == AudioCompression::Adpcm {
                        stream.adpcm_block_offsets.push(stream.audio_data.len());
                        Some(&stream.adpcm_block_offsets[..])
                    } else {
                        None
                    },
                ) {
                    let handle = self.sounds.insert(Sound {
                        format: stream.format,
                        source: SoundSource::AudioBuffer(audio_buffer),
                        num_sample_frames: stream.num_sample_frames,
                        skip_sample_frames: stream.skip_sample_frames,
                        stream_segments: stream.stream_segments,
                        size: stream.audio_data.len() as u32,
                    });
                    return Some(handle);
                }
            }
        }

        None
    }

    fn start_sound(
        &mut self,
        sound: SoundHandle,
        sound_info: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, Error> {
        let handle = self.start_sound_internal(sound, Some(sound_info))?;
        Ok(handle)
    }

    fn start_stream(
        &mut self,
        stream_handle: Option<SoundHandle>,
        clip_frame: u16,
        _clip_data: ruffle_core::tag_utils::SwfSlice,
        _stream_info: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, Error> {
        if let Some(stream) = stream_handle {
            let mut sound_info = None;
            if clip_frame > 1 {
                if let Some(sound) = self.sounds.get(stream) {
                    // Figure out the frame and sample where this stream segment first starts.
                    let start_pos = match sound
                        .stream_segments
                        .binary_search_by(|(f, _)| f.cmp(&clip_frame))
                    {
                        Ok(i) => sound.stream_segments[i].1,
                        Err(i) => {
                            if i > 0 {
                                let (segment_frame, segment_sample) = sound.stream_segments[i - 1];
                                let frames_skipped = clip_frame.saturating_sub(segment_frame);
                                let samples_per_frame = 44100.0 / self.frame_rate;
                                segment_sample
                                    + u32::from(frames_skipped) * (samples_per_frame as u32)
                            } else {
                                0
                            }
                        }
                    };
                    sound_info = Some(swf::SoundInfo {
                        event: swf::SoundEvent::Event,
                        in_sample: Some(start_pos),
                        out_sample: None,
                        num_loops: 1,
                        envelope: None,
                    });
                }
            }
            let instance = self.start_sound_internal(stream, sound_info.as_ref())?;
            Ok(instance)
        } else {
            let msg = format!("Missing stream for sound ID {:?}", stream_handle);
            log::error!("{}", msg);
            Err(msg.into())
        }
    }

    fn stop_sound(&mut self, sound: SoundInstanceHandle) {
        SOUND_INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            instances.remove(sound);
        })
    }

    fn is_loading_complete(&self) -> bool {
        NUM_SOUNDS_LOADING.with(|n| n.get() == 0)
    }

    fn play(&mut self) {
        // Allow audio to start playing after a user gesture.
        let _ = self.context.resume();
    }

    fn pause(&mut self) {
        // Suspend audio to be resumed later.
        let _ = self.context.suspend();
    }

    fn stop_all_sounds(&mut self) {
        SOUND_INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            // This is a workaround for a bug in generational-arena:
            // Arena::clear does not properly bump the generational index, allowing for stale references
            // to continue to work (this caused #1315). Arena::remove will force a generation bump.
            // See https://github.com/fitzgen/generational-arena/issues/30
            if let Some((i, _)) = instances.iter().next() {
                instances.remove(i);
            }
            instances.clear();
        })
    }

    fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<u32> {
        SOUND_INSTANCES.with(|instances| {
            let instances = instances.borrow();
            // TODO: Return actual position
            instances.get(instance).map(|_| 0)
        })
    }

    fn get_sound_duration(&self, sound: SoundHandle) -> Option<u32> {
        if let Some(sound) = self.sounds.get(sound) {
            // AS duration does not subtract `skip_sample_frames`.
            let num_sample_frames: f64 = sound.num_sample_frames.into();
            let sample_rate: f64 = sound.format.sample_rate.into();
            let ms = (num_sample_frames * 1000.0 / sample_rate).round();
            Some(ms as u32)
        } else {
            None
        }
    }

    fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
        self.sounds.get(sound).map(|s| s.size)
    }

    fn set_sound_transform(&mut self, instance: SoundInstanceHandle, transform: SoundTransform) {
        SOUND_INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            if let Some(instance) = instances.get_mut(instance) {
                if let SoundInstanceType::AudioBuffer(sound) = &mut instance.instance_type {
                    sound.set_transform(&self.context, &transform);
                }
            }
        })
    }
}

#[wasm_bindgen(raw_module = "./ruffle-imports.js")]
extern "C" {
    /// Imported JS method to copy data into an `AudioBuffer`.
    /// We'd prefer to use `AudioBuffer.copyToChannel`, but this isn't supported
    /// on Safari.
    #[wasm_bindgen(js_name = "copyToAudioBuffer")]
    fn copy_to_audio_buffer(
        audio_buffer: &web_sys::AudioBuffer,
        left_data: Option<&[f32]>,
        right_data: Option<&[f32]>,
    );
}

// Janky resmapling code.
// TODO: Clean this up.
#[allow(unused_assignments)]
fn resample(
    mut input: impl Iterator<Item = [i16; 2]>,
    input_sample_rate: u16,
    output_sample_rate: u16,
) -> impl Iterator<Item = [i16; 2]> {
    let (mut left0, mut right0) = if let Some(frame) = input.next() {
        (Some(frame[0]), Some(frame[1]))
    } else {
        (None, None)
    };
    let (mut left1, mut right1) = if let Some(frame) = input.next() {
        (Some(frame[0]), Some(frame[1]))
    } else {
        (None, None)
    };
    let (mut left, mut right) = (left0.unwrap(), right0.unwrap());
    let dt_input = 1.0 / f64::from(input_sample_rate);
    let dt_output = 1.0 / f64::from(output_sample_rate);
    let mut t = 0.0;
    std::iter::from_fn(move || {
        if let (Some(l0), Some(r0), Some(l1), Some(r1)) = (left0, right0, left1, right1) {
            let a = t / dt_input;
            let l0: f64 = l0.into();
            let l1: f64 = l1.into();
            let r0: f64 = r0.into();
            let r1: f64 = r1.into();
            left = (l0 + (l1 - l0) * a) as i16;
            right = (r0 + (r1 - r0) * a) as i16;
            t += dt_output;
            while t >= dt_input {
                t -= dt_input;
                left0 = left1;
                right0 = right1;
                if let Some(frame) = input.next() {
                    left1 = Some(frame[0]);
                    right1 = Some(frame[1]);
                } else {
                    left1 = None;
                    right1 = None;
                }
            }
            Some([left, right])
        } else {
            None
        }
    })
}
