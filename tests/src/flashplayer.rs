use crate::flashplayer::config::FlashPlayerDefinition;
use crate::flashplayer::environment::PlayerEnvironment;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

pub mod config;
pub mod environment;

pub struct FlashPlayer<'a> {
    player_definition: &'a FlashPlayerDefinition,
}

impl<'a> FlashPlayer<'a> {
    pub fn new(player_definition: &'a FlashPlayerDefinition) -> Self {
        Self { player_definition }
    }

    pub fn run(
        &self,
        environment: &PlayerEnvironment,
        swf: &Path,
        max_duration: Duration,
    ) -> String {
        let mut command = Command::new(self.player_definition.path.clone());
        environment.configure(&mut command);
        let mut child = command
            .arg(swf)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let start = Instant::now();
        while child.try_wait().unwrap().is_none() {
            std::thread::sleep(Duration::from_millis(200));
            if start.elapsed() > max_duration {
                child.kill().unwrap();
                break;
            }
        }
        environment.read_log()
    }
}
