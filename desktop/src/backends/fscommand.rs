use crate::custom_event::RuffleEvent;

use ruffle_core::external::FsCommandProvider;
use std::sync::Arc;
use winit::event_loop::EventLoopProxy;
use winit::window::{Fullscreen, Window};

pub struct DesktopFSCommandProvider {
    pub event_loop: EventLoopProxy<RuffleEvent>,
    pub window: Arc<Window>,
}

impl FsCommandProvider for DesktopFSCommandProvider {
    fn on_fs_command(&self, command: &str, args: &str) -> bool {
        match command {
            "quit" => {
                let _ = self.event_loop.send_event(RuffleEvent::ExitRequested);
            }
            "fullscreen" => {
                match args {
                    "true" => self
                        .window
                        .set_fullscreen(Some(Fullscreen::Borderless(None))),
                    "false" => self.window.set_fullscreen(None),
                    _ => {}
                };
            }
            _ => return false,
        };

        true
    }
}
