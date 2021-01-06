#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Controls whether the content is letterboxed or pillarboxed when the
/// player's aspect ratio does not match the movie's aspect ratio.
///
/// When letterboxed, black bars will be rendered around the exterior
/// margins of the content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename = "letterbox"))]
pub enum Letterbox {
    /// The content will never be letterboxed.
    #[cfg_attr(feature = "serde", serde(rename = "off"))]
    Off,

    /// The content will only be letterboxed if the content is running fullscreen.
    #[cfg_attr(feature = "serde", serde(rename = "fullscreen"))]
    Fullscreen,

    /// The content will always be letterboxed.
    #[cfg_attr(feature = "serde", serde(rename = "on"))]
    On,
}

impl Default for Letterbox {
    fn default() -> Self {
        Letterbox::Fullscreen
    }
}
