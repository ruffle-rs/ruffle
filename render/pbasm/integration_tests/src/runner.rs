//! Integration tests runner for pbasm.

use anyhow::{anyhow, Context, Result};
use libtest_mimic::Trial;
use pbasm::{run_main, Opt};
use ruffle_fs_tests_runner::{FsTestsRunner, TestLoaderParams};
use serde::Deserialize;
use std::path::Path;
use std::{borrow::Cow, path::PathBuf};
use vfs::VfsPath;

const TEST_TOML_NAME: &str = "test.toml";

#[derive(Clone, Copy, Deserialize)]
enum TestType {
    Roundtrip,
    Assemble,
    Dissassemble,
}

impl TestType {
    fn perform_assembly(self) -> bool {
        matches!(self, TestType::Assemble | TestType::Roundtrip)
    }

    fn perform_disassembly(self) -> bool {
        matches!(self, TestType::Dissassemble | TestType::Roundtrip)
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TestOptions {
    pub r#type: TestType,
    pub ignore: bool,
    pub pbj_path: String,
    pub pbasm_path: String,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            r#type: TestType::Roundtrip,
            ignore: false,
            pbj_path: "test.pbj".to_owned(),
            pbasm_path: "test.pbasm".to_owned(),
        }
    }
}

impl TestOptions {
    fn read(path: &VfsPath) -> Result<Self> {
        let result = toml::from_str(&path.read_to_string()?)?;
        Ok(result)
    }

    fn pbj_path(&self, test_dir: &VfsPath) -> Result<VfsPath> {
        test_dir
            .join(&self.pbj_path)
            .context("Failed to get pbj path")
    }

    fn pbasm_path(&self, test_dir: &VfsPath) -> Result<VfsPath> {
        test_dir
            .join(&self.pbasm_path)
            .context("Failed to get pbasm path")
    }
}

fn main() {
    let mut runner = FsTestsRunner::new();

    runner
        .with_descriptor_name(Cow::Borrowed(TEST_TOML_NAME))
        .with_test_loader(Box::new(|params| Some(load_test(params))));

    runner.run()
}

fn load_test(params: TestLoaderParams) -> Trial {
    let test_dir = params.test_dir.clone();
    let test_dir_real = params.test_dir_real.into_owned();
    let name = params.test_name;

    let descriptor_path = test_dir.join("test.toml").unwrap();

    let options = TestOptions::read(&descriptor_path)
        .map_err(|e| anyhow!("Failed to parse {}: {e}", descriptor_path.as_str()))
        .expect("Failed to parse test descriptor");
    let ignore = options.ignore;

    let mut trial = Trial::test(name.to_string(), move || {
        let pbj_path = options.pbj_path(&test_dir)?;
        let pbj_path_real = to_real_path(&test_dir_real, &pbj_path);
        let pbasm_path = options.pbasm_path(&test_dir)?;
        let pbasm_path_real = to_real_path(&test_dir_real, &pbasm_path);

        let pbj_actual_path = test_dir_real.join("actual.pbj");
        let pbasm_actual_path = test_dir_real.join("actual.pbasm");

        if options.r#type.perform_assembly() {
            let opt = Opt {
                source: pbasm_path_real.to_str().unwrap().to_string(),
                disassemble: false,
                output: Some(pbj_actual_path.to_str().unwrap().to_string()),
            };
            run_test(opt, &pbj_path_real)?;
        }

        if options.r#type.perform_disassembly() {
            let opt = Opt {
                source: pbj_path_real.to_str().unwrap().to_string(),
                disassemble: true,
                output: Some(pbasm_actual_path.to_str().unwrap().to_string()),
            };
            run_test(opt, &pbasm_path_real)?;
        }

        Ok(())
    });
    if ignore {
        trial = trial.with_ignored_flag(true);
    }
    trial
}

fn to_real_path(real_dir: &Path, file: &VfsPath) -> PathBuf {
    real_dir.join(file.as_str().strip_prefix('/').unwrap())
}

fn run_test(opt: Opt, expected_path: &Path) -> Result<()> {
    let actual_path = opt.output.clone().unwrap();
    run_main(opt).map_err(|e: anyhow::Error| anyhow!("Failed to execute pbasm: {e}"))?;

    let actual = std::fs::read(&actual_path)?;
    let expected = std::fs::read(expected_path).map_err(|e| {
        anyhow!(
            "Error reading test file {}: {e}",
            expected_path.to_string_lossy()
        )
    })?;

    if actual != expected {
        return Err(anyhow!(
            "Test failed: Output doesn't match: {}",
            actual_path.to_string()
        ));
    }

    let _ = std::fs::remove_file(actual_path);
    Ok(())
}
