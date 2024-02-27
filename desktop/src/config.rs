use crate::cli::Opt;
use anyhow::{Context, Error};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct GlobalPreferences {
    pub cli: Opt,
    preferences: Arc<Mutex<SavedGlobalPreferences>>,
}

impl GlobalPreferences {
    pub fn load(cli: Opt) -> Result<Self, Error> {
        std::fs::create_dir_all(&cli.config).context("Failed to create configuration directory")?;
        let preferences_path = cli.config.join("preferences.toml");
        let preferences = if preferences_path.exists() {
            toml::from_str(
                &std::fs::read_to_string(&preferences_path)
                    .context("Failed to read saved preferences")?,
            )
            .context("Failed to parse saved preferences")?
        } else {
            SavedGlobalPreferences::default()
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
                .graphics_backend,
        )
    }

    pub fn graphics_power_preference(&self) -> PowerPreference {
        self.cli.power.unwrap_or(
            self.preferences
                .lock()
                .expect("Preferences is not reentrant")
                .graphics_power_preference,
        )
    }

    pub fn write_preferences(
        &self,
        writer: impl FnOnce(&mut SavedGlobalPreferences),
    ) -> Result<(), Error> {
        let mut preferences = self
            .preferences
            .lock()
            .expect("Preferences is not reentrant");
        writer(&mut preferences);
        let serialized =
            toml::to_string(preferences.deref()).context("Could not serialize preferences")?;
        std::fs::write(self.cli.config.join("preferences.toml"), serialized)
            .context("Could not write preferences to disk")
    }
}

// [NA] Deliberately not "deny_unknown" here, trying to keep this backwards & forwards compatible.
// It's quite common to bisect, even users manually hop back and forth to test things.
// Therefore, try to take care to not error out if something is totally wrong.
// TODO: Maybe a custom deserializer?
#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SavedGlobalPreferences {
    #[serde(skip_serializing_if = "GraphicsBackend::is_default")]
    pub graphics_backend: GraphicsBackend,

    #[serde(skip_serializing_if = "PowerPreference::is_default")]
    pub graphics_power_preference: PowerPreference,
}
