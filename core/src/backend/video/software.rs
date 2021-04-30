//! Pure software video decoding backend.

use crate::backend::render::{BitmapHandle, BitmapInfo, RenderBackend};
use crate::backend::video::{
    EncodedFrame, Error, FrameDependency, VideoBackend, VideoStreamHandle,
};
use generational_arena::Arena;
use swf::{VideoCodec, VideoDeblocking};
use vp6_dec_rs::Vp6State;

/// A single preloaded video stream.
pub enum VideoStream {
    /// A VP6 video stream, with or without alpha channel.
    Vp6(Vp6State, (u16, u16), Option<BitmapHandle>),
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
        size: (u16, u16),
        codec: VideoCodec,
        _filter: VideoDeblocking,
    ) -> Result<VideoStreamHandle, Error> {
        match codec {
            VideoCodec::Vp6 => {
                Ok(self
                    .streams
                    .insert(VideoStream::Vp6(Vp6State::new(false), size, None)))
            }
            VideoCodec::Vp6WithAlpha => {
                Ok(self
                    .streams
                    .insert(VideoStream::Vp6(Vp6State::new(true), size, None)))
            }
            _ => Err(format!("Unsupported video codec type {:?}", codec).into()),
        }
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

        match stream {
            VideoStream::Vp6(_state, _bounds, _last_bitmap) => {
                // Luckily the very first bit of the encoded frames is exactly
                // this flag, so we don't have to bother asking any "proper"
                // decoder or parser.
                Ok(
                    if !encoded_frame.data.is_empty() && (encoded_frame.data[0] & 0b_1000_0000) == 0
                    {
                        FrameDependency::None
                    } else {
                        FrameDependency::Past
                    },
                )
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
            VideoStream::Vp6(state, bounds, last_bitmap) => {
                let (mut rgba, (mut width, mut height)) = state.decode(encoded_frame.data);

                if width < bounds.0 as usize || height < bounds.1 as usize {
                    log::warn!("A VP6 video frame is smaller than the bounds of the stream it belongs in. This is not supported.");
                    // Flash Player just produces a black image in this case!
                }

                if width > bounds.0 as usize {
                    // Removing the unwanted pixels on the right edge (most commonly: unused pieces of macroblocks)
                    // by squishing all the rows tightly next to each other.
                    // Even though the vp6 decoder in FFmpeg could do this trimming on its own (accepts parameters
                    // for it in the extradata field), the swscale colorspace conversion would still have to be done
                    // into a frame that has a stride which is a multiple of 32 or 64 bytes (for performance reasons;
                    // otherwise it leaves the right edge blank) and that is often greater than the actual width*4,
                    // so there will be gaps between the rows in these cases.
                    // And Bitmap at the moment does not allow these gaps, so we need to remove them.
                    // Also dropping any unwanted rows on the bottom while we're at it.
                    let new_width = bounds.0 as usize;
                    let new_height = usize::min(height, bounds.1 as usize);
                    // no need to move the first row, nor any rows on the bottom that will end up being cropped entirely
                    for row in 1..new_height {
                        rgba.copy_within(
                            row * width * 4..(row * width + new_width) * 4,
                            row * new_width * 4,
                        );
                    }
                    width = new_width;
                    height = new_height;
                }

                // Cropping the unwanted rows on the bottom, also dropping any unused space at the end left by the squish above
                height = usize::min(height, bounds.1 as usize);
                rgba.truncate(width * height * 4);

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
