use crate::events::KeyCode;
use downcast_rs::Downcast;

pub type Error = Box<dyn std::error::Error>;

pub trait UiBackend: Downcast {
    fn is_key_down(&self, key: KeyCode) -> bool;

    fn last_key_code(&self) -> KeyCode;

    fn last_key_char(&self) -> Option<char>;

    fn mouse_visible(&self) -> bool;

    fn set_mouse_visible(&mut self, visible: bool);

    /// Changes the mouse cursor image.
    fn set_mouse_cursor(&mut self, cursor: MouseCursor);

    /// Set the clipboard to the given content
    fn set_clipboard_content(&mut self, content: String);

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error>;

    /// Displays a warning about unsupported content in Ruffle.
    /// The user can still click an "OK" or "run anyway" message to dismiss the warning.
    fn display_unsupported_message(&self);

    /// Displays a message about an error during root movie download.
    /// In particular, on web this can be a CORS error, which we can sidestep
    /// by providing a direct .swf link instead.
    fn display_root_movie_download_failed_message(&self);

    // Unused, but kept in case we need it later
    fn message(&self, message: &str);
}
impl_downcast!(UiBackend);

/// A mouse cursor icon displayed by the Flash Player.
/// Communicated from the core to the UI backend via `UiBackend::set_mouse_cursor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseCursor {
    /// The default arrow icon.
    /// Equivalent to AS3 `MouseCursor.ARROW`.
    Arrow,

    /// The hand icon incdicating a button or link.
    /// Equivalent to AS3 `MouseCursor.BUTTON`.
    Hand,

    /// The text I-beam.
    /// Equivalent to AS3 `MouseCursor.IBEAM`.
    IBeam,

    /// The grabby-dragging hand icon.
    /// Equivalent to AS3 `MouseCursor.HAND`.
    Grab,
}

/// UiBackend that does nothing.
pub struct NullUiBackend {}

impl NullUiBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiBackend for NullUiBackend {
    fn is_key_down(&self, _key: KeyCode) -> bool {
        false
    }

    fn last_key_code(&self) -> KeyCode {
        KeyCode::Unknown
    }

    fn last_key_char(&self) -> Option<char> {
        None
    }

    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn set_clipboard_content(&mut self, _content: String) {}

    fn set_fullscreen(&mut self, _is_full: bool) -> Result<(), Error> {
        Ok(())
    }

    fn display_unsupported_message(&self) {}

    fn display_root_movie_download_failed_message(&self) {}

    fn message(&self, _message: &str) {}
}

impl Default for NullUiBackend {
    fn default() -> Self {
        NullUiBackend::new()
    }
}
