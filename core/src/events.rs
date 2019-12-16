#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum PlayerEvent {
    MouseMove { x: f64, y: f64 },
    MouseUp { x: f64, y: f64 },
    MouseDown { x: f64, y: f64 },
    MouseLeft,
}

/// The events that an AVM1 button can fire.
///
/// In Flash, these are created using `on` code on the button instance:
/// ```ignore
/// on(release) {
///     trace("Button clicked");
/// }
/// ```
#[derive(Debug)]
#[allow(dead_code)]
pub enum ButtonEvent {
    Press,
    Release,
    RollOut,
    RollOver,
    KeyPress(KeyCode),
}

/// An event type that can be handled by a movie clip
/// instance.
/// TODO: Move this representation in the swf crate?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClipEvent {
    Construct,
    Data,
    DragOut,
    DragOver,
    EnterFrame,
    Initialize,
    KeyUp,
    KeyDown,
    KeyPress { key_code: KeyCode },
    Load,
    MouseUp,
    MouseDown,
    MouseMove,
    Press,
    RollOut,
    RollOver,
    Release,
    ReleaseOutside,
    Unload,
}

/// Flash virtual keycode.
/// TODO: This will eventually move to a separate module.
pub type KeyCode = swf::KeyCode;
