pub mod approximations;
pub mod expression;
pub mod font;
pub mod image_comparison;
pub mod known_failure;
pub mod player;

use crate::image_trigger::ImageTrigger;
use crate::options::approximations::Approximations;
use crate::options::font::{DefaultFontsOptions, FontOptions, FontSortOptions};
use crate::options::image_comparison::ImageComparison;
use crate::options::known_failure::KnownFailure;
use crate::options::player::PlayerOptions;
use anyhow::{Result, bail};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use toml::Spanned;
use toml::de::{DeTable, DeValue};
use vfs::VfsPath;

fn merge_into_subtest<'a>(
    base: &Spanned<DeValue<'a>>,
    config: &mut Spanned<DeValue<'a>>,
    path: &mut String,
) -> Result<(), toml::de::Error> {
    use serde::de::Error as _;
    let err_msg = match (base.get_ref(), config.get_mut()) {
        // Table into table, recurse into fields
        (DeValue::Table(tbase), DeValue::Table(tconfig)) => {
            let old_path_len = path.len();
            for (k, vbase) in tbase {
                if let Some(vconfig) = tconfig.get_mut(k) {
                    path.push('.');
                    path.push_str(k.get_ref());
                    merge_into_subtest(vbase, vconfig, path)?;
                    path.truncate(old_path_len);
                } else {
                    tconfig.insert(k.clone(), vbase.clone());
                }
            }
            return Ok(());
        }
        (DeValue::Table(_), DeValue::Array(_)) => "cannot merge table into array",
        (DeValue::Array(_), DeValue::Table(_)) => "cannot merge array into table",
        (DeValue::Array(_), DeValue::Array(_)) => "merging arrays isn't supported",
        // Otherwise, overwrite the whole value.
        _ => {
            *config = base.clone();
            return Ok(());
        }
    };

    Err(toml::de::Error::custom(format_args!(
        "{err_msg} (path = {path})"
    )))
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TestOptions {
    // Only set when the `test.toml` file has multiple configs,
    // which we handle manually.
    #[serde(skip)]
    pub subtest_name: Option<String>,

    pub num_frames: Option<u32>,
    pub num_ticks: Option<u32>,
    pub tick_rate: Option<f64>,
    pub output_path: String,
    pub sleep_to_meet_frame_rate: bool,
    pub image_comparisons: HashMap<String, ImageComparison>,
    pub ignore: bool,
    pub known_failure: KnownFailure,
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
            subtest_name: None,
            num_frames: None,
            num_ticks: None,
            tick_rate: None,
            output_path: "output.txt".to_string(),
            sleep_to_meet_frame_rate: false,
            image_comparisons: Default::default(),
            ignore: false,
            known_failure: KnownFailure::None,
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
    pub fn read_with_subtests(path: &VfsPath) -> Result<Vec<Self>> {
        use serde::de::{Error, IntoDeserializer};

        let contents = path.read_to_string()?;

        // This improves TOML error messages.
        let err_with_input = |mut e: toml::de::Error| {
            e.set_input(Some(&contents));
            e
        };

        let mut raw = DeTable::parse(&contents)?;
        // `TestOptions` doesn't actually have this field, so remove it.
        let raw_subtests = raw.get_mut().remove("subtests");

        let subtests = match raw_subtests.map(|spanned| spanned.into_inner()) {
            // This is a single test, parse and return it.
            None => {
                let parsed = Self::deserialize(raw.into_deserializer()).map_err(err_with_input)?;
                parsed.validate()?;
                return Ok(vec![parsed]);
            }
            Some(DeValue::Table(table)) => table,
            Some(_) => bail!(err_with_input(Error::custom(
                "'configs' field should be a table"
            ))),
        };

        if subtests.len() < 2 {
            bail!("If present, the [subtests] table must have at least two entries.");
        }

        // Merge default values into each subtest, and deserialize the result.
        let raw = Spanned::new(0..contents.len(), DeValue::Table(raw.into_inner()));
        subtests
            .into_iter()
            .map(|(name, mut raw_config)| {
                let mut name = name.into_inner().into_owned();

                // Note: when this returns, `name` will be restored to its original value.
                merge_into_subtest(&raw, &mut raw_config, &mut name).map_err(err_with_input)?;

                let mut parsed =
                    Self::deserialize(raw_config.into_deserializer()).map_err(err_with_input)?;

                parsed.subtest_name = Some(name);
                parsed.validate()?;
                Ok(parsed)
            })
            .collect()
    }

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
                    bail!(
                        "Multiple captures are set to trigger {:?}. This likely isn't intended!",
                        comparison.trigger
                    );
                }
            }
        }

        Ok(())
    }

    pub fn has_known_failure(&self) -> bool {
        !matches!(self.known_failure, KnownFailure::None)
            || self.image_comparisons.values().any(|cmp| cmp.known_failure)
    }

    pub fn output_path(&self, test_directory: &VfsPath) -> Result<VfsPath> {
        Ok(test_directory.join(&self.output_path)?)
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
