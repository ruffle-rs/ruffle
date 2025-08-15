use ruffle_wstr::{FromWStr, WStr};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The quality setting of the `Stage`.
///
/// In the Flash Player, this settings affects anti-aliasing and bitmap smoothing.
/// These settings currently have no effect in Ruffle, but the active setting is still stored.
/// [StageQuality in the AS3 Reference](https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/StageQuality.html)
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
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

    /// Returns the preferred anti-aliasing sample count for this quality
    pub fn sample_count(self) -> u32 {
        match self {
            StageQuality::Low => 1,
            StageQuality::Medium => 2,
            StageQuality::High => 4,
            StageQuality::Best => 4,
            StageQuality::High8x8 => 8,
            StageQuality::High8x8Linear => 8,
            StageQuality::High16x16 => 16,
            StageQuality::High16x16Linear => 16,
        }
    }
}

impl Display for StageQuality {
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

pub struct StageQualityError;

impl FromStr for StageQuality {
    type Err = StageQualityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let quality = match s {
            "low" => StageQuality::Low,
            "medium" => StageQuality::Medium,
            "high" => StageQuality::High,
            _ => return Err(StageQualityError),
        };
        Ok(quality)
    }
}

impl FromWStr for StageQuality {
    type Err = StageQualityError;

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
            Err(StageQualityError)
        }
    }
}
