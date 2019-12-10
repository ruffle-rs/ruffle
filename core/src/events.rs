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

/// Flash virtual keycode.
/// TODO: This will eventually move to a separate module.
pub type KeyCode = u8;
