use gc_arena::Collect;
use serde::{Deserialize, Serialize};

/// Controls whether the content is letterboxed or pillarboxed when the
/// player's aspect ratio does not match the movie's aspect ratio.
///
/// When letterboxed, black bars will be rendered around the exterior
/// margins of the content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Collect, Serialize, Deserialize)]
#[collect(require_static)]
#[serde(rename = "letterbox")]
pub enum Letterbox {
    /// The content will never be letterboxed.
    #[serde(rename = "off")]
    Off,

    /// The content will only be letterboxed if the content is running fullscreen.
    #[serde(rename = "fullscreen")]
    Fullscreen,

    /// The content will always be letterboxed.
    #[serde(rename = "on")]
    On,
}
