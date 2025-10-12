use crate::options::TestOptions;
use anyhow::{anyhow, Error};
use pretty_assertions::Comparison;
use vfs::VfsPath;

pub fn compare_trace_output(
    expected_path: &VfsPath,
    options: &TestOptions,
    actual_output: &str,
) -> anyhow::Result<()> {
    let expected_output = expected_path.read_to_string()?.replace("\r\n", "\n");

    if let Some(approximations) = &options.approximations {
        let add_comparison_to_err = |err: Error| -> Error {
            let left_pretty = PrettyString(actual_output);
            let right_pretty = PrettyString(&expected_output);
            let comparison = Comparison::new(&left_pretty, &right_pretty);

            anyhow!("{}\n\n{}\n", err, comparison)
        };

        if actual_output.lines().count() != expected_output.lines().count() {
            return Err(anyhow!(
                "# of lines of output didn't match (expected {} from Flash, got {} from Ruffle",
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

                approximations
                    .compare(actual, expected)
                    .map_err(add_comparison_to_err)?;
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
                            approximations
                                .compare(actual_num, expected_num)
                                .map_err(add_comparison_to_err)?;
                        }
                        let modified_actual = pattern.replace_all(actual, "");
                        let modified_expected = pattern.replace_all(expected, "");

                        assert_text_matches(modified_actual.as_ref(), modified_expected.as_ref())?;
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

fn assert_text_matches(ruffle: &str, flash: &str) -> anyhow::Result<()> {
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
