use crate::util::options::TestOptions;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Test {
    pub options: TestOptions,
    pub swf_path: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub name: String,
}

impl Test {
    pub fn from_options(options_path: &Path, root_dir: &Path) -> Result<Self> {
        let test_dir = options_path
            .parent()
            .context("Couldn't get test directory")?;
        let options = TestOptions::read(options_path).context("Couldn't load test options")?;
        let swf_path = test_dir.join("test.swf");
        let input_path = test_dir.join("input.json");
        let output_path = test_dir.join("output.txt");
        let name = test_dir
            .strip_prefix(root_dir)
            .context("Couldn't strip root prefix from test dir")?
            .to_string_lossy()
            .replace('\\', "/");
        Ok(Self {
            options,
            swf_path,
            input_path,
            output_path,
            name,
        })
    }
}
