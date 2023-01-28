use crate::assert_eq;
use crate::set_logger;
use crate::util::options::TestOptions;
use crate::util::runner::run_swf;
use anyhow::{Context, Result};
use ruffle_core::Player;
use ruffle_input_format::InputInjector;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct Test {
    pub options: TestOptions,
    pub swf_path: PathBuf,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub name: String,
}

impl Test {
    pub fn from_options(options: TestOptions, test_dir: &Path, root_dir: &Path) -> Result<Self> {
        let swf_path = test_dir.join("test.swf");
        let input_path = test_dir.join("input.json");
        let output_path = options.output_path(test_dir);
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

    pub fn from_options_file(options_path: &Path, root_dir: &Path) -> Result<Self> {
        Self::from_options(
            TestOptions::read(options_path).context("Couldn't load test options")?,
            options_path
                .parent()
                .context("Couldn't get test directory")?,
            root_dir,
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
        let mut expected_output = std::fs::read_to_string(&self.output_path)?.replace("\r\n", "\n");

        // Strip a trailing newline if it has one.
        if expected_output.ends_with('\n') {
            expected_output = expected_output[0..expected_output.len() - "\n".len()].to_string();
        }

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

        Ok(())
    }
}
