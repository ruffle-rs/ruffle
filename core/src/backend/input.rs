use crate::events::KeyCode;
use downcast_rs::Downcast;

pub trait InputBackend: Downcast {
    fn is_key_down(&self, key: KeyCode) -> bool;

    fn get_last_key_code(&self) -> KeyCode;

    fn mouse_visible(&self) -> bool;

    fn hide_mouse(&mut self);

    fn show_mouse(&mut self);

    /// Changes the mouse cursor image.
    fn set_mouse_cursor(&mut self, cursor: MouseCursor);
}
impl_downcast!(InputBackend);

/// Input backend that does nothing
pub struct NullInputBackend {}

impl NullInputBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputBackend for NullInputBackend {
    fn is_key_down(&self, _key: KeyCode) -> bool {
        false
    }

    fn get_last_key_code(&self) -> KeyCode {
        KeyCode::Unknown
    }

    fn mouse_visible(&self) -> bool {
        true
    }

    fn hide_mouse(&mut self) {}

    fn show_mouse(&mut self) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}
}

impl Default for NullInputBackend {
    fn default() -> Self {
        NullInputBackend::new()
    }
}

/// A mouse cursor icon displayed by the Flash Player.
/// Communicated from the core to the input backend via `InputBackend::set_mouse_cursor`.
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
