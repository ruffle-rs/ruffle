//! NetStream implementation

use crate::backend::navigator::Request;
use crate::context::UpdateContext;
use crate::loader::Error;
use crate::string::AvmString;
use flv_rs::{
    AudioData as FlvAudioData, Error as FlvError, FlvReader, Header as FlvHeader,
    ScriptData as FlvScriptData, Tag as FlvTag, TagData as FlvTagData, Value as FlvValue,
    VideoData as FlvVideoData,
};
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_video::VideoStreamHandle;
use ruffle_wstr::WStr;
use std::cmp::max;
use std::io::Seek;
use swf::{VideoCodec, VideoDeblocking};

/// Manager for all media streams.
///
/// This does *not* handle data transport; which is delegated to `LoadManager`.
/// `StreamManager` *only* handles decoding or encoding of relevant media
/// streams.
#[derive(Collect)]
#[collect(no_drop)]
pub struct StreamManager<'gc> {
    /// List of actively playing streams.
    ///
    /// This is not the total list of all created NetStreams; only the ones
    /// that have been configured to play media.
    playing_streams: Vec<NetStream<'gc>>,
}

impl<'gc> Default for StreamManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> StreamManager<'gc> {
    pub fn new() -> Self {
        StreamManager {
            playing_streams: Vec::new(),
        }
    }

    pub fn ensure_playing(context: &mut UpdateContext<'_, 'gc>, stream: NetStream<'gc>) {
        if !context.stream_manager.playing_streams.contains(&stream) {
            context.stream_manager.playing_streams.push(stream);
        }
    }

    pub fn ensure_paused(context: &mut UpdateContext<'_, 'gc>, stream: NetStream<'gc>) {
        let index = context
            .stream_manager
            .playing_streams
            .iter()
            .position(|x| *x == stream);
        if let Some(index) = index {
            context.stream_manager.playing_streams.remove(index);
        }
    }

    pub fn toggle_paused(context: &mut UpdateContext<'_, 'gc>, stream: NetStream<'gc>) {
        let index = context
            .stream_manager
            .playing_streams
            .iter()
            .position(|x| *x == stream);
        if let Some(index) = index {
            context.stream_manager.playing_streams.remove(index);
        } else {
            context.stream_manager.playing_streams.push(stream);
        }
    }

