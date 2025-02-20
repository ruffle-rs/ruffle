use crate::decoder::VideoDecoder;
use ruffle_render::bitmap::BitmapFormat;
use ruffle_video::error::Error;

use nihav_codec_support::codecs::{NABufferRef, NAVideoBuffer, NAVideoInfo};
use nihav_codec_support::codecs::{NABufferType::Video, YUV420_FORMAT};
use nihav_core::codecs::NADecoderSupport;
use nihav_duck::codecs::vp6::{VP56Decoder, VP56Parser, VP6BR};
use nihav_duck::codecs::vpcommon::{BoolCoder, VP_YUVA420_FORMAT};
use ruffle_video::frame::{DecodedFrame, EncodedFrame, FrameDependency};

#[derive(thiserror::Error, Debug)]
pub enum Vp6Error {
    #[error("Decoder error: {0:?}")]
    // DecoderError doesn't impl Error... so this is manual.
    DecoderError(nihav_core::codecs::DecoderError),

    #[error("Unexpected skip frame")]
    UnexpectedSkipFrame,

    #[error("Invalid buffer type")]
    InvalidBufferType,
}

impl From<Vp6Error> for Error {
    fn from(error: Vp6Error) -> Self {
        Error::DecoderError(Box::new(error))
    }
}

impl From<nihav_core::codecs::DecoderError> for Vp6Error {
    fn from(error: nihav_core::codecs::DecoderError) -> Self {
        Vp6Error::DecoderError(error)
    }
}

/// VP6 video decoder.
pub struct Vp6Decoder {
    with_alpha: bool,
    bounds: (u16, u16),
    decoder: VP56Decoder,
    support: NADecoderSupport,
    bitreader: VP6BR,
    init_called: bool,
    last_frame: Option<NABufferRef<NAVideoBuffer<u8>>>,
}

impl Vp6Decoder {
    pub fn new(with_alpha: bool, bounds: (u16, u16)) -> Self {
        // Unfortunately, `init()` cannot be called on the decoder
        // just yet, because `bounds` is only the declared size of
        // the video, to which it will be cropped.
        // This can be (rarely) even much smaller than the actual
        // encoded size of the frames.
        // `VP56Decoder::init()` needs the full encoded frame size,
        // so it can allocate its internal buffers accordingly.
        // The encoded frame size will be parsed from the header of
        // the first encoded frame passed to `Self::decode_frame()`.

        Self {
            with_alpha,
            bounds,
            decoder: VP56Decoder::new(6, with_alpha, true),
            support: NADecoderSupport::new(),
            bitreader: VP6BR::new(),
            init_called: false,
            last_frame: None,
        }
    }
}

fn crop(data: &[u8], mut width: usize, to_size: (u16, u16)) -> Vec<u8> {
    debug_assert!(data.len() % width == 0);
    let mut height = data.len() / width;
    let mut data = data.to_vec();

    if width > to_size.0 as usize {
        // Removing the unwanted pixels on the right edge
        // by squishing all the rows tightly next to each other.
        let new_width = to_size.0 as usize;
        let new_height = usize::min(height, to_size.1 as usize);
        // no need to move the first row, nor any rows on the bottom that will end up being cropped entirely
        for row in 1..new_height {
            data.copy_within(row * width..(row * width + new_width), row * new_width);
        }
        width = new_width;
        height = new_height;
    }

    // Cropping the unwanted rows on the bottom, also dropping any unused space at the end left by the squish above
    height = usize::min(height, to_size.1 as usize);
    data.truncate(width * height);

    data
}

impl VideoDecoder for Vp6Decoder {
    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error> {
        // Luckily the very first bit of the encoded frames is exactly this flag,
        // so we don't have to bother asking any "proper" decoder or parser.
        // The first 24 bits are the alpha offset in VP6A, those need to be skipped.
        let flag_index = if self.with_alpha { 3 } else { 0 };
        Ok(
            // Empty frames are allowed, but they also can't be seeked to
            if encoded_frame.data.len() > flag_index
                && (encoded_frame.data[flag_index] & 0b_1000_0000) == 0
            {
                FrameDependency::None
            } else {
                FrameDependency::Past
            },
        )
    }

