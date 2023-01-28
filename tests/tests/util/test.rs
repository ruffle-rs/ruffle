use crate::assert_eq;
use crate::set_logger;
use crate::util::options::TestOptions;
use crate::util::runner::run_swf;
use anyhow::{Context, Result};
use notify::Config;
use notify::EventKind;
use notify::RecursiveMode;
use notify::Watcher;
use notify::event::ModifyKind;
use ruffle_core::Player;
use ruffle_input_format::InputInjector;
use tempfile::TempDir;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use anyhow::anyhow;

pub struct Test {
    pub options: TestOptions,
    pub swf_path: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub name: String,
}

impl Test {
    pub fn from_options(options: TestOptions, test_dir: &Path, name: String) -> Result<Self> {
        let swf_path = test_dir.join("test.swf");
        let input_path = test_dir.join("input.json");
        let output_path = options.output_path(test_dir);
        Ok(Self {
            options,
            swf_path,
            input_path,
            output_path,
            name,
        })
    }

    pub fn from_options_file(options_path: &Path, name: String) -> Result<Self> {
        Self::from_options(
            TestOptions::read(options_path).context("Couldn't load test options")?,
            options_path
                .parent()
                .context("Couldn't get test directory")?,
            name,
        )
    }

    pub fn run(
        self,
        before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
        before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    ) -> std::result::Result<(), libtest_mimic::Failed> {
        set_logger();
        let injector = if self.input_path.is_file() {
            InputInjector::from_file(&self.input_path)?
        } else {
            InputInjector::empty()
        };
        let output = run_swf(&self, injector, before_start, before_end)?;
        self.compare_output(&output)?;
        Ok(())
    }

    pub fn should_run(&self, check_renderer: bool) -> bool {
        if self.options.ignore {
            return false;
        }
        self.options.player_options.can_run(check_renderer)
    }

    pub fn compare_output(&self, actual_output: &str) -> Result<()> {
        let expected_output = std::fs::read_to_string(&self.output_path)?.replace("\r\n", "\n");

        if let Some(approximations) = &self.options.approximations {
            std::assert_eq!(
                actual_output.lines().count(),
                expected_output.lines().count(),
                "# of lines of output didn't match"
            );

            for (actual, expected) in actual_output.lines().zip(expected_output.lines()) {
                // If these are numbers, compare using approx_eq.
                if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>())
                {
                    // NaNs should be able to pass in an approx test.
                    if actual.is_nan() && expected.is_nan() {
                        continue;
                    }

                    approximations.compare(actual, expected);
                } else {
                    let mut found = false;

                    // Check each of the user-provided regexes for a match
                    for pattern in approximations.number_patterns() {
                        if let (Some(actual_captures), Some(expected_captures)) =
                            (pattern.captures(actual), pattern.captures(expected))
                        {
                            found = true;
                            std::assert_eq!(
                                actual_captures.len(),
                                expected_captures.len(),
                                "Differing numbers of regex captures"
                            );

                            // Each capture group (other than group 0, which is always the entire regex
                            // match) represents a floating-point value
                            for (actual_val, expected_val) in actual_captures
                                .iter()
                                .skip(1)
                                .zip(expected_captures.iter().skip(1))
                            {
                                let actual_num = actual_val
                                    .expect("Missing capture group value for 'actual'")
                                    .as_str()
                                    .parse::<f64>()
                                    .expect("Failed to parse 'actual' capture group as float");
                                let expected_num = expected_val
                                    .expect("Missing capture group value for 'expected'")
                                    .as_str()
                                    .parse::<f64>()
                                    .expect("Failed to parse 'expected' capture group as float");
                                approximations.compare(actual_num, expected_num);
                            }
                            let modified_actual = pattern.replace(actual, "");
                            let modified_expected = pattern.replace(expected, "");

                            assert_eq!(modified_actual, modified_expected);
                            break;
                        }
                    }

                    if !found {
                        assert_eq!(actual, expected);
                    }
                }
            }
        } else {
            assert_eq!(
                actual_output, expected_output,
                "ruffle output != flash player output"
            );
        }

        #[cfg(feature = "fpcompare")]
        run_flash_player(&swf_path, &expected_output, options);

