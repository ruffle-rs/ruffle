use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, AudioMixerProxy, DecodeError, RegisterError, SoundHandle,
    SoundInstanceHandle, SoundStreamInfo, SoundTransform,
};
use ruffle_core::impl_audio_mixer_backend;
use ruffle_web_common::JsResult;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::Registry;
use tracing_wasm::WASMLayer;
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioScheduledSourceNode};

#[allow(dead_code)]
pub struct WebAudioBackend {
    mixer: AudioMixer,
    context: AudioContext,
    /// The current length of both buffers, in frames (pairs of left/right samples).
    buffer_size: Rc<Cell<u32>>,
    buffers: Vec<Rc<RefCell<Buffer>>>,
    /// When the last submitted buffer is expected to play out completely, in seconds.
    time: Rc<Cell<f64>>,
    /// For how many seconds were we able to continuously fill the next buffer "at a sufficiently early time".
    probation_elapsed: Rc<Cell<f32>>,
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
}

impl WebAudioBackend {
    /// These govern the adaptive buffer size algorithm, all are in number of frames (pairs of samples).
    /// They must all be integer powers of 2 (due to how the algorithm works).
    const INITIAL_BUFFER_SIZE: u32 = 2048; // 46.44 ms at 44.1 kHz
    const MIN_BUFFER_SIZE: u32 = 1024; // 23.22 ms at 44.1 kHz
    const MAX_BUFFER_SIZE: u32 = 16384; // 371.52 ms at 44.1 kHz
    /// Buffer size will not be increased until this many seconds have elapsed after startup,
    /// to account for any initialization (shape tessellation, WASM JIT, etc.) hitches.
    const WARMUP_PERIOD: f32 = 2.0;

    /// For how long we need to fill every single buffer "quickly enough" in order to decrease buffer size.
    /// Measured in seconds. A higher value is more conservative.
    const PROBATION_LENGTH: f32 = 10.0;
    /// The limit of playout ratio (progress) when filling the next buffer, under which it is
    /// considered "quick". Must be in 0..1, and less than `0.5 * NORMAL_PROGRESS_RANGE_MAX`.
    const NORMAL_PROGRESS_RANGE_MIN: f64 = 0.25;
    /// The limit of playout ratio (progress) when filling the next buffer, over which buffer size
    /// is increased immediately. Must be in 0..1, and greater than `2 * NORMAL_PROGRESS_RANGE_MIN`.
    const NORMAL_PROGRESS_RANGE_MAX: f64 = 0.75;

