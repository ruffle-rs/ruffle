use crate::flashplayer::config::FlashPlayerDefinition;
use anyhow::{Context, anyhow};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;
use temp_dir::TempDir;

#[derive(Debug)]
pub struct PlayerEnvironment {
    root: TempDir,
    log_file: PathBuf,
    display: Option<Xvfb>,
}

impl PlayerEnvironment {
    pub fn new(log_warnings: bool, player: &FlashPlayerDefinition) -> Self {
        let root = TempDir::new().unwrap();
        let log_file = root.child(".macromedia/Flash_Player/Logs/flashlog.txt");

        let mut config = "TraceOutputFileEnable=1\n".to_string();
        if log_warnings {
            config.push_str("ErrorReportingEnable=1\n");
        }

        // Automatically dismiss exception dialogs as they require user
        // interaction.
        config.push_str("SuppressDebuggerExceptionDialogs=1\n");

        fs::write(root.child("mm.cfg"), config).unwrap();
        Self {
            root,
            log_file,
            display: player.xvfb.then(|| Xvfb::start().unwrap()),
        }
    }

    pub fn read_log(&self) -> String {
        fs::read_to_string(self.log_file.clone()).unwrap()
    }

    pub fn log_file_last_modified(&self) -> Option<SystemTime> {
        self.log_file.metadata().and_then(|m| m.modified()).ok()
    }

    pub fn configure(&self, command: &mut Command) {
        command.env("HOME", self.root.path().to_str().unwrap());
        command.env("LC_ALL", "C");
        if let Some(display) = &self.display {
            command.env("DISPLAY", &display.name);
        }
        // Some VDPAU drivers (e.g. NVIDIA's) kill Flash Player's X connection
        // on displays without DRI3, such as Xvfb. Flash Player works fine
        // without VDPAU, so don't load any driver.
        command.env("VDPAU_DRIVER", "none");
    }
}

/// A virtual X server, so that Flash Player doesn't need the user's display.
#[derive(Debug)]
struct Xvfb {
    process: Child,
    name: String,
}

impl Xvfb {
    fn start() -> anyhow::Result<Self> {
        // `-displayfd 1` makes Xvfb pick a free display number and write it
        // to stdout once the server is ready to accept connections.
        let mut process = Command::new("Xvfb")
            .args(["-displayfd", "1", "-screen", "0", "1920x1080x24"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start Xvfb, is it installed?")?;

        let mut line = String::new();
        BufReader::new(process.stdout.take().unwrap()).read_line(&mut line)?;
        let number = line.trim();
        if number.is_empty() {
            let status = process.wait()?;
            return Err(anyhow!(
                "Xvfb exited without reporting a display ({status})"
            ));
        }

        Ok(Self {
            name: format!(":{number}"),
            process,
        })
    }
}

impl Drop for Xvfb {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
