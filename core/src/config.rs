use std::str::FromStr;

/// Controls whether the content is letterboxed or pillarboxed when the
/// player's aspect ratio does not match the movie's aspect ratio.
///
/// When letterboxed, black bars will be rendered around the exterior
/// margins of the content.
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename = "letterbox")
)]
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

pub struct ParseEnumError;

impl FromStr for Letterbox {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let letterbox = match s {
            "off" => Letterbox::Off,
            "fullscreen" => Letterbox::Fullscreen,
            "on" => Letterbox::On,
            _ => return Err(ParseEnumError),
        };
        Ok(letterbox)
    }
}

/// The networking API access mode of the Ruffle player.
/// This setting is only used on web.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NetworkingAccessMode {
    /// All networking APIs are permitted in the SWF file.
    #[cfg_attr(feature = "serde", serde(rename = "all"))]
    All,

    /// The SWF file may not call browser navigation or browser interaction APIs.
    ///
    /// The APIs getURL(), navigateToURL(), fscommand() and ExternalInterface.call()
    /// are prevented in this mode.
    #[cfg_attr(feature = "serde", serde(rename = "internal"))]
    Internal,

    /// The SWF file may not call browser navigation or browser interaction APIs
    /// and it cannot use any SWF-to-SWF communication APIs.
    ///
    /// Additionally to the ones in internal mode, the APIs sendToURL(),
    /// FileReference.download(), FileReference.upload(), Loader.load(),
    /// LocalConnection.connect(), LocalConnection.send(), NetConnection.connect(),
    /// NetStream.play(), Security.loadPolicyFile(), SharedObject.getLocal(),
    /// SharedObject.getRemote(), Socket.connect(), Sound.load(), URLLoader.load(),
    /// URLStream.load() and XMLSocket.connect() are prevented in this mode.
    ///
    /// This mode is not implemented yet.
    #[cfg_attr(feature = "serde", serde(rename = "none"))]
    None,
}
