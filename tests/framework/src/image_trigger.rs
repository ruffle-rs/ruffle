use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum ImageTrigger {
    #[default]
    LastFrame,
    SpecificIteration(u32),
    FsCommand,
}

impl<'de> Deserialize<'de> for ImageTrigger {
    fn deserialize<D>(deserializer: D) -> Result<ImageTrigger, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_i32(ImageTriggerVisitor)
    }
}

struct ImageTriggerVisitor;

impl Visitor<'_> for ImageTriggerVisitor {
    type Value = ImageTrigger;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("one of either: a numeric frame/tick number, the string \"last_frame\", or the string \"fs_command\"")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value >= 0 && value <= i64::from(u32::MAX) {
            Ok(ImageTrigger::SpecificIteration(value as u32))
        } else {
            Err(E::custom(format!("i64 out of range: {}", value)))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value <= u64::from(u32::MAX) {
            Ok(ImageTrigger::SpecificIteration(value as u32))
        } else {
            Err(E::custom(format!("u64 out of range: {}", value)))
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(frame) = value.parse::<u32>() {
            Ok(ImageTrigger::SpecificIteration(frame))
        } else if value == "last_frame" {
            Ok(ImageTrigger::LastFrame)
        } else if value == "fs_command" {
            Ok(ImageTrigger::FsCommand)
        } else {
            Err(E::unknown_variant(
                value,
                &["'last_frame'", "'fs_command'", "a frame/tick number"],
            ))
        }
    }
}
