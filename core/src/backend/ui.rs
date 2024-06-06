use crate::backend::navigator::OwnedFuture;
use crate::events::{KeyCode, PlayerEvent, TextControlCode};
pub use crate::loader::Error as DialogLoaderError;
use chrono::{DateTime, Utc};
use downcast_rs::Downcast;
use fluent_templates::loader::langid;
pub use fluent_templates::LanguageIdentifier;
use std::borrow::Cow;
use std::collections::HashSet;
use url::Url;

pub type FullscreenError = Cow<'static, str>;
pub static US_ENGLISH: LanguageIdentifier = langid!("en-US");

pub enum FontDefinition<'a> {
    /// A singular DefineFont tag extracted from a swf.
    SwfTag(swf::Font<'a>, &'static swf::Encoding),

    /// A font contained in an external file, such as a ttf.
    FontFile {
        name: String,
        is_bold: bool,
        is_italic: bool,
        data: Vec<u8>,
        index: u32,
    },
}

/// A filter specifying a category that can be selected from a file chooser dialog
pub struct FileFilter {
    /// The description of the category
    pub description: String,
    /// A semicolon ';' delimited list of acceptable windows file extensions that can be selected
    /// in this category, with a */wildcard before each extension
    pub extensions: String,
    /// A semicolon ';' delimited list of acceptable MacOs file extensions that can be selected in
    /// this category, with a */wildcard before each extension
    /// Note that a list of file filters will either all have Some(_) mac_type or all will have None
    pub mac_type: Option<String>,
}

/// A result of a file selection
pub trait FileDialogResult: Downcast {
    /// Was the file selection canceled by the user
    fn is_cancelled(&self) -> bool;
    fn creation_time(&self) -> Option<DateTime<Utc>>;
    fn modification_time(&self) -> Option<DateTime<Utc>>;
    fn file_name(&self) -> Option<String>;
    fn size(&self) -> Option<u64>;
    fn file_type(&self) -> Option<String>;
    fn creator(&self) -> Option<String> {
        None
    }
    fn contents(&self) -> &[u8];
    /// Write the given data to the chosen file and refresh any internal metadata.
    /// Any future calls to other functions (such as [FileDialogResult::size]) will reflect
    /// the state at the time of the last refresh
    fn write_and_refresh(&mut self, data: &[u8]);
}
impl_downcast!(FileDialogResult);

/// Future representing a file selection in process
pub type DialogResultFuture = OwnedFuture<Box<dyn FileDialogResult>, DialogLoaderError>;

pub trait UiBackend: Downcast {
    fn mouse_visible(&self) -> bool;

    fn set_mouse_visible(&mut self, visible: bool);

    /// Changes the mouse cursor image.
    fn set_mouse_cursor(&mut self, cursor: MouseCursor);

    /// Get the clipboard content
    fn clipboard_content(&mut self) -> String;

    /// Check if the clipboard is available and not empty
    fn clipboard_available(&mut self) -> bool {
        !self.clipboard_content().is_empty()
    }

    /// Sets the clipboard to the given content.
    fn set_clipboard_content(&mut self, content: String);

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError>;

    /// Displays a message about an error during root movie download.
    /// In particular, on web this can be a CORS error, which we can sidestep
    /// by providing a direct .swf link instead.
    fn display_root_movie_download_failed_message(&self, _invalid_swf: bool);

    // Unused, but kept in case we need it later.
    fn message(&self, message: &str);

    // Only used on web.
    fn open_virtual_keyboard(&self);

    fn language(&self) -> LanguageIdentifier;

    fn display_unsupported_video(&self, url: Url);

    /// Called when a previously unknown device font is requested by a movie.
    /// The backend is requested to call `register` with any fonts that match the given name.
    ///
    /// You may call `register` any amount of times with any amount of found device fonts.
    /// If you do not call `register` with any fonts that match the request,
    /// then the font will simply be marked as not found - this may or may not fall back to another font.  
    fn load_device_font(
        &self,
        name: &str,
        is_bold: bool,
        is_italic: bool,
        register: &mut dyn FnMut(FontDefinition),
    );

    /// Displays a file selection dialog, returning None if the dialog cannot be displayed
    /// (e.g because it is already open)
    /// * `filters` represents a list of filters to the possible file types that can be selected
    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture>;