        Ok(())
    }
}

#[allow(dead_code)]
fn run_flash_player(swf_path: &str, expected_output: &str, options: &TestOptions) {
    if !options.fpcompare {
        return;
    }

    let dir = TempDir::new().unwrap();
    let agent_script_path = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("fprunner")
        .join("agent.js");

    let mm_cfg_path = PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join("fprunner")
        .join("mm.cfg")
        .to_str()
        .unwrap()
        .to_string();

    let frida_globals = format!("{{\"MM_CFG_PATH\": \"{mm_cfg_path}\"}}");

    let abs_swf_path = std::fs::canonicalize(swf_path).unwrap();

    let fp_path = if let Ok(path) = std::env::var("FLASH_PLAYER_DEBUGGER") {
        path
    } else {
        PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("fprunner")
            .join("download")
            .join("flashplayerdebugger")
            .to_str()
            .unwrap()
            .to_string()
    };

    let frida_out_path = dir.path().join("frida_stdout.log");
    let frida_out = File::create(&frida_out_path).unwrap();

    let flash_log_path = dir.path().join("flash_log.txt");
    // Create it in advance so that the `notify` crate can watch it
    File::create(&flash_log_path).unwrap();

    struct FlashPlayerKiller {
        pid_location: PathBuf,
    }

    impl Drop for FlashPlayerKiller {
        fn drop(&mut self) {
            if let Ok(pid) = std::fs::read_to_string(&self.pid_location) {
                let pid = pid.trim().parse::<u32>().unwrap();
                let status = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .status()
                    .unwrap_or_else(|e| {
                        panic!("Failed to spawn `kill` for flash player with pid {pid}: {e}",)
                    });
                if !status.success() {
                    eprintln!("Failed to kill flash player with pid {pid}");
                }
            }
        }
    }

    let _killer = FlashPlayerKiller {
        pid_location: dir.path().join("flashplayer_pid"),
    };

    let _child = Command::new("frida")
        .args(&[
            "-l",
            agent_script_path.to_str().unwrap(),
            "--parameters",
            &frida_globals,
            "-f",
            &fp_path,
            abs_swf_path.to_str().unwrap(),
        ])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(frida_out.try_clone().unwrap())
        .stderr(frida_out.try_clone().unwrap())
        .spawn()
        .unwrap();

    let mut fp_output = match tail_lines(
        &flash_log_path,
        Duration::new(5, 0),
        Duration::new(5, 0),
        expected_output.lines().count(),
    ) {
        Ok(fp_output) => fp_output,
        Err(e) => {
            eprintln!(
                "Failed to get output from Flash Player. Frida logs: {}",
                std::fs::read_to_string(frida_out_path).unwrap()
            );
            panic!("Failed to get output: {e:?}");
        }
    };

    // Normalize this in the same way that we normalize our own trace output.
    fp_output = fp_output.replace('\r', "\n");

    assert_eq!(
        fp_output,
        expected_output,
        "Real Flash Player output does not match expected output\n\nFrida output:\n```\n{}```\n",
        std::fs::read_to_string(&frida_out_path).unwrap(),
    );
}

fn tail_lines(
    path: &Path,
    write_wait_timeout: Duration,
    overall_timeout: Duration,
    expected_lines: usize,
) -> Result<String, anyhow::Error> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = sender.send(res);
    })?;

    watcher.configure(Config::default().with_poll_interval(Duration::from_secs(1)))?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    let mut file = std::fs::File::open(path)?;
    let mut data = String::new();
    let mut found_lines = 0;

    let start = Instant::now();

    while Instant::now() - start < overall_timeout {
        match receiver.recv_timeout(write_wait_timeout)? {
            Ok(event) if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) => {
                let mut new_data = String::new();
                file.read_to_string(&mut new_data)?;
                found_lines += new_data
                    .chars()
                    .filter(|c| *c == '\r' || *c == '\n')
                    .count();
                data += &new_data;
                if found_lines >= expected_lines {
                    return Ok(data);
                }
            }
            _ => {}
        }
    }

    Err(anyhow!("Timed out waiting for all lines to be read"))
}