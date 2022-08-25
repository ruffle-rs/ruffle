use crate::decoder::VideoDecoder;
use generational_arena::Arena;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::{Bitmap, BitmapFormat, BitmapHandle, BitmapInfo};
use ruffle_video::backend::VideoBackend;
use ruffle_video::error::Error;
use ruffle_video::frame::{EncodedFrame, FrameDependency};
use ruffle_video::VideoStreamHandle;
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
        size: (u16, u16),
        codec: VideoCodec,
        _filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        let decoder: Box<dyn VideoDecoder> = match codec {
            #[cfg(feature = "h263")]
            VideoCodec::H263 => Box::new(h263::H263Decoder::new()),
            #[cfg(feature = "vp6")]
            VideoCodec::Vp6 => Box::new(vp6::Vp6Decoder::new(false, size)),
            #[cfg(feature = "vp6")]
            VideoCodec::Vp6WithAlpha => Box::new(vp6::Vp6Decoder::new(true, size)),
            #[cfg(feature = "screenvideo")]
            VideoCodec::ScreenVideo => Box::new(screen::ScreenVideoDecoder::new()),
            other => return Err(Error::UnsupportedCodec(other)),
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
            .ok_or(Error::VideoStreamIsNotRegistered)?;

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
            .ok_or(Error::VideoStreamIsNotRegistered)?;

        let frame = stream.decoder.decode_frame(encoded_frame)?;
        let handle = if let Some(bitmap) = stream.bitmap {
            renderer.update_texture(bitmap, frame.width.into(), frame.height.into(), frame.rgba)?
        } else {
            let bitmap = Bitmap::new(
                frame.width.into(),
                frame.height.into(),
                BitmapFormat::Rgba,
                frame.rgba,
            );
            renderer.register_bitmap(bitmap)?
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
pub struct VideoStream {
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