    /// Display a dialog allowing a user to select a destination to save a file to
    ///
    /// * `file_name` is a suggestion for the file name to save the file as
    /// * `title` is a title that should be displayed in the dialog
    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        title: String,
    ) -> Option<DialogResultFuture>;

    /// Mark that any previously open dialog has been closed
    fn close_file_dialog(&mut self);
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

pub struct InputManager {
    keys_down: HashSet<KeyCode>,
    keys_toggled: HashSet<KeyCode>,
    last_key: KeyCode,
    last_char: Option<char>,
    last_text_control: Option<TextControlCode>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_toggled: HashSet::new(),
            last_key: KeyCode::Unknown,
            last_char: None,
            last_text_control: None,
        }
    }

    fn add_key(&mut self, key_code: KeyCode) {
        self.last_key = key_code;
        if key_code != KeyCode::Unknown {
            self.keys_down.insert(key_code);
        }
    }

    fn toggle_key(&mut self, key_code: KeyCode) {
        if key_code == KeyCode::Unknown || self.keys_down.contains(&key_code) {
            return;
        }
        if self.keys_toggled.contains(&key_code) {
            self.keys_toggled.remove(&key_code);
        } else {
            self.keys_toggled.insert(key_code);
        }
    }

    fn remove_key(&mut self, key_code: KeyCode) {
        self.last_key = key_code;
        if key_code != KeyCode::Unknown {
            self.keys_down.remove(&key_code);
        }
    }

    pub fn handle_event(&mut self, event: &PlayerEvent) {
        match *event {
            PlayerEvent::KeyDown { key_code, key_char } => {
                self.last_char = key_char;
                self.toggle_key(key_code);
                self.add_key(key_code);
            }
            PlayerEvent::KeyUp { key_code, key_char } => {
                self.last_char = key_char;
                self.remove_key(key_code);
                self.last_text_control = None;
            }
            PlayerEvent::TextControl { code } => {
                self.last_text_control = Some(code);
            }
            PlayerEvent::MouseDown { button, .. } => {
                self.toggle_key(button.into());
                self.add_key(button.into())
            }
            PlayerEvent::MouseUp { button, .. } => self.remove_key(button.into()),
            _ => {}
        }
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_toggled(&self, key: KeyCode) -> bool {
        self.keys_toggled.contains(&key)
    }

    pub fn last_key_code(&self) -> KeyCode {
        self.last_key
    }

    pub fn last_key_char(&self) -> Option<char> {
        self.last_char
    }

    pub fn last_text_control(&self) -> Option<TextControlCode> {
        self.last_text_control
    }

    pub fn is_mouse_down(&self) -> bool {
        self.is_key_down(KeyCode::MouseLeft)
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// UiBackend that does nothing.
pub struct NullUiBackend {}

impl NullUiBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiBackend for NullUiBackend {
    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn clipboard_content(&mut self) -> String {
        "".into()
    }

    fn set_clipboard_content(&mut self, _content: String) {}

    fn set_fullscreen(&mut self, _is_full: bool) -> Result<(), FullscreenError> {
        Ok(())
    }

    fn display_root_movie_download_failed_message(&self, _invalid_swf: bool) {}

    fn message(&self, _message: &str) {}

    fn display_unsupported_video(&self, _url: Url) {}

    fn load_device_font(
        &self,
        _name: &str,
        _is_bold: bool,
        _is_italic: bool,
        _register: &mut dyn FnMut(FontDefinition),
    ) {
    }

    fn open_virtual_keyboard(&self) {}

    fn language(&self) -> LanguageIdentifier {
        US_ENGLISH.clone()
    }

    fn display_file_open_dialog(
        &mut self,
        _filters: Vec<FileFilter>,
    ) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            let result: Result<Box<dyn FileDialogResult>, DialogLoaderError> =
                Ok(Box::new(NullFileDialogResult::new()));
            result
        }))
    }

    fn close_file_dialog(&mut self) {}

    fn display_file_save_dialog(
        &mut self,
        _file_name: String,
        _domain: String,
    ) -> Option<DialogResultFuture> {
        None
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
    fn contents(&self) -> &[u8] {
        &[]
    }

    fn write_and_refresh(&mut self, _data: &[u8]) {}
}
