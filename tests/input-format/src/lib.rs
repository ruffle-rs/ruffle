use serde::{Deserialize, Serialize};

/// Position of a mouse cursor on the screen.
///
/// Mouse cursor positions are sized relative to the Flash stage's dimensions,
/// regardless of the native window's size or pixel density. For example, a
/// (640x480) stage movie on a 2x display (on platforms that report physical
/// pixels) or at 2x the size will see mouse clicks at its bottom right corner
/// on (1280x960), relative to the window. That coordinate needs to be scaled
/// down to match the desired stage.
#[derive(Serialize, Deserialize)]
pub struct MousePosition(f64, f64);

/// Which mouse button is being pressed or released.
#[derive(Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// All automated event types supported by FlashTAS.
///
/// A FlashTAS input file consists of a string of `AutomatedEvent`s which are
/// played back by FlashTAS.
#[derive(Serialize, Deserialize)]
pub enum AutomatedEvent {
    /// End the current frame's input and wait for the next frame before
    /// continuing to inject input.
    Wait,

    /// Move the mouse to a new cursor position.
    MouseMove(MousePosition),

    /// Click a mouse button.
    MouseDown(MousePosition, MouseButton),

    /// Release a mouse button.
    MouseUp(MousePosition, MouseButton),
}
