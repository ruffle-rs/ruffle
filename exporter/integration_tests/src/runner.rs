//! Integration tests runner for exporter.

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use exporter::cli::Opt;
use libtest_mimic::Trial;
use ruffle_fs_tests_runner::{FsTestsRunner, TestLoaderParams};
use serde::Deserialize;
use std::borrow::Cow;
use std::io::Read;
use std::process::Stdio;
use vfs::{VfsError, VfsPath};

const TEST_TOML_NAME: &str = "test.toml";

/// This environment variable is set for child processes.
/// If it's defined, it means that it's the child process.
const CHILD_PROCESS_ENV_NAME: &str = "__RUFFLE_EXPORTER_TEST_CHILD_PROCESS__";

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct TestOptions {
    pub args: Vec<String>,
    pub swf: String,
    pub ignore: bool,
    pub input_dir: Option<String>,
    pub output_dir: Option<String>,
    pub expected_status: Option<i32>,
    pub expected_stdout: Option<String>,
    pub expected_stderr: Option<String>,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            args: Vec::new(),
            swf: "test.swf".to_owned(),
            ignore: false,
            input_dir: None,
            output_dir: None,
            expected_status: None,
            expected_stdout: None,
            expected_stderr: None,
        }
    }
}

impl TestOptions {
    fn read(path: &VfsPath) -> Result<Self> {
        let result: Self = toml::from_str(&path.read_to_string()?)?;
        Ok(result)
    }

    fn input_directory(&self, test_dir: &VfsPath) -> Result<VfsPath> {
        self.input_dir
            .as_ref()
            .map(|d| test_dir.join(d))
            .unwrap_or_else(|| test_dir.join("input"))
            .context("Failed to get input directory")
    }

    fn output_directory(&self, test_dir: &VfsPath) -> Result<VfsPath> {
        self.output_dir
            .as_ref()
            .map(|d| test_dir.join(d))
            .unwrap_or_else(|| test_dir.join("output"))
            .context("Failed to get output directory")
    }

    fn actual_directory(&self, test_dir: &VfsPath) -> Result<VfsPath> {
        test_dir
            .join("actual")
            .context("Failed to get actual directory")
    }
}

fn main() -> Result<()> {
    if std::env::var(CHILD_PROCESS_ENV_NAME).is_ok() {
        let opt: Opt = Opt::parse();
        return exporter::run_main(opt);
    }

    let mut runner = FsTestsRunner::new();

    runner
        // We're switching directories, so we cannot use relative paths.
        .with_canonicalize_paths(true)
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
        .map_err(|e| {
            anyhow!(
                "Failed to parse {} in {}: {e}",
                descriptor_path.as_str(),
                test_dir_real.to_string_lossy()
            )
        })
        .expect("Failed to parse test descriptor");
    let ignore = options.ignore;
    let swf_path = test_dir
        .join(&options.swf)
        .map_err(|e| anyhow!("Failed to get SWF path: {e}"))
        .unwrap();

    let mut trial = Trial::test(name.to_string(), move || {
        let input_dir = options.input_directory(&test_dir)?;
        let output_dir = options.output_directory(&test_dir)?;
        let actual_dir = options.actual_directory(&test_dir)?;

        let _ = actual_dir.remove_dir_all();

        if input_dir.exists()? {
            input_dir
                .copy_dir(&actual_dir)
                .map_err(|e| anyhow!("Failed to prepare actual directory: {e}"))?;
        } else {
            actual_dir.create_dir()?;
        }

        let actual_dir_real = test_dir_real.join(actual_dir.as_str().trim_start_matches(['/']));
        let swf_path_real = test_dir_real.join(swf_path.as_str().trim_start_matches(['/']));
        let swf_path_real_str = swf_path_real.to_string_lossy();

        let mut args: Vec<&str> = Vec::new();
        args.push(&swf_path_real_str);
        for arg in &options.args {
            args.push(arg);
        }

        // Spawn a child process with a different working directory.
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(args)
            .current_dir(actual_dir_real)
            .env(CHILD_PROCESS_ENV_NAME, "")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child process");

        let exist_status = child.wait().expect("Failed to wait for child process");

        let stdout = std::io::read_to_string(child.stdout.take().expect("handle present"))?;
        let stderr = std::io::read_to_string(child.stderr.take().expect("handle present"))?;

        match (exist_status.code(), options.expected_status) {
            (Some(0), None) => {
                // Works as expected!
            }
            (Some(actual), None) => {
                return Err(
                    anyhow!("Expected exporter to succeed, but it returned {actual}, stderr:\n{stderr}\nstdout:\n{stdout}").into(),
                );
            }
            (Some(actual), Some(expected)) if actual == expected => {
                // Works as expected!
            }
            (Some(actual), Some(expected)) => {
                return Err(anyhow!(
                    "Expected exporter to return {expected}, but it returned {actual}, stderr:\n{stderr}\nstdout:\n{stdout}"
                )
                .into());
            }
            (None, _) => {
                return Err(anyhow!("Child process was terminated by a signal").into());
            }
        }

        if let Some(expected_stdout) = &options.expected_stdout {
            if &stdout != expected_stdout {
                return Err(anyhow!(
                    "Unexpected stdout. Expected:\n{expected_stdout}\nActual:\n{stdout}"
                )
                .into());
            }
        }

        if let Some(expected_stderr) = &options.expected_stderr {
            if &stderr != expected_stderr {
                return Err(anyhow!(
                    "Unexpected stderr. Expected:\n{expected_stderr}\nActual:\n{stderr}"
                )
                .into());
            }
        }

        verify_dirs(&actual_dir, &output_dir, &input_dir)
            .map_err(|err| anyhow!("Failed to verify files: {err}"))?;

        let _ = actual_dir.remove_dir_all();

        Ok(())
    });
    if ignore {
        trial = trial.with_ignored_flag(true);
    }
    trial
}

