use crate::events::KeyCode;
use chrono::{DateTime, Utc};
use downcast_rs::Downcast;
use std::future::Future;
use std::pin::Pin;

pub use crate::loader::Error;

/// Type alias for pinned, boxed, and owned futures that output a falliable
/// result of type `Result<T, E>`.
pub type OwnedFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>;

pub struct FileFilter {
    pub description: String,
    pub extensions: String,
    pub mac_type: Option<String>,
}

pub trait FileDialogResult: Downcast {
    fn is_cancelled(&self) -> bool;
    fn creation_time(&self) -> Option<DateTime<Utc>>;
    fn modification_time(&self) -> Option<DateTime<Utc>>;
    fn file_name(&self) -> Option<String>;
    fn size(&self) -> Option<u64>;
    fn file_type(&self) -> Option<String>;
    fn creator(&self) -> Option<String>;
}
impl_downcast!(FileDialogResult);

pub type DialogResultFuture = OwnedFuture<Box<dyn FileDialogResult>, Error>;

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

    fn is_fullscreen(&self) -> bool;

    /// Displays a warning about unsupported content in Ruffle.
    /// The user can still click an "OK" or "run anyway" message to dismiss the warning.
    fn display_unsupported_message(&self);

    /// Displays a message about an error during root movie download.
    /// In particular, on web this can be a CORS error, which we can sidestep
    /// by providing a direct .swf link instead.
    fn display_root_movie_download_failed_message(&self);

    // Unused, but kept in case we need it later
    fn message(&self, message: &str);

    /// Displays a file dialog
    fn display_file_dialog(&self, filters: Vec<FileFilter>) -> DialogResultFuture;
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

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn display_unsupported_message(&self) {}

    fn display_root_movie_download_failed_message(&self) {}

    fn message(&self, _message: &str) {}

    fn display_file_dialog(&self, _filters: Vec<FileFilter>) -> DialogResultFuture {
        Box::pin(async move {
            let result: Result<Box<dyn FileDialogResult>, Error> =
                Ok(Box::new(NullFileDialogResult::new()));
            result
        })
    }
}

impl Default for NullUiBackend {
    fn default() -> Self {
        NullUiBackend::new()
    }
}

pub struct NullFileDialogResult {}

impl NullFileDialogResult {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for NullFileDialogResult {
    fn default() -> Self {
        NullFileDialogResult::new()
    }
}

impl FileDialogResult for NullFileDialogResult {
    fn is_cancelled(&self) -> bool {
        true
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        None
    }
    fn modification_time(&self) -> Option<DateTime<Utc>> {
        None
    }
    fn file_name(&self) -> Option<String> {
        None
    }
    fn size(&self) -> Option<u64> {
        None
    }
    fn file_type(&self) -> Option<String> {
        None
    }
    fn creator(&self) -> Option<String> {
        None
    }
}