    /// Process all playing media streams.
    ///
    /// This is an unlocked timestep; the `dt` parameter indicates how many
    /// milliseconds have elapsed since the last tick. This is intended to
    /// support video framerates separate from the Stage frame rate.
    ///
    /// This does not borrow `&mut self` as we need the `UpdateContext`, too.
    pub fn tick(context: &mut UpdateContext<'_, 'gc>, dt: f64) {
        let streams = context.stream_manager.playing_streams.clone();
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
pub struct NetStream<'gc>(GcCell<'gc, NetStreamData>);

impl<'gc> PartialEq for NetStream<'gc> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<'gc> Eq for NetStream<'gc> {}

/// The current type of the data in the stream buffer.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub enum NetStreamType {
    /// The stream is an FLV.
    Flv {
        header: FlvHeader,
        stream: Option<VideoStreamHandle>,
    },
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct NetStreamData {
    /// All data currently loaded in the stream.
    buffer: Vec<u8>,

    /// The buffer position that we are currently seeking to.
    offset: usize,

    /// The buffer position for processing incoming data.
    ///
    /// Certain data, such as the header or metadata of an FLV, should only
    /// ever be processed one time, even if we seek backwards to it later on.
    /// We call this data "preloaded", whether or not there is actually a
    /// separate preload step for that given format.
    preload_offset: usize,

    /// The current stream type, if known.
    stream_type: Option<NetStreamType>,

    /// The current seek offset in the stream.
    stream_time: f64,
}

impl<'gc> NetStream<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(
            gc_context,
            NetStreamData {
                buffer: Vec::new(),
                offset: 0,
                preload_offset: 0,
                stream_type: None,
                stream_time: 0.0,
            },
        ))
    }

    pub fn load_buffer(self, gc_context: MutationContext<'gc, '_>, data: &mut Vec<u8>) {
        self.0.write(gc_context).buffer.append(data);
    }

    pub fn report_error(self, _error: Error) {
        //TODO: Report an `asyncError` to AVM1 or 2.
    }

    pub fn bytes_loaded(self) -> usize {
        self.0.read().buffer.len()
    }

    pub fn bytes_total(self) -> usize {
        self.0.read().buffer.len()
    }

    /// Start playing media from this NetStream.
    ///
    /// If `name` is specified, this will also trigger streaming download of
    /// the given resource. Otherwise, the stream will play whatever data is
    /// available in the buffer.
    pub fn play(self, context: &mut UpdateContext<'_, 'gc>, name: Option<AvmString<'gc>>) {
        if let Some(name) = name {
            let request = Request::get(name.to_string());
            context
                .load_manager
                .load_netstream(context.player.clone(), self, request);
        }

        StreamManager::ensure_playing(context, self);
    }

    /// Pause stream playback.
    pub fn pause(self, context: &mut UpdateContext<'_, 'gc>) {
        StreamManager::ensure_paused(context, self);
    }

    /// Resume stream playback.
    pub fn resume(self, context: &mut UpdateContext<'_, 'gc>) {
        StreamManager::ensure_playing(context, self);
    }

    /// Resume stream playback if paused, pause otherwise.
    pub fn toggle_paused(self, context: &mut UpdateContext<'_, 'gc>) {
        StreamManager::toggle_paused(context, self);
    }

    pub fn tick(self, context: &mut UpdateContext<'_, 'gc>, dt: f64) {
        let mut write = self.0.write(context.gc_context);

        // First, try to sniff the stream's container format from headers.
        if write.stream_type.is_none() {
            // A nonzero preload offset indicates that we tried and failed to
            // sniff the container format, so in that case do not process the
            // stream anymore.
            if write.preload_offset > 0 {
                return;
            }

            match write.buffer.get(0..3) {
                Some([0x46, 0x4C, 0x56]) => {
                    let mut reader = FlvReader::from_parts(&write.buffer, write.offset);
                    match FlvHeader::parse(&mut reader) {
                        Ok(header) => {
                            write.offset = reader.into_parts().1;
                            write.preload_offset = write.offset;
                            write.stream_type = Some(NetStreamType::Flv {
                                header,
                                stream: None,
                            });
                        }
                        Err(FlvError::EndOfData) => return,
                        Err(e) => {
                            //TODO: Fire an error event to AS & stop playing too
                            tracing::error!("FLV header parsing failed: {}", e);
                            write.preload_offset = 3;
                            return;
                        }
                    }
                }
                _ => {
                    write.preload_offset = 3;
                    return;
                }
            }
        }

        let end_time = write.stream_time + dt;

        //At this point we should know our stream type.
        if matches!(write.stream_type, Some(NetStreamType::Flv { .. })) {
            let mut reader = FlvReader::from_parts(&write.buffer, write.offset);

            loop {
                let tag = FlvTag::parse(&mut reader);
                if let Err(e) = tag {
                    //Corrupt tag or out of data
                    if !matches!(e, FlvError::EndOfData) {
                        //TODO: Stop the stream so we don't repeatedly yield the same error
                        //and fire an error event to AS
                        tracing::error!("FLV tag parsing failed: {}", e);
                    }

                    break;
                }

                let tag = tag.expect("valid tag");
                if tag.timestamp as f64 >= end_time {
                    //All tags processed
                    if let Err(e) = FlvTag::skip_back(&mut reader) {
                        tracing::error!("FLV skip back failed: {}", e);
                    }

                    break;
                }

                let tag_needs_preloading = reader.stream_position().expect("valid position")
                    as usize
                    >= write.preload_offset;

                match tag.data {
                    FlvTagData::Audio(FlvAudioData {
                        format,
                        rate,
                        size,
                        sound_type,
                        data,
                    }) => {
                        tracing::warn!("Stub: Stream audio processing");
                    }
                    FlvTagData::Video(FlvVideoData {
                        frame_type,
                        codec_id,
                        data,
                    }) => {
                        tracing::warn!("Stub: Stream video processing");
                    }
                    FlvTagData::Script(FlvScriptData(vars)) => {
                        let has_stream_already = match write.stream_type {
                            Some(NetStreamType::Flv { stream, .. }) => stream.is_some(),
                            _ => unreachable!(),
                        };

                        let mut width = None;
                        let mut height = None;
                        let mut video_codec_id = None;
                        let mut frame_rate = None;
                        let mut duration = None;

                        for var in vars {
                            if var.name == b"onMetaData" && !has_stream_already {
                                match var.data {
                                    FlvValue::Object(subvars) => {
                                        for subvar in subvars {
                                            match (subvar.name, subvar.data) {
                                                (b"width", FlvValue::Number(val)) => {
                                                    width = Some(val)
                                                }
                                                (b"height", FlvValue::Number(val)) => {
                                                    height = Some(val)
                                                }
                                                (b"videocodecid", FlvValue::Number(val)) => {
                                                    video_codec_id = Some(val)
                                                }
                                                (b"framerate", FlvValue::Number(val)) => {
                                                    frame_rate = Some(val)
                                                }
                                                (b"duration", FlvValue::Number(val)) => {
                                                    duration = Some(val)
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => tracing::error!("Invalid FLV metadata tag!"),
                                }
                            } else {
                                tracing::warn!(
                                    "Stub: Stream data processing (name: {})",
                                    WStr::from_units(var.name)
                                );
                            }
                        }

                        let (_, position) = reader.into_parts();

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
                                if let Some(video_codec) = VideoCodec::from_u8(video_codec_id as u8)
                                {
                                    match context.video.register_video_stream(
                                        num_frames as u32,
                                        (width as u16, height as u16),
                                        video_codec,
                                        VideoDeblocking::UseVideoPacketValue,
                                    ) {
                                        Ok(stream_handle) => match &mut write.stream_type {
                                            Some(NetStreamType::Flv { stream, .. }) => {
                                                *stream = Some(stream_handle)
                                            }
                                            _ => unreachable!(),
                                        },
                                        Err(e) => tracing::error!(
                                            "Got error when registring FLV video stream: {}",
                                            e
                                        ),
                                    }
                                } else {
                                    tracing::error!(
                                        "FLV video stream has invalid codec ID {}",
                                        video_codec_id
                                    );
                                }
                            }
                        }

                        reader = FlvReader::from_parts(&write.buffer, position);
                    }
                    FlvTagData::Invalid(e) => {
                        tracing::error!("FLV data parsing failed: {}", e)
                    }
                }

                // We cannot mutate stream state while also holding an active
                // reader or any tags.
                let (_, position) = reader.into_parts();
                write.offset = position;
                write.preload_offset = max(write.offset, write.preload_offset);
                reader = FlvReader::from_parts(&write.buffer, position);
            }
        }
    }
}
