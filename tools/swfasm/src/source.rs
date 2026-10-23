//! The textual (RON) representation of a SWF file.
//!
//! These types are designed to be written by hand, so most fields have defaults
//! and values use convenient units (e.g. pixels instead of twips).
//!
//! They are lowered into `swf` crate types by [`crate::compile`].

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use swf::{SwfStr, TagCode};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Swf {
    pub version: u8,

    #[serde(default)]
    pub stage: Stage,

    /// Frames per second.
    #[serde(default = "default_frame_rate")]
    pub frame_rate: f64,

    /// The frame count stored in the header.
    ///
    /// Defaults to the number of tags with the `ShowFrame` code on the main
    /// timeline.
    #[serde(default)]
    pub num_frames: Option<u16>,

    #[serde(default)]
    pub tags: Vec<Tag>,
}

fn default_frame_rate() -> f64 {
    24.0
}

/// The stage rectangle, in pixels.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for Stage {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 550.0,
            height: 400.0,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Tag {
    ShowFrame,
    End,
    FileAttributes {
        #[serde(default)]
        as3: bool,
        #[serde(default)]
        use_network_sandbox: bool,
        #[serde(default)]
        has_metadata: bool,
        #[serde(default)]
        use_gpu: bool,
        #[serde(default)]
        use_direct_blit: bool,
    },

    /// A color in the `0xRRGGBB` format.
    SetBackgroundColor(u32),

    DoAction(DoActionContent),

    /// A tag with an arbitrary code and body.
    Raw {
        code: u16,
        data: Bytes,

        /// Whether to use the long (6 byte) header form even when the body is
        /// short enough for the short one.
        #[serde(default)]
        force_long: bool,
    },
}

impl Tag {
    /// Whether this tag is written with the `ShowFrame` tag code.
    pub fn is_show_frame(&self) -> bool {
        match self {
            Self::ShowFrame => true,
            Self::Raw { code, .. } => *code == TagCode::ShowFrame as u16,
            _ => false,
        }
    }
}

/// The actions of a `DoAction` tag.
#[derive(Debug, Deserialize)]
pub enum DoActionContent {
    /// AVM1 bytecode as bytes.
    Bytes(Bytes),
}

/// A string stored in a SWF file.
///
/// Strings in SWF files are not necessarily UTF-8.
///
/// TODO Add support for UTF-16.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct SwfString(pub String);

impl SwfString {
    pub fn as_swf_str(&self) -> &SwfStr {
        SwfStr::from_utf8_str(&self.0)
    }

    /// Converts a string from a SWF file, if it can be represented.
    pub fn from_swf_str(s: &SwfStr) -> Option<Self> {
        std::str::from_utf8(s.as_bytes())
            .ok()
            .map(|s| Self(s.to_owned()))
    }
}

/// Binary data, written as a hex string with optional whitespace (`"01 02 ff"`),
/// or a list of such strings, which are concatenated.
///
/// The list form allows commenting individual parts of the data:
///
/// ```ron
/// data: [
///     "0a 00", // frame count
///     "ff ff", // flags
/// ],
/// ```
#[derive(Debug, Default, PartialEq)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    fn parse_hex(hex: &str) -> Result<Vec<u8>, String> {
        let digits = hex
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .map(|c| {
                c.to_digit(16)
                    .map(|d| d as u8)
                    .ok_or_else(|| format!("invalid hex digit '{c}'"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !digits.len().is_multiple_of(2) {
            return Err(format!("hex string \"{hex}\" has an odd number of digits"));
        }
        Ok(digits
            .chunks(2)
            .map(|pair| pair[0] << 4 | pair[1])
            .collect())
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = Bytes;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a hex string or a list of hex strings")
            }

            fn visit_str<E: de::Error>(self, hex: &str) -> Result<Bytes, E> {
                Bytes::parse_hex(hex).map(Bytes).map_err(E::custom)
            }

            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Bytes, A::Error> {
                let mut bytes = Vec::new();
                while let Some(hex) = seq.next_element::<&str>()? {
                    bytes.extend(Bytes::parse_hex(hex).map_err(de::Error::custom)?);
                }
                Ok(Bytes(bytes))
            }
        }

        deserializer.deserialize_any(BytesVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_swf_string() {
        let parse = |text| ron::from_str::<SwfString>(text).map(|s| s.0);
        assert_eq!(parse(r#""""#), Ok(String::new()));
        assert_eq!(parse(r#""hello""#), Ok("hello".to_owned()));
        assert_eq!(parse(r#""a\"b\\c\n""#), Ok("a\"b\\c\n".to_owned()));
        assert_eq!(parse(r#""\u{17c}""#), Ok("ż".to_owned()));
        assert!(parse("1").is_err());
        assert!(parse(r#"SwfString("hello")"#).is_err());
    }

    #[test]
    fn swf_string_as_swf_str() {
        let as_bytes = |s: &str| SwfString(s.to_owned()).as_swf_str().as_bytes().to_vec();
        assert_eq!(as_bytes(""), b"");
        assert_eq!(as_bytes("hello"), b"hello");
        assert_eq!(as_bytes("ż"), b"\xc5\xbc");
    }

    fn parse_bytes(text: &str) -> Result<Vec<u8>, String> {
        ron::from_str::<Bytes>(text)
            .map(|bytes| bytes.0)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn deserialize_bytes_from_string() {
        assert_eq!(parse_bytes(r#""""#), Ok(vec![]));
        assert_eq!(parse_bytes(r#""01 02 ff""#), Ok(vec![0x01, 0x02, 0xff]));
        assert_eq!(parse_bytes(r#""0102ff""#), Ok(vec![0x01, 0x02, 0xff]));
        assert_eq!(parse_bytes(r#""AbCd""#), Ok(vec![0xab, 0xcd]));
        assert_eq!(
            parse_bytes("\" 0 1\t02\n\r f f \""),
            Ok(vec![0x01, 0x02, 0xff])
        );
    }

    #[test]
    fn deserialize_bytes_from_list() {
        assert_eq!(parse_bytes("[]"), Ok(vec![]));
        assert_eq!(parse_bytes(r#"["01"]"#), Ok(vec![0x01]));
        assert_eq!(
            parse_bytes(r#"["01 02", "", "ff",]"#),
            Ok(vec![0x01, 0x02, 0xff])
        );
        assert_eq!(
            parse_bytes("[\n    \"01\", // one\n    \"02\", /* two */\n]"),
            Ok(vec![0x01, 0x02])
        );
    }

    #[test]
    fn deserialize_bytes_errors() {
        let parse_err = |text| parse_bytes(text).unwrap_err();
        assert!(parse_err(r#""0""#).contains("odd number of digits"));
        assert!(parse_err(r#""01 2""#).contains("odd number of digits"));
        assert!(parse_err(r#"["0", "1"]"#).contains("odd number of digits"));
        assert!(parse_err(r#""0g""#).contains("invalid hex digit 'g'"));
        assert!(parse_err(r#""0x01""#).contains("invalid hex digit 'x'"));
        assert!(parse_err(r#"["01", "zz"]"#).contains("invalid hex digit 'z'"));
        assert!(parse_err("1").contains("a hex string or a list of hex strings"));
        assert!(parse_bytes("[1]").is_err());
    }
}
