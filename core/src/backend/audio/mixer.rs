use super::decoders::{
    self, AdpcmDecoder, Decoder, G711ALawDecoder, G711MuLawDecoder, PcmDecoder, SeekableDecoder,
};
use super::{SampleQueue, SoundHandle, SoundInstanceHandle, SoundStreamInfo, SoundTransform};
use crate::backend::audio::{DecodeError, RegisterError};
use crate::tag_utils::SwfSlice;
use ruffle_common::buffer::Substream;
use ruffle_common::duration::FloatDuration;
use slotmap::{SecondaryMap, SlotMap};
use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::{Arc, Mutex, RwLock};
use swf::AudioCompression;

/// Holds the last 2048 output audio frames. Frames can be written to it one by
/// one, and the last completely filled 1024-wide window can be read from it.
struct CircBuf {
    pub samples: [[f32; 2]; 2048],
    pub pos: usize,
}

impl CircBuf {
    /// Creates an empty circular buffer.
    pub fn new() -> Self {
        Self {
            samples: [[0.0; 2]; 2048],
            pos: 0,
        }
    }

    /// Writes a value into the buffer, pushing the write position forward.
    pub fn push(&mut self, sample: [f32; 2]) {
        self.samples[self.pos] = sample;
        self.pos = (self.pos + 1) % 2048;
    }

    /// Returns one half of the inner buffer, the one that is not currently
    /// being written to.
    pub fn get(&self) -> &[[f32; 2]; 1024] {
        if self.pos < 1024 {
            self.samples[1024..2048]
                .try_into()
                .expect("Length is 1024, cast is infallible")
        } else {
            self.samples[0..1024]
                .try_into()
                .expect("Length is 1024, cast is infallible")
        }
    }
}

/// An audio mixer for a Flash movie.
///
/// `AudioMixer` manages the audio state for a Flash movie. This can be used by any backend that
/// can output a raw audio stream.
///
/// An `AudioBackend` can forward audio events to the `AudioMixer`, and it will track the state of
// all sounds and mix the audio into an output buffer audio stream.
pub struct AudioMixer {
    /// The currently registered sounds.
    sounds: SlotMap<SoundHandle, Sound>,

    /// The sounds decoded so far by `extract_sound`.
    extracted_sounds: SecondaryMap<SoundHandle, ExtractedSound>,

    /// The list of actively playing sound instances.
    sound_instances: Arc<Mutex<SlotMap<SoundInstanceHandle, SoundInstance>>>,

    /// The master volume of the audio from [0.0, 1.0].
    volume: Arc<RwLock<f32>>,

    /// The number of channels in the output stream. Must be 1 or 2.
    num_output_channels: u8,

    /// The sample rate of the output stream in Hz.
    output_sample_rate: u32,

    /// The last two windows of output samples.
    output_memory: Arc<RwLock<CircBuf>>,
}

/// An audio stream.
trait Stream: dasp::signal::Signal<Frame = [i16; 2]> + Send + Sync {
    /// The position of this stream in sample frames.
    ///
    /// For infinite streams, this will be the number of sample frames since the start of the
    /// stream, starting from 0.
    /// For finite streams, this will be the sample position in the underlying audio data. This may
    /// not start from 0 if this sound did not start playing from the beginning.
    fn source_position(&self) -> u32;

    /// The sample rate of the underlying audio source of this stream. For example, this will return
    /// 22050 when playing a 22KHz audio file, even if the output rate is 44KHz.
    fn source_sample_rate(&self) -> u16;
}

/// A stream that wraps a `Decoder`.
struct DecoderStream<D> {
    decoder: D,
    position: u32,
    is_exhausted: bool,
}

impl<D> DecoderStream<D> {
    /// Creates a `DecoderStream` using the given decoder as a source.
    fn new(decoder: D) -> Self {
        Self {
            decoder,
            position: 0,
            is_exhausted: false,
        }
    }
}

impl<D: Decoder> Stream for DecoderStream<D> {
    #[inline]
    fn source_position(&self) -> u32 {
        self.position
    }

    #[inline]
    fn source_sample_rate(&self) -> u16 {
        self.decoder.sample_rate()
    }
}

impl<D: Decoder> dasp::signal::Signal for DecoderStream<D> {
    type Frame = [i16; 2];

    #[inline]
    fn next(&mut self) -> [i16; 2] {
        if let Some(frame) = self.decoder.next() {
            self.position += 1;
            frame
        } else {
            self.is_exhausted = true;
            Default::default()
        }
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.is_exhausted
    }
}

/// A stream that plays the sample frames of a `SampleQueue` as they are queued.
struct SampleQueueStream {
    queue: SampleQueue,

    /// Sample frames taken from the queue, still to be played.
    buffer: VecDeque<[i16; 2]>,

    /// The number of sample frames played so far.
    position: u32,

    /// Silent sample frames to play before checking the queue again, after
    /// finding it empty.
    starved_frames: u16,

    is_exhausted: bool,
}

impl SampleQueueStream {
    /// How often to check an empty queue for samples, in sample frames.
    const STARVED_RETRY_FRAMES: u16 = 256;

    fn new(queue: SampleQueue) -> Self {
        Self {
            queue,
            buffer: VecDeque::new(),
            position: 0,
            starved_frames: 0,
            is_exhausted: false,
        }
    }
}

impl Stream for SampleQueueStream {
    #[inline]
    fn source_position(&self) -> u32 {
        self.position
    }

    #[inline]
    fn source_sample_rate(&self) -> u16 {
        SampleQueue::SAMPLE_RATE
    }
}

impl dasp::signal::Signal for SampleQueueStream {
    type Frame = [i16; 2];

