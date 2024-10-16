//! NetStream implementation

use crate::avm1::{
    Activation as Avm1Activation, ActivationIdentifier as Avm1ActivationIdentifier,
    ExecutionReason as Avm1ExecutionReason, FlvValueAvm1Ext, ScriptObject as Avm1ScriptObject,
    TObject as Avm1TObject, Value as Avm1Value,
};
use crate::avm2::object::TObject as Avm2TObject;
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Error as Avm2Error, EventObject as Avm2EventObject,
    FlvValueAvm2Ext, Object as Avm2Object,
};
use crate::backend::audio::{
    DecodeError, SoundInstanceHandle, SoundStreamInfo, SoundStreamWrapping,
};
use crate::backend::navigator::Request;
use crate::buffer::{Buffer, Slice, Substream, SubstreamError};
use crate::context::UpdateContext;
use crate::display_object::{MovieClip, TDisplayObject};
use crate::loader::Error;
use crate::string::AvmString;
use crate::vminterface::AvmObject;
use flv_rs::{
    AudioData as FlvAudioData, AudioDataType as FlvAudioDataType, Error as FlvError, FlvReader,
    FrameType as FlvFrameType, Header as FlvHeader, ScriptData as FlvScriptData,
    SoundFormat as FlvSoundFormat, SoundRate as FlvSoundRate, SoundSize as FlvSoundSize,
    SoundType as FlvSoundType, Tag as FlvTag, TagData as FlvTagData, Value as FlvValue,
    VideoData as FlvVideoData, VideoPacket as FlvVideoPacket,
};
use gc_arena::{Collect, GcCell, Mutation};
use ruffle_render::bitmap::BitmapInfo;
use ruffle_video::frame::EncodedFrame;
use ruffle_video::VideoStreamHandle;
use std::cmp::max;
use std::io::{Seek, SeekFrom};
use swf::{AudioCompression, SoundFormat, VideoCodec, VideoDeblocking};
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
enum NetstreamError {
    #[error("Decoding failed because {0}")]
    DecodeError(DecodeError),

    #[error("Substream management error {0}")]
    SubstreamError(SubstreamError),

    #[error("Unknown codec")]
    UnknownCodec,
}

impl From<DecodeError> for NetstreamError {
    fn from(err: DecodeError) -> NetstreamError {
        NetstreamError::DecodeError(err)
    }
}

impl From<SubstreamError> for NetstreamError {
    fn from(err: SubstreamError) -> NetstreamError {
        NetstreamError::SubstreamError(err)
    }
}

/// Manager for all media streams.
///
/// This does *not* handle data transport; which is delegated to `LoadManager`.
/// `StreamManager` *only* handles decoding or encoding of relevant media
/// streams.
#[derive(Collect)]
#[collect(no_drop)]
pub struct StreamManager<'gc> {
    /// List of streams that need tick processing.
    ///
    /// This is not the total list of all created NetStreams; only the ones
    /// that have been configured to play media.
    ///
    /// A stream becomes active if it is either playing streaming media or is
    /// doing other tick-time processing such as seeking.
    active_streams: Vec<NetStream<'gc>>,
}

impl Default for StreamManager<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> StreamManager<'gc> {
    pub fn new() -> Self {
        StreamManager {
            active_streams: Vec::new(),
        }
    }

    /// Activate a `NetStream`.
    ///
    /// This can be called at any time to flag that a `NetStream` has work to
    /// do, and called multiple times. The `NetStream` will determine what to
    /// do at tick time.
    pub fn activate(context: &mut UpdateContext<'gc>, stream: NetStream<'gc>) {
        if !context.stream_manager.active_streams.contains(&stream) {
            context.stream_manager.active_streams.push(stream);
        }
    }

    /// Deactivate a `NetStream`.
    ///
    /// This should only ever be called at tick time if the stream itself has
    /// determined there is no future work for it to do.
    pub fn deactivate(context: &mut UpdateContext<'gc>, stream: NetStream<'gc>) {
        let index = context
            .stream_manager
            .active_streams
            .iter()
            .position(|x| *x == stream);
        if let Some(index) = index {
            context.stream_manager.active_streams.remove(index);
        }
    }

    /// Process all active media streams.
    ///
    /// This is an unlocked timestep; the `dt` parameter indicates how many
    /// milliseconds have elapsed since the last tick. This is intended to
    /// support video framerates separate from the Stage frame rate.
    ///
    /// This does not borrow `&mut self` as we need the `UpdateContext`, too.
    pub fn tick(context: &mut UpdateContext<'gc>, dt: f64) {
        let streams = context.stream_manager.active_streams.clone();
        for stream in streams {
            stream.tick(context, dt)
        }
    }
}

/// A stream representing download of some (audiovisual) data.
///
/// `NetStream` interacts with several different parts of player
/// infrastructure:
///
///  * `LoadManager` fills individual `NetStream` buffers with data (or, in the
///    future, empties them out for media upload)
///  * `StreamManager` processes media data in the `NetStream` buffer (in the
///    future, sending it to the audio backend or `SoundManager`)
///  * `Video` display objects linked to this `NetStream` display the latest
///    decoded frame.
///
/// It corresponds directly to the AVM1 and AVM2 `NetStream` classes; it's API
/// is intended to be a VM-agnostic version of those.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct NetStream<'gc>(GcCell<'gc, NetStreamData<'gc>>);

impl PartialEq for NetStream<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl Eq for NetStream<'_> {}

