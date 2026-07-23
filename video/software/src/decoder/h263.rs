use crate::decoder::VideoDecoder;
use h263_rs::parser::H263Reader;
use h263_rs::{DecoderOption, H263State, PictureOption, PictureTypeCode};
use h263_rs_deblock::deblock::{QUANT_TO_STRENGTH, deblock};
use ruffle_render::bitmap::BitmapFormat;
use ruffle_video::error::Error;
use ruffle_video::frame::{DecodedFrame, EncodedFrame, FrameDependency};
use swf::VideoDeblocking;

#[derive(thiserror::Error, Debug)]
pub enum H263Error {
    #[error("Picture wasn't found in the video stream")]
    NoPictureInVideoStream,

    #[error("Decoder error")]
    DecoderError(#[from] h263_rs::Error),

    #[error("Invalid picture type code: {0:?}")]
    InvalidPictureType(PictureTypeCode),

    #[error("Picture is missing width and height details")]
    MissingWidthHeight,
}

impl From<H263Error> for Error {
    fn from(error: H263Error) -> Self {
        Error::DecoderError(Box::new(error))
    }
}

/// H263 video decoder.
pub struct H263Decoder {
    state: H263State,
    deblock: VideoDeblocking,
    scratch: Vec<u8>,
}

impl H263Decoder {
    pub fn new(deblock: VideoDeblocking) -> Self {
        Self {
            state: H263State::new(DecoderOption::SORENSON_SPARK_BITSTREAM),
            deblock,
            scratch: Vec::new(),
        }
    }
}

impl VideoDecoder for H263Decoder {
    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error> {
        let mut reader = H263Reader::from_source(encoded_frame.data());
        let picture = self
            .state
            .parse_picture(&mut reader, None)
            .map_err(H263Error::DecoderError)?
            .ok_or(H263Error::NoPictureInVideoStream)?;

        match picture.picture_type {
            PictureTypeCode::IFrame => Ok(FrameDependency::None),
            PictureTypeCode::PFrame => Ok(FrameDependency::Past),
            PictureTypeCode::DisposablePFrame => Ok(FrameDependency::Past),
            code => Err(H263Error::InvalidPictureType(code).into()),
        }
    }

    fn decode_frame_dyn(
        &mut self,
        encoded_frame: EncodedFrame<'_>,
        callback: &mut dyn FnMut(DecodedFrame<'_>),
    ) -> Result<(), Error> {
        let mut reader = H263Reader::from_source(encoded_frame.data());

        self.state
            .decode_next_picture(&mut reader)
            .map_err(H263Error::DecoderError)?;

        let picture = self
            .state
            .get_last_picture()
            .expect("Decoding a picture should let us grab that picture");

        let (width, height) = picture
            .format()
            .into_width_and_height()
            .ok_or(H263Error::MissingWidthHeight)?;
        let (y, b, r) = picture.as_yuv();

        self.scratch.clear();
        let data = &mut self.scratch;
        data.reserve_exact(y.len() + b.len() + r.len());

        let hdr = picture.as_header();
        if self.deblock == VideoDeblocking::Level1
            || (self.deblock == VideoDeblocking::UseVideoPacketValue
                && hdr.options.contains(PictureOption::USE_DEBLOCKER))
        {
            let chroma_width = picture.chroma_samples_per_row();
            let quantizer = hdr.quantizer as usize;
            let strength = QUANT_TO_STRENGTH[quantizer];

            // TODO: it'd be nice if `h263_rs_deblock` provided in-place deblocking
            let y = deblock(y, width as usize, strength);
            let b = deblock(b, chroma_width, strength);
            let r = deblock(r, chroma_width, strength);

            data.extend_from_slice(&y);
            data.extend_from_slice(&b);
            data.extend_from_slice(&r);
        } else {
            data.extend_from_slice(y);
            data.extend_from_slice(b);
            data.extend_from_slice(r);
        }

        callback(DecodedFrame::new(
            width as u32,
            height as u32,
            BitmapFormat::Yuv420p,
            data.as_slice(),
        ));
        Ok(())
    }
}

impl Default for H263Decoder {
    fn default() -> Self {
        Self::new(VideoDeblocking::UseVideoPacketValue)
    }
}