    #[inline]
    fn next(&mut self) -> [i16; 2] {
        if self.buffer.is_empty() && self.starved_frames == 0 {
            let ended = self.queue.take(&mut self.buffer);
            self.is_exhausted = ended && self.buffer.is_empty();
            // Before the first samples, keep checking: they are about to be
            // queued, and waiting would delay the start of the sound.
            if self.buffer.is_empty() && self.position > 0 {
                self.starved_frames = Self::STARVED_RETRY_FRAMES;
            }
        }
        if let Some(frame) = self.buffer.pop_front() {
            self.position = self.position.saturating_add(1);
            frame
        } else {
            // No samples yet: play silence until more are queued.
            self.starved_frames = self.starved_frames.saturating_sub(1);
            Default::default()
        }
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.is_exhausted
    }
}

/// Contains the data and metadata for a sound in an SWF file.
///
/// A sound is defined by the `DefineSound` SWF tags and contains the audio data for the sound.
struct Sound {
    /// The format of the sound, including sample rate and compression format.
    format: swf::SoundFormat,

    /// The audio data of this sound.
    ///
    /// This will be compressed in the format indicated by `format.compression`.
    data: Arc<[u8]>,

    /// Number of samples in this audio.
    /// This does not include `skip_sample_frames`.
    num_sample_frames: u32,

    /// The number of sample frames to skip to reach the start of the audio.
    ///
    /// MP3 encoders introduce some silence at the beginning of the audio due to encoder delay.
    /// `skip_sample_frames` indicates how many sample frames to skip to bypass the delay.
    /// This is `0` unless `format.compression` is `AudioCompression::Mp3`.
    skip_sample_frames: u16,

    /// The number of silent sample frames that Flash decodes before the
    /// audio, and our decoder does not: the encoder's info frame of an MP3
    /// file. These count towards the duration and are extracted.
    lead_in_sample_frames: u32,

    /// Whether `num_sample_frames` is exact, as declared by an SWF, rather
    /// than estimated from an MP3 file.
    num_sample_frames_is_exact: bool,
}

/// A sound being decoded by `AudioMixer::extract_sound`. Sample frames are
/// at `AudioMixer::EXTRACT_SAMPLE_RATE`.
struct ExtractedSound {
    /// The sample frames of silence the sound starts with (see
    /// `Sound::lead_in_sample_frames`).
    lead_in: usize,

    /// The length of the sound in sample frames, if it is known exactly.
    /// Decoding stops there, before any padding the codec adds.
    length: Option<usize>,

    /// The stream decoding the sound, until it ends.
    stream: Option<Box<dyn Stream>>,

    /// The sample frame the stream is at.
    position: usize,

    /// The last sample frames decoded, up to `WINDOW_LENGTH`, ending at
    /// `position`. Stepping back within them, as reverse playback does,
    /// doesn't decode the sound from the start again.
    window: VecDeque<[i16; 2]>,
}

impl ExtractedSound {
    /// How many of the last sample frames decoded are kept (2 seconds).
    const WINDOW_LENGTH: usize = 2 * AudioMixer::EXTRACT_SAMPLE_RATE as usize;

    /// The sample frame that `window` starts at.
    fn window_start(&self) -> usize {
        self.position - self.window.len()
    }

    /// Decodes the next sample frame, or returns `None` at the end.
    fn next_frame(&mut self) -> Option<[i16; 2]> {
        if self.length.is_some_and(|length| self.position >= length) {
            return None;
        }
        let frame = if self.position < self.lead_in {
            [0, 0]
        } else {
            let stream = self.stream.as_mut()?;
            let frame = stream.next();
            if stream.is_exhausted() {
                self.stream = None;
                return None;
            }
            frame
        };
        self.position += 1;
        if self.window.len() == Self::WINDOW_LENGTH {
            self.window.pop_front();
        }
        self.window.push_back(frame);
        Some(frame)
    }
}

/// An actively playing instance of a sound.
/// This sound can be either an event sound (`StartSound`) or
/// a stream sound (`SoundStreamBlock`).
/// The audio thread will iterate through all `SoundInstance`s
/// to fill the audio buffer.
struct SoundInstance {
    /// The handle the sound definition inside `sounds`.
    /// `None` if this is a stream sound.
    #[expect(dead_code)]
    handle: Option<SoundHandle>,

    /// The audio stream. Call `next()` to yield sample frames.
    stream: Box<dyn Stream>,

    /// Flag indicating whether this sound is still playing.
    /// If this flag is false, the sound will be cleaned up during the
    /// next loop of the sound thread.
    active: bool,

    /// The transform for the left channel of this sound instance.
    left_transform: [f32; 2],

    /// The transform for the right channel of this sound instance.
    right_transform: [f32; 2],

    /// Stores the per-channel "peak amplitude" (volume) of this sound
    /// over the last completely mixed 1024-frame long window.
    /// Updated whenever a new buffer is filled completely.
    peak: [f32; 2],

    /// Accumulates the per-channel minimum and maximum sample values
    /// (respectively) of this sound over the buffer currently being
    /// mixed. Used to compute `peak`, and is reset after every time.
    range: ([f32; 2], [f32; 2]),
}

impl SoundInstance {
    /// Creates a new `SoundInstance` from a `Stream`, with a SoundHandle.
    fn new_sound(handle: SoundHandle, stream: Box<dyn Stream>) -> Self {
        SoundInstance {
            handle: Some(handle),
            stream,
            active: true,
            left_transform: [1.0, 0.0],
            right_transform: [0.0, 1.0],
            peak: [0.0, 0.0],
            range: ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]),
        }
    }

    /// Creates a new `SoundInstance` from a `Stream`, for stream sounds.
    ///
    /// Substream-backed sounds also use this.
    fn new_stream(stream: Box<dyn Stream>) -> Self {
        SoundInstance {
            handle: None,
            stream,
            active: true,
            left_transform: [1.0, 0.0],
            right_transform: [0.0, 1.0],
            peak: [0.0, 0.0],
            range: ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]),
        }
    }

    /// Updates `peak` from `range`, and resets the latter to default.
    fn update_peak(&mut self) {
        self.peak[0] = (self.range.1[0] - self.range.0[0]) / 2.0;
        self.peak[1] = (self.range.1[1] - self.range.0[1]) / 2.0;

        self.range = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    }
}

