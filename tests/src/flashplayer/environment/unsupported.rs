use std::time::SystemTime;

#[derive(Debug)]
pub struct PlayerEnvironment;

impl PlayerEnvironment {
    pub fn new(_log_warnings: bool) -> Self {
        panic!("Unsupported platform :( Feel free to add support and open a PR!");
    }

    pub fn read_log(&self) -> String {
        panic!("Unsupported platform :( Feel free to add support and open a PR!");
    }

    pub fn log_file_last_modified(&self) -> Option<SystemTime> {
        None
    }

    pub fn configure(&self, _command: &mut std::process::Command) {
        panic!("Unsupported platform :( Feel free to add support and open a PR!");
    }
}
