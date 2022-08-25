use crate::error::Error;
use crate::frame::{EncodedFrame, FrameDependency};
use crate::VideoStreamHandle;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::BitmapInfo;
use swf::{VideoCodec, VideoDeblocking};

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
