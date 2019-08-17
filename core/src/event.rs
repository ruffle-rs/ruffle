use swf::Twips;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Event {
    MouseMove { x: Twips, y: Twips },
    MouseUp { x: Twips, y: Twips },
    MouseDown { x: Twips, y: Twips },
    MouseLeft,
}

#[derive(Debug)]
pub enum ClipEvent {
    Press,
    Release,
    RollOut,
    RollOver,
    KeyPress(KeyCode),
}

type KeyCode = u8;