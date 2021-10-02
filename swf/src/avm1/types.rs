use crate::string::SwfStr;
use bitflags::bitflags;

#[derive(Clone, Debug, PartialEq)]
pub enum Action<'a> {
    Add,
    Add2,
    And,
    AsciiToChar,
    BitAnd,
    BitLShift,
    BitOr,
    BitRShift,
    BitURShift,
    BitXor,
    Call,
    CallFunction,
    CallMethod,
    CastOp,
    CharToAscii,
    CloneSprite,
    ConstantPool(Vec<&'a SwfStr>),
    Decrement,
    DefineFunction {
        name: &'a SwfStr,
        params: Vec<&'a SwfStr>,
        actions: &'a [u8],
    },
    DefineFunction2(Function<'a>),
    DefineLocal,
    DefineLocal2,
    Delete,
    Delete2,
    Divide,
    EndDrag,
    Enumerate,
    Enumerate2,
    Equals,
    Equals2,
    Extends,
    GetMember,
    GetProperty,
    GetTime,
    GetUrl {
        url: &'a SwfStr,
        target: &'a SwfStr,
    },
    GetUrl2 {
        send_vars_method: SendVarsMethod,
        is_target_sprite: bool,
        is_load_vars: bool,
    },
    GetVariable,
    GotoFrame(u16),
    GotoFrame2 {
        set_playing: bool,
        scene_offset: u16,
    },
    GotoLabel(&'a SwfStr),
    Greater,
    If {
        offset: i16,
    },
    ImplementsOp,
    Increment,
    InitArray,
    InitObject,
    InstanceOf,
    Jump {
        offset: i16,
    },
    Less,
    Less2,
    MBAsciiToChar,
    MBCharToAscii,
    MBStringExtract,
    MBStringLength,
    Modulo,
    Multiply,
    NewMethod,
    NewObject,
    NextFrame,
    Not,
    Or,
    Play,
    Pop,
    PreviousFrame,
    Push(Vec<Value<'a>>),
    PushDuplicate,
    RandomNumber,
    RemoveSprite,
    Return,
    SetMember,
    SetProperty,
    SetTarget(&'a SwfStr),
    SetTarget2,
    SetVariable,
    StackSwap,
    StartDrag,
    Stop,
    StopSounds,
    StoreRegister(u8),
    StrictEquals,
    StringAdd,
    StringEquals,
    StringExtract,
    StringGreater,
    StringLength,
    StringLess,
    Subtract,
    TargetPath,
    Throw,
    ToInteger,
    ToNumber,
    ToString,
    ToggleQuality,
    Trace,
    Try(TryBlock<'a>),
    TypeOf,
    WaitForFrame {
        frame: u16,
        num_actions_to_skip: u8,
    },
    WaitForFrame2 {
        num_actions_to_skip: u8,
    },
    With {
        actions: &'a [u8],
    },
    Unknown {
        opcode: u8,
        data: &'a [u8],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    Double(f64),
    Str(&'a SwfStr),
    Register(u8),
    ConstantPool(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendVarsMethod {
    None,
    Get,
    Post,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function<'a> {
    pub name: &'a SwfStr,
    pub register_count: u8,
    pub params: Vec<FunctionParam<'a>>,
    pub flags: FunctionFlags,
    pub actions: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionParam<'a> {
    pub name: &'a SwfStr,
    pub register_index: Option<u8>,
}

bitflags! {
    pub struct FunctionFlags: u16 {
        const PRELOAD_THIS = 1 << 0;
        const SUPPRESS_THIS = 1 << 1;
        const PRELOAD_ARGUMENTS = 1 << 2;
        const SUPPRESS_ARGUMENTS = 1 << 3;
        const PRELOAD_SUPER = 1 << 4;
        const SUPPRESS_SUPER = 1 << 5;
        const PRELOAD_ROOT = 1 << 6;
        const PRELOAD_PARENT = 1 << 7;
        const PRELOAD_GLOBAL = 1 << 8;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TryBlock<'a> {
    pub try_body: &'a [u8],
    pub catch_body: Option<(CatchVar<'a>, &'a [u8])>,
    pub finally_body: Option<&'a [u8]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatchVar<'a> {
    Var(&'a SwfStr),
    Register(u8),
}
