use crate::image_trigger::ImageTrigger;
use crate::options::expression::TestExpression;
use anyhow::anyhow;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct ImageComparison {
    tolerance: Option<u8>,
    max_outliers: Option<usize>,
    checks: Vec<ImageComparisonCheck>,
    pub trigger: ImageTrigger,
    pub known_failure: bool,
}

impl ImageComparison {
    pub fn checks(&self) -> anyhow::Result<Cow<'_, [ImageComparisonCheck]>> {
        let has_simple_check = self.tolerance.is_some() || self.max_outliers.is_some();
        if has_simple_check && !self.checks.is_empty() {
            return Err(anyhow!(
                "Both simple and advanced checks are defined. \
                Either remove 'tolerance' & 'max_outliers', or move it to 'checks'."
            ));
        }

        if !self.checks.is_empty() {
            Ok(Cow::Borrowed(&self.checks))
        } else {
            Ok(Cow::Owned(vec![ImageComparisonCheck {
                tolerance: self.tolerance.unwrap_or_default(),
                max_outliers: self.max_outliers.unwrap_or_default(),
                filter: None,
            }]))
        }
    }
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct ImageComparisonCheck {
    pub tolerance: u8,
    pub max_outliers: usize,

    pub filter: Option<TestExpression>,
}
