use clipboard::{ClipboardContext, ClipboardProvider};
use ruffle_core::backend::ui::{Error, MouseCursor, UiBackend};
use std::rc::Rc;
use tinyfiledialogs::{message_box_ok, MessageBoxIcon};
use winit::window::{Fullscreen, Window};

pub struct DesktopUiBackend {
    window: Rc<Window>,
    cursor_visible: bool,
    clipboard: ClipboardContext,
}

impl DesktopUiBackend {
    pub fn new(window: Rc<Window>) -> Self {
        Self {
            window,
            cursor_visible: true,
            clipboard: ClipboardProvider::new().unwrap(),
        }
    }
}

// TODO: Move link to https://ruffle.rs/faq or similar
const UNSUPPORTED_CONTENT_MESSAGE: &str = "\
This content is not yet supported by Ruffle and will likely not run as intended.

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
        self.clipboard.set_contents(content).unwrap();
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error> {
        self.window.set_fullscreen(if is_full {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
        Ok(())
    }

    fn display_unsupported_message(&self) {
        message_box_ok(
            "Ruffle - Unsupported content",
            UNSUPPORTED_CONTENT_MESSAGE,
            MessageBoxIcon::Warning,
        );
    }

    fn display_root_movie_download_failed_message(&self) {
        message_box_ok(
            "Ruffle - Load failed",
            DOWNLOAD_FAILED_MESSAGE,
            MessageBoxIcon::Warning,
        );
    }

    fn message(&self, message: &str) {
        message_box_ok("Ruffle", message, MessageBoxIcon::Info)
    }
}
