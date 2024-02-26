use crate::cli::Opt;
use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

pub struct GlobalPreferences {
    pub cli: Opt,
    _preferences: SavedGlobalPreferences,
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
            _preferences: preferences,
        })
    }
}

#[derive(Default, Deserialize, Serialize)]
struct SavedGlobalPreferences {}
