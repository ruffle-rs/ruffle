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
    debug_event_queue_out: Arc<RwLock<Vec<ruffle_core::player::DebugMessageOut>>>,
}

impl DesktopUiBackend {
    pub fn new(window: Rc<Window>) -> Result<Self, Error> {
        Ok(Self {
            window,
            cursor_visible: true,
            clipboard: Clipboard::new().context("Couldn't get platform clipboard")?,
            debug_event_queue: Default::default(),
            debug_event_queue_out: Default::default(),
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
        let queue_out_local = Arc::clone(&self.debug_event_queue_out);

        // spawn debugger thread
        std::thread::spawn(move || {
            let mut stream = TcpStream::connect("localhost:7979").unwrap();
            stream.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();

            loop {
                {
                    let mut data = [0u8; 1024];
                    if let Ok(len) = stream.read(&mut data) {

                        if len == 0 {
                            break;
                        }

                        let data = &data[..len];
                        let s = String::from_utf8(data.to_vec()).unwrap();

                        if let Ok(msg) = serde_json::from_str::<ruffle_core::player::DebugMessageIn>(&s) {
                            println!("Got data: {:?}", msg);
                            queue_local.write().unwrap().push(msg);
                        }
                    }
                }

                if let Some(out_msg) = queue_out_local.write().unwrap().pop() {
                    std::io::Write::write(&mut stream, serde_json::to_string(&out_msg).unwrap().as_bytes()).unwrap();
                }
            }
        });
    }

    fn submit_debug_message(&mut self, evt: ruffle_core::player::DebugMessageOut) {
        self.debug_event_queue_out.write().unwrap().push(evt.clone());
    }


    fn get_debug_event(&mut self) -> Option<ruffle_core::player::DebugMessageIn> {
        self.debug_event_queue.write().unwrap().pop()
    }
}

