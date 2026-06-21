//! Various structs related to FTE used across the whole codebase.

use crate::avm2::activation::Activation;
use crate::string::AvmString;
use gc_arena::Collect;
use ruffle_macros::istr;
use ruffle_wstr::WStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum TextBaselineValue {
    Ascent,
    Descent,
    IdeographicBottom,
    IdeographicCenter,
    IdeographicTop,
    Roman,
    UseDominantBaseline,
}

impl TextBaselineValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"ascent" {
            Self::Ascent
        } else if s == b"descent" {
            Self::Descent
        } else if s == b"ideographicBottom" {
            Self::IdeographicBottom
        } else if s == b"ideographicCenter" {
            Self::IdeographicCenter
        } else if s == b"ideographicTop" {
            Self::IdeographicTop
        } else if s == b"roman" {
            Self::Roman
        } else if s == b"useDominantBaseline" {
            Self::UseDominantBaseline
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            TextBaselineValue::Ascent => istr!("ascent"),
            TextBaselineValue::Descent => istr!("descent"),
            TextBaselineValue::IdeographicBottom => istr!("ideographicBottom"),
            TextBaselineValue::IdeographicCenter => istr!("ideographicCenter"),
            TextBaselineValue::IdeographicTop => istr!("ideographicTop"),
            TextBaselineValue::Roman => istr!("roman"),
            TextBaselineValue::UseDominantBaseline => istr!("useDominantBaseline"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum BreakOpportunityValue {
    All,
    Any,
    Auto,
    None,
}

impl BreakOpportunityValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"all" {
            Self::All
        } else if s == b"any" {
            Self::Any
        } else if s == b"auto" {
            Self::Auto
        } else if s == b"none" {
            Self::None
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::All => istr!("all"),
            Self::Any => istr!("any"),
            Self::Auto => istr!("auto"),
            Self::None => istr!("none"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum DigitCaseValue {
    Default,
    Lining,
    OldStyle,
}

impl DigitCaseValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"default" {
            Self::Default
        } else if s == b"lining" {
            Self::Lining
        } else if s == b"oldStyle" {
            Self::OldStyle
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::Default => istr!("default"),
            Self::Lining => istr!("lining"),
            Self::OldStyle => istr!("oldStyle"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum DigitWidthValue {
    Default,
    Proportional,
    Tabular,
}

impl DigitWidthValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"default" {
            Self::Default
        } else if s == b"proportional" {
            Self::Proportional
        } else if s == b"tabular" {
            Self::Tabular
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::Default => istr!("default"),
            Self::Proportional => istr!("proportional"),
            Self::Tabular => istr!("tabular"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum KerningValue {
    Auto,
    Off,
    On,
}

impl KerningValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"auto" {
            Self::Auto
        } else if s == b"off" {
            Self::Off
        } else if s == b"on" {
            Self::On
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::Auto => istr!("auto"),
            Self::Off => istr!("off"),
            Self::On => istr!("on"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum LigatureLevelValue {
    Common,
    Exotic,
    Minimum,
    None,
    Uncommon,
}

impl LigatureLevelValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"common" {
            Self::Common
        } else if s == b"exotic" {
            Self::Exotic
        } else if s == b"minimum" {
            Self::Minimum
        } else if s == b"none" {
            Self::None
        } else if s == b"uncommon" {
            Self::Uncommon
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::Common => istr!("common"),
            Self::Exotic => istr!("exotic"),
            Self::Minimum => istr!("minimum"),
            Self::None => istr!("none"),
            Self::Uncommon => istr!("uncommon"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum TextRotationValue {
    Auto,
    Rotate0,
    Rotate180,
    Rotate270,
    Rotate90,
}

impl TextRotationValue {
    pub fn parse_string(s: &WStr) -> Option<Self> {
        Some(if s == b"auto" {
            Self::Auto
        } else if s == b"rotate0" {
            Self::Rotate0
        } else if s == b"rotate180" {
            Self::Rotate180
        } else if s == b"rotate270" {
            Self::Rotate270
        } else if s == b"rotate90" {
            Self::Rotate90
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::Auto => istr!("auto"),
            Self::Rotate0 => istr!("rotate0"),
            Self::Rotate180 => istr!("rotate180"),
            Self::Rotate270 => istr!("rotate270"),
            Self::Rotate90 => istr!("rotate90"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect)]
#[collect(require_static)]
pub enum TypographicCaseValue {
    Caps,
    CapsAndSmallCaps,
    Default,
    Lowercase,
    SmallCaps,
    Title,
    Uppercase,
}

impl TypographicCaseValue {
    pub fn parse_avm2_string(s: &WStr) -> Option<Self> {
        Some(if s == b"caps" {
            Self::Caps
        } else if s == b"capsAndSmallCaps" {
            Self::CapsAndSmallCaps
        } else if s == b"default" {
            Self::Default
        } else if s == b"lowercase" {
            Self::Lowercase
        } else if s == b"smallCaps" {
            Self::SmallCaps
        } else if s == b"title" {
            Self::Title
        } else if s == b"uppercase" {
            Self::Uppercase
        } else {
            return None;
        })
    }

    pub fn as_string<'gc>(self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match self {
            Self::Caps => istr!("caps"),
            Self::CapsAndSmallCaps => istr!("capsAndSmallCaps"),
            Self::Default => istr!("default"),
            Self::Lowercase => istr!("lowercase"),
            Self::SmallCaps => istr!("smallCaps"),
            Self::Title => istr!("title"),
            Self::Uppercase => istr!("uppercase"),
        }
    }
}
