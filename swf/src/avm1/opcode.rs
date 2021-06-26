#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum OpCode {
    End = 0x00,

    NextFrame = 0x04,
    PreviousFrame = 0x05,
    Play = 0x06,
    Stop = 0x07,
    ToggleQuality = 0x08,
    StopSounds = 0x09,
    Add = 0x0A,
    Subtract = 0x0B,
    Multiply = 0x0C,
    Divide = 0x0D,
    Equals = 0x0E,
    Less = 0x0F,
    And = 0x10,
    Or = 0x11,
    Not = 0x12,
    StringEquals = 0x13,
    StringLength = 0x14,
    StringExtract = 0x15,

    Pop = 0x17,
    ToInteger = 0x18,

    GetVariable = 0x1C,
    SetVariable = 0x1D,

    SetTarget2 = 0x20,
    StringAdd = 0x21,
    GetProperty = 0x22,
    SetProperty = 0x23,
    CloneSprite = 0x24,
    RemoveSprite = 0x25,
    Trace = 0x26,
    StartDrag = 0x27,
    EndDrag = 0x28,
    StringLess = 0x29,
    Throw = 0x2A,
    CastOp = 0x2B,
    ImplementsOp = 0x2C,

    RandomNumber = 0x30,
    MBStringLength = 0x31,
    CharToAscii = 0x32,
    AsciiToChar = 0x33,
    GetTime = 0x34,
    MBStringExtract = 0x35,
    MBCharToAscii = 0x36,
    MBAsciiToChar = 0x37,

    Delete = 0x3A,
    Delete2 = 0x3B,
    DefineLocal = 0x3C,
    CallFunction = 0x3D,
    Return = 0x3E,
    Modulo = 0x3F,
    NewObject = 0x40,
    DefineLocal2 = 0x41,
    InitArray = 0x42,
    InitObject = 0x43,
    TypeOf = 0x44,
    TargetPath = 0x45,
    Enumerate = 0x46,
    Add2 = 0x47,
    Less2 = 0x48,
    Equals2 = 0x49,
    ToNumber = 0x4A,
    ToString = 0x4B,
    PushDuplicate = 0x4C,
    StackSwap = 0x4D,
    GetMember = 0x4E,
    SetMember = 0x4F,
    Increment = 0x50,
    Decrement = 0x51,
    CallMethod = 0x52,
    NewMethod = 0x53,
    InstanceOf = 0x54,
    Enumerate2 = 0x55,

    BitAnd = 0x60,
    BitOr = 0x61,
    BitXor = 0x62,
    BitLShift = 0x63,
    BitRShift = 0x64,
    BitURShift = 0x65,
    StrictEquals = 0x66,
    Greater = 0x67,
    StringGreater = 0x68,
    Extends = 0x69,

    GotoFrame = 0x81,

    GetUrl = 0x83,

    StoreRegister = 0x87,
    ConstantPool = 0x88,

    WaitForFrame = 0x8A,
    SetTarget = 0x8B,
    GotoLabel = 0x8C,
    WaitForFrame2 = 0x8D,
    DefineFunction2 = 0x8E,
    Try = 0x8F,

    With = 0x94,

    Push = 0x96,

    Jump = 0x99,
    GetUrl2 = 0x9A,
    DefineFunction = 0x9B,
    If = 0x9D,
    Call = 0x9E,
    GotoFrame2 = 0x9F,
}

impl OpCode {
    pub fn from_u8(n: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(n)
    }

    pub fn format(opcode: u8) -> String {
        if let Some(op) = Self::from_u8(opcode) {
            format!("{:?}", op)
        } else {
            format!("Unknown({})", opcode)
        }
    }
}
