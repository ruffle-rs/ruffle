//! Video decoder backends

use crate::backend::render::RenderBackend;
use generational_arena::{Arena, Index};
use ruffle_render::bitmap::BitmapInfo;
use swf::{VideoCodec, VideoDeblocking};

mod software;

pub use crate::backend::video::software::SoftwareVideoBackend;

pub type VideoStreamHandle = Index;

pub type Error = Box<dyn std::error::Error>;

/// An encoded video frame of some video codec.
pub struct EncodedFrame<'a> {
    /// The codec used to encode the frame.
    pub codec: VideoCodec,

    /// The raw bitstream data to funnel into the codec.
    pub data: &'a [u8],

    /// A caller-specified frame ID. Frame IDs must be consistent between
    /// subsequent uses of the same data stream.
    pub frame_id: u32,
}

impl<'a> EncodedFrame<'a> {
    /// Borrow this frame's data.
    pub fn data(&'a self) -> &'a [u8] {
        self.data
    }
}

/// A decoded frame of video in RGBA format.
struct DecodedFrame {
    width: u16,
    height: u16,
    rgba: Vec<u8>,
}

/// What dependencies a given video frame has on any previous frames.
#[derive(Copy, Clone, Debug)]
pub enum FrameDependency {
    /// This frame has no reference frames and can be seeked to at any time.
    None,

    /// This frame has some number of reference frames that prohibit any
    /// out-of-order decoding.
    ///
    /// The only legal way to decode a `Past` frame is to decode every prior
    /// frame from the last `None` frame. In the event that there is no prior
    /// `None` frame, then video decoding should start from the beginning.
    Past,
}

impl FrameDependency {
    /// Determine if this given frame is a keyframe.
    ///
    /// A keyframe is a frame that can be independently seeked to without
    /// decoding any prior or future frames.
    pub fn is_keyframe(self) -> bool {
        matches!(self, FrameDependency::None)
    }
}

/// A backend that provides access to some number of video decoders.
///
/// Implementations of `VideoBackend` are not required to actually support
/// decoding any video formats. However, they must interoperate with at least
/// one `RenderBackend` so that renderable video frames may be passed from the
/// decoder to the renderer.
pub trait VideoBackend {
    /// Register a new video stream.
    ///
    /// Most of the parameters provided to this function are advisory: the
    /// actual video data stream provided to the decoder may vary in size or
    /// number of frames. This function should return an `Error` if it is not
    /// possible to decode video with the given parameters.
    fn register_video_stream(
        &mut self,
        num_frames: u32,
        size: (u16, u16),
        codec: VideoCodec,
        filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error>;

    /// Preload a frame of a given video stream.
    ///
    /// No decoding is intended to happen at this point in time. Instead, the
    /// video data should be inspected to determine inter-frame dependencies
    /// between this and any previous frames in the stream.
    ///
    /// Frames should be preloaded in the order that they are recieved.
    ///
    /// Any dependencies listed here are inherent to the video bitstream. The
    /// containing video stream is also permitted to introduce additional
    /// interframe dependencies.
    fn preload_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        encoded_frame: EncodedFrame<'_>,
    ) -> Result<FrameDependency, Error>;

    /// Decode a frame of a given video stream.
    ///
    /// This function is provided the external index of the frame, the codec
    /// used to decode the data, and what codec to decode it with. The codec
    /// provided here must match the one used to register the video stream.
    ///
    /// Frames may be decoded in any order that does not violate the frame
    /// dependencies declared by the output of `preload_video_stream_frame`.
    ///
    /// The resulting `BitmapInfo` will be renderable only on the given
    /// `RenderBackend`. `VideoBackend` implementations are allowed to return
    /// an error if a drawable bitmap cannot be produced for the given
    /// renderer.
    ///
    /// Any previously returned bitmaps may be updated, invalidated, or
    /// reclaimed by whatever means the decoder implementation chooses.
    fn decode_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        encoded_frame: EncodedFrame<'_>,
        renderer: &mut dyn RenderBackend,
    ) -> Result<BitmapInfo, Error>;
}

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
        Err("Video decoding not implemented".into())
    }
}
