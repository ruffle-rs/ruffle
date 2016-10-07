#[derive(Debug,PartialEq)]
pub enum Action {
    NextFrame,
    Play,
    PreviousFrame,
    Stop,
    StopSounds,
    ToggleQuality,
    Unknown { opcode: u8, data: Vec<u8> },
}