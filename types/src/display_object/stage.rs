use bitflags::bitflags;
use gc_arena::Collect;
use ruffle_wstr::{FromWStr, WStr};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct ParseEnumError;

/// The scale mode of a stage.
/// This controls the behavior when the player viewport size differs from the SWF size.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum StageScaleMode {
    /// The movie will be stretched to fit the container.
    ExactFit,

    /// The movie will maintain its aspect ratio, but will be cropped.
    NoBorder,

    /// The movie is not scaled to fit the container.
    /// With this scale mode, `Stage.stageWidth` and `stageHeight` will return the dimensions of the container.
    /// SWF content uses this scale mode to resize dynamically and create responsive layouts.
    NoScale,

    /// The movie will scale to fill the container and maintain its aspect ratio, but will be letterboxed.
    /// This is the default scale mode.
    #[default]
    ShowAll,
}

impl Display for StageScaleMode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageScaleMode::ExactFit => "exactFit",
            StageScaleMode::NoBorder => "noBorder",
            StageScaleMode::NoScale => "noScale",
            StageScaleMode::ShowAll => "showAll",
        };
        f.write_str(s)
    }
}

impl FromStr for StageScaleMode {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scale_mode = match s.to_ascii_lowercase().as_str() {
            "exactfit" => StageScaleMode::ExactFit,
            "noborder" => StageScaleMode::NoBorder,
            "noscale" => StageScaleMode::NoScale,
            "showall" => StageScaleMode::ShowAll,
            _ => return Err(ParseEnumError),
        };
        Ok(scale_mode)
    }
}

impl FromWStr for StageScaleMode {
    type Err = ParseEnumError;

    #[inline]
    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"exactfit")) {
            Ok(StageScaleMode::ExactFit)
        } else if s.eq_ignore_case(WStr::from_units(b"noborder")) {
            Ok(StageScaleMode::NoBorder)
        } else if s.eq_ignore_case(WStr::from_units(b"noscale")) {
            Ok(StageScaleMode::NoScale)
        } else if s.eq_ignore_case(WStr::from_units(b"showall")) {
            Ok(StageScaleMode::ShowAll)
        } else {
            Err(ParseEnumError)
        }
    }
}

/// The scale mode of a stage.
/// This controls the behavior when the player viewport size differs from the SWF size.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum StageDisplayState {
    /// Sets AIR application or content in Flash Player to expand the stage over the user's entire screen.
    /// Keyboard input is disabled, with the exception of a limited set of non-printing keys.
    FullScreen,

    /// Sets the application to expand the stage over the user's entire screen, with keyboard input allowed.
    /// (Available in AIR and Flash Player, beginning with Flash Player 11.3.)
    FullScreenInteractive,

    /// Sets the stage back to the standard stage display mode.
    #[default]
    Normal,
}

impl Display for StageDisplayState {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageDisplayState::FullScreen => "fullScreen",
            StageDisplayState::FullScreenInteractive => "fullScreenInteractive",
            StageDisplayState::Normal => "normal",
        };
        f.write_str(s)
    }
}

impl FromStr for StageDisplayState {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let display_state = match s.to_ascii_lowercase().as_str() {
            "fullscreen" => StageDisplayState::FullScreen,
            "fullscreeninteractive" => StageDisplayState::FullScreenInteractive,
            "normal" => StageDisplayState::Normal,
            _ => return Err(ParseEnumError),
        };
        Ok(display_state)
    }
}

impl FromWStr for StageDisplayState {
    type Err = ParseEnumError;

    #[inline]
    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"fullscreen")) {
            Ok(StageDisplayState::FullScreen)
        } else if s.eq_ignore_case(WStr::from_units(b"fullscreeninteractive")) {
            Ok(StageDisplayState::FullScreenInteractive)
        } else if s.eq_ignore_case(WStr::from_units(b"normal")) {
            Ok(StageDisplayState::Normal)
        } else {
            Err(ParseEnumError)
        }
    }
}