    pub fn new(log_subscriber: Arc<Layered<WASMLayer, Registry>>) -> Result<Self, JsError> {
        let context = AudioContext::new().into_js_result()?;
        let sample_rate = context.sample_rate();
        let mut audio = Self {
            context,
            mixer: AudioMixer::new(2, sample_rate as u32),
            buffer_size: Rc::new(Cell::new(Self::INITIAL_BUFFER_SIZE)),
            buffers: Vec::with_capacity(2),
            time: Rc::new(Cell::new(0.0)),
            probation_elapsed: Rc::new(Cell::new(0.0)),
            log_subscriber,
        };

        // Create and start the audio buffers.
        // These buffers ping-pong as the audio stream plays.
        for _ in 0..2 {
            let buffer = Buffer::new(&audio)?;
            buffer.borrow_mut().play()?;
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
        Some(Duration::from_secs_f64(
            f64::from(self.buffer_size.get()) / f64::from(self.context.sample_rate()),
        ))
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
    buffer_size: Rc<Cell<u32>>,
    audio_buffer: Vec<f32>,
    js_buffer: web_sys::AudioBuffer,
    audio_node: Option<web_sys::AudioBufferSourceNode>,
    on_ended_handler: Closure<dyn FnMut()>,
    time: Rc<Cell<f64>>,
    probation_elapsed: Rc<Cell<f32>>,
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
}

impl Buffer {
    fn new(audio: &WebAudioBackend) -> Result<Rc<RefCell<Self>>, JsError> {
        let sample_rate = audio.context.sample_rate();
        let buffer = Rc::new(RefCell::new(Self {
            context: audio.context.clone(),
            mixer_proxy: audio.mixer.proxy(),
            buffer_size: audio.buffer_size.clone(),
            audio_buffer: vec![0.0; 2 * WebAudioBackend::INITIAL_BUFFER_SIZE as usize],
            js_buffer: audio
                .context
                .create_buffer(2, WebAudioBackend::INITIAL_BUFFER_SIZE, sample_rate)
                .into_js_result()?,
            audio_node: None,
            on_ended_handler: Closure::new(|| {}),
            time: audio.time.clone(),
            probation_elapsed: audio.probation_elapsed.clone(),
            log_subscriber: audio.log_subscriber.clone(),
        }));

        // Swap in the onended handler.
        let buffer_handle = buffer.clone();
        buffer.borrow_mut().on_ended_handler = Closure::new(move || {
            // Refill and schedule the buffer for playback.
            let _ = buffer_handle.borrow_mut().play();
        });

        Ok(buffer)
    }

    fn play(&mut self) -> Result<(), JsError> {
        let _subscriber = tracing::subscriber::set_default(self.log_subscriber.clone());

        let time_left = self.time.get() - self.context.current_time();
        let mut buffer_timestep =
            f64::from(self.buffer_size.get()) / f64::from(self.context.sample_rate());

        // How far along the other buffer is in playing out right now:
        //  ~0: it has just started playing, we are well within time
        // 0.25 .. 0.75: "optimal range"
        //  ~1: we are just barely keeping up with feeding the output
        //  >1: we are falling behind, audio stutters
        let progress = (buffer_timestep - time_left) / buffer_timestep;
        tracing::trace!(
            "Audio buffer progress when filling the next one: {}%",
            progress * 100.0
        );

        if progress < WebAudioBackend::NORMAL_PROGRESS_RANGE_MIN {
            // This fill is considered quick, let's count it.
            self.probation_elapsed
                .set(self.probation_elapsed.get() + buffer_timestep as f32);
        } else if progress < WebAudioBackend::NORMAL_PROGRESS_RANGE_MAX {
            // This fill is in the "normal" range, only resetting the probation time.
            self.probation_elapsed.set(0.0);
        } else {
            // This fill is considered slow (maybe even too slow), increasing the buffer size.
            self.probation_elapsed.set(0.0);
            if progress >= 1.0 {
                tracing::debug!("Audio underrun detected!");
            }
            if self.time.get() as f32 > WebAudioBackend::WARMUP_PERIOD {
                if self.buffer_size.get() < WebAudioBackend::MAX_BUFFER_SIZE {
                    self.buffer_size.set(self.buffer_size.get() * 2);
                    tracing::debug!(
                        "Increased audio buffer size to {} frames",
                        self.buffer_size.get()
                    );
                } else {
                    tracing::debug!("Not increasing audio buffer size, already at max size");
                }
            } else {
                tracing::debug!(
                    "Not increasing audio buffer size, still in warmup period (at {} of {} sec)",
                    self.time.get(),
                    WebAudioBackend::WARMUP_PERIOD
                );
            }
        }

        // If enough quick fills happened, we decrease the buffer size.
        if self.probation_elapsed.get() > WebAudioBackend::PROBATION_LENGTH
            && self.buffer_size.get() > WebAudioBackend::MIN_BUFFER_SIZE
        {
            self.buffer_size.set(self.buffer_size.get() / 2);
            tracing::debug!(
                "Decreased audio buffer size to {} frames",
                self.buffer_size.get()
            );
            self.probation_elapsed.set(0.0);
        }

        // In case buffer_size changed above (or in the latest call in the other instance),
        // we need to recaulculate/recreate/resize a couple of things that depend on it.
        if self.js_buffer.length() != self.buffer_size.get() {
            tracing::trace!("Recreating JS side buffer with new length");
            buffer_timestep =
                f64::from(self.buffer_size.get()) / f64::from(self.context.sample_rate());
            self.js_buffer = self
                .context
                .create_buffer(2, self.buffer_size.get(), self.context.sample_rate())
                .into_js_result()?;
            self.audio_buffer
                .resize(2 * self.buffer_size.get() as usize, 0.0);
        }

        // Mix new audio into the output buffer and copy to JS.
        self.mixer_proxy.mix(&mut self.audio_buffer);
        copy_to_audio_buffer_interleaved(&self.js_buffer, &self.audio_buffer);

        // Create the audio node to play back the audio buffer.
        let audio_node = self.context.create_buffer_source().into_js_result()?;
        audio_node.set_buffer(Some(&self.js_buffer));
        audio_node
            .connect_with_audio_node(&self.context.destination())
            .into_js_result()?;
        let scheduled: &AudioScheduledSourceNode = &audio_node;
        scheduled.set_onended(Some(self.on_ended_handler.as_ref().unchecked_ref()));

        // Sanity: ensure our player time is not in the past. This can happen due to underruns.
        self.time
            .set(f64::max(self.time.get(), self.context.current_time()));

        // Schedule this buffer for playback and advance the player time.
        audio_node
            .start_with_when(self.time.get())
            .into_js_result()?;
        self.time.set(self.time.get() + buffer_timestep);

        self.audio_node = Some(audio_node);
        Ok(())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if let Some(audio_node) = self.audio_node.take() {
            let scheduled: &AudioScheduledSourceNode = &audio_node;
            scheduled.set_onended(None);
        }
    }
}

#[wasm_bindgen(raw_module = "./ruffle-imports")]
extern "C" {
    // Imported JS method to copy interleaved audio data into an `AudioBuffer`.
    #[wasm_bindgen(js_name = "copyToAudioBufferInterleaved")]
    fn copy_to_audio_buffer_interleaved(
        audio_buffer: &web_sys::AudioBuffer,
        interleaved_data: &[f32],
    );
}
