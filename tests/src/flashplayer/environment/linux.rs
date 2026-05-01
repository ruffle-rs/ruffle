use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use temp_dir::TempDir;

#[derive(Debug)]
pub struct PlayerEnvironment {
    root: TempDir,
    log_file: PathBuf,
}

impl PlayerEnvironment {
    pub fn new() -> Self {
        let root = TempDir::new().unwrap();
        let log_file = root.child(".macromedia/Flash_Player/Logs/flashlog.txt");
        fs::write(root.child("mm.cfg"), "TraceOutputFileEnable=1\n").unwrap();
        Self { root, log_file }
    }

    pub fn read_log(&self) -> String {
        fs::read_to_string(self.log_file.clone()).unwrap()
    }

    pub fn log_file_last_modified(&self) -> Option<SystemTime> {
        self.log_file.metadata().and_then(|m| m.modified()).ok()
    }

    pub fn configure(&self, command: &mut std::process::Command) {
        command.env("HOME", self.root.path().to_str().unwrap());
        command.env("LC_ALL", "C");
    }
}
