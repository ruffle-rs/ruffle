//! Pure software video decoding backend.

use crate::backend::render::{BitmapHandle, BitmapInfo, RenderBackend};
use crate::backend::video::{
    EncodedFrame, Error, FrameDependency, VideoBackend, VideoStreamHandle,
};
use generational_arena::Arena;
use swf::{VideoCodec, VideoDeblocking};
use vp6_dec_rs::VP6State;

/// A single preloaded video stream.
pub enum VideoStream {
    /// A VP6 video stream.
    Vp6(VP6State, Option<BitmapHandle>),
}

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
    fn register_video_stream(
        &mut self,
        _num_frames: u32,
        _size: (u16, u16),
        codec: VideoCodec,
        _filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        match codec {
            VideoCodec::Vp6 => Ok(self.streams.insert(VideoStream::Vp6(VP6State::new(), None))),
            _ => Err(format!("Unsupported video codec type {:?}", codec).into()),
        }
    }

    fn preload_video_stream_frame(
        &mut self,
        stream: VideoStreamHandle,
        _encoded_frame: EncodedFrame<'_>,
    ) -> Result<FrameDependency, Error> {
        let stream = self
            .streams
            .get_mut(stream)
            .ok_or("Unregistered video stream")?;

        match stream {
            VideoStream::Vp6(_state, _last_bitmap) => {
                // TODO actually parse the frame header and report correctly
                Ok(FrameDependency::None)
            }
        }
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

        match stream {
            VideoStream::Vp6(state, last_bitmap) => {
                let (rgba, (width, height)) = state.decode(encoded_frame.data);

                let handle = if let Some(lb) = last_bitmap {
                    renderer.update_texture(*lb, width as u32, height as u32, rgba)?
                } else {
                    renderer.register_bitmap_raw(width as u32, height as u32, rgba)?
                };

                *last_bitmap = Some(handle);

                Ok(BitmapInfo {
                    handle,
                    width: width as u16,
                    height: height as u16,
                })
            }
        }
    }
}
