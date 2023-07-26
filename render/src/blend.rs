use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

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

impl FromStr for ExtendedBlendMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mode = match s {
            "normal" => ExtendedBlendMode::Normal,
            "layer" => ExtendedBlendMode::Layer,
            "multiply" => ExtendedBlendMode::Multiply,
            "screen" => ExtendedBlendMode::Screen,
            "lighten" => ExtendedBlendMode::Lighten,
            "darken" => ExtendedBlendMode::Darken,
            "difference" => ExtendedBlendMode::Difference,
            "add" => ExtendedBlendMode::Add,
            "subtract" => ExtendedBlendMode::Subtract,
            "invert" => ExtendedBlendMode::Invert,
            "alpha" => ExtendedBlendMode::Alpha,
            "erase" => ExtendedBlendMode::Erase,
            "overlay" => ExtendedBlendMode::Overlay,
            "hardlight" => ExtendedBlendMode::HardLight,
            "shader" => ExtendedBlendMode::Shader,
            _ => return Err(()),
        };
        Ok(mode)
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
