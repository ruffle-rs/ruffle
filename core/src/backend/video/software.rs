//! Pure software video decoding backend.

use crate::backend::render::{BitmapInfo, RenderBackend};
use crate::backend::video::{
    EncodedFrame, Error, FrameDependency, VideoBackend, VideoStreamHandle,
};
use generational_arena::Arena;
use h263_rs::parser::{decode_picture, H263Reader};
use h263_rs::{DecoderOption, H263State, PictureTypeCode};
use h263_rs_yuv::bt601::yuv420_to_rgba;
use swf::{VideoCodec, VideoDeblocking};

/// A single preloaded video stream.
pub enum VideoStream {
    H263(H263State),
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
            VideoCodec::H263 => Ok(self.streams.insert(VideoStream::H263(H263State::new(
                DecoderOption::SORENSON_SPARK_BITSTREAM,
            )))),
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
            VideoStream::H263(_state) => {
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
            VideoStream::H263(state) => {
                let mut reader = H263Reader::from_source(encoded_frame.data());

                state.decode_next_picture(&mut reader)?;

                let picture = state
                    .get_last_picture()
                    .expect("Decoding a picture should let us grab that picture");

                let (width, height) = picture
                    .format()
                    .into_width_and_height()
                    .ok_or("H.263 decoder error!")?;
                let chroma_width = picture.chroma_samples_per_row();
                let (y, b, r) = picture.as_yuv();
                let rgba = yuv420_to_rgba(y, b, r, width.into(), chroma_width);

                let handle = renderer.register_bitmap_raw(width.into(), height.into(), rgba)?;

                Ok(BitmapInfo {
                    handle,
                    width,
                    height,
                })
            }
        }
    }
}
