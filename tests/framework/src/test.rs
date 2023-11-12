use crate::environment::Environment;
use crate::options::TestOptions;
use crate::runner::run_swf;
use crate::set_logger;
use crate::util::read_bytes;
use anyhow::{anyhow, Result};
use pretty_assertions::Comparison;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::Player;
use ruffle_input_format::InputInjector;
use ruffle_socket_format::SocketEvent;
use std::sync::{Arc, Mutex};
use vfs::VfsPath;

pub struct Test {
    pub options: TestOptions,
    pub swf_path: VfsPath,
    pub input_path: VfsPath,
    pub socket_path: VfsPath,
    pub output_path: VfsPath,
    pub root_path: VfsPath,
    pub name: String,
}

impl Test {
    pub fn from_options(options: TestOptions, test_dir: VfsPath, name: String) -> Result<Self> {
        let swf_path = test_dir.join("test.swf")?;
        let input_path = test_dir.join("input.json")?;
        let socket_path = test_dir.join("socket.json")?;
        let output_path = options.output_path(&test_dir)?;

        Ok(Self {
            options,
            swf_path,
            input_path,
            socket_path,
            output_path,
            root_path: test_dir,
            name,
        })
    }

    pub fn run(
        &self,
        mut before_start: impl FnMut(Arc<Mutex<Player>>) -> Result<()>,
        mut before_end: impl FnMut(Arc<Mutex<Player>>) -> Result<()>,
        environment: &impl Environment,
    ) -> Result<()> {
        set_logger();

        let data = read_bytes(&self.swf_path)?;
        let movie = SwfMovie::from_data(&data, format!("file:///{}", self.swf_path.as_str()), None)
            .map_err(|e| anyhow!(e.to_string()))?;
        let viewport_dimensions = self.options.player_options.viewport_dimensions(&movie);
        let renderers = self
            .options
            .player_options
            .create_renderer(environment, viewport_dimensions);

        if renderers.is_empty() {
            let output = run_swf(
                self,
                movie,
                self.input_injector()?,
                self.socket_events()?,
                &mut before_start,
                &mut before_end,
                None,
                viewport_dimensions,
            )?;
            self.compare_output(&output)?;
        } else {
            for renderer in renderers {
                let output = run_swf(
                    self,
                    movie.clone(),
                    self.input_injector()?,
                    self.socket_events()?,
                    &mut before_start,
                    &mut before_end,
                    Some(renderer),
                    viewport_dimensions,
                )?;
                self.compare_output(&output)?;
            }
        }

        Ok(())
    }

    fn socket_events(&self) -> Result<Option<Vec<SocketEvent>>> {
        Ok(if self.socket_path.is_file()? {
            Some(SocketEvent::from_reader(
                &read_bytes(&self.socket_path)?[..],
            )?)
        } else {
            None
        })
    }

    fn input_injector(&self) -> Result<InputInjector> {
        Ok(if self.input_path.is_file()? {
            InputInjector::from_reader(&read_bytes(&self.input_path)?[..])?
        } else {
            InputInjector::empty()
        })
    }

    pub fn should_run(&self, check_renderer: bool, environment: &impl Environment) -> bool {
        if self.options.ignore {
            return false;
        }
        self.options.required_features.can_run()
            && self
                .options
                .player_options
                .can_run(check_renderer, environment)
    }

    pub fn compare_output(&self, actual_output: &str) -> Result<()> {
        let expected_output = self.output_path.read_to_string()?.replace("\r\n", "\n");

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

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
/// Courtesy of https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl<'a> std::fmt::Debug for PrettyString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
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