impl AudioMixer {
    /// Creates a new `AudioMixer` with the given number of channels and sample rate.
    pub fn new(num_output_channels: u8, output_sample_rate: u32) -> Self {
        Self {
            sounds: SlotMap::with_key(),
            extracted_sounds: SecondaryMap::new(),
            sound_instances: Arc::new(Mutex::new(SlotMap::with_key())),
            volume: Arc::new(RwLock::new(1.0)),
            num_output_channels,
            output_sample_rate,
            output_memory: Arc::new(RwLock::new(CircBuf::new())),
        }
    }

    /// Creates a proxy that may be sent to a different thread.
    pub fn proxy(&self) -> AudioMixerProxy {
        AudioMixerProxy {
            sound_instances: Arc::clone(&self.sound_instances),
            volume: Arc::clone(&self.volume),
            num_output_channels: self.num_output_channels,
            output_memory: Arc::clone(&self.output_memory),
        }
    }

    /// Mixes audio into the given `output_buffer`.
    ///
    /// All playing sound instances will be sampled and mixed to fill `output_buffer`.
    /// `output_buffer` is expected to be in 2-channel interleaved format.
    pub fn mix<'a, T>(&mut self, output_buffer: &mut [T])
    where
        T: 'a
            + Default
            + dasp::Sample<Signed = T>
            + dasp::sample::ToSample<f32>
            + dasp::sample::FromSample<i16>,
    {
        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        let volume = *self.volume.read().expect("Cannot be called reentrant");
        let mut output_memory = self
            .output_memory
            .write()
            .expect("Cannot be called reentrant");
        Self::mix_audio::<T>(
            &mut sound_instances,
            volume,
            self.num_output_channels,
            output_buffer,
            &mut output_memory,
        );
    }

    /// Instantiate a seekable decoder for audio data with the given format.
    ///
    /// A seekable decoder is used for:
    ///  * "Event" sounds on the timeline with custom start/loop settings
    ///  * ActionScript sounds that may have a custom start and loop setting
    fn make_seekable_decoder(
        format: &swf::SoundFormat,
        data: Cursor<ArcAsRef>,
    ) -> Result<Box<dyn SeekableDecoder>, decoders::Error> {
        let decoder: Box<dyn SeekableDecoder> = match format.compression {
            AudioCompression::UncompressedUnknownEndian => {
                // Cross fingers that it's little endian.
                tracing::warn!("make_decoder: PCM sound is unknown endian; assuming little endian");
                Box::new(PcmDecoder::new(
                    data,
                    format.is_stereo,
                    format.sample_rate,
                    format.is_16_bit,
                ))
            }
            AudioCompression::Uncompressed => Box::new(PcmDecoder::new(
                data,
                format.is_stereo,
                format.sample_rate,
                format.is_16_bit,
            )),
            AudioCompression::Adpcm => Box::new(AdpcmDecoder::new(
                data,
                format.is_stereo,
                format.sample_rate,
            )?),
            #[cfg(feature = "mp3")]
            AudioCompression::Mp3 => Box::new(decoders::Mp3Decoder::new_seekable(data)?),
            #[cfg(feature = "nellymoser")]
            AudioCompression::Nellymoser => Box::new(decoders::NellymoserDecoder::new(
                data,
                format.sample_rate.into(),
            )),
            AudioCompression::G711ALawPCM => Box::new(G711ALawDecoder::new(data)),
            AudioCompression::G711MuLawPCM => Box::new(G711MuLawDecoder::new(data)),
            _ => return Err(decoders::Error::UnhandledCompression(format.compression)),
        };
        Ok(decoder)
    }

    /// Transforms a `Stream` into a new `Stream` that matches the output sample rate.
    fn make_resampler<S: Stream>(&self, stream: S) -> impl Stream + use<S> {
        Self::make_resampler_to(stream, self.output_sample_rate)
    }

    /// Transforms a `Stream` into a new `Stream` with the given sample rate.
    fn make_resampler_to<S: Stream>(mut stream: S, sample_rate: u32) -> impl Stream + use<S> {
        // TODO: Allow interpolator to be user-configurable?
        let left = stream.next();
        let right = stream.next();
        let interpolator = dasp::interpolate::linear::Linear::new(left, right);
        let source_sample_rate = stream.source_sample_rate().into();
        ConverterStream(dasp::signal::interpolate::Converter::from_hz_to_hz(
            stream,
            interpolator,
            source_sample_rate,
            sample_rate.into(),
        ))
    }

    /// Creates a `Stream` for an "event" that decodes and resamples the audio stream to the
    /// output format.
    ///
    /// This also applies the custom envelope, start/end, and looping parameters from `settings`.
    fn make_stream_from_event_sound(
        &self,
        sound: &Sound,
        settings: &swf::SoundInfo,
        data: Cursor<ArcAsRef>,
    ) -> Result<Box<dyn Stream>, DecodeError> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = Self::make_seekable_decoder(&sound.format, data)?;

