//! Pixel Bender assembly tests runner.

use anyhow::{anyhow, Context, Result};
use libtest_mimic::Trial;
use pixel_bender::assembly::PixelBenderShaderAssembly;
use pixel_bender::disassembly::PixelBenderShaderDisassembly;
use pixel_bender::parse_shader;
use ruffle_fs_tests_runner::{FsTestsRunner, TestLoaderParams};
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
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
    fn performs_assembly(self) -> bool {
        matches!(self, TestType::Assemble | TestType::Roundtrip)
    }

    fn performs_disassembly(self) -> bool {
        matches!(self, TestType::Dissassemble | TestType::Roundtrip)
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TestOptions {
    pub r#type: TestType,
    pub ignore: bool,
    pub pbj_path: String,
    pub asm_path: String,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            r#type: TestType::Roundtrip,
            ignore: false,
            pbj_path: "test.pbj".to_owned(),
            asm_path: "test.pbasm".to_owned(),
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

    fn asm_path(&self, test_dir: &VfsPath) -> Result<VfsPath> {
        test_dir
            .join(&self.asm_path)
            .context("Failed to get asm path")
    }
}

fn main() {
    let mut runner = FsTestsRunner::new();

    runner
        .with_descriptor_name(Cow::Borrowed(TEST_TOML_NAME))
        .with_test_loader(Box::new(|params, register_trial| {
            register_trial(load_test(params))
        }));

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
        let asm_path = options.asm_path(&test_dir)?;
        let asm_path_real = to_real_path(&test_dir_real, &asm_path);

        let pbj_actual_path = test_dir_real.join("actual.pbj");
        let asm_actual_path = test_dir_real.join("actual.pbasm");

        if options.r#type.performs_assembly() {
            let source = asm_path_real.to_str().unwrap().to_string();
            let output = pbj_actual_path.to_str().unwrap().to_string();

            run_test(&pbj_path_real, &pbj_actual_path, move || {
                let input =
                    std::fs::read_to_string(&source).context("Failed to open source file")?;
                let mut write =
                    File::create(output).map_err(|e| anyhow!("Failed to create file: {e}"))?;
                let assembly = PixelBenderShaderAssembly::new(&input, &mut write);
                assembly.assemble()?;
                Ok(())
            })?;
        }

        if options.r#type.performs_disassembly() {
            let source = pbj_path_real.to_str().unwrap().to_string();
            let output = asm_actual_path.to_str().unwrap().to_string();

            run_test(&asm_path_real, &asm_actual_path, move || {
                let data = std::fs::read(&source).context("Failed to open source file")?;
                let parsed = parse_shader(&data, false)
                    .map_err(|e| anyhow!("Failed to parse the shader: {e}"))?;
                let mut write =
                    File::create(output).map_err(|e| anyhow!("Failed to create file: {e}"))?;
                write!(write, "{}", PixelBenderShaderDisassembly(&parsed))
                    .context("Failed to write disassembly")?;
                Ok(())
            })?;
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

fn run_test<F>(expected_path: &Path, actual_path: &Path, proc: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    proc().map_err(|e: anyhow::Error| anyhow!("Failed to execute (dis)assembly: {e}"))?;

    let actual = std::fs::read(actual_path)?;
    let expected = std::fs::read(expected_path).map_err(|e| {
        anyhow!(
            "Error reading test file {}: {e}",
            expected_path.to_string_lossy()
        )
    })?;

    if actual != expected {
        return Err(anyhow!(
            "Test failed: Output doesn't match: {}",
            actual_path.to_string_lossy()
        ));
    }

    let _ = std::fs::remove_file(actual_path);
    Ok(())
}
