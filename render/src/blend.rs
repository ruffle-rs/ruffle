use std::fmt::{self, Display, Formatter};
use ruffle_wstr::{FromWStr, WStr};

/// Like `swf::BlendMode`, but contains variants that cannot be read from
/// a SWF (currently, just `Shader`).
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtendedBlendMode {
    #[default]
    Normal,
    Layer,
    Multiply,
    Screen,
    Lighten,
    Darken,
    Difference,
    Add,
    Subtract,
    Invert,
    Alpha,
    Erase,
    Overlay,
    HardLight,
    Shader,
}

impl FromWStr for ExtendedBlendMode {
    type Err = ();

    fn from_wstr(s: &WStr) -> Result<Self, Self::Err> {
        if s == b"normal" {
            Ok(ExtendedBlendMode::Normal)
        } else if s == b"layer" {
            Ok(ExtendedBlendMode::Layer)
        } else if s == b"multiply" {
            Ok(ExtendedBlendMode::Multiply)
        } else if s == b"screen" {
            Ok(ExtendedBlendMode::Screen)
        } else if s == b"lighten" {
            Ok(ExtendedBlendMode::Lighten)
        } else if s == b"darken" {
            Ok(ExtendedBlendMode::Darken)
        } else if s == b"difference" {
            Ok(ExtendedBlendMode::Difference)
        } else if s == b"add" {
            Ok(ExtendedBlendMode::Add)
        } else if s == b"subtract" {
            Ok(ExtendedBlendMode::Subtract)
        } else if s == b"invert" {
            Ok(ExtendedBlendMode::Invert)
        } else if s == b"alpha" {
            Ok(ExtendedBlendMode::Alpha)
        } else if s == b"erase" {
            Ok(ExtendedBlendMode::Erase)
        } else if s == b"overlay" {
            Ok(ExtendedBlendMode::Overlay)
        } else if s == b"hardlight" {
            Ok(ExtendedBlendMode::HardLight)
        } else if s == b"shader" {
            Ok(ExtendedBlendMode::Shader)
        } else {
            Err(())
        }
    }
}

impl Display for ExtendedBlendMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match *self {
            ExtendedBlendMode::Normal => "normal",
            ExtendedBlendMode::Layer => "layer",
            ExtendedBlendMode::Multiply => "multiply",
            ExtendedBlendMode::Screen => "screen",
            ExtendedBlendMode::Lighten => "lighten",
            ExtendedBlendMode::Darken => "darken",
            ExtendedBlendMode::Difference => "difference",
            ExtendedBlendMode::Add => "add",
            ExtendedBlendMode::Subtract => "subtract",
            ExtendedBlendMode::Invert => "invert",
            ExtendedBlendMode::Alpha => "alpha",
            ExtendedBlendMode::Erase => "erase",
            ExtendedBlendMode::Overlay => "overlay",
            ExtendedBlendMode::HardLight => "hardlight",
            ExtendedBlendMode::Shader => "shader",
        };
        f.write_str(s)
    }
}

impl From<swf::BlendMode> for ExtendedBlendMode {
    fn from(b: swf::BlendMode) -> Self {
        match b {
            swf::BlendMode::Normal => ExtendedBlendMode::Normal,
            swf::BlendMode::Layer => ExtendedBlendMode::Layer,
            swf::BlendMode::Multiply => ExtendedBlendMode::Multiply,
            swf::BlendMode::Screen => ExtendedBlendMode::Screen,
            swf::BlendMode::Lighten => ExtendedBlendMode::Lighten,
            swf::BlendMode::Darken => ExtendedBlendMode::Darken,
            swf::BlendMode::Difference => ExtendedBlendMode::Difference,
            swf::BlendMode::Add => ExtendedBlendMode::Add,
            swf::BlendMode::Subtract => ExtendedBlendMode::Subtract,
            swf::BlendMode::Invert => ExtendedBlendMode::Invert,
            swf::BlendMode::Alpha => ExtendedBlendMode::Alpha,
            swf::BlendMode::Erase => ExtendedBlendMode::Erase,
            swf::BlendMode::Overlay => ExtendedBlendMode::Overlay,
            swf::BlendMode::HardLight => ExtendedBlendMode::HardLight,
        }
    }
}

impl TryInto<swf::BlendMode> for ExtendedBlendMode {
    type Error = ();
    fn try_into(self) -> Result<swf::BlendMode, Self::Error> {
        match self {
            ExtendedBlendMode::Normal => Ok(swf::BlendMode::Normal),
            ExtendedBlendMode::Layer => Ok(swf::BlendMode::Layer),
            ExtendedBlendMode::Multiply => Ok(swf::BlendMode::Multiply),
            ExtendedBlendMode::Screen => Ok(swf::BlendMode::Screen),
            ExtendedBlendMode::Lighten => Ok(swf::BlendMode::Lighten),
            ExtendedBlendMode::Darken => Ok(swf::BlendMode::Darken),
            ExtendedBlendMode::Difference => Ok(swf::BlendMode::Difference),
            ExtendedBlendMode::Add => Ok(swf::BlendMode::Add),
            ExtendedBlendMode::Subtract => Ok(swf::BlendMode::Subtract),
            ExtendedBlendMode::Invert => Ok(swf::BlendMode::Invert),
            ExtendedBlendMode::Alpha => Ok(swf::BlendMode::Alpha),
            ExtendedBlendMode::Erase => Ok(swf::BlendMode::Erase),
            ExtendedBlendMode::Overlay => Ok(swf::BlendMode::Overlay),
            ExtendedBlendMode::HardLight => Ok(swf::BlendMode::HardLight),
            ExtendedBlendMode::Shader => Err(()),
        }
    }
}