        // Wrap the decoder into an event sound stream (controls looping/envelope)
        let stream = EventSoundStream::new_with_settings(
            decoder,
            settings,
            sound
                .num_sample_frames_is_exact
                .then_some(sound.num_sample_frames),
            sound.skip_sample_frames,
            sound.lead_in_sample_frames,
        );
        // Resample the stream to the output sample rate.
        let stream = self.make_resampler(stream);
        if let Some(envelope) = &settings.envelope {
            let envelope_signal = EnvelopeSignal::new(&envelope[..], self.output_sample_rate);
            Ok(Box::new(MulAmpStream::new(stream, envelope_signal)) as Box<dyn Stream>)
        } else {
            Ok(Box::new(stream) as Box<dyn Stream>)
        }
    }

    /// Creates a `Stream` for a simple "event" sound that decodes and resamples the audio stream
    /// to the output format.
    ///
    /// This is used for cases where there is no custom envelope or looping on the sound instance.
    /// Otherwise, `AudioMixer::make_stream_from_event_sound` should be used.
    fn make_stream_from_simple_event_sound<R: 'static + std::io::Read + Send + Sync>(
        &self,
        format: &swf::SoundFormat,
        data_stream: R,
    ) -> Result<Box<dyn Stream>, DecodeError> {
        // Instantiate a decoder for the compression that the sound data uses.
        let decoder = decoders::make_decoder(format, data_stream)?;

        // Convert the `Decoder` to a `Stream`, and resample it to output sample rate.
        let stream = DecoderStream::new(decoder);
        let stream = self.make_resampler(stream);
        Ok(Box::new(stream))
    }

    /// Creates a `Stream` that decodes and resamples a timeline "stream" sound.
    fn make_stream_from_swf_slice(
        &self,
        stream_info: &swf::SoundStreamHead,
        data_stream: SwfSlice,
    ) -> Result<Box<dyn Stream>, DecodeError> {
        // Instantiate a decoder for the compression that the sound data uses.
        let clip_stream_decoder = decoders::make_stream_decoder(stream_info, data_stream)?;

        // Convert the `Decoder` to a `Stream`, and resample it to the output sample rate.
        let stream = DecoderStream::new(clip_stream_decoder);
        let stream = Box::new(self.make_resampler(stream));
        Ok(stream)
    }

    fn make_stream_from_buffer_substream(
        &self,
        stream_info: &SoundStreamInfo,
        data_stream: Substream,
    ) -> Result<Box<dyn Stream>, DecodeError> {
        // Instantiate a decoder for the compression that the sound data uses.
        let clip_stream_decoder = decoders::make_substream_decoder(stream_info, data_stream)?;

        // Convert the `Decoder` to a `Stream`, and resample it to the output sample rate.
        let stream = DecoderStream::new(clip_stream_decoder);
        let stream = Box::new(self.make_resampler(stream));
        Ok(stream)
    }

    /// Callback to the audio thread.
    /// Refill the output buffer by stepping through all active sounds
    /// and mixing in their output.
    fn mix_audio<'a, T>(
        sound_instances: &mut SlotMap<SoundInstanceHandle, SoundInstance>,
        volume: f32,
        num_channels: u8,
        mut output_buffer: &mut [T],
        output_memory: &mut CircBuf,
    ) where
        T: 'a
            + Default
            + dasp::Sample<Signed = T>
            + dasp::sample::ToSample<f32>
            + dasp::sample::FromSample<i16>,
    {
        use dasp::{
            Sample,
            frame::{Frame, Stereo},
        };
        use std::ops::DerefMut;

        // Adapt the volume for logarithmic hearing.
        let volume = ((10_f32.powf(81_f32.log10() * volume) - 1.0) / 80.0).to_sample();

        // For each sample, mix the samples from all active sound instances.
        for buf_frame in output_buffer
            .deref_mut()
            .chunks_exact_mut(num_channels.into())
        {
            let mut output_frame = Stereo::<T::Signed>::EQUILIBRIUM;
            for (_, sound) in sound_instances.iter_mut() {
                if sound.active && !sound.stream.is_exhausted() {
                    let sound_frame = sound.stream.next();
                    let [left_0, left_1] = sound_frame.mul_amp(sound.left_transform);
                    let [right_0, right_1] = sound_frame.mul_amp(sound.right_transform);
                    let mut sound_frame: Stereo<T> = [
                        Sample::add_amp(left_0, left_1).to_sample(),
                        Sample::add_amp(right_0, right_1).to_sample(),
                    ];
                    sound_frame = sound_frame.scale_amp(volume);

                    sound.range.0[0] = sound.range.0[0].min(sound_frame[0].to_sample());
                    sound.range.0[1] = sound.range.0[1].min(sound_frame[1].to_sample());

                    sound.range.1[0] = sound.range.1[0].max(sound_frame[0].to_sample());
                    sound.range.1[1] = sound.range.1[1].max(sound_frame[1].to_sample());

                    output_frame = output_frame.add_amp(sound_frame);
                } else {
                    sound.active = false;
                }
            }

            output_memory.push([output_frame[0].to_sample(), output_frame[1].to_sample()]);

            if output_memory.pos == 0 || output_memory.pos == 1024 {
                for (_, sound) in sound_instances.iter_mut() {
                    sound.update_peak();
                }
            }

            for (buf_sample, output_sample) in buf_frame.iter_mut().zip(output_frame.iter()) {
                *buf_sample = *output_sample;
            }
        }

        // Remove all dead sounds.
        sound_instances.retain(|_, sound| sound.active);
    }

    pub fn get_sample_history(&self) -> [[f32; 2]; 1024] {
        let output_memory = self
            .output_memory
            .read()
            .expect("Cannot be called reentrant");

        *output_memory.get()
    }

    /// Registers an embedded SWF sound with the audio mixer.
    pub fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, RegisterError> {
        // Slice off latency seek for MP3 data.
        let (skip_sample_frames, data) = if swf_sound.format.compression == AudioCompression::Mp3 {
            if swf_sound.data.len() < 2 {
                return Err(RegisterError::ShortMp3);
            }
            let skip_sample_frames = u16::from_le_bytes([swf_sound.data[0], swf_sound.data[1]]);
            (skip_sample_frames, &swf_sound.data[2..])
        } else {
            (0, swf_sound.data)
        };

        let sound = Sound {
            format: swf_sound.format.clone(),
            data: Arc::from(data),
            num_sample_frames: swf_sound.num_samples,
            skip_sample_frames,
            lead_in_sample_frames: 0,
            num_sample_frames_is_exact: true,
        };
        Ok(self.sounds.insert(sound))
    }

    /// Registers an external MP3 with the audio mixer.
    #[cfg(feature = "mp3")]
    pub fn register_mp3(&mut self, data: &[u8]) -> Result<SoundHandle, DecodeError> {
        let data = Arc::from(data);
        // Validate that this is actually MP3 data, and calculate duration and sample rate.
        let metadata = decoders::mp3_metadata(&data)?;
        let sound = Sound {
            format: swf::SoundFormat {
                compression: AudioCompression::Mp3,
                sample_rate: metadata.sample_rate,
                is_stereo: true,
                is_16_bit: true,
            },
            data,
            num_sample_frames: metadata.num_sample_frames,
            skip_sample_frames: 0,
            lead_in_sample_frames: metadata.info_frame_sample_frames,
            num_sample_frames_is_exact: false,
        };
        Ok(self.sounds.insert(sound))
    }

    #[cfg(not(feature = "mp3"))]
    pub fn register_mp3(&mut self, _data: &[u8]) -> Result<SoundHandle, DecodeError> {
        Err(decoders::Error::UnhandledCompression(AudioCompression::Mp3))
    }

    /// Starts a timeline audio stream.
    pub fn start_stream(
        &mut self,
        clip_data: SwfSlice,
        stream_info: &swf::SoundStreamHead,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        // The audio data for stream sounds is distributed among the frames of a
        // movie clip. The stream tag reader will parse through the SWF and
        // feed the decoder audio data on the fly.
        let stream = self.make_stream_from_swf_slice(stream_info, clip_data)?;

        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        let handle = sound_instances.insert(SoundInstance::new_stream(stream));
        Ok(handle)
    }

    /// Starts a sound.
    ///
    /// The sound must have been registered using `AudioMixer::register_sound`.
    pub fn start_sound(
        &mut self,
        sound_handle: SoundHandle,
        settings: &swf::SoundInfo,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        let sound = &self.sounds[sound_handle];
        let data = Cursor::new(ArcAsRef(Arc::clone(&sound.data)));
        // Create a stream that decodes and resamples the sound.
        let stream = if sound.skip_sample_frames == 0
            && sound.lead_in_sample_frames == 0
            && settings.in_sample.is_none()
            && settings.out_sample.is_none()
            && settings.num_loops <= 1
            && settings.envelope.is_none()
        {
            // For simple event sounds, use a standard decoder stream.
            self.make_stream_from_simple_event_sound(&sound.format, data)?
        } else {
            // For event sounds with envelopes/other properties, wrap it in `EventSoundStream`.
            self.make_stream_from_event_sound(sound, settings, data)?
        };

        // Add sound instance to active list.
        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        let handle = sound_instances.insert(SoundInstance::new_sound(sound_handle, stream));
        Ok(handle)
    }

    /// Starts a `Substream` backed audio stream.
    pub fn start_substream(
        &mut self,
        stream_data: Substream,
        stream_info: &SoundStreamInfo,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        // The audio data for substream sounds is already de-multiplexed by the
        // caller. The substream tag reader will feed the decoder audio data
        // from each chunk.
        let stream = self.make_stream_from_buffer_substream(stream_info, stream_data)?;

        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        let handle = sound_instances.insert(SoundInstance::new_stream(stream));
        Ok(handle)
    }

    /// Starts playing the sample frames of a `SampleQueue`, as they are queued.
    pub fn start_sample_queue(
        &mut self,
        queue: SampleQueue,
    ) -> Result<SoundInstanceHandle, DecodeError> {
        let stream = Box::new(self.make_resampler(SampleQueueStream::new(queue)));

        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        let handle = sound_instances.insert(SoundInstance::new_stream(stream));
        Ok(handle)
    }

    /// The sample rate of the sample frames returned by `extract_sound`.
    pub const EXTRACT_SAMPLE_RATE: u32 = 44100;

    /// Decodes the sample frames `start..start + length` of a sound as
    /// stereo at `EXTRACT_SAMPLE_RATE`, or fewer if the sound ends first.
    /// Returns `None` if the sound can't be decoded.
    ///
    /// Decoding goes on from where the last call ended, so that a sound
    /// extracted piece by piece is decoded once.
    pub fn extract_sound(
        &mut self,
        sound: SoundHandle,
        start: usize,
        length: usize,
    ) -> Option<Vec<[f32; 2]>> {
        let sound_data = self.sounds.get(sound)?;
        let make_stream = || {
            Self::make_stream_for_extracting(sound_data)
                .map_err(|e| tracing::warn!("Cannot extract sound: {e}"))
                .ok()
        };
        if !self.extracted_sounds.contains_key(sound) {
            let stream = make_stream()?;
            let to_extract_rate = |sample_frames: u32| {
                (u64::from(sample_frames) * u64::from(Self::EXTRACT_SAMPLE_RATE)
                    / u64::from(sound_data.format.sample_rate.max(1))) as usize
            };
            let lead_in = to_extract_rate(sound_data.lead_in_sample_frames);
            self.extracted_sounds.insert(
                sound,
                ExtractedSound {
                    lead_in,
                    length: sound_data
                        .num_sample_frames_is_exact
                        .then(|| lead_in + to_extract_rate(sound_data.num_sample_frames)),
                    stream: Some(stream),
                    position: 0,
                    window: VecDeque::new(),
                },
            );
        }
        let extracted = &mut self.extracted_sounds[sound];

        let mut end = start.saturating_add(length);
        if let Some(length) = extracted.length {
            end = end.min(length);
        }
        if start >= end {
            return Some(Vec::new());
        }

        if start < extracted.window_start() {
            // Too far back: decode again from the start.
            extracted.stream = make_stream();
            extracted.position = 0;
            extracted.window.clear();
        }
        while extracted.position < start && extracted.next_frame().is_some() {}
        if extracted.position < start {
            // The sound ends before `start`.
            return Some(Vec::new());
        }

        let to_f32 =
            |[left, right]: [i16; 2]| [f32::from(left) / 32768.0, f32::from(right) / 32768.0];
        let mut samples = Vec::with_capacity(end - start);
        let window_start = extracted.window_start();
        samples.extend(
            extracted
                .window
                .range(start - window_start..end.min(extracted.position) - window_start)
                .map(|&frame| to_f32(frame)),
        );
        while extracted.position < end
            && let Some(frame) = extracted.next_frame()
        {
            samples.push(to_f32(frame));
        }
        Some(samples)
    }

    /// Creates a `Stream` that decodes a sound from the start, resampled to
    /// `EXTRACT_SAMPLE_RATE`.
    fn make_stream_for_extracting(sound: &Sound) -> Result<Box<dyn Stream>, DecodeError> {
        let data = Cursor::new(ArcAsRef(Arc::clone(&sound.data)));
        let mut decoder = decoders::make_decoder(&sound.format, data)?;
        // Skip the MP3 encoder delay of sounds embedded in SWFs, like playback does.
        for _ in 0..sound.skip_sample_frames {
            decoder.next();
        }
        let stream = DecoderStream::new(decoder);
        if u32::from(stream.source_sample_rate()) == Self::EXTRACT_SAMPLE_RATE {
            Ok(Box::new(stream))
        } else {
            Ok(Box::new(Self::make_resampler_to(
                stream,
                Self::EXTRACT_SAMPLE_RATE,
            )))
        }
    }

    /// Stops a playing sound instance.
    pub fn stop_sound(&mut self, sound: SoundInstanceHandle) {
        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        sound_instances.remove(sound);
    }

    pub fn stop_all_sounds(&mut self) {
        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        sound_instances.clear();
    }

    /// Returns the position of a playing sound in milliseconds.
    ///
    ////// Returns `None` if the sound is no longer playing.
    pub fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<f64> {
        let sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        sound_instances.get(instance).map(|instance| {
            // Get the current sample position from the underlying audio source.
            let num_sample_frames: f64 = instance.stream.source_position().into();
            let sample_rate: f64 = instance.stream.source_sample_rate().into();
            num_sample_frames * 1000.0 / sample_rate
        })
    }

    pub fn get_sound_peak(&self, instance: SoundInstanceHandle) -> Option<[f32; 2]> {
        let sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        sound_instances.get(instance).map(|instance| instance.peak)
    }

    /// Returns the duration of a registered sound.
    ///
    /// Returns `None` if the sound is not registered or invalid.
    pub fn get_sound_duration(&self, sound: SoundHandle) -> Option<FloatDuration> {
        if let Some(sound) = self.sounds.get(sound) {
            // AS duration does not subtract `skip_sample_frames`.
            let num_sample_frames =
                f64::from(sound.num_sample_frames) + f64::from(sound.lead_in_sample_frames);
            let sample_rate: f64 = sound.format.sample_rate.into();
            let ms = num_sample_frames * 1000.0 / sample_rate;
            Some(FloatDuration::from_millis(ms))
        } else {
            None
        }
    }

    pub fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
        self.sounds.get(sound).map(|s| s.data.len() as u32)
    }

    pub fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat> {
        self.sounds.get(sound).map(|s| &s.format)
    }

    /// Sets the sound transform for the given playing sound.
    pub fn set_sound_transform(
        &mut self,
        instance: SoundInstanceHandle,
        transform: SoundTransform,
    ) {
        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        if let Some(instance) = sound_instances.get_mut(instance) {
            instance.left_transform = [transform.left_to_left, transform.right_to_left];
            instance.right_transform = [transform.left_to_right, transform.right_to_right];
        }
    }

    pub fn volume(&self) -> f32 {
        *self.volume.read().expect("Cannot be called reentrant")
    }

    pub fn set_volume(&mut self, volume: f32) {
        *self.volume.write().expect("Cannot be called reentrant") = volume
    }
}

