#[derive(Debug,PartialEq)]
pub enum Action {
    GetUrl { url: String, target: String },
    GotoFrame(u16),
    NextFrame,
    Play,
    PreviousFrame,
    Stop,
    StopSounds,
    ToggleQuality,
    WaitForFrame { frame: u16, num_actions_to_skip: u8 },
    Unknown { opcode: u8, data: Vec<u8> },
}