    fn decode_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<DecodedFrame, Error> {
        // If this is the first frame, the decoder needs to be initialized.

        if !self.init_called {
            let mut bool_coder = BoolCoder::new(if self.with_alpha {
                // The 24 bits alpha offset needs to be skipped first in this case
                &encoded_frame.data[3..]
            } else {
                encoded_frame.data
            })
            .map_err(Vp6Error::DecoderError)?;

            let header = self
                .bitreader
                .parse_header(&mut bool_coder)
                .map_err(Vp6Error::DecoderError)?;

            let video_info = NAVideoInfo::new(
                header.disp_w as usize * 16,
                header.disp_h as usize * 16,
                true,
                if self.with_alpha {
                    VP_YUVA420_FORMAT
                } else {
                    YUV420_FORMAT
                },
            );

            self.decoder
                .init(&mut self.support, video_info)
                .map_err(Vp6Error::DecoderError)?;

            self.init_called = true;
        }

        let frame = if encoded_frame.data.is_empty()
            || (self.with_alpha && encoded_frame.data.len() <= 3)
        {
            // This frame is empty, so it's a "skip frame"; reusing the last frame, if there is one.

            match &self.last_frame {
                Some(frame) => frame.clone(),
                None => return Err(Vp6Error::UnexpectedSkipFrame.into()),
            }
        } else {
            // Actually decoding the frame and extracting the buffer it is stored in.

            let decoded = self
                .decoder
                .decode_frame(&mut self.support, encoded_frame.data, &mut self.bitreader)
                .map_err(Vp6Error::DecoderError)?;

            let frame = match decoded {
                (Video(buffer), _) => Ok(buffer),
                _ => Err(Vp6Error::InvalidBufferType),
            }?;

            self.last_frame = Some(frame.clone());

            frame
        };

        let yuv = frame.get_data();

        let (mut width, mut height) = frame.get_dimensions(0);
        let (chroma_width, chroma_height) = frame.get_dimensions(1);
        debug_assert_eq!(chroma_width, width.div_ceil(2));
        debug_assert_eq!(chroma_height, height.div_ceil(2));

        // We assume that there is no padding between rows
        debug_assert!(frame.get_stride(0) == frame.get_dimensions(0).0);
        debug_assert!(frame.get_stride(1) == frame.get_dimensions(1).0);
        debug_assert!(frame.get_stride(2) == frame.get_dimensions(2).0);

        // Where each plane starts in the buffer
        let offsets = (
            frame.get_offset(0),
            frame.get_offset(1),
            frame.get_offset(2),
        );

        let y = &yuv[offsets.0..offsets.0 + width * height];
        let u = &yuv[offsets.1..offsets.1 + chroma_width * chroma_height];
        let v = &yuv[offsets.2..offsets.2 + chroma_width * chroma_height];

        // Cropping the encoded frame (containing whole macroblocks) to the
        // size requested by the bounds attribute.

        let &bounds = &self.bounds;

        if width < bounds.0 as usize || height < bounds.1 as usize {
            log::warn!("A VP6 video frame is smaller than the bounds of the stream it belongs in. This is not supported.");
            // Flash Player just produces a black image in this case!
        }

        //(most commonly: unused pieces of macroblocks)
        // Bitmap at the moment does not allow these gaps, so we need to remove them.

        let y = crop(y, width, bounds);
        let u = crop(
            u,
            chroma_width,
            (bounds.0.div_ceil(2), bounds.1.div_ceil(2)),
        );
        let v = crop(
            v,
            chroma_width,
            (bounds.0.div_ceil(2), bounds.1.div_ceil(2)),
        );

        width = bounds.0 as usize;
        height = bounds.1 as usize;

        // Adding in the alpha component, if present.
        if self.with_alpha {
            // Apparently it's possible for the alpha channel to be coded in a different size than the Y channel.
            let (alpha_width, alpha_height) = frame.get_dimensions(3);
            debug_assert!(frame.get_stride(3) == frame.get_dimensions(3).0);

            let alpha_offset = frame.get_offset(3);
            let alpha = &yuv[alpha_offset..alpha_offset + alpha_width * alpha_height];
            let a = crop(alpha, alpha_width, bounds);

            let mut data = y.to_vec();
            data.extend(u);
            data.extend(v);
            data.extend(a);

            Ok(DecodedFrame::new(
                width as u32,
                height as u32,
                BitmapFormat::Yuva420p,
                data,
            ))
        } else {
            let mut data = y.to_vec();
            data.extend(u);
            data.extend(v);

            Ok(DecodedFrame::new(
                width as u32,
                height as u32,
                BitmapFormat::Yuv420p,
                data,
            ))
        }
    }
}

impl Default for Vp6Decoder {
    fn default() -> Self {
        Self::new(false, (0, 0))
    }
}