/// A thread-safe proxy to the main `AudioMixer`, allowing for mixing audio from a different thread.
///
/// An `AudioMixerProxy` can be created via `AudioMixer::proxy`. The proxy can be sent to another thread
/// to perform audio mixing on a different thread.
pub struct AudioMixerProxy {
    /// The list of actively playing sound instances.
    sound_instances: Arc<Mutex<SlotMap<SoundInstanceHandle, SoundInstance>>>,

    /// The master volume of the audio from [0.0, 1.0].
    volume: Arc<RwLock<f32>>,

    /// The number of channels in the output stream. Must be 1 or 2.
    num_output_channels: u8,

    output_memory: Arc<RwLock<CircBuf>>,
}

impl AudioMixerProxy {
    /// Mixes audio into the given `output_buffer`.
    ///
    /// All playing sound instances will be sampled and mixed to fill `output_buffer`.
    /// `output_buffer` is expected to be in 2-channel interleaved format.
    pub fn mix<'a, T>(&self, output_buffer: &mut [T])
    where
        T: 'a
            + Default
            + dasp::Sample<Signed = T>
            + dasp::sample::ToSample<f32>
            + dasp::sample::FromSample<i16>,
    {
        let mut sound_instances = self
            .sound_instances
            .lock()
            .expect("Cannot be called reentrant");
        let volume = *self.volume.read().expect("Cannot be called reentrant");
        let mut output_memory = self
            .output_memory
            .write()
            .expect("Cannot be called reentrant");
        AudioMixer::mix_audio::<T>(
            &mut sound_instances,
            volume,
            self.num_output_channels,
            output_buffer,
            &mut output_memory,
        )
    }
}