/// The current type of the data in the stream buffer.
#[derive(Clone, Debug)]
pub enum NetStreamType {
    /// The stream is an FLV.
    Flv {
        #[allow(dead_code)] // set but never read
        header: FlvHeader,

        /// The currently playing video track's stream instance.
        video_stream: Option<VideoStreamHandle>,

        /// The index of the last processed frame.
        ///
        /// FLV does not store this information directly and we are not holding
        /// onto a table of data buffers like `Video` does, so we must maintain
        /// frame IDs ourselves for various API related purposes.
        frame_id: u32,
    },
}

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct NetStreamData<'gc> {
    /// All data currently loaded in the stream.
    buffer: Buffer,

    /// The buffer position that we are currently seeking to.
    offset: usize,

    /// The expected length of the buffer once downloading is complete.
    ///
    /// `None` indicates that downloading is already complete and that the
    /// length of the associated `Buffer` is the final length.
    expected_length: Option<usize>,

    /// The buffer position for processing incoming data.
    ///
    /// This points to the first byte that the stream has *never* processed
    /// before in the buffer. It should always be greater than or equal to the
    /// offset position.
    ///
    /// Certain data, such as the header or metadata of an FLV, should only
    /// ever be processed one time, even if we seek backwards to it later on.
    /// We call this data "preloaded", whether or not there is actually a
    /// separate preload step for that given format.
    preload_offset: usize,

    /// The current stream type, if known.
    #[collect(require_static)]
    stream_type: Option<NetStreamType>,

    /// The current seek offset in the stream in milliseconds.
    stream_time: f64,

    /// The next queued seek offset in milliseconds.
    ///
    /// Seeks are only executed on the next stream tick.
    queued_seek_time: Option<f64>,

    /// The number of seconds of video data that should be buffered. This is
    /// currently unsupported and changing it has no effect.
    buffer_time: f64,

    /// The last decoded bitmap.
    ///
    /// Any `Video`s on the stage will display the bitmap here when attached to
    /// this `NetStream`.
    #[collect(require_static)]
    last_decoded_bitmap: Option<BitmapInfo>,

    /// The AVM side of this stream.
    avm_object: Option<AvmObject<'gc>>,

    /// The AVM2 client object, which corresponds to `NetStream.client`.
    avm2_client: Option<Avm2Object<'gc>>,

    /// The URL of the requested FLV if one exists.
    url: Option<String>,

    /// The `Substream` associated with the currently playing audio track and
    /// the expected playback format of that audio.
    #[collect(require_static)]
    audio_stream: Option<(Substream, SoundStreamInfo)>,

    /// The currently playing sound stream
    #[collect(require_static)]
    sound_instance: Option<SoundInstanceHandle>,

    /// The MovieClip this `NetStream` is attached to.
    attached_to: Option<MovieClip<'gc>>,

    /// True if the stream should play when ticked.
    playing: bool,
}

impl<'gc> NetStream<'gc> {
    pub fn new(gc_context: &Mutation<'gc>, avm_object: Option<AvmObject<'gc>>) -> Self {
        // IMPORTANT: When adding new fields consider if they need to be
        // initialized in `reset_buffer` as well.
        Self(GcCell::new(
            gc_context,
            NetStreamData {
                buffer: Buffer::new(),
                offset: 0,
                preload_offset: 0,
                stream_type: None,
                stream_time: 0.0,
                queued_seek_time: None,
                buffer_time: 0.1,
                last_decoded_bitmap: None,
                avm_object,
                avm2_client: None,
                url: None,
                audio_stream: None,
                sound_instance: None,
                attached_to: None,
                playing: false,
                expected_length: Some(0),
            },
        ))
    }

    pub fn set_client(self, gc_context: &Mutation<'gc>, new_client: Avm2Object<'gc>) {
        self.0.write(gc_context).avm2_client = Some(new_client);
    }

