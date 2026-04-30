use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Default)]
pub struct FlashPlayerDefinition {
    pub version: u8,
    pub path: PathBuf,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct FlashHarnessConfig {
    pub players: Vec<FlashPlayerDefinition>,
}

static CONFIG_TEMPLATE: &str = r#"# To use `cargo testutils` to execute tests within Flash Player, download some flash players (ensure it's the debug version!) and list them below as shown in the commented out examples.
# (hint: there's a whole bunch on archive.org, e.g. https://archive.org/details/Adobe_Flash_Player_Complete_Collection)

# [[players]]
# version = 32
# path = "/path/to/a/flashplayer/debugger"

# [[players]]
# version = 15
# path = "/path/to/another/flashplayer"
"#;

impl FlashHarnessConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path: PathBuf = std::env::var("FLASH_PLAYERS_TEST_CONFIG")
            .expect("FLASH_PLAYERS_TEST_CONFIG environment variable must be set")
            .into();
        if !path.is_file() {
            std::fs::write(path.clone(), CONFIG_TEMPLATE)?;
        }
        let config: Self = toml::from_str(&std::fs::read_to_string(&path)?)?;
        if config.players.is_empty() {
            return Err(anyhow::anyhow!(
                "No Flash Player executables have been configured (please edit {})",
                path.display()
            ));
        }
        Ok(config)
    }

    pub fn get_player(&self, version: u8) -> Option<&FlashPlayerDefinition> {
        self.players.iter().find(|player| player.version == version)
    }
}
