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
        max_idle: Duration,
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
        let mut last_activity = Instant::now();
        let mut last_log_time = None;
        while child.try_wait().unwrap().is_none() {
            std::thread::sleep(Duration::from_millis(100));
            let log_time = environment.log_file_last_modified();
            if log_time != last_log_time {
                last_log_time = log_time;
                last_activity = Instant::now();
            }
            if start.elapsed() > max_duration || last_activity.elapsed() > max_idle {
                child.kill().unwrap();
                break;
            }
        }
        environment.read_log()
    }
}
