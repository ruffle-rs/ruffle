//! File results type.
//!
//! The `FileResults` type in this module is used to report results of a scan.

use serde::de::{Error as DesError, Unexpected, Visitor};
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Write;

#[derive(Serialize, Deserialize, Debug)]
pub enum AvmType {
    Avm1,
    Avm2,
}

/// How far we got through the scan before getting an error.
#[derive(Serialize, Deserialize, Debug)]
pub enum Progress {
    /// Nothing was able to be completed.
    ///
    /// Usually this indicates a significant problem
    Nothing,
    Read,
    Decompressed,
    Parsed,
    Executed,
    Completed,
}

/// The result of a single scan.
#[derive(Serialize, Deserialize, Debug)]
pub struct FileResults {
    pub name: String,

    #[serde(serialize_with = "into_hex", deserialize_with = "from_hex")]
    pub hash: Vec<u8>,
    pub progress: Progress,
    pub testing_time: u128,
    pub error: Option<String>,
    pub vm_type: Option<AvmType>,
}

/// Formats data as capital hex
fn into_hex<S>(hash: &[u8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut out = String::with_capacity(2 * hash.len());
    for byte in hash {
        write!(out, "{:02X}", byte).map_err(|e| SerError::custom(e.to_string()))?;
    }

    s.serialize_str(&out)
}

/// Parses hex strings into data
fn from_hex<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct HexVisitor();

    impl Visitor<'_> for HexVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "A string of hex digits with even length")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DesError,
        {
            let mut result = Vec::with_capacity(v.len() / 2);

            for i in (0..v.len()).step_by(2) {
                result.push(
                    u8::from_str_radix(
                        v.get(i..i + 2)
                            .ok_or_else(|| DesError::invalid_length(v.len(), &self))?,
                        16,
                    )
                    .map_err(|_| DesError::invalid_value(Unexpected::Str(v), &self))?,
                );
            }

            Ok(result)
        }
    }

    d.deserialize_str(HexVisitor())
}
