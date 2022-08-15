use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, AudioMixerProxy, SoundHandle, SoundInstanceHandle,
    SoundTransform,
};
use ruffle_core::impl_audio_mixer_backend;
use ruffle_web_common::JsResult;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::AudioContext;

type Error = Box<dyn std::error::Error>;

#[allow(dead_code)]
pub struct WebAudioBackend {
    mixer: AudioMixer,
    context: AudioContext,
    buffers: Vec<Arc<RwLock<Buffer>>>,
    time: Arc<RwLock<f64>>,
    position_resolution: Duration,
}

impl WebAudioBackend {
    const BUFFER_SIZE: u32 = 4096;

    pub fn new() -> Result<Self, Error> {
        let context = AudioContext::new().into_js_result()?;
        let sample_rate = context.sample_rate();
        let mut audio = Self {
            context,
            mixer: AudioMixer::new(2, sample_rate as u32),
            buffers: Vec::with_capacity(2),
            time: Arc::new(RwLock::new(0.0)),
            position_resolution: Duration::from_secs_f64(
                f64::from(Self::BUFFER_SIZE) / f64::from(sample_rate),
            ),
        };

        // Create and start the audio buffers.
        // These buffers ping-pong as the audio stream plays.
        for _ in 0..2 {
            let buffer = Buffer::new(&audio)?;
            let _ = buffer.write().unwrap().play();
            audio.buffers.push(buffer);
        }

        Ok(audio)
    }

    /// Returns the JavaScript AudioContext.
    pub fn audio_context(&self) -> &AudioContext {
        &self.context
    }
}

impl AudioBackend for WebAudioBackend {
    impl_audio_mixer_backend!(mixer);

    fn play(&mut self) {
        let _ = self.context.resume();
    }

    fn pause(&mut self) {
        let _ = self.context.suspend();
    }

    fn position_resolution(&self) -> Option<Duration> {
        Some(self.position_resolution)
    }
}

impl Drop for WebAudioBackend {
    fn drop(&mut self) {
        let _ = self.context.close();
    }
}

struct Buffer {
    context: AudioContext,
    mixer_proxy: AudioMixerProxy,
    audio_buffer: Vec<f32>,
    js_buffer: web_sys::AudioBuffer,
    audio_node: Option<web_sys::AudioBufferSourceNode>,
    on_ended_handler: Closure<dyn FnMut()>,
    time: Arc<RwLock<f64>>,
    buffer_timestep: f64,
}

impl Buffer {
    fn new(audio: &WebAudioBackend) -> Result<Arc<RwLock<Self>>, Error> {
        let sample_rate = audio.context.sample_rate();
        let buffer = Arc::new(RwLock::new(Buffer {
            context: audio.context.clone(),
            mixer_proxy: audio.mixer.proxy(),
            audio_node: None,
            audio_buffer: vec![0.0; 2 * WebAudioBackend::BUFFER_SIZE as usize],
            js_buffer: audio
                .context
                .create_buffer(2, WebAudioBackend::BUFFER_SIZE, sample_rate)
                .into_js_result()?,
            on_ended_handler: Closure::new(|| {}),
            time: audio.time.clone(),
            buffer_timestep: f64::from(WebAudioBackend::BUFFER_SIZE) / f64::from(sample_rate),
        }));

        // Swap in the onended handler.
        let buffer_handle = buffer.clone();
        buffer.write().unwrap().on_ended_handler = Closure::wrap(Box::new(move || {
            // Refill and schedule the buffer for playback.
            let _ = buffer_handle.write().unwrap().play();
        }) as Box<dyn FnMut()>);

        Ok(buffer)
    }

    fn play(&mut self) -> Result<(), Error> {
        // Mix new audio into the output buffer and copy to JS.
        self.mixer_proxy.mix(&mut self.audio_buffer);
        copy_to_audio_buffer_interleaved(&self.js_buffer, &self.audio_buffer);

        // Create the audio node to play back the audio buffer.
        let audio_node = self.context.create_buffer_source().into_js_result()?;
        audio_node.set_buffer(Some(&self.js_buffer));
        audio_node
            .connect_with_audio_node(&self.context.destination())
            .into_js_result()?;
        audio_node.set_onended(Some(self.on_ended_handler.as_ref().unchecked_ref()));

        // Sanity: ensure our player time is not in the past. This can happen due to underruns.
        let mut time = self.time.write().unwrap();
        *time = f64::max(*time, self.context.current_time());

        // Schedule this buffer for playback and advance the player time.
        audio_node.start_with_when(*time).into_js_result()?;
        *time += self.buffer_timestep;

        self.audio_node = Some(audio_node);
        Ok(())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if let Some(audio_node) = self.audio_node.take() {
            audio_node.set_onended(None);
        }
    }
}

#[wasm_bindgen(raw_module = "./ruffle-imports.js")]
extern "C" {
    // Imported JS method to copy interleaved audio data into an `AudioBuffer`.
    #[wasm_bindgen(js_name = "copyToAudioBufferInterleaved")]
    fn copy_to_audio_buffer_interleaved(
        audio_buffer: &web_sys::AudioBuffer,
        interleaved_data: &[f32],
    );
}
