use crate::backend::video::software::VideoDecoder;
use crate::backend::video::{DecodedFrame, EncodedFrame, FrameDependency};
use h263_rs::parser::H263Reader;
use h263_rs::{DecoderOption, H263State, PictureTypeCode};
use h263_rs_yuv::bt601::yuv420_to_rgba;
use ruffle_video::error::Error;

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
pub struct H263Decoder(H263State);

impl H263Decoder {
    pub fn new() -> Self {
        Self(H263State::new(DecoderOption::SORENSON_SPARK_BITSTREAM))
    }
}

impl VideoDecoder for H263Decoder {
    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error> {
        let mut reader = H263Reader::from_source(encoded_frame.data());
        let picture = self
            .0
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

    fn decode_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<DecodedFrame, Error> {
        let mut reader = H263Reader::from_source(encoded_frame.data());

        self.0
            .decode_next_picture(&mut reader)
            .map_err(H263Error::DecoderError)?;

        let picture = self
            .0
            .get_last_picture()
            .expect("Decoding a picture should let us grab that picture");

        let (width, height) = picture
            .format()
            .into_width_and_height()
            .ok_or(H263Error::MissingWidthHeight)?;
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