bitflags! {
    /// The alignment of the stage.
    /// This controls the position of the movie after scaling to fill the viewport.
    /// The default alignment is centered (no bits set).
    ///
    /// This is a bitflags instead of an enum to mimic Flash Player behavior.
    /// You can theoretically have both TOP and BOTTOM bits set, for example.
    #[derive(Default, Collect)]
    #[collect(require_static)]
    pub struct StageAlign: u8 {
        /// Align to the top of the viewport.
        const TOP    = 1 << 0;

        /// Align to the bottom of the viewport.
        const BOTTOM = 1 << 1;

        /// Align to the left of the viewport.
        const LEFT   = 1 << 2;

        /// Align to the right of the viewport.;
        const RIGHT  = 1 << 3;
    }
}

impl FromStr for StageAlign {
    type Err = std::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in s.bytes().map(|c| c.to_ascii_uppercase()) {
            match c {
                b'T' => align.insert(StageAlign::TOP),
                b'B' => align.insert(StageAlign::BOTTOM),
                b'L' => align.insert(StageAlign::LEFT),
                b'R' => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
        Ok(align)
    }
}

impl FromWStr for StageAlign {
    type Err = std::convert::Infallible;

    #[inline]
    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        // Chars get converted into flags.
        // This means "tbbtlbltblbrllrbltlrtbl" is valid, resulting in "TBLR".
        let mut align = StageAlign::default();
        for c in s.iter() {
            match u8::try_from(c).map(|c| c.to_ascii_uppercase()) {
                Ok(b'T') => align.insert(StageAlign::TOP),
                Ok(b'B') => align.insert(StageAlign::BOTTOM),
                Ok(b'L') => align.insert(StageAlign::LEFT),
                Ok(b'R') => align.insert(StageAlign::RIGHT),
                _ => (),
            }
        }
        Ok(align)
    }
}

/// The quality setting of the `Stage`.
///
/// In the Flash Player, this settings affects anti-aliasing and bitmap smoothing.
/// These settings currently have no effect in Ruffle, but the active setting is still stored.
/// [StageQuality in the AS3 Reference](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/StageQuality.html)
#[derive(Default, Clone, Collect, Copy, Debug, Eq, PartialEq)]
#[collect(require_static)]
pub enum StageQuality {
    /// No anti-aliasing, and bitmaps are never smoothed.
    Low,

    /// 2x anti-aliasing.
    Medium,

    /// 4x anti-aliasing.
    #[default]
    High,

    /// 4x anti-aliasing with high quality downsampling.
    /// Bitmaps will use high quality downsampling when scaled down.
    /// Despite the name, this is not the best quality setting as 8x8 and 16x16 modes were added to
    /// Flash Player 11.3.
    Best,

    /// 8x anti-aliasing.
    /// Bitmaps will use high quality downsampling when scaled down.
    High8x8,

    /// 8x anti-aliasing done in linear sRGB space.
    /// Bitmaps will use high quality downsampling when scaled down.
    High8x8Linear,

    /// 16x anti-aliasing.
    /// Bitmaps will use high quality downsampling when scaled down.
    High16x16,

    /// 16x anti-aliasing done in linear sRGB space.
    /// Bitmaps will use high quality downsampling when scaled down.
    High16x16Linear,
}

impl StageQuality {
    /// Returns the string representing the quality setting as returned by AVM1 `_quality` and
    /// AVM2 `Stage.quality`.
    #[inline]
    pub fn into_avm_str(self) -> &'static str {
        // Flash Player always returns quality in uppercase, despite the AVM2 `StageQuality` being
        // lowercase.
        match self {
            StageQuality::Low => "LOW",
            StageQuality::Medium => "MEDIUM",
            StageQuality::High => "HIGH",
            StageQuality::Best => "BEST",
            // The linear sRGB quality settings are not returned even if they are active.
            StageQuality::High8x8 | StageQuality::High8x8Linear => "8X8",
            StageQuality::High16x16 | StageQuality::High16x16Linear => "16X16",
        }
    }
}