fn verify_dirs(actual_dir: &VfsPath, expected_dir: &VfsPath, input_dir: &VfsPath) -> Result<()> {
    if matches!(expected_dir.exists(), Ok(true)) {
        for expected_file in expected_dir
            .walk_dir()
            .map_err(|err| anyhow!("Error reading output directory: {err}"))?
        {
            let expected_file = expected_file?;
            let actual_file = rebase_path(&expected_file, expected_dir, actual_dir)?;

            if expected_file.is_dir()? {
                if !actual_file.is_dir()? {
                    return Err(anyhow!(
                        "Expected {} to be a directory",
                        actual_file.as_str()
                    ));
                }
            } else if !actual_file.is_file()? {
                return Err(anyhow!("Expected {} to be a file", actual_file.as_str()));
            } else {
                let expected_content = read_bytes(&expected_file)?;
                let actual_content = read_bytes(&actual_file)?;

                if expected_file.as_str().ends_with(".png") {
                    if !images_equal(&expected_content, &actual_content)? {
                        return Err(anyhow!(
                            "Image {} is different than expected",
                            actual_file.as_str()
                        ));
                    }
                } else if expected_content != actual_content {
                    return Err(anyhow!(
                        "File {} has different content than expected",
                        actual_file.as_str()
                    ));
                }
            }
        }
    }

    for actual_file in actual_dir
        .walk_dir()
        .map_err(|err| anyhow!("Error reading actual directory: {err}"))?
    {
        let actual_file = actual_file?;
        let expected_file = rebase_path(&actual_file, actual_dir, expected_dir)?;
        let input_file = rebase_path(&actual_file, actual_dir, input_dir)?;

        // In case there's a new file (not present in input) that we didn't expect.
        if !expected_file.exists()? && !input_file.exists()? {
            return Err(anyhow!("Unexpected file: {}", actual_file.as_str()));
        }
    }

    Ok(())
}

fn rebase_path(path: &VfsPath, base: &VfsPath, new_base: &VfsPath) -> Result<VfsPath, VfsError> {
    let relative_path = path
        .as_str()
        .strip_prefix(&format!("{}/", base.as_str()))
        .expect("Path does not start with base");
    new_base.join(relative_path)
}

fn read_bytes(file: &VfsPath) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    file.open_file()
        .map_err(|err| anyhow!("Failed opening file: {err}"))?
        .read_to_end(&mut content)
        .map_err(|err| anyhow!("Failed reading file: {err}"))?;
    Ok(content)
}

fn images_equal(expected_content: &[u8], actual_content: &[u8]) -> Result<bool> {
    // TODO Maybe use the same method of image comparison as in core tests?
    let expected = image::load_from_memory(expected_content)
        .map_err(|e| anyhow!("Failed to open expected image: {e}"))?;
    let actual = image::load_from_memory(actual_content)
        .map_err(|e| anyhow!("Failed to open actual image: {e}"))?;
    Ok(expected == actual)
}
