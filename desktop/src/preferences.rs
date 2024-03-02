mod read;
mod write;

use crate::cli::Opt;
use crate::preferences::read::read_preferences;
use crate::preferences::write::PreferencesWriter;
use anyhow::{Context, Error};
use ruffle_core::backend::ui::US_ENGLISH;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::sync::{Arc, Mutex};
use sys_locale::get_locale;
use toml_edit::Document;
use unic_langid::LanguageIdentifier;

#[derive(Clone)]
pub struct GlobalPreferences {
    pub cli: Opt,
    preferences: Arc<Mutex<PreferencesAndDocument>>,
}

impl GlobalPreferences {
    pub fn load(cli: Opt) -> Result<Self, Error> {
        std::fs::create_dir_all(&cli.config).context("Failed to create configuration directory")?;
        let preferences_path = cli.config.join("preferences.toml");
        let preferences = if preferences_path.exists() {
            let contents = std::fs::read_to_string(&preferences_path)
                .context("Failed to read saved preferences")?;
            let (result, document) = read_preferences(&contents);
            for warning in result.warnings {
                // TODO: A way to display warnings to users, generally
                tracing::warn!("{warning}");
            }
            PreferencesAndDocument {
                document,
                values: result.result,
            }
        } else {
            Default::default()
        };

        Ok(Self {
            cli,
            preferences: Arc::new(Mutex::new(preferences)),
        })
    }

    pub fn graphics_backends(&self) -> GraphicsBackend {
        self.cli.graphics.unwrap_or(
            self.preferences
                .lock()
                .expect("Preferences is not reentrant")
                .values
                .graphics_backend,
        )
    }

    pub fn graphics_power_preference(&self) -> PowerPreference {
        self.cli.power.unwrap_or(
            self.preferences
                .lock()
                .expect("Preferences is not reentrant")
                .values
                .graphics_power_preference,
        )
    }

    pub fn language(&self) -> LanguageIdentifier {
        self.preferences
            .lock()
            .expect("Preferences is not reentrant")
            .values
            .language
            .clone()
    }

    pub fn output_device_name(&self) -> Option<String> {
        self.preferences
            .lock()
            .expect("Preferences is not reentrant")
            .values
            .output_device
            .clone()
    }

    pub fn write_preferences(&self, fun: impl FnOnce(&mut PreferencesWriter)) -> Result<(), Error> {
        let mut preferences = self
            .preferences
            .lock()
            .expect("Preferences is not reentrant");

        let mut writer = PreferencesWriter::new(&mut preferences);
        fun(&mut writer);

        let serialized = preferences.document.to_string();
        std::fs::write(self.cli.config.join("preferences.toml"), serialized)
            .context("Could not write preferences to disk")
    }
}

#[derive(Default)]
struct PreferencesAndDocument {
    document: Document,
    values: SavedGlobalPreferences,
}

#[derive(PartialEq, Debug)]
pub struct SavedGlobalPreferences {
    pub graphics_backend: GraphicsBackend,
    pub graphics_power_preference: PowerPreference,
    pub language: LanguageIdentifier,
    pub output_device: Option<String>,
}

impl Default for SavedGlobalPreferences {
    fn default() -> Self {
        let preferred_locale = get_locale();
        let locale = preferred_locale
            .and_then(|l| l.parse().ok())
            .unwrap_or_else(|| US_ENGLISH.clone());
        Self {
            graphics_backend: Default::default(),
            graphics_power_preference: Default::default(),
            language: locale,
            output_device: None,
        }
    }
}
