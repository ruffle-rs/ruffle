use crate::cli::Opt;
use anyhow::{Context, Error};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use serde::{Deserialize, Serialize};

pub struct GlobalPreferences {
    pub cli: Opt,
    preferences: SavedGlobalPreferences,
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

        Ok(Self { cli, preferences })
    }

    pub fn graphics_backends(&self) -> wgpu::Backends {
        self.cli
            .graphics
            .or(self.preferences.graphics_backend)
            .unwrap_or(GraphicsBackend::Default)
            .into()
    }

    pub fn graphics_power_preference(&self) -> wgpu::PowerPreference {
        self.cli
            .power
            .or(self.preferences.graphics_power_preference)
            .unwrap_or(PowerPreference::High)
            .into()
    }
}

#[derive(Default, Deserialize, Serialize)]
struct SavedGlobalPreferences {
    graphics_backend: Option<GraphicsBackend>,
    graphics_power_preference: Option<PowerPreference>,
}
