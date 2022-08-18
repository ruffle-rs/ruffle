use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Bitmap texture is larger than the rendering device supports")]
    TooLarge,

    #[error("Unknown bitmap format")]
    UnknownType,

    #[error("Invalid ZLIB compression")]
    InvalidZlibCompression,

    #[error("Invalid JPEG")]
    InvalidJpeg(#[from] jpeg_decoder::Error),

    #[error("Invalid PNG")]
    InvalidPng(#[from] png::DecodingError),

    #[error("Invalid GIF")]
    InvalidGif(#[from] gif::DecodingError),

    #[error("Empty GIF")]
    EmptyGif,

    #[error("Unsupported DefineBitsLossless{0} format {1:?}")]
    UnsupportedLosslessFormat(u8, swf::BitmapFormat),

    #[cfg(feature = "web")]
    #[error("Javascript error")]
    JavascriptError(wasm_bindgen::JsValue),
}