/// A dummy wrapper struct to implement `AsRef<[u8]>` for `Arc<Vec<u8>>`.
/// Not having this trait causes problems when trying to use `Cursor<Vec<u8>>`.
struct ArcAsRef(Arc<[u8]>);

impl AsRef<[u8]> for ArcAsRef {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Default for ArcAsRef {
    fn default() -> Self {
        ArcAsRef(Arc::new([]))
    }
}

/// A stream for event sound instances with custom envelopes, start/end point, or loop settings.
struct EventSoundStream {
    decoder: Box<dyn SeekableDecoder>,
    num_loops: u16,
    start_sample_frame: u32,
    end_sample_frame: Option<u32>,
    cur_sample_frame: u32,
    skip_sample_frames: u32,
    /// Sample frames of silence that the sound starts with, before the
    /// decoder's (see `Sound::lead_in_sample_frames`).
    lead_in_sample_frames: u32,
    /// Sample frames of the lead-in that each loop starts with.
    start_lead_in: u32,
    /// Sample frames of the lead-in still to play in this loop.
    lead_in_left: u32,
    is_exhausted: bool,
}

impl EventSoundStream {
    fn new_with_settings(
        decoder: Box<dyn SeekableDecoder>,
        settings: &swf::SoundInfo,
        num_sample_frames: Option<u32>,
        skip_sample_frames: u16,
        lead_in_sample_frames: u32,
    ) -> Self {
        let skip_sample_frames: u32 = skip_sample_frames.into();
        let sample_divisor = 44100.0 / f64::from(decoder.sample_rate());
        // In and out points count the lead-in, which comes before the
        // decoder's first sample frame.
        let in_sample_frame = (f64::from(settings.in_sample.unwrap_or(0)) / sample_divisor) as u32;
        let start_lead_in = lead_in_sample_frames.saturating_sub(in_sample_frame);
        let start_sample_frame =
            in_sample_frame.saturating_sub(lead_in_sample_frames) + skip_sample_frames;
        // Without an out point, a sound whose length is only an estimate
        // plays to the end of its data.
        let end_sample_frame = settings
            .out_sample
            .map(|n| ((f64::from(n) / sample_divisor) as u32).saturating_sub(lead_in_sample_frames))
            .or(num_sample_frames)
            .map(|end| end + skip_sample_frames);

        let mut stream = Self {
            decoder,
            num_loops: settings.num_loops,
            start_sample_frame,
            end_sample_frame,
            cur_sample_frame: start_sample_frame,
            skip_sample_frames,
            lead_in_sample_frames,
            start_lead_in,
            lead_in_left: 0,
            is_exhausted: false,
        };
        stream.next_loop();
        stream
    }

