pub mod expression;
pub mod font;
pub mod image_comparison;
pub mod player;

use crate::image_trigger::ImageTrigger;
use crate::options::font::{DefaultFontsOptions, FontOptions, FontSortOptions};
use crate::options::image_comparison::ImageComparison;
use crate::options::player::PlayerOptions;
use anyhow::{anyhow, Result};
use approx::relative_eq;
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use vfs::VfsPath;

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TestOptions {
    pub num_frames: Option<u32>,
    pub num_ticks: Option<u32>,
    pub tick_rate: Option<f64>,
    pub output_path: String,
    pub sleep_to_meet_frame_rate: bool,
    pub image_comparisons: HashMap<String, ImageComparison>,
    pub ignore: bool,
    pub known_failure: bool,
    pub approximations: Option<Approximations>,
    pub player_options: PlayerOptions,
    pub log_fetch: bool,
    pub required_features: RequiredFeatures,
    pub fonts: HashMap<String, FontOptions>,
    pub font_sorts: HashMap<String, FontSortOptions>,
    pub default_fonts: DefaultFontsOptions,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            num_frames: None,
            num_ticks: None,
            tick_rate: None,
            output_path: "output.txt".to_string(),
            sleep_to_meet_frame_rate: false,
            image_comparisons: Default::default(),
            ignore: false,
            known_failure: false,
            approximations: None,
            player_options: PlayerOptions::default(),
            log_fetch: false,
            required_features: RequiredFeatures::default(),
            fonts: Default::default(),
            font_sorts: Default::default(),
            default_fonts: Default::default(),
        }
    }
}

impl TestOptions {
    pub fn read(path: &VfsPath) -> Result<Self> {
        let result: Self = toml::from_str(&path.read_to_string()?)?;
        result.validate()?;
        Ok(result)
    }

    fn validate(&self) -> Result<()> {
        if !self.image_comparisons.is_empty() {
            let mut seen_triggers = HashSet::new();
            for comparison in self.image_comparisons.values() {
                if comparison.trigger != ImageTrigger::FsCommand
                    && !seen_triggers.insert(comparison.trigger)
                {
                    return Err(anyhow!(
                        "Multiple captures are set to trigger {:?}. This likely isn't intended!",
                        comparison.trigger
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn output_path(&self, test_directory: &VfsPath) -> Result<VfsPath> {
        Ok(test_directory.join(&self.output_path)?)
    }
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Approximations {
    number_patterns: Vec<String>,
    epsilon: Option<f64>,
    max_relative: Option<f64>,
}

impl Approximations {
    pub fn compare(&self, actual: f64, expected: f64) -> Result<()> {
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
                "Approximation failed: expected {}, found {}. Episilon = {:?}, Max Relative = {:?}",
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

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RequiredFeatures {
    lzma: bool,
    jpegxr: bool,
}

impl RequiredFeatures {
    pub fn can_run(&self) -> bool {
        (!self.lzma || cfg!(feature = "lzma")) && (!self.jpegxr || cfg!(feature = "jpegxr"))
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    optional: bool,
    pub sample_count: u32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            optional: false,
            sample_count: 1,
        }
    }
}
