//! Various structs related to FTE used across the whole codebase.

use gc_arena::Collect;
use ruffle_macros::Avm2Enum;
use ruffle_wstr::WStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum FontWeightValue {
    #[avm2_variant("normal")]
    Normal,
    #[avm2_variant("bold")]
    Bold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum FontPostureValue {
    #[avm2_variant("normal")]
    Normal,
    #[avm2_variant("italic")]
    Italic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum FontLookupValue {
    #[avm2_variant("device")]
    Device,
    #[avm2_variant("embeddedCFF")]
    EmbeddedCFF,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum RenderingModeValue {
    #[avm2_variant("normal")]
    Normal,
    #[avm2_variant("cff")]
    Cff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum CffHintingValue {
    #[avm2_variant("none")]
    None,
    #[avm2_variant("horizontalStem")]
    HorizontalStem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum TextBaselineValue {
    #[avm2_variant("ascent")]
    Ascent,
    #[avm2_variant("descent")]
    Descent,
    #[avm2_variant("ideographicBottom")]
    IdeographicBottom,
    #[avm2_variant("ideographicCenter")]
    IdeographicCenter,
    #[avm2_variant("ideographicTop")]
    IdeographicTop,
    #[avm2_variant("roman")]
    Roman,
    #[avm2_variant("useDominantBaseline")]
    UseDominantBaseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum BreakOpportunityValue {
    #[avm2_variant("all")]
    All,
    #[avm2_variant("any")]
    Any,
    #[avm2_variant("auto")]
    Auto,
    #[avm2_variant("none")]
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum DigitCaseValue {
    #[avm2_variant("default")]
    Default,
    #[avm2_variant("lining")]
    Lining,
    #[avm2_variant("oldStyle")]
    OldStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum DigitWidthValue {
    #[avm2_variant("default")]
    Default,
    #[avm2_variant("proportional")]
    Proportional,
    #[avm2_variant("tabular")]
    Tabular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum KerningValue {
    #[avm2_variant("auto")]
    Auto,
    #[avm2_variant("off")]
    Off,
    #[avm2_variant("on")]
    On,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum LigatureLevelValue {
    #[avm2_variant("common")]
    Common,
    #[avm2_variant("exotic")]
    Exotic,
    #[avm2_variant("minimum")]
    Minimum,
    #[avm2_variant("none")]
    None,
    #[avm2_variant("uncommon")]
    Uncommon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum TextRotationValue {
    #[avm2_variant("auto")]
    Auto,
    #[avm2_variant("rotate0")]
    Rotate0,
    #[avm2_variant("rotate180")]
    Rotate180,
    #[avm2_variant("rotate270")]
    Rotate270,
    #[avm2_variant("rotate90")]
    Rotate90,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Collect, Avm2Enum)]
#[collect(require_static)]
pub enum TypographicCaseValue {
    #[avm2_variant("caps")]
    Caps,
    #[avm2_variant("capsAndSmallCaps")]
    CapsAndSmallCaps,
    #[avm2_variant("default")]
    Default,
    #[avm2_variant("lowercase")]
    Lowercase,
    #[avm2_variant("smallCaps")]
    SmallCaps,
    #[avm2_variant("title")]
    Title,
    #[avm2_variant("uppercase")]
    Uppercase,
}

#[derive(Clone, Copy, Collect, PartialEq)]
#[collect(require_static)]
pub enum TextLineValidity {
    Valid,
    Invalid,
    Static,
    PossiblyInvalid,
    UserInvalid,
}

impl TextLineValidity {
    pub fn parse(string: &WStr) -> Self {
        if string == b"valid" {
            Self::Valid
        } else if string == b"invalid" {
            Self::Invalid
        } else if string == b"static" {
            Self::Static
        } else if string == b"possiblyInvalid" {
            Self::PossiblyInvalid
        } else {
            Self::UserInvalid
        }
    }
}
