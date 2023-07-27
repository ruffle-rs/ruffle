use crate::set_logger;
use crate::util::options::TestOptions;
use crate::util::runner::run_swf;
use crate::util::PrettyString;
use anyhow::{anyhow, Context, Result};
use pretty_assertions::Comparison;
use ruffle_core::Player;
use ruffle_input_format::InputInjector;
use ruffle_socket_format::SocketEvent;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct Test {
    pub options: TestOptions,
    pub swf_path: PathBuf,
    pub input_path: PathBuf,
    pub socket_path: PathBuf,
    pub output_path: PathBuf,
    pub name: String,
}

impl Test {
    pub fn from_options(options: TestOptions, test_dir: &Path, name: String) -> Result<Self> {
        let swf_path = test_dir.join("test.swf");
        let input_path = test_dir.join("input.json");
        let socket_path = test_dir.join("socket.json");
        let output_path = options.output_path(test_dir);

        Ok(Self {
            options,
            swf_path,
            input_path,
            socket_path,
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
        &self,
        before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
        before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    ) -> std::result::Result<(), libtest_mimic::Failed> {
        set_logger();
        let injector = if self.input_path.is_file() {
            InputInjector::from_file(&self.input_path)?
        } else {
            InputInjector::empty()
        };
        let socket_events = if self.socket_path.is_file() {
            Some(SocketEvent::from_file(&self.socket_path)?)
        } else {
            None
        };
        let output = run_swf(self, injector, socket_events, before_start, before_end)?;
        self.compare_output(&output)?;
        Ok(())
    }

    pub fn should_run(&self, check_renderer: bool) -> bool {
        if self.options.ignore {
            return false;
        }
        self.options.required_features.can_run()
            && self.options.player_options.can_run(check_renderer)
    }

    pub fn compare_output(&self, actual_output: &str) -> Result<()> {
        let expected_output = std::fs::read_to_string(&self.output_path)?.replace("\r\n", "\n");

        if let Some(approximations) = &self.options.approximations {
            if actual_output.lines().count() != expected_output.lines().count() {
                return Err(anyhow!(
                    "# of lines of output didn't match (expected {} from Flash, got {} from Ruffle",
                    expected_output.lines().count(),
                    actual_output.lines().count()
                ));
            }

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
                            if expected_captures.len() != actual_captures.len() {
                                return Err(anyhow!(
                                    "Differing numbers of regex captures (expected {}, actually {})",
                                    expected_captures.len(),
                                    actual_captures.len(),
                                ));
                            }

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
                            let modified_actual = pattern.replace_all(actual, "");
                            let modified_expected = pattern.replace_all(expected, "");

                            assert_text_matches(
                                modified_actual.as_ref(),
                                modified_expected.as_ref(),
                            )?;
                            break;
                        }
                    }

                    if !found {
                        assert_text_matches(actual, expected)?;
                    }
                }
            }
        } else {
            assert_text_matches(actual_output, &expected_output)?;
        }

        Ok(())
    }
}

fn assert_text_matches(ruffle: &str, flash: &str) -> Result<()> {
    if flash != ruffle {
        let left_pretty = PrettyString(ruffle);
        let right_pretty = PrettyString(flash);
        let comparison = Comparison::new(&left_pretty, &right_pretty);

        Err(anyhow!(
            "assertion failed: `(flash_expected == ruffle_actual)`\
                       \n\
                       \n{}\
                       \n",
            comparison
        ))
    } else {
        Ok(())
    }
}