    pub fn client(self) -> Option<Avm2Object<'gc>> {
        self.0.read().avm2_client
    }

    pub fn set_avm_object(self, gc_context: &Mutation<'gc>, avm_object: AvmObject<'gc>) {
        self.0.write(gc_context).avm_object = Some(avm_object);
    }

    /// Reset the `NetStream` buffer to accept new source data.
    ///
    /// This must be done once per source change and should ideally be done
    /// immediately before the first `load_buffer` call for a particular source
    /// file.
    ///
    /// Externally visible AVM state must not be reinitialized here - i.e. the
    /// AS3 `client` doesn't go away because you played a new video file.
    pub fn reset_buffer(self, context: &mut UpdateContext<'gc>) {
        let mut write = self.0.write(context.gc_context);

        if let Some(instance) = write.sound_instance {
            // We stop the sound twice because sounds may have either been
            // played through the audio manager or through the backend directly
            // depending on the attachment state at the time of first audio
            // playback.
            context.audio.stop_sound(instance);
            context.audio_manager.stop_sound(context.audio, instance);
        }

        write.buffer = Buffer::new();
        write.offset = 0;
        write.preload_offset = 0;
        write.stream_type = None;
        write.stream_time = 0.0;
        write.queued_seek_time = None;
        write.audio_stream = None;
        write.sound_instance = None;
        write.expected_length = Some(0);
    }

    /// Set the total number of bytes expected to be downloaded.
    pub fn set_expected_length(self, context: &mut UpdateContext<'gc>, expected: usize) {
        let mut write = self.0.write(context.gc_context);
        let len = write.buffer.len();

        // The subtract is to avoid reserving space for already-downloaded data.
        if expected > len {
            write.buffer.reserve(expected - len);
        }

        write.expected_length = Some(expected);
    }

    /// Append data to the `NetStream`'s current internal buffer.
    ///
    /// If you are loading data from a new source, you must first initialize
    /// the buffer, otherwise existing buffer contents will remain and be
    /// incorrectly parsed.
    ///
    /// Buffer loading can be done in chunks but must be done in such a way
    /// that all data is appended in the correct order and that data from
    /// separate streams is not mixed together.
    pub fn load_buffer(self, context: &mut UpdateContext<'gc>, data: &mut Vec<u8>) {
        self.0.write(context.gc_context).buffer.append(data);

        StreamManager::activate(context, self);

        // NOTE: The onMetaData event triggers before this event in Flash due to its streaming behavior.
        self.trigger_status_event(
            context,
            vec![("code", "NetStream.Buffer.Full"), ("level", "status")],
        );
    }

    /// Indicate that the buffer has finished loading and that no further data
    /// is expected to be downloaded to it.
    pub fn finish_buffer(self, context: &mut UpdateContext<'gc>) {
        self.0.write(context.gc_context).expected_length = None;
    }

    pub fn report_error(self, _error: Error) {
        // TODO: Report an `asyncError` to AVM1 or 2.
    }

    pub fn bytes_loaded(self) -> usize {
        self.0.read().buffer.len()
    }

    pub fn bytes_total(self) -> usize {
        let read = self.0.read();
        let buflen = read.buffer.len();

        std::cmp::max(read.expected_length.unwrap_or(buflen), buflen)
    }

    pub fn time(self) -> f64 {
        self.0.read().stream_time
    }

    pub fn buffer_time(self) -> f64 {
        self.0.read().buffer_time
    }

    pub fn set_buffer_time(self, mc: &Mutation<'gc>, buffer_time: f64) {
        self.0.write(mc).buffer_time = buffer_time;
    }

    /// Queue a seek to be executed on the next frame tick.
    ///
    /// `offset` is in milliseconds.
    pub fn seek(self, context: &mut UpdateContext<'gc>, offset: f64, notify: bool) {
        self.0.write(context.gc_context).queued_seek_time = Some(offset);
        StreamManager::activate(context, self);

        if notify {
            let trigger = AvmString::new_utf8(
                context.gc_context,
                format!("Start Seeking {}", offset as u64),
            );
            self.trigger_status_event(
                context,
                vec![
                    ("description", trigger),
                    ("level", "status".into()),
                    ("code", "NetStream.SeekStart.Notify".into()),
                ],
            );
        }
    }

    /// Seek to a new position in the stream.
    ///
    /// All existing audio will be paused. The stream offset will be snapped to
    /// either the prior or next keyframe depending on seek direction. If the
    /// stream is playing then new tag processing will occur when the stream
    /// ticks next.
    ///
    /// This always does an in-buffer seek. Seek-driven requests are not
    /// currently supported. When progressive download is implemented this seek
    /// algorithm will need to detect out-of-buffer seeks and trigger fresh
    /// downloads.
    ///
    /// `offset` is in milliseconds.
    ///
    /// This function should be run during stream ticks and *not* called by AVM
    /// code to service seek requests.
    pub fn execute_seek(self, context: &mut UpdateContext<'gc>, offset: f64) {
        self.trigger_status_event(
            context,
            vec![("code", "NetStream.Seek.Notify"), ("level", "status")],
        );

        // Ensure the container stream type is known before continuing.
        if self.0.read().stream_type.is_none() && !self.sniff_stream_type(context) {
            return;
        }

        let mut write = self.0.write(context.gc_context);
        if write.stream_time == offset {
            //Don't do anything for no-op seeks.
            return;
        }

        if let Some(sound) = write.sound_instance {
            context.stop_sound(sound);
            context.audio.stop_sound(sound);

            write.sound_instance = None;
            write.audio_stream = None;
        }

        if matches!(write.stream_type, Some(NetStreamType::Flv { .. })) {
            let slice = write.buffer.to_full_slice();
            let buffer = slice.data();
            let mut reader = FlvReader::from_parts(&buffer, write.offset);
            let skipping_back = write.stream_time > offset;

            loop {
                if skipping_back {
                    let res = FlvTag::skip_back(&mut reader);
                    if matches!(res, Err(FlvError::EndOfData)) {
                        //At start of video, can't skip further back
                        break;
                    }

                    if let Err(e) = res {
                        tracing::error!("FLV tag parsing failed during seek backward: {}", e);
                        break;
                    }
                }

                let old_position = reader
                    .stream_position()
                    .expect("valid stream position when seeking");

                let tag = FlvTag::parse(&mut reader);
                if matches!(tag, Err(FlvError::EndOfData)) {
                    //At end of video, can't skip further forward
                    break;
                }

                if let Err(e) = tag {
                    tracing::error!("FLV tag parsing failed during seek forward: {}", e);
                    break;
                }

                if skipping_back {
                    //Tag position won't actually move backwards if we don't do this.
                    reader
                        .seek(SeekFrom::Start(old_position))
                        .expect("valid backseek position");
                }

                let tag = tag.unwrap();
                write.stream_time = tag.timestamp as f64;

                if skipping_back && write.stream_time > offset
                    || !skipping_back && write.stream_time < offset
                {
                    continue;
                }

                match tag.data {
                    FlvTagData::Video(FlvVideoData {
                        frame_type: FlvFrameType::Keyframe,
                        ..
                    }) => {
                        // If we don't backseek when we find the keyframe,
                        // we will miss the keyframe.
                        reader
                            .seek(SeekFrom::Start(old_position))
                            .expect("valid backseek position");

                        break;
                    }
                    _ => continue,
                }
            }

            write.offset = reader
                .stream_position()
                .expect("FLV reader stream position") as usize;
        }

        drop(write);

        if let Some(AvmObject::Avm2(_)) = self.0.read().avm_object {
            self.trigger_status_event(
                context,
                vec![
                    ("description", "Seek Complete -1"),
                    ("level", "status"),
                    ("code", "NetStream.Seek.Complete"),
                ],
            );
        }
    }

    /// Start playing media from this NetStream.
    ///
    /// If `name` is specified, this will also trigger streaming download of
    /// the given resource. Otherwise, the stream will play whatever data is
    /// available in the buffer.
    pub fn play(self, context: &mut UpdateContext<'gc>, name: Option<AvmString<'gc>>) {
        if let Some(name) = name {
            let request = if let Ok(stream_url) =
                Url::parse(context.swf.url()).and_then(|url| url.join(name.to_string().as_str()))
            {
                Request::get(stream_url.to_string())
            } else {
                Request::get(name.to_string())
            };
            let mut write = self.0.write(context.gc_context);
            write.url = Some(request.url().to_string());
            write.preload_offset = 0;
            let future = context
                .load_manager
                .load_netstream(context.player.clone(), self, request);

            context.navigator.spawn_future(future);
        }

        self.0.write(context.gc_context).playing = true;
        StreamManager::activate(context, self);

        self.trigger_status_event(
            context,
            vec![("code", "NetStream.Play.Start"), ("level", "status")],
        );
    }

    /// Pause stream playback.
    pub fn pause(self, context: &mut UpdateContext<'gc>, notify: bool) {
        // NOTE: We do not deactivate the stream here as there may be other
        // work to be done at tick time.
        self.0.write(context.gc_context).playing = false;

        if notify {
            self.trigger_status_event(
                context,
                vec![
                    ("description", "Pausing"),
                    ("level", "status"),
                    ("code", "NetStream.Pause.Notify"),
                ],
            );
        }
    }

    /// Resume stream playback.
    pub fn resume(self, context: &mut UpdateContext<'gc>) {
        self.0.write(context.gc_context).playing = true;
        StreamManager::activate(context, self);
    }

    /// Resume stream playback if paused, pause otherwise.
    pub fn toggle_paused(self, context: &mut UpdateContext<'gc>) {
        let mut write = self.0.write(context.gc_context);
        write.playing = !write.playing;

        if write.playing {
            StreamManager::activate(context, self);
        }
    }

    /// Indicates that this `NetStream`'s audio was detached from a `MovieClip` (AVM1)
    pub fn was_detached(self, context: &mut UpdateContext<'gc>) {
        let mut write = self.0.write(context.gc_context);

        if let Some(sound_instance) = &write.sound_instance {
            context
                .audio_manager
                .stop_sound(context.audio, *sound_instance);
        }

        write.audio_stream = None;
        write.attached_to = None;
    }

    /// Indicates that this `NetStream`'s audio was attached to a `MovieClip` (AVM1)
    pub fn was_attached(self, context: &mut UpdateContext<'gc>, clip: MovieClip<'gc>) {
        let mut write = self.0.write(context.gc_context);

        // A `NetStream` cannot be attached to two `MovieClip`s at once.
        // Stop the old sound; the new one will stream at the next tag read.
        // TODO: Change this to have `audio_manager` just switch the sound
        // transforms around
        if let Some(sound_instance) = &write.sound_instance {
            context
                .audio_manager
                .stop_sound(context.audio, *sound_instance);
        }

        write.audio_stream = None;
        write.attached_to = Some(clip);
    }

    /// Process a parsed FLV audio tag.
    ///
    /// `write` must be an active borrow of the current `NetStream`. `slice`
    /// must reference the underlying backing buffer.
    fn flv_audio_tag(
        self,
        write: &mut NetStreamData<'gc>,
        slice: &Slice,
        audio_data: FlvAudioData<'_>,
    ) -> Result<(), NetstreamError> {
        let data = match audio_data.data {
            FlvAudioDataType::Raw(data)
            | FlvAudioDataType::AacSequenceHeader(data)
            | FlvAudioDataType::AacRaw(data) => slice.to_subslice(data),
        };
        let substream = match &mut write.audio_stream {
            Some((substream, _sound_stream_info)) => {
                if substream
                    .last_chunk()
                    .map(|lc| lc.end() > data.start())
                    .unwrap_or(false)
                {
                    // Reject repeats of existing tags.
                    // We need to do this because of lookahead - we will
                    // encounter the same audio tag multiple times as we buffer
                    // a few ahead for the audio backend.
                    // This assumes that tags are processed in-order - which
                    // should always be the case. Seeks should cancel the audio
                    // stream before processing new tags.
                    return Ok(());
                }

                substream
            }
            audio_stream => {
                // None
                let substream = Substream::new(slice.buffer().clone());
                let swf_format = SoundFormat {
                    compression: match audio_data.format {
                        FlvSoundFormat::LinearPCMPlatformEndian => {
                            AudioCompression::UncompressedUnknownEndian
                        }
                        FlvSoundFormat::Adpcm => AudioCompression::Adpcm,
                        FlvSoundFormat::MP3 => AudioCompression::Mp3,
                        FlvSoundFormat::LinearPCMLittleEndian => AudioCompression::Uncompressed,
                        FlvSoundFormat::Nellymoser16kHz => AudioCompression::Nellymoser16Khz,
                        FlvSoundFormat::Nellymoser8kHz => AudioCompression::Nellymoser8Khz,
                        FlvSoundFormat::Nellymoser => AudioCompression::Nellymoser,
                        FlvSoundFormat::G711ALawPCM => return Err(NetstreamError::UnknownCodec),
                        FlvSoundFormat::G711MuLawPCM => return Err(NetstreamError::UnknownCodec),
                        FlvSoundFormat::Aac => return Err(NetstreamError::UnknownCodec),
                        FlvSoundFormat::Speex => AudioCompression::Speex,
                        FlvSoundFormat::MP38kHz => AudioCompression::Mp3,
                        FlvSoundFormat::DeviceSpecific => return Err(NetstreamError::UnknownCodec),
                    },
                    sample_rate: match (audio_data.format, audio_data.rate) {
                        (FlvSoundFormat::MP38kHz, _) => 8_000,
                        (_, FlvSoundRate::R5_500) => 5_500,
                        (_, FlvSoundRate::R11_000) => 11_000,
                        (_, FlvSoundRate::R22_000) => 22_000,
                        (_, FlvSoundRate::R44_000) => 44_000,
                    },
                    is_stereo: match audio_data.sound_type {
                        FlvSoundType::Mono => false,
                        FlvSoundType::Stereo => true,
                    },
                    is_16_bit: match audio_data.size {
                        FlvSoundSize::Bits8 => false,
                        FlvSoundSize::Bits16 => true,
                    },
                };

                let sound_stream_head = SoundStreamInfo {
                    wrapping: SoundStreamWrapping::Unwrapped,
                    stream_format: swf_format,
                    num_samples_per_block: 0,
                    latency_seek: 0,
                };

                *audio_stream = Some((substream, sound_stream_head));

                &mut audio_stream.as_mut().unwrap().0
            }
        };

        Ok(substream.append(data)?)
    }

    /// Determine if the given sound is currently playing.
    fn sound_currently_playing(
        context: &mut UpdateContext<'gc>,
        sound: &Option<SoundInstanceHandle>,
    ) -> bool {
        sound
            .map(|si| context.audio.is_sound_playing(si))
            .unwrap_or(false)
    }

    /// Clean up after a sound instance that has finished playing.
    ///
    /// Generally speaking, streams are only to be used once. However, the
    /// audio backend will only retain information about sounds that are
    /// currently playing, so if the sound has finished since the last tick, we
    /// need to restart it.
    ///
    /// Intended to be called at the start of tag processing, before any new
    /// audio data has been streamed.
    fn cleanup_sound_stream(
        self,
        context: &mut UpdateContext<'gc>,
        write: &mut NetStreamData<'gc>,
    ) {
        if !Self::sound_currently_playing(context, &write.sound_instance) {
            write.audio_stream = None;
            write.sound_instance = None;
        }
    }

    /// Ensure that if we have queued up audio into a sound stream, that said
    /// stream gets sent over to the audio backend.
    ///
    /// Intended to be called at the end of tag processing. Audio processing
    /// should occur only after a minimum number of tags have been processed to
    /// avoid audio underruns.
    fn commit_sound_stream(
        self,
        context: &mut UpdateContext<'gc>,
        write: &mut NetStreamData<'gc>,
    ) -> Result<(), NetstreamError> {
        if !Self::sound_currently_playing(context, &write.sound_instance) {
            if let Some((substream, sound_stream_head)) = &mut write.audio_stream {
                let attached_to = write.attached_to;

                if let Some(mc) = attached_to {
                    write.sound_instance = Some(context.audio_manager.start_substream(
                        context.audio,
                        substream.clone(),
                        mc,
                        sound_stream_head,
                    )?);
                } else {
                    write.sound_instance = Some(
                        context
                            .audio
                            .start_substream(substream.clone(), sound_stream_head)?,
                    );
                }
            }
        }

        Ok(())
    }

    /// Attempt to sniff the stream type from data in the buffer.
    ///
    /// Returns true if the stream type was successfully sniffed. False
    /// indicates that there is either not enough data in the buffer, or the
    /// data is of an unrecognized format. This should be used as a signal to
    /// stop stream processing until new data has been retrieved.
    pub fn sniff_stream_type(self, context: &mut UpdateContext<'gc>) -> bool {
        let mut write = self.0.write(context.gc_context);
        let slice = write.buffer.to_full_slice();
        let buffer = slice.data();

        // A nonzero preload offset indicates that we tried and failed to
        // sniff the container format, so in that case do not process the
        // stream anymore.
        if write.preload_offset > 0 {
            return false;
        }

        match buffer.get(0..3) {
            Some([0x46, 0x4C, 0x56]) => {
                let mut reader = FlvReader::from_parts(&buffer, write.offset);
                match FlvHeader::parse(&mut reader) {
                    Ok(header) => {
                        write.offset = reader.into_parts().1;
                        write.preload_offset = write.offset;
                        write.stream_type = Some(NetStreamType::Flv {
                            header,
                            video_stream: None,
                            frame_id: 0,
                        });
                        true
                    }
                    Err(FlvError::EndOfData) => false,
                    Err(e) => {
                        //TODO: Fire an error event to AS & stop playing too
                        tracing::error!("FLV header parsing failed: {}", e);
                        write.preload_offset = 3;
                        false
                    }
                }
            }
            Some(magic) => {
                //Unrecognized signature
                //TODO: Fire an error event to AS & stop playing too
                tracing::error!("Unrecognized file signature: {:?}", magic);
                write.preload_offset = 3;
                if let Some(url) = &write.url {
                    if url.is_empty() {
                        return false;
                    }
                    let parsed_url = match context.navigator.resolve_url(url) {
                        Ok(parsed_url) => parsed_url,
                        Err(e) => {
                            tracing::error!(
                                "Could not parse URL because of {}, the corrupt URL was: {}",
                                e,
                                url
                            );
                            return false;
                        }
                    };
                    context.ui.display_unsupported_video(parsed_url);
                }
                false
            }
            None => false, //Data not yet loaded
        }
    }

    /// Process a parsed FLV video tag.
    ///
    /// `write` must be an active borrow of the current `NetStream`. `slice`
    /// must reference the underlying backing buffer.
    ///
    /// `tag_needs_preloading` indicates that this video tag has not been
    /// encountered before.
    fn flv_video_tag(
        self,
        context: &mut UpdateContext<'gc>,
        write: &mut NetStreamData<'gc>,
        slice: &Slice,
        video_data: FlvVideoData<'_>,
        tag_needs_preloading: bool,
    ) {
        let (video_handle, frame_id) = match write.stream_type {
            Some(NetStreamType::Flv {
                video_stream,
                frame_id,
                ..
            }) => (video_stream, frame_id),
            _ => unreachable!(),
        };
        let codec = VideoCodec::from_u8(video_data.codec_id as u8);
        let buffer = slice.data();

        match (video_handle, codec, video_data.data) {
            (maybe_video_handle, Some(codec), FlvVideoPacket::Data(mut data))
            | (
                maybe_video_handle,
                Some(codec),
                FlvVideoPacket::Vp6Data {
                    hadjust: _,
                    vadjust: _,
                    mut data,
                },
            ) => {
                //Some movies don't actually have metadata, so let's register a
                //dummy stream just in case. All the actual data in the registration
                //is lies, of course.
                let video_handle = match maybe_video_handle {
                    Some(stream) => stream,
                    None => {
                        match context.video.register_video_stream(
                            1,
                            (8, 8),
                            codec,
                            VideoDeblocking::UseVideoPacketValue,
                        ) {
                            Ok(new_handle) => {
                                match &mut write.stream_type {
                                    Some(NetStreamType::Flv { video_stream, .. }) => {
                                        *video_stream = Some(new_handle)
                                    }
                                    _ => unreachable!(),
                                }

                                new_handle
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Got error when registering FLV video stream: {}",
                                    e
                                );
                                return; //TODO: This originally breaks and halts tag processing
                            }
                        }
                    }
                };

                if codec == VideoCodec::ScreenVideo || codec == VideoCodec::ScreenVideoV2 {
                    // ScreenVideo streams consider the FLV
                    // video data byte to be integral to their
                    // own bitstream.
                    let offset = data.as_ptr() as usize - buffer.as_ptr() as usize;
                    let len = data.len();
                    data = buffer
                        .get(offset - 1..offset + len)
                        .expect("screenvideo flvs have video data bytes");
                }

                // NOTE: Currently, no implementation of the decoder backend actually requires
                if tag_needs_preloading {
                    let encoded_frame = EncodedFrame {
                        codec,
                        data, //TODO: ScreenVideo's decoder wants the FLV header bytes
                        frame_id,
                    };

                    if let Err(e) = context
                        .video
                        .preload_video_stream_frame(video_handle, encoded_frame)
                    {
                        tracing::error!("Preloading video frame {} failed: {}", frame_id, e);
                    }
                }

                let encoded_frame = EncodedFrame {
                    codec,
                    data, //TODO: ScreenVideo's decoder wants the FLV header bytes
                    frame_id,
                };

                match context.video.decode_video_stream_frame(
                    video_handle,
                    encoded_frame,
                    context.renderer,
                ) {
                    Ok(bitmap_info) => {
                        write.last_decoded_bitmap = Some(bitmap_info);
                        if let Some(mc) = write.attached_to {
                            mc.invalidate_cached_bitmap(context.gc_context);
                            *context.needs_render = true;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Decoding video frame {} failed: {}", frame_id, e);
                    }
                }
            }
            (_, _, FlvVideoPacket::CommandFrame(_command)) => {
                tracing::warn!("Stub: FLV command frame processing")
            }
            (Some(video_handle), _, FlvVideoPacket::AvcSequenceHeader(data)) => {
                match context
                    .video
                    .configure_video_stream_decoder(video_handle, data)
                {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Configuring video decoder {} failed: {}", frame_id, e);
                    }
                }
            }
            (
                Some(video_handle),
                Some(codec),
                FlvVideoPacket::AvcNalu {
                    composition_time_offset: _,
                    data,
                },
            ) => {
                let encoded_frame = EncodedFrame {
                    codec,
                    data,
                    frame_id,
                };

                match context.video.decode_video_stream_frame(
                    video_handle,
                    encoded_frame,
                    context.renderer,
                ) {
                    Ok(bitmap_info) => {
                        write.last_decoded_bitmap = Some(bitmap_info);
                    }
                    Err(e) => {
                        tracing::error!("Decoding video frame {} failed: {}", frame_id, e);
                    }
                }
            }
            (_, _, FlvVideoPacket::AvcEndOfSequence) => {
                tracing::warn!("Stub: FLV AVC/H.264 End of Sequence processing")
            }
            (_, None, _) => {
                tracing::error!(
                    "FLV video tag has invalid codec id {}",
                    video_data.codec_id as u8
                )
            }
            (None, _, _) => {
                tracing::error!("No video handle")
            }
        }

        match &mut write.stream_type {
            Some(NetStreamType::Flv {
                ref mut frame_id, ..
            }) => *frame_id += 1,
            _ => unreachable!(),
        };
    }

    /// Process a parsed FLV script tag.
    ///
    /// This function attempts to borrow the current `NetStream`, you must drop
    /// any existing borrows and pick them back up when you're done.
    ///
    /// `tag_needs_preloading` indicates that this script tag has not been
    /// encountered before.
    fn flv_script_tag(
        self,
        context: &mut UpdateContext<'gc>,
        script_data: FlvScriptData<'_>,
        tag_needs_preloading: bool,
    ) {
        let mut write = self.0.write(context.gc_context);
        let has_stream_already = match write.stream_type {
            Some(NetStreamType::Flv { video_stream, .. }) => video_stream.is_some(),
            _ => unreachable!(),
        };

        let mut width = None;
        let mut height = None;
        let mut video_codec_id = None;
        let mut frame_rate = None;
        let mut duration = None;

        for var in script_data.0 {
            if var.name == b"onMetaData" && !has_stream_already {
                match var.data.clone() {
                    FlvValue::Object(subvars) | FlvValue::EcmaArray(subvars) => {
                        for subvar in subvars {
                            match (subvar.name, subvar.data) {
                                (b"width", FlvValue::Number(val)) => width = Some(val),
                                (b"height", FlvValue::Number(val)) => height = Some(val),
                                (b"videocodecid", FlvValue::Number(val)) => {
                                    video_codec_id = Some(val)
                                }
                                (b"framerate", FlvValue::Number(val)) => frame_rate = Some(val),
                                (b"duration", FlvValue::Number(val)) => duration = Some(val),
                                _ => {}
                            }
                        }
                    }
                    _ => tracing::error!("Invalid FLV metadata tag!"),
                }
            }
            let avm_object = write.avm_object;
            drop(write);
            // This is necessary because the script callback functions can call back into
            // these methods, (e.g. NetStream::play), so we need to avoid holding a borrow
            // while the script data is being handled.
            let _ = self.handle_script_data(avm_object, context, var.name, var.data); // Any errors while trying to lookup or call AVM2 properties are silently swallowed.
            write = self.0.write(context.gc_context);
        }

        if tag_needs_preloading {
            if let (
                Some(width),
                Some(height),
                Some(video_codec_id),
                Some(frame_rate),
                Some(duration),
            ) = (width, height, video_codec_id, frame_rate, duration)
            {
                let num_frames = frame_rate * duration;
                if let Some(video_codec) = VideoCodec::from_u8(video_codec_id as u8) {
                    match context.video.register_video_stream(
                        num_frames as u32,
                        (width as u16, height as u16),
                        video_codec,
                        VideoDeblocking::UseVideoPacketValue,
                    ) {
                        Ok(stream_handle) => match &mut write.stream_type {
                            Some(NetStreamType::Flv { video_stream, .. }) => {
                                *video_stream = Some(stream_handle)
                            }
                            _ => unreachable!(),
                        },
                        Err(e) => {
                            tracing::error!("Got error when registering FLV video stream: {}", e)
                        }
                    }
                } else {
                    tracing::error!("FLV video stream has invalid codec ID {}", video_codec_id);
                }
            }
        }
    }

    /// Process stream data.
    ///
    /// `dt` is in milliseconds.
    pub fn tick(self, context: &mut UpdateContext<'gc>, dt: f64) {
        let seek_offset = self.0.write(context.gc_context).queued_seek_time.take();
        if let Some(offset) = seek_offset {
            self.execute_seek(context, offset);
        }

        // Paused streams deactivate themselves after seek processing.
        if !self.0.read().playing {
            StreamManager::deactivate(context, self);
            return;
        }

        // Ensure the container stream type is known before continuing.
        if self.0.read().stream_type.is_none() && !self.sniff_stream_type(context) {
            return;
        }

        let mut write = self.0.write(context.gc_context);

        self.cleanup_sound_stream(context, &mut write);
        let slice = write.buffer.to_full_slice();
        let buffer = slice.data();

        let max_time = write.stream_time + dt;
        let mut buffer_underrun = false;
        let mut error = false;
        let mut max_lookahead_audio_tags = 5;
        let mut is_lookahead_tag = false;

        //At this point we should know our stream type.
        if matches!(write.stream_type, Some(NetStreamType::Flv { .. })) {
            let mut reader = FlvReader::from_parts(&buffer, write.offset);

            loop {
                let tag = FlvTag::parse(&mut reader);
                if let Err(e) = tag {
                    // `is_lookahead_tag` gets set once we start reading tags
                    // after the end & won't ever be set back. We don't want
                    // error states or playback ending to trip until we run
                    // those tags "for realsies"
                    if !is_lookahead_tag && matches!(e, FlvError::EndOfData) {
                        buffer_underrun = true;
                    } else if !is_lookahead_tag {
                        //Corrupt tag or out of data
                        tracing::error!("FLV tag parsing failed: {}", e);
                        error = true;
                    }

                    break;
                }

                let tag = tag.expect("valid tag");
                is_lookahead_tag = tag.timestamp as f64 >= max_time; //FLV timestamps are also ms
                if is_lookahead_tag && max_lookahead_audio_tags == 0 {
                    break;
                }

                let tag_needs_preloading = reader.stream_position().expect("valid position")
                    as usize
                    >= write.preload_offset;

                match tag.data {
                    FlvTagData::Audio(audio_data) => {
                        if is_lookahead_tag {
                            max_lookahead_audio_tags -= 1;
                        }

                        if let Err(e) = self.flv_audio_tag(&mut write, &slice, audio_data) {
                            //TODO: Fire an error event at AS.
                            tracing::error!("Error committing sound stream: {}", e);
                        }
                    }
                    FlvTagData::Video(video_data) if !is_lookahead_tag => self.flv_video_tag(
                        context,
                        &mut write,
                        &slice,
                        video_data,
                        tag_needs_preloading,
                    ),
                    FlvTagData::Script(script_data) if !is_lookahead_tag => {
                        drop(write);
                        self.flv_script_tag(context, script_data, tag_needs_preloading);
                        write = self.0.write(context.gc_context);
                    }
                    FlvTagData::Invalid(e) => {
                        tracing::error!("FLV data parsing failed: {}", e)
                    }
                    FlvTagData::Video(_) | FlvTagData::Script(_) => {}
                }

                if !is_lookahead_tag {
                    write.offset = reader
                        .stream_position()
                        .expect("FLV reader stream position")
                        as usize;
                    write.preload_offset = max(write.offset, write.preload_offset);
                }
            }
        }

        write.stream_time = max_time;
        if let Err(e) = self.commit_sound_stream(context, &mut write) {
            //TODO: Fire an error event at AS.
            tracing::error!("Error committing sound stream: {}", e);
        }
        drop(write);

        if buffer_underrun {
            let is_end_of_video = self.0.read().expected_length.is_none();

            self.trigger_status_event(
                context,
                vec![("code", "NetStream.Buffer.Flush"), ("level", "status")],
            );

            if is_end_of_video {
                self.trigger_status_event(
                    context,
                    vec![("code", "NetStream.Play.Stop"), ("level", "status")],
                );
            }

            self.trigger_status_event(
                context,
                vec![("code", "NetStream.Buffer.Empty"), ("level", "status")],
            );

            if is_end_of_video {
                self.pause(context, false);
            }
        }

        if error {
            //TODO: Fire an error event at AS.
            self.pause(context, false);
        }
    }

    pub fn last_decoded_bitmap(self) -> Option<BitmapInfo> {
        self.0.read().last_decoded_bitmap.clone()
    }

    /// Trigger a status event on the stream.
    pub fn trigger_status_event(
        self,
        context: &mut UpdateContext<'gc>,
        values: Vec<(impl Into<AvmString<'gc>>, impl Into<AvmString<'gc>>)>,
    ) {
        let object = self.0.read().avm_object;
        match object {
            Some(AvmObject::Avm1(object)) => {
                let root = context.stage.root_clip().expect("root");
                let object_proto = context.avm1.prototypes().object;
                let mut activation = Avm1Activation::from_nothing(
                    context,
                    Avm1ActivationIdentifier::root("[NetStream Status Event]"),
                    root,
                );
                let info_object =
                    Avm1ScriptObject::new(activation.context.gc_context, Some(object_proto));

                for (key, value) in values {
                    info_object
                        .set(key.into(), Avm1Value::String(value.into()), &mut activation)
                        .expect("valid set");
                }

                if let Err(e) = object.call_method(
                    "onStatus".into(),
                    &[info_object.into()],
                    &mut activation,
                    Avm1ExecutionReason::Special,
                ) {
                    tracing::error!(
                        "Got error when dispatching AVM1 onStatus event from NetStream: {}",
                        e
                    );
                }
            }
            Some(AvmObject::Avm2(object)) => {
                let domain = context.avm2.stage_domain();
                let mut activation = Avm2Activation::from_domain(context, domain);
                let net_status_event =
                    Avm2EventObject::net_status_event(&mut activation, "netStatus", values);
                Avm2::dispatch_event(activation.context, net_status_event, object);
            }
            None => {}
        }
    }

    fn handle_script_data(
        self,
        avm_object: Option<AvmObject<'gc>>,
        context: &mut UpdateContext<'gc>,
        variable_name: &[u8],
        variable_data: FlvValue,
    ) -> Result<(), Avm2Error<'gc>> {
        match avm_object {
            Some(AvmObject::Avm1(object)) => {
                let avm_string_name = AvmString::new_utf8_bytes(context.gc_context, variable_name);

                let root = context.stage.root_clip().expect("root");
                let mut activation = Avm1Activation::from_nothing(
                    context,
                    Avm1ActivationIdentifier::root(format!("[FLV {}]", avm_string_name)),
                    root,
                );

                let avm1_object_value = variable_data.to_avm1_value(&mut activation);

                if let Err(e) = object.call_method(
                    avm_string_name,
                    &[avm1_object_value],
                    &mut activation,
                    Avm1ExecutionReason::Special,
                ) {
                    tracing::error!(
                        "Got error when dispatching AVM1 {} script data handler from NetStream: {}",
                        avm_string_name,
                        e,
                    );
                }
            }
            Some(AvmObject::Avm2(_object)) => {
                let mut activation = Avm2Activation::from_nothing(context);
                let client_object = self
                    .client()
                    .expect("Client should be initialized if script data is being accessed");

                let data_object = variable_data.to_avm2_value(&mut activation);

                client_object.call_public_property(
                    AvmString::new_utf8_bytes(activation.context.gc_context, variable_name),
                    &[data_object],
                    &mut activation,
                )?;
            }
            None => {}
        };

        Ok(())
    }
}
