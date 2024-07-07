use serde::{Deserialize, Serialize};

/// Position of a mouse cursor on the screen.
///
/// Mouse cursor positions are sized relative to the Flash stage's dimensions,
/// regardless of the native window's size or pixel density. For example, a
/// (640x480) stage movie on a 2x display (on platforms that report physical
/// pixels) or at 2x the size will see mouse clicks at its bottom right corner
/// on (1280x960), relative to the window. That coordinate needs to be scaled
/// down to match the desired stage.
#[derive(Serialize, Deserialize, Debug)]
pub struct MousePosition(pub f64, pub f64);

/// Which mouse button is being pressed or released.
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Control inputs to a text field
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextControlCode {
    MoveLeft,
    MoveLeftWord,
    MoveLeftLine,
    MoveLeftDocument,
    MoveRight,
    MoveRightWord,
    MoveRightLine,
    MoveRightDocument,
    SelectLeft,
    SelectLeftWord,
    SelectLeftLine,
    SelectLeftDocument,
    SelectRight,
    SelectRightWord,
    SelectRightLine,
    SelectRightDocument,
    SelectAll,
    Copy,
    Paste,
    Cut,
    Backspace,
    Enter,
    Delete,
}

/// All automated event types supported by FlashTAS.
///
/// A FlashTAS input file consists of a string of `AutomatedEvent`s which are
/// played back by FlashTAS.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AutomatedEvent {
    /// End the current frame's input and wait for the next frame before
    /// continuing to inject input.
    Wait,

    /// Move the mouse to a new cursor position.
    MouseMove { pos: MousePosition },

    /// Click a mouse button.
    MouseDown {
        pos: MousePosition,
        btn: MouseButton,
        index: Option<usize>,
        assert_handled: Option<EventHandledAssertion>,
    },

    /// Release a mouse button.
    MouseUp {
        pos: MousePosition,
        btn: MouseButton,
    },

    /// Mouse scroll.
    MouseWheel {
        lines: Option<f64>,
        pixels: Option<f64>,
    },

    /// Press a key
    KeyDown { key_code: u8 },

    /// Release a key
    KeyUp { key_code: u8 },

    /// Input a character code
    TextInput { codepoint: char },

    /// Input a control character code
    TextControl { code: TextControlCode },

    /// Populate clipboard with the given text
    SetClipboardText { text: String },

    /// Inform the player that the focus has been gained (i.e. the window has been focused).
    FocusGained,

    /// Inform the player that the focus has been lost (i.e. the user focused another window).
    FocusLost,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventHandledAssertion {
    pub value: bool,
    pub message: String,
}
