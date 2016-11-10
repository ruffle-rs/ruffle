#[derive(Debug,PartialEq)]
pub enum Action {
    Add,
    Divide,
    GetUrl { url: String, target: String },
    GotoFrame(u16),
    GotoLabel(String),
    Multiply,
    NextFrame,
    Play,
    Pop,
    PreviousFrame,
    Push(Vec<Value>),
    SetTarget(String),
    Stop,
    StopSounds,
    Subtract,
    ToggleQuality,
    WaitForFrame { frame: u16, num_actions_to_skip: u8 },
    Unknown { opcode: u8, data: Vec<u8> },
}

#[derive(Debug,PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Int(u32),
    Float(f32),
    Double(f64),
    Str(String),
    Register(u8),
    ConstantPool(u16),
}