use crate::backends::TestLogBackend;
use crate::options::approximations::Approximations;
use crate::options::known_failure::KnownFailure;
use anyhow::{Error, anyhow};
use pretty_assertions::Comparison;
use vfs::VfsPath;

pub fn compare_trace_output(
    log: &TestLogBackend,
    expected_path: &VfsPath,
    approx: Option<&Approximations>,
    known_failure: &KnownFailure,
) -> anyhow::Result<()> {
    let expected_trace = expected_path.read_to_string()?.replace("\r\n", "\n");

    let ruffle_expected_path = path_with_suffix(expected_path, "ruffle")?;

    // Null bytes are invisible, and interfere with constructing
    // the expected output.txt file. Any tests dealing with null
    // bytes should explicitly test for them in ActionScript.
    let actual_trace = log.trace_output().replace('\0', "");

    let result = test("flash_expected", approx, &expected_trace, &actual_trace);

    match known_failure {
        KnownFailure::None | KnownFailure::Panic { .. } => {
            if result.is_ok() && ruffle_expected_path.exists()? {
                Err(anyhow!(
                    "Unexpected `{}` file for passing check, please remove it!",
                    ruffle_expected_path.as_str(),
                ))
            } else {
                result
            }
        }
        KnownFailure::TraceOutput {
            ruffle_check: false,
        } => {
            if result.is_ok() {
                Err(anyhow!(
                    "Trace output check was known to be failing, but now passes successfully. \
                    Please update the test and remove `known_failure = true`!"
                ))
            } else {
                Ok(())
            }
        }
        KnownFailure::TraceOutput { ruffle_check: true } => {
            if result.is_ok() {
                return Err(anyhow!(
                    "Trace output check was known to be failing, but now passes successfully. \
                    Please update the test, and remove `known_failure = true` and `{}`!",
                    ruffle_expected_path.as_str(),
                ));
            }

            let expected_trace = if ruffle_expected_path.exists()? {
                ruffle_expected_path.read_to_string()?.replace("\r\n", "\n")
            } else {
                ruffle_expected_path
                    .create_file()?
                    .write_all(actual_trace.as_bytes())?;
                return Err(anyhow!(
                    "No trace to compare to! Saved actual trace as Ruffle-expected."
                ));
            };

            test("ruffle_expected", approx, &expected_trace, &actual_trace)
        }
    }
}

fn test(
    expected_name: &str,
    approx: Option<&Approximations>,
    expected_output: &str,
    actual_output: &str,
) -> anyhow::Result<()> {
    if let Some(approx) = approx {
        let add_comparison_to_err = |err: Error| -> Error {
            let left_pretty = PrettyString(actual_output);
            let right_pretty = PrettyString(expected_output);
            let comparison = Comparison::new(&left_pretty, &right_pretty);

            anyhow!("{}\n\n{}\n", err, comparison)
        };

        if actual_output.lines().count() != expected_output.lines().count() {
            return Err(anyhow!(
                "# of lines of output didn't match ({expected_name}: {}, ruffle_actual: {})",
                expected_output.lines().count(),
                actual_output.lines().count()
            ));
        }

        for (actual, expected) in actual_output.lines().zip(expected_output.lines()) {
            // If these are numbers, compare using approx_eq.
            if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
                // NaNs should be able to pass in an approx test.
                if actual.is_nan() && expected.is_nan() {
                    continue;
                }

                approx
                    .compare(actual, expected)
                    .map_err(add_comparison_to_err)?;
            } else {
                let mut found = false;

                // Check each of the user-provided regexes for a match
                for pattern in approx.number_patterns() {
                    if let (Some(actual_captures), Some(expected_captures)) =
                        (pattern.captures(actual), pattern.captures(expected))
                    {
                        found = true;
                        if expected_captures.len() != actual_captures.len() {
                            return Err(anyhow!(
                                "Differing numbers of regex captures ({expected_name}: {}, ruffle_actual: {})",
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
                            approx
                                .compare(actual_num, expected_num)
                                .map_err(add_comparison_to_err)?;
                        }
                        let modified_actual = pattern.replace_all(actual, "");
                        let modified_expected = pattern.replace_all(expected, "");

                        assert_text_matches(
                            modified_actual.as_ref(),
                            modified_expected.as_ref(),
                            expected_name,
                        )?;
                        break;
                    }
                }

                if !found {
                    assert_text_matches(actual, expected, expected_name)?;
                }
            }
        }
    } else {
        assert_text_matches(actual_output, expected_output, expected_name)?;
    }

    Ok(())
}

/// Wrapper around string slice that makes debug output `{:?}` to print string same way as `{}`.
/// Used in different `assert*!` macros in combination with `pretty_assertions` crate to make
/// test failures to show nice diffs.
/// Courtesy of https://github.com/colin-kiegel/rust-pretty-assertions/issues/24
#[derive(PartialEq, Eq)]
#[doc(hidden)]
struct PrettyString<'a>(pub &'a str);

/// Make diff to display string as multi-line string
impl std::fmt::Debug for PrettyString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

fn assert_text_matches(ruffle: &str, expected: &str, expected_name: &str) -> anyhow::Result<()> {
    if ruffle != expected {
        let left_pretty = PrettyString(ruffle);
        let right_pretty = PrettyString(expected);
        let comparison = Comparison::new(&left_pretty, &right_pretty);

        Err(anyhow!(
            "assertion failed: `(ruffle_actual == {expected_name})`\
                       \n\
                       \n{}\
                       \n",
            comparison
        ))
    } else {
        Ok(())
    }
}

/// Adds a suffix to the filename: e.g. `foo/bar.txt` -> `foo/bar.suffix.txt`
fn path_with_suffix(path: &VfsPath, suffix: &str) -> anyhow::Result<VfsPath> {
    // `VfsPath`'s API is less nice than std's `Path`... :(
    let mut name = path.filename();
    name.insert_str(0, "../");
    if let Some(ext) = path.extension() {
        name.truncate(name.len() - ext.len());
        name.push_str(suffix);
        name.push('.');
        name.push_str(&ext);
    } else {
        name.push('.');
        name.push_str(suffix);
    }
    Ok(path.join(&name)?)
}
