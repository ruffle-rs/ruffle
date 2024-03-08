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

/// The preferences that relate to the application itself.
///
/// This structure is safe to clone, internally it holds an Arc to any mutable properties.
///
/// The general priority order for preferences should look as follows, where top is "highest priority":
/// - User-selected movie-specific setting (if applicable, such as through Open Advanced)
/// - Movie-specific settings (if applicable and we implement this, stored on disk)
/// - CLI (if applicable)
/// - Persisted preferences (if applicable, saved to toml)
/// - Ruffle defaults
#[derive(Clone)]
pub struct GlobalPreferences {
    /// As the CLI holds properties ranging from initial movie settings (ie url),
    /// to application itself (ie render backend),
    /// this field is available for checking where needed.
    // TODO: This should really not be public and we should split up CLI somehow,
    // or make it all getters in here?
    pub cli: Opt,

    /// The actual, mutable user preferences that are persisted to disk.
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
                toml_document: document,
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
        self.cli.graphics.unwrap_or_else(|| {
            self.preferences
                .lock()
                .expect("Preferences is not reentrant")
                .values
                .graphics_backend
        })
    }

    pub fn graphics_power_preference(&self) -> PowerPreference {
        self.cli.power.unwrap_or_else(|| {
            self.preferences
                .lock()
                .expect("Preferences is not reentrant")
                .values
                .graphics_power_preference
        })
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

    pub fn mute(&self) -> bool {
        self.preferences
            .lock()
            .expect("Preferences is not reentrant")
            .values
            .mute
    }

    pub fn preferred_volume(&self) -> f32 {
        self.cli.volume.unwrap_or_else(|| {
            self.preferences
                .lock()
                .expect("Preferences is not reentrant")
                .values
                .volume
        })
    }

    pub fn write_preferences(&self, fun: impl FnOnce(&mut PreferencesWriter)) -> Result<(), Error> {
        let mut preferences = self
            .preferences
            .lock()
            .expect("Preferences is not reentrant");

        let mut writer = PreferencesWriter::new(&mut preferences);
        fun(&mut writer);

        let serialized = preferences.toml_document.to_string();
        std::fs::write(self.cli.config.join("preferences.toml"), serialized)
            .context("Could not write preferences to disk")
    }
}

/// The actual preferences that are persisted to disk, and mutable at runtime.
/// Care should be taken to only modify what's actually changed, using `GlobalPreferences::write_preferences`.
///
/// Two versions of Ruffle may have different preferences, or different values available for each preference.
/// For this reason, we store both the original toml document *and* the parsed values as we understand them.
/// Whenever we persist values back to the toml, we only edit the values we changed and leave the remaining
/// values as they originally were.
/// This way, switching between different versions will *not* wipe your settings or get Ruffle into an
/// invalid state.
#[derive(Default)]
struct PreferencesAndDocument {
    /// The original toml document
    toml_document: Document,

    /// The actual preferences stored within the toml document, as this version of Ruffle understands them.
    values: SavedGlobalPreferences,
}

#[derive(PartialEq, Debug)]
pub struct SavedGlobalPreferences {
    pub graphics_backend: GraphicsBackend,
    pub graphics_power_preference: PowerPreference,
    pub language: LanguageIdentifier,
    pub output_device: Option<String>,
    pub mute: bool,
    pub volume: f32,
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
            mute: false,
            volume: 1.0,
        }
    }
}
