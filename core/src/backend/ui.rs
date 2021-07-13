use crate::events::KeyCode;
use crate::prelude::*;
use std::collections::HashSet;

pub trait UiBackend {
    fn mouse_visible(&self) -> bool;

    fn set_mouse_visible(&mut self, visible: bool);

    /// Changes the mouse cursor image.
    fn set_mouse_cursor(&mut self, cursor: MouseCursor);

    /// Sets the clipboard to the given content.
    fn set_clipboard_content(&mut self, content: String);

    fn is_fullscreen(&self) -> bool;

    /// Displays a warning about unsupported content in Ruffle.
    /// The user can still click an "OK" or "run anyway" message to dismiss the warning.
    fn display_unsupported_message(&self);

    // Unused, but kept in case we need it later.
    fn message(&self, message: &str);
}

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
    last_key: KeyCode,
    pub mouse_position: (Twips, Twips),
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            last_key: KeyCode::Unknown,
            mouse_position: (Twips::ZERO, Twips::ZERO),
        }
    }

    pub fn key_down(&mut self, key: KeyCode) {
        self.last_key = key;
        self.keys_down.insert(key);
    }

    pub fn key_up(&mut self, key: KeyCode) {
        self.last_key = key;
        self.keys_down.remove(&key);
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn last_key_code(&self) -> KeyCode {
        self.last_key
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

    fn set_clipboard_content(&mut self, _content: String) {}

    fn is_fullscreen(&self) -> bool {
        false
    }

    fn display_unsupported_message(&self) {}

    fn message(&self, _message: &str) {}
}

impl Default for NullUiBackend {
    fn default() -> Self {
        NullUiBackend::new()
    }
}
