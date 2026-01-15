use std::fmt;

use serde::{
    Deserialize, Deserializer,
    de::{self, value::MapAccessDeserializer},
};

#[derive(Clone, Debug, Default)]
pub enum KnownFailure {
    #[default]
    None,
    TraceOutput {
        ruffle_check: bool,
    },
    Panic {
        message: String,
    },
}

impl<'de> Deserialize<'de> for KnownFailure {
    fn deserialize<D: Deserializer<'de>>(deser: D) -> Result<Self, D::Error> {
        deser.deserialize_any(KnownFailureVisitor)
    }
}

struct KnownFailureVisitor;

impl<'de> de::Visitor<'de> for KnownFailureVisitor {
    type Value = KnownFailure;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a boolean, `.ruffle_check = false`, or `.panic = 'message'`")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
        if v {
            Ok(KnownFailure::TraceOutput { ruffle_check: true })
        } else {
            Ok(KnownFailure::None)
        }
    }

    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        enum Raw {
            #[serde(rename = "panic")]
            Panic(String),
            #[serde(rename = "ruffle_check")]
            RuffleCheck(bool),
        }

        match Raw::deserialize(MapAccessDeserializer::new(map))? {
            Raw::Panic(message) => Ok(KnownFailure::Panic { message }),
            Raw::RuffleCheck(ruffle_check) => Ok(KnownFailure::TraceOutput { ruffle_check }),
        }
    }
}
