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
    ConstantPool(ConstantPool<'a>),
    Decrement,
    DefineFunction(DefineFunction<'a>),
    DefineFunction2(DefineFunction2<'a>),
    DefineLocal,
    DefineLocal2,
    Delete,
    Delete2,
    Divide,
    End,
    EndDrag,
    Enumerate,
    Enumerate2,
    Equals,
    Equals2,
    Extends,
    GetMember,
    GetProperty,
    GetTime,
    GetUrl(GetUrl<'a>),
    GetUrl2(GetUrl2),
    GetVariable,
    GotoFrame(GotoFrame),
    GotoFrame2(GotoFrame2),
    GotoLabel(GotoLabel<'a>),
    Greater,
    If(If),
    ImplementsOp,
    Increment,
    InitArray,
    InitObject,
    InstanceOf,
    Jump(Jump),
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
    Push(Push<'a>),
    PushDuplicate,
    RandomNumber,
    RemoveSprite,
    Return,
    SetMember,
    SetProperty,
    SetTarget(SetTarget<'a>),
    SetTarget2,
    SetVariable,
    StackSwap,
    StartDrag,
    Stop,
    StopSounds,
    StoreRegister(StoreRegister),
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
    Try(Try<'a>),
    TypeOf,
    WaitForFrame(WaitForFrame),
    WaitForFrame2(WaitForFrame2),
    With(With<'a>),
    Unknown(Unknown<'a>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantPool<'a> {
    pub strings: Vec<&'a SwfStr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineFunction<'a> {
    pub name: &'a SwfStr,
    pub params: Vec<&'a SwfStr>,
    pub actions: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineFunction2<'a> {
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
pub struct GetUrl<'a> {
    pub url: &'a SwfStr,
    pub target: &'a SwfStr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GetUrl2 {
    pub send_vars_method: SendVarsMethod,
    pub is_target_sprite: bool,
    pub is_load_vars: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendVarsMethod {
    None,
    Get,
    Post,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GotoFrame {
    pub frame: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GotoFrame2 {
    pub set_playing: bool,
    pub scene_offset: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GotoLabel<'a> {
    pub label: &'a SwfStr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    pub offset: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Jump {
    pub offset: i16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Push<'a> {
    pub values: Vec<Value<'a>>,
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

#[derive(Clone, Debug, PartialEq)]
pub struct SetTarget<'a> {
    pub target: &'a SwfStr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreRegister {
    pub register: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Try<'a> {
    pub try_body: &'a [u8],
    pub catch_body: Option<(CatchVar<'a>, &'a [u8])>,
    pub finally_body: Option<&'a [u8]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatchVar<'a> {
    Var(&'a SwfStr),
    Register(u8),
}

bitflags! {
    pub struct TryFlags: u8 {
        const CATCH_BLOCK = 1 << 0;
        const FINALLY_BLOCK = 1 << 1;
        const CATCH_IN_REGISTER = 1 << 2;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaitForFrame {
    pub frame: u16,
    pub num_actions_to_skip: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaitForFrame2 {
    pub num_actions_to_skip: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct With<'a> {
    pub actions: &'a [u8],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unknown<'a> {
    pub opcode: u8,
    pub data: &'a [u8],
}
