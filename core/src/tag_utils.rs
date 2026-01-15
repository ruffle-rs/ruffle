//! SWF decoding support

use swf::{CharacterId, TagCode};
use thiserror::Error;

pub use ruffle_common::tag_utils::{SwfMovie, SwfSlice, SwfStream};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't read SWF: {0}")]
    InvalidSwf(#[from] swf::error::Error),

    #[error("Couldn't register bitmap: {0}")]
    InvalidBitmap(#[from] ruffle_render::error::Error),

    #[error("Couldn't register font: {0}")]
    InvalidFont(#[from] ttf_parser::FaceParsingError),

    #[error("Attempted to preload video frames into non-video character {0}")]
    PreloadVideoIntoInvalidCharacter(CharacterId),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Invalid SWF url")]
    InvalidSwfUrl,
}

/// Whether or not to end tag decoding.
pub enum ControlFlow {
    /// Stop decoding after this tag.
    Exit,

    /// Continue decoding the next tag.
    Continue,
}

/// Decode tags from a SWF stream reader.
///
/// The given `tag_callback` will be called for each decoded tag. It will be
/// provided with the stream to read from, the tag code read, and the tag's
/// size. The callback is responsible for (optionally) parsing the contents of
/// the tag; otherwise, it will be skipped.
///
/// Decoding will terminate when the following conditions occur:
///
///  * The `tag_callback` calls for the decoding to finish.
///  * The decoder encounters a tag longer than the underlying SWF slice
///    (indicated by returning false)
///  * The SWF stream is otherwise corrupt or unreadable (indicated as an error
///    result)
///
/// Decoding will also log tags longer than the SWF slice, error messages
/// yielded from the tag callback, and unknown tags. It will *only* return an
/// error message if the SWF tag itself could not be parsed. Other forms of
/// irregular decoding will be signalled by returning false.
pub fn decode_tags<'a, F>(reader: &mut SwfStream<'a>, mut tag_callback: F) -> Result<bool, Error>
where
    F: for<'b> FnMut(&'b mut SwfStream<'a>, TagCode, usize) -> Result<ControlFlow, Error>,
{
    loop {
        let (tag_code, tag_len) = reader.read_tag_code_and_length()?;
        if tag_len > reader.get_ref().len() {
            tracing::error!("Unexpected EOF when reading tag");
            *reader.get_mut() = &reader.get_ref()[reader.get_ref().len()..];
            return Ok(false);
        }

        let tag_slice = &reader.get_ref()[..tag_len];
        let end_slice = &reader.get_ref()[tag_len..];
        if let Some(tag) = TagCode::from_u16(tag_code) {
            *reader.get_mut() = tag_slice;
            let result = tag_callback(reader, tag, tag_len);

            match result {
                Err(e) => {
                    tracing::error!("Error running definition tag: {:?}, got {}", tag, e)
                }
                Ok(ControlFlow::Exit) => {
                    *reader.get_mut() = end_slice;
                    break;
                }
                Ok(ControlFlow::Continue) => {}
            }
        } else {
            tracing::warn!("Unknown tag code: {:?}", tag_code);
        }

        *reader.get_mut() = end_slice;
    }

    Ok(true)
}

/// Utility method to construct a movie from a file on disk.
#[cfg(any(unix, windows, target_os = "redox"))]
pub fn movie_from_path<P: AsRef<std::path::Path>>(
    path: P,
    loader_url: Option<String>,
) -> Result<SwfMovie, Error> {
    let data = std::fs::read(&path)?;

    let abs_path = path.as_ref().canonicalize()?;
    let url = url::Url::from_file_path(abs_path).map_err(|()| Error::InvalidSwfUrl)?;

    SwfMovie::from_data(&data, url.into(), loader_url).map_err(Error::InvalidSwf)
}
