use anyhow::{Context, Error};
use arboard::Clipboard;
use rfd::{MessageButtons, MessageDialog, MessageLevel};
use ruffle_core::backend::ui::{FullscreenError, MouseCursor, UiBackend};
use std::rc::Rc;
use tracing::error;
use winit::window::{Fullscreen, Window};
use std::net::TcpStream;
use std::io::Read;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DesktopUiBackend {
    window: Rc<Window>,
    cursor_visible: bool,
    clipboard: Clipboard,
    debug_event_queue: Arc<RwLock<Vec<ruffle_core::player::DebugMessageIn>>>,
}

impl DesktopUiBackend {
    pub fn new(window: Rc<Window>) -> Result<Self, Error> {
        Ok(Self {
            window,
            cursor_visible: true,
            clipboard: Clipboard::new().context("Couldn't get platform clipboard")?,
            debug_event_queue: Default::default(),
        })
    }
}

// TODO: Move link to https://ruffle.rs/faq or similar
const UNSUPPORTED_CONTENT_MESSAGE: &str = "\
The Ruffle emulator does not yet support ActionScript 3, required by this content.
If you choose to run it anyway, interactivity will be missing or limited.

See the following link for more info:
https://github.com/ruffle-rs/ruffle/wiki/Frequently-Asked-Questions-For-Users";

const DOWNLOAD_FAILED_MESSAGE: &str = "Ruffle failed to open or download this file.";

impl UiBackend for DesktopUiBackend {
    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn set_mouse_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
        self.cursor_visible = visible;
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        use winit::window::CursorIcon;
        let icon = match cursor {
            MouseCursor::Arrow => CursorIcon::Arrow,
            MouseCursor::Hand => CursorIcon::Hand,
            MouseCursor::IBeam => CursorIcon::Text,
            MouseCursor::Grab => CursorIcon::Grab,
        };
        self.window.set_cursor_icon(icon);
    }

    fn set_clipboard_content(&mut self, content: String) {
        if let Err(e) = self.clipboard.set_text(content) {
            error!("Couldn't set clipboard contents: {:?}", e);
        }
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError> {
        self.window.set_fullscreen(if is_full {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
        Ok(())
    }

    fn display_unsupported_message(&self) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Ruffle - Unsupported content")
            .set_description(UNSUPPORTED_CONTENT_MESSAGE)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn display_root_movie_download_failed_message(&self) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Ruffle - Load failed")
            .set_description(DOWNLOAD_FAILED_MESSAGE)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn message(&self, message: &str) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("Ruffle")
            .set_description(message)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn connect_debugger(&mut self) {
        let queue_local = Arc::clone(&self.debug_event_queue);

        // spawn debugger thread
        std::thread::spawn(move || {
            let mut stream = TcpStream::connect("localhost:7979").unwrap();

            loop {
                let mut data = [0u8; 1024];
                let len = stream.read(&mut data).unwrap();

                if len == 0 {
                    break;
                }

                let data = &data[..len];
                println!("Got data: {:?}", data);
                if data[0] == 0x1 {
                    queue_local.write().unwrap().push(ruffle_core::player::DebugMessageIn::Pause);
                }
                if data[0] == 0x2 {
                    queue_local.write().unwrap().push(ruffle_core::player::DebugMessageIn::Play);
                }
            }
        });
    }


    fn get_debug_event(&mut self) -> Option<ruffle_core::player::DebugMessageIn> {
        self.debug_event_queue.write().unwrap().pop()
    }
}
