use anyhow::anyhow;
use approx::relative_eq;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Approximations {
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
            Err(anyhow!(
                "Approximation failed: expected {}, found {}. Epsilon = {:?}, Max Relative = {:?}",
                expected,
                actual,
                self.epsilon,
                self.max_relative
            ))
        }
    }

    pub fn number_patterns(&self) -> Vec<Regex> {
        self.number_patterns
            .iter()
            .map(|p| Regex::new(p).unwrap())
            .collect()
    }
}
