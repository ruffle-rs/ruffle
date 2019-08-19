use swf::Twips;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum PlayerEvent {
    MouseMove { x: Twips, y: Twips },
    MouseUp { x: Twips, y: Twips },
    MouseDown { x: Twips, y: Twips },
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
