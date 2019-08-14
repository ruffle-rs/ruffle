use swf::Twips;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Event {
    MouseMove { x: Twips, y: Twips },
    MouseUp { x: Twips, y: Twips },
    MouseDown { x: Twips, y: Twips },
}

#[derive(Debug)]
pub enum PlayerEvent {
    RollOver,
    Click,
}
