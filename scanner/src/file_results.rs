//! File results type.
//!
//! The `FileResults` type in this module is used to report results of a scan.

use serde::{Deserialize, Serialize};

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
    pub progress: Progress,
    pub testing_time: u128,
    pub error: Option<String>,
    pub vm_type: Option<AvmType>,
}
