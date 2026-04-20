use anyhow::anyhow;
use serde::Deserialize;

/// Test expression is a cfg-like expression that evaluates to a boolean
/// and can be used in test configuration.
///
/// Currently the following variables are supported:
/// * `os` --- refers to [`std::env::consts::OS`],
/// * `arch` --- refers to [`std::env::consts::ARCH`],
/// * `family` --- refers to [`std::env::consts::FAMILY`].
///
/// Example expression:
///
/// ```text
/// not(os = "aarch64")
/// ```
#[derive(Deserialize, Clone, Debug)]
pub struct TestExpression(String);

impl TestExpression {
    pub fn evaluate(&self) -> anyhow::Result<bool> {
        let cfg_parsed = cfg_expr::Expression::parse(&self.0)
            .map_err(|err| anyhow!("Cannot parse expression:\n{err}"))?;
        let mut unknown_pred = None;
        let cfg_matches = cfg_parsed.eval(|pred| match pred {
            cfg_expr::Predicate::KeyValue { key, val } if *key == "os" => {
                *val == std::env::consts::OS
            }
            cfg_expr::Predicate::KeyValue { key, val } if *key == "arch" => {
                *val == std::env::consts::ARCH
            }
            cfg_expr::Predicate::KeyValue { key, val } if *key == "family" => {
                *val == std::env::consts::FAMILY
            }
            _ => {
                unknown_pred = Some(format!("{pred:?}"));
                false
            }
        });
        if let Some(pred) = unknown_pred {
            return Err(anyhow!("Unknown predicate used in expression: {pred}"));
        }
        Ok(cfg_matches)
    }
}
