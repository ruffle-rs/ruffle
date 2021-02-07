//! Pure software video decoding backend.

use crate::backend::render::{BitmapInfo, RenderBackend};
use crate::backend::video::{
    EncodedFrame, Error, FrameDependency, VideoBackend, VideoStreamHandle,
};
use generational_arena::Arena;
use swf::{VideoCodec, VideoDeblocking};

/// A single preloaded video stream.
pub enum VideoStream {}

/// Desktop video backend.
///
/// TODO: Currently, this just proxies out to `ruffle_h263`, in the future it
/// should support desktop media playback APIs so we can take advantage of
/// hardware-accelerated video decoding.
pub struct SoftwareVideoBackend {
    streams: Arena<VideoStream>,
}

impl Default for SoftwareVideoBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftwareVideoBackend {
    pub fn new() -> Self {
        Self {
            streams: Arena::new(),
        }
    }
}

impl VideoBackend for SoftwareVideoBackend {
    fn register_video_stream(
        &mut self,
        _num_frames: u32,
        _size: (u16, u16),
        codec: VideoCodec,
        _filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        Err(format!("Unsupported video codec type {:?}", codec).into())
    }

    fn preload_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        _encoded_frame: EncodedFrame<'_>,
    ) -> Result<FrameDependency, Error> {
        let _stream = self
            .streams
            .get_mut(stream)
            .ok_or("Unregistered video stream")?;

        unreachable!()
    }

    fn decode_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        _encoded_frame: EncodedFrame<'_>,
        _renderer: &mut dyn RenderBackend,
    ) -> Result<BitmapInfo, Error> {
        let _stream = self
            .streams
            .get_mut(stream)
            .ok_or("Unregistered video stream")?;

        unreachable!()
    }
}