    /// Resets the decoder to the start point of the loop.
    fn next_loop(&mut self) {
        if self.num_loops > 0 {
            self.num_loops -= 1;
            self.decoder.seek_to_sample_frame(self.start_sample_frame);
            self.cur_sample_frame = self.start_sample_frame;
            self.lead_in_left = self.start_lead_in;
        } else {
            self.is_exhausted = true;
        }
    }
}

impl dasp::signal::Signal for EventSoundStream {
    type Frame = [i16; 2];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        // Loop the sound if necessary, and get the next frame.
        if !self.is_exhausted {
            if self.lead_in_left > 0 {
                self.lead_in_left -= 1;
                return [0, 0];
            }
            if let Some(frame) = self.decoder.next() {
                self.cur_sample_frame += 1;
                if let Some(end) = self.end_sample_frame
                    && self.cur_sample_frame > end
                {
                    self.next_loop();
                }
                frame
            } else {
                self.next_loop();
                self.next()
            }
        } else {
            [0, 0]
        }
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.is_exhausted
    }
}

impl Stream for EventSoundStream {
    #[inline]
    fn source_position(&self) -> u32 {
        (self
            .cur_sample_frame
            .saturating_sub(self.skip_sample_frames)
            + self.lead_in_sample_frames)
            .saturating_sub(self.lead_in_left)
    }

    #[inline]
    fn source_sample_rate(&self) -> u16 {
        self.decoder.sample_rate()
    }
}

/// A stream that converts a source stream to a different sample rate.
struct ConverterStream<S, I>(dasp::signal::interpolate::Converter<S, I>)
where
    S: Stream,
    I: dasp::interpolate::Interpolator<Frame = [i16; 2]>;

impl<S, I> Stream for ConverterStream<S, I>
where
    S: Stream,
    I: dasp::interpolate::Interpolator<Frame = [i16; 2]> + Send + Sync,
{
    #[inline]
    fn source_position(&self) -> u32 {
        self.0.source().source_position()
    }

    #[inline]
    fn source_sample_rate(&self) -> u16 {
        self.0.source().source_sample_rate()
    }
}

impl<S, I> dasp::signal::Signal for ConverterStream<S, I>
where
    S: Stream,
    I: dasp::interpolate::Interpolator<Frame = [i16; 2]> + Send + Sync,
{
    type Frame = [i16; 2];

    #[inline]
    fn next(&mut self) -> [i16; 2] {
        self.0.next()
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.0.is_exhausted()
    }
}

/// A stream that multiples a source stream by an amplitude stream to produce an enveloped stream.
struct MulAmpStream<S, E>
where
    S: Stream,
    E: dasp::signal::Signal<Frame = [f32; 2]> + Send + Sync,
{
    stream: S,
    envelope: E,
}

impl<S, E> MulAmpStream<S, E>
where
    S: Stream,
    E: dasp::signal::Signal<Frame = [f32; 2]> + Send + Sync,
{
    fn new(stream: S, envelope: E) -> Self {
        Self { stream, envelope }
    }
}

impl<S, E> Stream for MulAmpStream<S, E>
where
    S: Stream,
    E: dasp::signal::Signal<Frame = [f32; 2]> + Send + Sync,
{
    #[inline]
    fn source_position(&self) -> u32 {
        self.stream.source_position()
    }

    #[inline]
    fn source_sample_rate(&self) -> u16 {
        self.stream.source_sample_rate()
    }
}

impl<S, E> dasp::signal::Signal for MulAmpStream<S, E>
where
    S: Stream,
    E: dasp::signal::Signal<Frame = [f32; 2]> + Send + Sync,
{
    type Frame = [i16; 2];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        dasp::frame::Frame::mul_amp(self.stream.next(), self.envelope.next())
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.stream.is_exhausted() || self.envelope.is_exhausted()
    }
}

/// A signal that represents the sound envelope for an event sound.
/// The sound stream gets multiplied by the envelope for volume/panning effects.
struct EnvelopeSignal {
    /// Iterator through the envelope points specified in the SWF file.
    envelope: std::vec::IntoIter<swf::SoundEnvelopePoint>,

    /// The starting envelope point.
    prev_point: swf::SoundEnvelopePoint,

    /// The ending envelope point.
    next_point: swf::SoundEnvelopePoint,

    /// The current sample index.
    cur_sample: u32,
}

