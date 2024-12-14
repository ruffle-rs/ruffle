use chrono::Utc;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub enum FilenamePattern {
    #[default]
    SingleFile,
    WithTimestamp,
}

impl FromStr for FilenamePattern {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single_file" => Ok(FilenamePattern::SingleFile),
            "with_timestamp" => Ok(FilenamePattern::WithTimestamp),
            _ => Err(()),
        }
    }
}

impl FilenamePattern {
    pub fn create_path(&self, directory: &Path) -> PathBuf {
        match self {
            FilenamePattern::SingleFile => directory.join("ruffle.log"),
            FilenamePattern::WithTimestamp => {
                directory.join(Utc::now().format("ruffle_%F_%H-%M-%S.log").to_string())
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FilenamePattern::SingleFile => "single_file",
            FilenamePattern::WithTimestamp => "with_timestamp",
        }
    }
}