impl Display for StageQuality {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Match string values returned by AS.
        let s = match *self {
            StageQuality::Low => "low",
            StageQuality::Medium => "medium",
            StageQuality::High => "high",
            StageQuality::Best => "best",
            StageQuality::High8x8 => "8x8",
            StageQuality::High8x8Linear => "8x8linear",
            StageQuality::High16x16 => "16x16",
            StageQuality::High16x16Linear => "16x16linear",
        };
        f.write_str(s)
    }
}

impl FromStr for StageQuality {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let quality = match s.to_ascii_lowercase().as_str() {
            "low" => StageQuality::Low,
            "medium" => StageQuality::Medium,
            "high" => StageQuality::High,
            "best" => StageQuality::Best,
            "8x8" => StageQuality::High8x8,
            "8x8linear" => StageQuality::High8x8Linear,
            "16x16" => StageQuality::High16x16,
            "16x16linear" => StageQuality::High16x16Linear,
            _ => return Err(ParseEnumError),
        };
        Ok(quality)
    }
}

impl FromWStr for StageQuality {
    type Err = ParseEnumError;

    #[inline]
    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s.eq_ignore_case(WStr::from_units(b"low")) {
            Ok(StageQuality::Low)
        } else if s.eq_ignore_case(WStr::from_units(b"medium")) {
            Ok(StageQuality::Medium)
        } else if s.eq_ignore_case(WStr::from_units(b"high")) {
            Ok(StageQuality::High)
        } else if s.eq_ignore_case(WStr::from_units(b"best")) {
            Ok(StageQuality::Best)
        } else if s.eq_ignore_case(WStr::from_units(b"8x8")) {
            Ok(StageQuality::High8x8)
        } else if s.eq_ignore_case(WStr::from_units(b"8x8linear")) {
            Ok(StageQuality::High8x8Linear)
        } else if s.eq_ignore_case(WStr::from_units(b"16x16")) {
            Ok(StageQuality::High16x16)
        } else if s.eq_ignore_case(WStr::from_units(b"16x16linear")) {
            Ok(StageQuality::High16x16Linear)
        } else {
            Err(ParseEnumError)
        }
    }
}

/// The window mode of the Ruffle player.
///
/// This setting controls how the Ruffle container is layered and rendered with other content on
/// the page. This setting is only used on web.
///
/// [Apply OBJECT and EMBED tag attributes in Adobe Flash Professional](https://helpx.adobe.com/flash/kb/flash-object-embed-tag-attributes.html)
#[derive(Default, Clone, Collect, Copy, Debug, Eq, PartialEq)]
#[collect(require_static)]
pub enum WindowMode {
    /// The Flash content is rendered in its own window and layering is done with the browser's
    /// default behavior.
    ///
    /// In Ruffle, this mode functions like `WindowMode::Opaque` and will layer the Flash content
    /// together with other HTML elements.
    #[default]
    Window,

    /// The Flash content is layered together with other HTML elements, and the stage color is
    /// opaque. Content can render above or below Ruffle based on CSS rendering order.
    Opaque,

    /// The Flash content is layered together with other HTML elements, and the stage color is
    /// transparent. Content beneath Ruffle will be visible through transparent areas.
    Transparent,

    /// Request compositing with hardware acceleration when possible.
    ///
    /// This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
    Gpu,

    /// Request a direct rendering path, bypassing browser compositing when possible.
    ///
    /// This mode has no effect in Ruffle and will function like `WindowMode::Opaque`.
    Direct,
}

impl Display for WindowMode {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match *self {
            WindowMode::Window => "window",
            WindowMode::Opaque => "opaque",
            WindowMode::Transparent => "transparent",
            WindowMode::Direct => "direct",
            WindowMode::Gpu => "gpu",
        };
        f.write_str(s)
    }
}

impl FromStr for WindowMode {
    type Err = ParseEnumError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let window_mode = match s.to_ascii_lowercase().as_str() {
            "window" => WindowMode::Window,
            "opaque" => WindowMode::Opaque,
            "transparent" => WindowMode::Transparent,
            "direct" => WindowMode::Direct,
            "gpu" => WindowMode::Gpu,
            _ => return Err(ParseEnumError),
        };
        Ok(window_mode)
    }
}
