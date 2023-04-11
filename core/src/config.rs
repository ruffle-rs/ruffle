use gc_arena::Collect;
use serde::{Deserialize, Serialize};

/// Controls whether the content is letterboxed or pillarboxed when the
/// player's aspect ratio does not match the movie's aspect ratio.
///
/// When letterboxed, black bars will be rendered around the exterior
/// margins of the content.
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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

/// The networking api restriction mode of the Ruffle player.
/// This setting is only used on web.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NetworkingRestrictionMode {
    /// All networking APIs are permitted in the SWF file.
    #[serde(rename = "all")]
    All,

    /// The SWF file may not call browser navigation or browser interaction APIs.
    #[serde(rename = "internal")]
    Internal,

    /// The SWF file may not call browser navigation or browser interaction APIs
    /// and it cannot use any SWF-to-SWF communication APIs.
    #[serde(rename = "none")]
    None,
}
