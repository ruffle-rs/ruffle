use crate::backend::VideoBackend;
use crate::error::Error;
use crate::frame::{EncodedFrame, FrameDependency};
use crate::VideoStreamHandle;
use generational_arena::Arena;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::BitmapInfo;
use swf::{VideoCodec, VideoDeblocking};

pub struct NullVideoBackend {
    streams: Arena<()>,
}

/// Implementation of video that does not decode any video.
///
/// Specifically:
///
///  * Registering a video stream succeeds but does nothing
///  * All video frames are silently marked as keyframes (`None` dependency)
///  * Video stream decoding fails with an error that video decoding is
///    unimplemented
impl NullVideoBackend {
    pub fn new() -> Self {
        Self {
            streams: Arena::new(),
        }
    }
}

impl Default for NullVideoBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoBackend for NullVideoBackend {
    fn register_video_stream(
        &mut self,
        _num_frames: u32,
        _size: (u16, u16),
        _codec: VideoCodec,
        _filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        Ok(self.streams.insert(()))
    }

    fn preload_video_stream_frame(
        &mut self,
        _stream: VideoStreamHandle,
        _encoded_frame: EncodedFrame<'_>,
    ) -> Result<FrameDependency, Error> {
        Ok(FrameDependency::None)
    }

    fn decode_video_stream_frame(
        &mut self,
        _stream: VideoStreamHandle,
        _encoded_frame: EncodedFrame<'_>,
        _renderer: &mut dyn RenderBackend,
    ) -> Result<BitmapInfo, Error> {
        Err(Error::DecodingNotSupported)
    }
}
