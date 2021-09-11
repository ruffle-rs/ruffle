//! File results type.
//!
//! The `FileResults` type in this module is used to report results of a scan.

use serde::de::{Error as DesError, Unexpected, Visitor};
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AvmType {
    Avm1,
    Avm2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Compression {
    None,
    Zlib,
    Lzma,
}

impl From<swf::Compression> for Compression {
    fn from(sc: swf::Compression) -> Self {
        match sc {
            swf::Compression::None => Compression::None,
            swf::Compression::Zlib => Compression::Zlib,
            swf::Compression::Lzma => Compression::Lzma,
        }
    }
}

/// A particular step in the scanner process.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Step {
    /// Nothing has been done yet.
    ///
    /// Usually this indicates a significant problem unrelated to Ruffle, or a
    /// scanner child process panic.
    Start,

    /// Reading of the file into memory and computing it's SHA256 hash.
    Read,

    /// Decompression of the file data into a SWF bytestream.
    Decompress,

    /// Parsing of the decompressed SWF.
    Parse,

    /// Execution of the SWF in Ruffle.
    Execute,

    /// Completion of all prior steps without error.
    Complete,
}

/// The result of a single scan.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileResults {
    /// The file name scanned (including path).
    #[serde(rename = "Filename")]
    pub name: String,

    /// The SHA256 hash of the SWF file.
    #[serde(
        rename = "SHA256 Hash",
        serialize_with = "into_hex",
        deserialize_with = "from_hex"
    )]
    pub hash: Vec<u8>,

    /// How far we were able to process this particular SWF
    #[serde(rename = "Progress")]
    pub progress: Step,

    /// How long testing took to complete
    #[serde(rename = "Test Duration")]
    pub testing_time: u128,

    /// The compressed length of the SWF file.
    #[serde(rename = "Compressed Length")]
    pub compressed_len: Option<usize>,

    /// The uncompressed length of the SWF file.
    #[serde(rename = "Uncompressed Length")]
    pub uncompressed_len: Option<u32>,

    /// Any errors encountered while testing.
    #[serde(rename = "Error")]
    pub error: Option<String>,

    /// The compression type this SWF uses.
    #[serde(rename = "Compression")]
    pub compression: Option<Compression>,

    /// The file format version of this SWF.
    #[serde(rename = "SWF Version")]
    pub version: Option<u8>,

    /// The stage size of this SWF.
    #[serde(rename = "Stage Size")]
    pub stage_size: Option<String>,

    /// The frame rate of this SWF.
    #[serde(rename = "Frame Rate")]
    pub frame_rate: Option<f32>,

    /// The number of frames this SWF claims to contain.
    #[serde(rename = "Number of Frames")]
    pub num_frames: Option<u16>,

    /// Whether or not the SWF requests hardware-accelerated presentation.
    #[serde(rename = "Direct Blit")]
    pub use_direct_blit: Option<bool>,

    /// Whether or not the SWF requests hardware-accelerated compositing.
    #[serde(rename = "GPU")]
    pub use_gpu: Option<bool>,

    /// Whether or not the SWF requests network access when ran locally.
    #[serde(rename = "Network Sandbox")]
    pub use_network_sandbox: Option<bool>,

    /// The AVM type of the movie.
    #[serde(rename = "AVM Version")]
    pub vm_type: Option<AvmType>,
}

impl Default for FileResults {
    fn default() -> Self {
        FileResults::new("")
    }
}

impl FileResults {
    pub fn new(name: &str) -> Self {
        FileResults {
            name: name.to_string(),
            hash: vec![],
            progress: Step::Start,
            testing_time: 0,
            compressed_len: None,
            uncompressed_len: None,
            error: None,
            compression: None,
            version: None,
            stage_size: None,
            frame_rate: None,
            num_frames: None,
            use_direct_blit: None,
            use_gpu: None,
            use_network_sandbox: None,
            vm_type: None,
        }
    }
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
