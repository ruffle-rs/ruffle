use ruffle_core::external::FsCommandProvider;

#[derive(Default)]
pub struct Ruffle3DSCommandProvider {
    should_quit: bool,
}

impl FsCommandProvider for Ruffle3DSCommandProvider {
    fn on_fs_command(&self, command: &str, args: &str) -> bool {
        match command {
            "quit" => {
                self.should_quit = true;
                true
            }
            _ => false
        }
    }
}