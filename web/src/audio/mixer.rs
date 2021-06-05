use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, SoundHandle, SoundInstanceHandle, SoundTransform,
};
use ruffle_core::impl_audio_mixer_backend;
use ruffle_web_common::JsResult;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::{AudioContext, AudioProcessingEvent, ScriptProcessorNode};

type Error = Box<dyn std::error::Error>;

#[allow(dead_code)]
pub struct WebAudioMixerBackend {
    mixer: AudioMixer,
    context: AudioContext,
    script_processor: ScriptProcessorNode,
    on_audio_process: Closure<dyn FnMut(AudioProcessingEvent)>,
}

impl WebAudioMixerBackend {
    pub fn new() -> Result<Self, Error> {
        let context = AudioContext::new().map_err(|_| "Unable to create AudioContext")?;
        let mixer = AudioMixer::new(2, context.sample_rate() as u32);

        let script_processor = context.create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(0, 0, 2).map_err(|_| "Unable to create ScriptProcessorNode")?;

        let mixer_proxy = mixer.proxy();
        let buffer_samples = 2 * script_processor.buffer_size() as usize;
        let mut out_data = Vec::new();
        out_data.resize(buffer_samples, 0.0);
        let on_audio_process = move |event: AudioProcessingEvent| {
            if let Ok(output_buffer) = event.output_buffer() {
                mixer_proxy.mix(&mut out_data);
                copy_to_audio_buffer_interleaved(&output_buffer, &out_data);
            }
        };
        let on_audio_process =
            Closure::wrap(Box::new(on_audio_process) as Box<dyn FnMut(AudioProcessingEvent)>);
        script_processor.set_onaudioprocess(Some(on_audio_process.as_ref().unchecked_ref()));
        script_processor
            .connect_with_audio_node(&context.destination())
            .warn_on_error();

        Ok(Self {
            mixer,
            context,
            script_processor,
            on_audio_process,
        })
    }

    /// Returns the JavaScript AudioContext.
    pub fn audio_context(&self) -> &AudioContext {
        &self.context
    }
}

impl AudioBackend for WebAudioMixerBackend {
    impl_audio_mixer_backend!(mixer);

    fn play(&mut self) {
        let _ = self.context.resume();
    }

    fn pause(&mut self) {
        let _ = self.context.suspend();
    }
}

impl Drop for WebAudioMixerBackend {
    fn drop(&mut self) {
        self.script_processor.set_onaudioprocess(None);
        let _ = self.context.close();
    }
}

#[wasm_bindgen(raw_module = "./ruffle-imports.js")]
extern "C" {
    /// Imported JS method to copy interleaved audio data into an `AudioBuffer`.
    #[wasm_bindgen(js_name = "copyToAudioBufferInterleaved")]
    fn copy_to_audio_buffer_interleaved(
        audio_buffer: &web_sys::AudioBuffer,
        interleaved_data: &[f32],
    );
}
