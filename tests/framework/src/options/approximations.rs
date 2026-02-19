use anyhow::bail;
use approx::relative_eq;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Approximations {
    #[serde(default)]
    pub bare_numbers: bool,
    number_patterns: Vec<String>,
    epsilon: Option<f64>,
    max_relative: Option<f64>,
}

impl Approximations {
    pub fn compare(&self, actual: f64, expected: f64) -> anyhow::Result<()> {
        let result = match (self.epsilon, self.max_relative) {
            (Some(epsilon), Some(max_relative)) => relative_eq!(
                actual,
                expected,
                epsilon = epsilon,
                max_relative = max_relative
            ),
            (Some(epsilon), None) => relative_eq!(actual, expected, epsilon = epsilon),
            (None, Some(max_relative)) => {
                relative_eq!(actual, expected, max_relative = max_relative)
            }
            (None, None) => relative_eq!(actual, expected),
        };

        if result {
            Ok(())
        } else {
            bail!(
                "Approximation failed: expected {}, found {}. Epsilon = {:?}, Max Relative = {:?}",
                expected,
                actual,
                self.epsilon,
                self.max_relative
            )
        }
    }

    pub(super) fn validate(&self) -> anyhow::Result<()> {
        if !self.bare_numbers && self.number_patterns.is_empty() {
            bail!(
                "approximations with `bare_numbers = false` should have at least one explicit number pattern"
            )
        }

        // TODO: consider checking number_pattern regexes too?

        Ok(())
    }

    pub fn number_patterns(&self) -> Vec<Regex> {
        self.number_patterns
            .iter()
            .map(|p| Regex::new(p).unwrap())
            .collect()
    }
}
