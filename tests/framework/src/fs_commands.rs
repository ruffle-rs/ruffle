use ruffle_core::external::FsCommandProvider;
use std::sync::mpsc;

#[derive(Debug)]
pub struct TestFsCommandProvider {
    sender: mpsc::Sender<FsCommand>,
}

impl TestFsCommandProvider {
    pub fn new() -> (Self, mpsc::Receiver<FsCommand>) {
        let (sender, receiver) = mpsc::channel();
        (Self { sender }, receiver)
    }
}

impl FsCommandProvider for TestFsCommandProvider {
    fn on_fs_command(&self, command: &str, args: &str) -> bool {
        if let Some(command) = FsCommand::from_command(command, args) {
            self.sender.send(command).expect(
                "Test FS command channel should be available for the lifetime of the movie",
            );
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub enum FsCommand {
    Quit,
    CaptureImage(String),
}

impl FsCommand {
    pub fn from_command(command: &str, args: &str) -> Option<Self> {
        match command {
            "quit" => Some(Self::Quit),
            "captureImage" => Some(Self::CaptureImage(args.to_string())),
            _ => None,
        }
    }
}
