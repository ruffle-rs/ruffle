pub mod approximations;
pub mod expression;
pub mod font;
pub mod image_comparison;
pub mod known_failure;
pub mod player;
pub mod shared_object;

use crate::compiler::SwfCompilerOptions;
use crate::environment::Environment;
use crate::image_trigger::ImageTrigger;
use crate::options::approximations::Approximations;
use crate::options::expression::TestExpression;
use crate::options::font::{DefaultFontsOptions, FontOptions, FontSortOptions};
use crate::options::image_comparison::ImageComparison;
use crate::options::known_failure::KnownFailure;
use crate::options::player::PlayerOptions;
use anyhow::{Result, bail};
use ruffle_render::quality::StageQuality;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, OnceLock};
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
                    // No subtest-specific field, use the base value
                    tconfig.insert(k.clone(), vbase.clone());
                }
            }
            return Ok(());
        }
        (DeValue::Table(_), DeValue::Array(_)) => "cannot merge table into array",
        (DeValue::Array(_), DeValue::Table(_)) => "cannot merge array into table",
        (DeValue::Array(_), DeValue::Array(_)) => "merging arrays isn't supported",
        // Otherwise, the subtest-specific value wins
        _ => return Ok(()),
    };

    Err(toml::de::Error::custom(format_args!(
        "{err_msg} (path = {path})"
    )))
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum FrameSelection {
    Array(Vec<u32>),
    Range { from: u32, to: u32 },
}

