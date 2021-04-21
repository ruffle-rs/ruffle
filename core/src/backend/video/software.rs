//! Pure software video decoding backend.

use crate::backend::render::{BitmapHandle, BitmapInfo, RenderBackend};
use crate::backend::video::{
    DecodedFrame, EncodedFrame, Error, FrameDependency, VideoBackend, VideoStreamHandle,
};
use generational_arena::Arena;
use swf::{VideoCodec, VideoDeblocking};

/// Software video backend that proxies to CPU-only codec implementations that
/// ship with Ruffle.
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
    #[allow(unreachable_code, unused_variables)]
    fn register_video_stream(
        &mut self,
        _num_frames: u32,
        _size: (u16, u16),
        codec: VideoCodec,
        _filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        let decoder: Box<dyn VideoDecoder> = match codec {
            #[cfg(feature = "h263")]
            VideoCodec::H263 => Box::new(h263::H263Decoder::new()),
            _ => return Err(format!("Unsupported video codec type {:?}", codec).into()),
        };
        let stream = VideoStream::new(decoder);
        let stream_handle = self.streams.insert(stream);
        Ok(stream_handle)
    }

    fn preload_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        encoded_frame: EncodedFrame<'_>,
    ) -> Result<FrameDependency, Error> {
        let stream = self
            .streams
            .get_mut(stream)
            .ok_or("Unregistered video stream")?;

        stream.decoder.preload_frame(encoded_frame)
    }

    fn decode_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        encoded_frame: EncodedFrame<'_>,
        renderer: &mut dyn RenderBackend,
    ) -> Result<BitmapInfo, Error> {
        let stream = self
            .streams
            .get_mut(stream)
            .ok_or("Unregistered video stream")?;

        let frame = stream.decoder.decode_frame(encoded_frame)?;
        let handle = if let Some(bitmap) = stream.bitmap {
            renderer.update_texture(bitmap, frame.width.into(), frame.height.into(), frame.rgba)?
        } else {
            renderer.register_bitmap_raw(frame.width.into(), frame.height.into(), frame.rgba)?
        };
        stream.bitmap = Some(handle);

        Ok(BitmapInfo {
            handle,
            width: frame.width,
            height: frame.height,
        })
    }
}

/// A single preloaded video stream.
struct VideoStream {
    bitmap: Option<BitmapHandle>,
    decoder: Box<dyn VideoDecoder>,
}

impl VideoStream {
    fn new(decoder: Box<dyn VideoDecoder>) -> Self {
        Self {
            decoder,
            bitmap: None,
        }
    }
}

/// Trait for video decoders.
/// This should be implemented for each video codec.
trait VideoDecoder {
    /// Preload a frame.
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
    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error>;

    /// Decode a frame of a given video stream.
    ///
    /// This function is provided the external index of the frame, the codec
    /// used to decode the data, and what codec to decode it with. The codec
    /// provided here must match the one used to register the video stream.
    ///
    /// Frames may be decoded in any order that does not violate the frame
    /// dependencies declared by the output of `preload_video_stream_frame`.
    ///
    /// The decoded frame should be returned. An `Error` can be returned if
    /// a drawable bitmap can not be produced.
    fn decode_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<DecodedFrame, Error>;
}

#[cfg(feature = "h263")]
mod h263 {
    use crate::backend::video::software::VideoDecoder;
    use crate::backend::video::{DecodedFrame, EncodedFrame, Error, FrameDependency};
    use h263_rs::parser::{decode_picture, H263Reader};
    use h263_rs::{DecoderOption, H263State, PictureTypeCode};
    use h263_rs_yuv::bt601::yuv420_to_rgba;

    /// H263 video decoder.
    pub struct H263Decoder(H263State);

    impl H263Decoder {
        pub fn new() -> Self {
            Self(H263State::new(DecoderOption::SORENSON_SPARK_BITSTREAM))
        }
    }

    impl VideoDecoder for H263Decoder {
        fn preload_frame(
            &mut self,
            encoded_frame: EncodedFrame<'_>,
        ) -> Result<FrameDependency, Error> {
            let mut reader = H263Reader::from_source(encoded_frame.data());
            let picture =
                decode_picture(&mut reader, DecoderOption::SORENSON_SPARK_BITSTREAM, None)?
                    .ok_or("Picture in video stream is not a picture")?;

            match picture.picture_type {
                PictureTypeCode::IFrame => Ok(FrameDependency::None),
                PictureTypeCode::PFrame => Ok(FrameDependency::Past),
                PictureTypeCode::DisposablePFrame => Ok(FrameDependency::Past),
                _ => Err("Invalid picture type code!".into()),
            }
        }

        fn decode_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<DecodedFrame, Error> {
            let mut reader = H263Reader::from_source(encoded_frame.data());

            self.0.decode_next_picture(&mut reader)?;

            let picture = self
                .0
                .get_last_picture()
                .expect("Decoding a picture should let us grab that picture");

            let (width, height) = picture
                .format()
                .into_width_and_height()
                .ok_or("H.263 decoder error!")?;
            let chroma_width = picture.chroma_samples_per_row();
            let (y, b, r) = picture.as_yuv();
            let rgba = yuv420_to_rgba(y, b, r, width.into(), chroma_width);
            Ok(DecodedFrame {
                width,
                height,
                rgba,
            })
        }
    }

    impl Default for H263Decoder {
        fn default() -> Self {
            Self::new()
        }
    }
}