impl EnvelopeSignal {
    fn new(envelope: &[swf::SoundEnvelopePoint], output_sample_rate: u32) -> Self {
        // Envelope samples are always in 44.1KHz.
        const ENVELOPE_SAMPLE_RATE: u32 = 44100;

        // Scale the envelope points from 44.1KHz to the output rate.
        let scale = f64::from(output_sample_rate) / f64::from(ENVELOPE_SAMPLE_RATE);
        let mut envelope = envelope
            .iter()
            .map(|pt| swf::SoundEnvelopePoint {
                sample: (f64::from(pt.sample) * scale) as u32,
                ..*pt
            })
            .collect::<swf::SoundEnvelope>()
            .into_iter();
        let first_point = envelope.next().unwrap_or(swf::SoundEnvelopePoint {
            sample: 0,
            left_volume: 1.0,
            right_volume: 1.0,
        });
        Self {
            // The initial volume is the first point's volume.
            prev_point: swf::SoundEnvelopePoint {
                sample: 0,
                left_volume: first_point.left_volume,
                right_volume: first_point.right_volume,
            },
            next_point: first_point,
            cur_sample: 0,
            envelope,
        }
    }
}

impl dasp::signal::Signal for EnvelopeSignal {
    type Frame = [f32; 2];

    fn next(&mut self) -> Self::Frame {
        // Calculate interpolated volume.
        let out = if self.prev_point.sample < self.next_point.sample {
            let a: f64 = (self.cur_sample - self.prev_point.sample).into();
            let b: f64 = (self.next_point.sample - self.prev_point.sample).into();
            let lerp = a / b;
            let interpolator = dasp::interpolate::linear::Linear::new(
                [self.prev_point.left_volume, self.prev_point.right_volume],
                [self.next_point.left_volume, self.next_point.right_volume],
            );
            use dasp::interpolate::Interpolator;
            interpolator.interpolate(lerp)
        } else {
            [self.next_point.left_volume, self.next_point.right_volume]
        };

        // Update envelope endpoints.
        self.cur_sample = self.cur_sample.saturating_add(1);
        while self.cur_sample > self.next_point.sample {
            self.prev_point = self.next_point.clone();
            self.next_point = self.envelope.next().unwrap_or(swf::SoundEnvelopePoint {
                sample: u32::MAX,
                left_volume: self.prev_point.left_volume,
                right_volume: self.prev_point.right_volume,
            });

            if self.prev_point.sample > self.next_point.sample {
                self.next_point.sample = self.prev_point.sample;
                tracing::error!("Invalid sound envelope; sample indices are out of order");
            }
        }

        out
    }

    fn is_exhausted(&self) -> bool {
        false
    }
}

#[macro_export]
macro_rules! impl_audio_mixer_backend {
    ($mixer:ident) => {
        #[inline]
        fn register_sound(&mut self, swf_sound: &swf::Sound) -> Result<SoundHandle, RegisterError> {
            self.$mixer.register_sound(swf_sound)
        }

        #[inline]
        fn register_mp3(&mut self, data: &[u8]) -> Result<SoundHandle, DecodeError> {
            self.$mixer.register_mp3(data)
        }

        #[inline]
        fn start_stream(
            &mut self,
            clip_data: $crate::tag_utils::SwfSlice,
            stream_info: &swf::SoundStreamHead,
        ) -> Result<SoundInstanceHandle, DecodeError> {
            self.$mixer.start_stream(clip_data, stream_info)
        }

        #[inline]
        fn start_sound(
            &mut self,
            sound_handle: SoundHandle,
            settings: &swf::SoundInfo,
        ) -> Result<SoundInstanceHandle, DecodeError> {
            self.$mixer.start_sound(sound_handle, settings)
        }

        #[inline]
        fn start_substream(
            &mut self,
            stream_data: ruffle_core::backend::audio::Substream,
            stream_info: &SoundStreamInfo,
        ) -> Result<SoundInstanceHandle, DecodeError> {
            self.$mixer.start_substream(stream_data, stream_info)
        }

        #[inline]
        fn start_sample_queue(
            &mut self,
            queue: ruffle_core::backend::audio::SampleQueue,
        ) -> Result<SoundInstanceHandle, DecodeError> {
            self.$mixer.start_sample_queue(queue)
        }

        #[inline]
        fn extract_sound(
            &mut self,
            sound: SoundHandle,
            start: usize,
            length: usize,
        ) -> Option<Vec<[f32; 2]>> {
            self.$mixer.extract_sound(sound, start, length)
        }

        #[inline]
        fn stop_sound(&mut self, sound: SoundInstanceHandle) {
            self.$mixer.stop_sound(sound)
        }

        #[inline]
        fn stop_all_sounds(&mut self) {
            self.$mixer.stop_all_sounds()
        }

        #[inline]
        fn get_sound_position(&self, instance: SoundInstanceHandle) -> Option<f64> {
            self.$mixer.get_sound_position(instance)
        }

        #[inline]
        fn get_sound_duration(&self, sound: SoundHandle) -> Option<$crate::FloatDuration> {
            self.$mixer.get_sound_duration(sound)
        }

        #[inline]
        fn get_sound_size(&self, sound: SoundHandle) -> Option<u32> {
            self.$mixer.get_sound_size(sound)
        }

        #[inline]
        fn get_sound_format(&self, sound: SoundHandle) -> Option<&swf::SoundFormat> {
            self.$mixer.get_sound_format(sound)
        }

        #[inline]
        fn set_sound_transform(
            &mut self,
            instance: SoundInstanceHandle,
            transform: SoundTransform,
        ) {
            self.$mixer.set_sound_transform(instance, transform)
        }

        #[inline]
        fn get_sound_peak(&mut self, instance: SoundInstanceHandle) -> Option<[f32; 2]> {
            self.$mixer.get_sound_peak(instance)
        }

        #[inline]
        fn volume(&self) -> f32 {
            self.$mixer.volume()
        }

        #[inline]
        fn set_volume(&mut self, volume: f32) {
            self.$mixer.set_volume(volume)
        }

        fn get_sample_history(&self) -> [[f32; 2]; 1024] {
            self.$mixer.get_sample_history()
        }
    };
}