impl FrameSelection {
    pub fn includes_frame(&self, frame: u32) -> bool {
        match self {
            FrameSelection::Array(frames) => frames.contains(&frame),
            &FrameSelection::Range { from, to } => from <= frame && frame <= to,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioAssertion {
    pub frames: FrameSelection,
    pub max_amplitude: Option<f32>,
    pub min_max_amplitude: Option<f32>,
    #[serde(default)]
    pub known_failure: bool,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedObjectConfig {
    pub expected: String,
    #[serde(default)]
    pub known_failure: bool,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestSharedOptions {
    #[serde(default)]
    pub compilers: Vec<SwfCompilerOptions>,

    // Used to only run compilers once per test group.
    #[serde(skip, default)]
    pub(crate) compile_ok: OnceLock<bool>,
}

impl TestSharedOptions {
    const FIELD_NAMES: &[&str] = &["compilers"];

    fn default_arc() -> Arc<Self> {
        static DEFAULT: LazyLock<Arc<TestSharedOptions>> = LazyLock::new(|| {
            Arc::new(TestSharedOptions {
                compilers: Vec::new(),
                compile_ok: OnceLock::from(true),
            })
        });
        Arc::clone(&*DEFAULT)
    }

    fn validate(&self) -> Result<()> {
        for options in &self.compilers {
            options.validate()?;
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TestOptions {
    // Only set when the `test.toml` file has multiple subtest,
    // which we handle manually.
    #[serde(skip)]
    pub subtest_name: Option<String>,

    // Options shared by every subtest in a test group.
    // This will also be handled manually.
    #[serde(skip, default = "TestSharedOptions::default_arc")]
    pub shared: Arc<TestSharedOptions>,

    pub num_frames: Option<u32>,
    pub num_ticks: Option<u32>,
    pub tick_rate: Option<f64>,
    pub output_path: String,
    pub sleep_to_meet_frame_rate: bool,
    pub image_comparisons: HashMap<String, ImageComparison>,
    pub audio_assertions: HashMap<String, AudioAssertion>,
    pub log_warnings: bool,
    pub ignore: bool,
    pub filter: Option<TestExpression>,
    pub known_failure: KnownFailure,
    pub approximations: Option<Approximations>,
    pub player_options: PlayerOptions,
    pub log_fetch: bool,
    pub required_features: RequiredFeatures,
    pub fonts: HashMap<String, FontOptions>,
    pub font_sorts: HashMap<String, FontSortOptions>,
    pub default_fonts: DefaultFontsOptions,
    pub shared_objects: HashMap<String, SharedObjectConfig>,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            subtest_name: None,
            shared: TestSharedOptions::default_arc(),
            num_frames: None,
            num_ticks: None,
            tick_rate: None,
            output_path: "output.txt".to_string(),
            sleep_to_meet_frame_rate: false,
            image_comparisons: Default::default(),
            audio_assertions: Default::default(),
            log_warnings: true,
            ignore: false,
            filter: None,
            known_failure: KnownFailure::None,
            approximations: None,
            player_options: PlayerOptions::default(),
            log_fetch: false,
            required_features: RequiredFeatures::default(),
            fonts: Default::default(),
            font_sorts: Default::default(),
            default_fonts: Default::default(),
            shared_objects: Default::default(),
        }
    }
}

impl TestOptions {
    pub fn read_with_subtests(path: &VfsPath) -> Result<Vec<Self>> {
        let contents = path.read_to_string()?;
        let tests = Self::deserialize_toml(&contents).map_err(|mut e| {
            // This improves TOML error messages.
            e.set_input(Some(&contents));
            e
        })?;

        if let Some(first) = tests.first() {
            first.shared.validate()?;
        }
        for test in &tests {
            test.validate()?;
        }

        Ok(tests)
    }

    fn deserialize_toml(contents: &str) -> Result<Vec<Self>, toml::de::Error> {
        use serde::de::{Error, IntoDeserializer};

        let mut raw = DeTable::parse(contents)?;

        // `TestOptions` doesn't actually have this field, so remove it.
        let raw_subtests = raw.get_mut().remove("subtests");

        // Manually pull out shared options.
        let shared = Arc::new({
            let shared: DeTable = TestSharedOptions::FIELD_NAMES
                .iter()
                .filter_map(|name| raw.get_mut().remove_entry(*name))
                .collect();
            let shared = Spanned::new(raw.span(), shared);
            TestSharedOptions::deserialize(shared.into_deserializer())?
        });

        let subtests = match raw_subtests.map(|spanned| spanned.into_inner()) {
            // This is a single test, parse and return it.
            None => {
                let mut parsed = Self::deserialize(raw.into_deserializer())?;
                parsed.shared = shared;
                return Ok(vec![parsed]);
            }
            Some(DeValue::Table(table)) if table.len() >= 2 => table,
            Some(_) => {
                return Err(Error::custom(
                    "'subtests' field should be a table with at least two entries",
                ));
            }
        };

        // Merge default values into each subtest, and deserialize the result.
        let raw = Spanned::new(0..contents.len(), DeValue::Table(raw.into_inner()));
        subtests
            .into_iter()
            .map(|(name, mut raw_config)| {
                // Don't allow shared options in subtests.
                if let DeValue::Table(table) = raw_config.get_ref() {
                    for field in TestSharedOptions::FIELD_NAMES {
                        if table.contains_key(*field) {
                            return Err(Error::custom(
                                "The '{field}' field cannot be overriden in subtests.",
                            ));
                        }
                    }
                }

                let mut name = name.into_inner().into_owned();

                // Note: when this returns, `name` will be restored to its original value.
                merge_into_subtest(&raw, &mut raw_config, &mut name)?;

                let mut parsed = Self::deserialize(raw_config.into_deserializer())?;
                parsed.subtest_name = Some(name);
                parsed.shared = shared.clone();
                Ok(parsed)
            })
            .collect()
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

        if let Some(approx) = &self.approximations {
            approx.validate()?;
        }

        Ok(())
    }

    pub fn has_known_failure(&self) -> bool {
        !matches!(self.known_failure, KnownFailure::None)
            || self.image_comparisons.values().any(|cmp| cmp.known_failure)
            || self.shared_objects.values().any(|so| so.known_failure)
    }

    pub fn output_path(&self, test_directory: &VfsPath) -> Result<VfsPath> {
        Ok(test_directory.join(&self.output_path)?)
    }

    pub fn can_run(&self, check_renderer: bool, environment: &impl Environment) -> bool {
        self.required_features.can_run()
            && self
                .filter
                .as_ref()
                .is_none_or(|f| f.evaluate().expect("invalid 'filter' expression"))
            && self.player_options.can_run(check_renderer, environment)
    }
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RequiredFeatures {
    lzma: bool,
    jpegxr: bool,
}

impl RequiredFeatures {
    fn can_run(&self) -> bool {
        (!self.lzma || cfg!(feature = "lzma")) && (!self.jpegxr || cfg!(feature = "jpegxr"))
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    optional: bool,
    quality: Quality,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            optional: false,
            quality: Quality(StageQuality::High),
        }
    }
}

impl RenderOptions {
    pub fn quality(&self) -> StageQuality {
        self.quality.0
    }
}

#[derive(Clone, Copy)]
struct Quality(StageQuality);

impl<'de> Deserialize<'de> for Quality {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let quality = std::str::FromStr::from_str(&s)
            .map_err(|_| serde::de::Error::custom(format!("Unknown quality: {s}")))?;
        Ok(Quality(quality))
    }
}
