use crate::string::SwfStr;
use crate::types::FrameNumber;

pub type ActionsData<'a> = &'a [u8];

pub type RegisterIndex = u8;

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
    DefineFunction(DefineFunction<'a>),
    DefineFunction2(DefineFunction2<'a>),
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
    GetUrl(GetUrl<'a>),
    GetUrl2(GetUrl2),
    GetVariable,
    GotoFrame(FrameNumber),
    GotoFrame2(GotoFrame2),
    GotoLabel(&'a SwfStr),
    Greater,
    If(InstructionOffset),
    ImplementsOp,
    Increment,
    InitArray,
    InitObject,
    InstanceOf,
    Jump(InstructionOffset),
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
    StoreRegister(RegisterIndex),
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
    WaitForFrame(WaitForFrame),
    WaitForFrame2(WaitForFrame2),
    With(ActionsData<'a>),
    Unknown { opcode: u8, data: ActionsData<'a> },
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
    Register(RegisterIndex),
    ConstantPool(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendVarsMethod {
    None,
    Get,
    Post,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineFunction<'a> {
    pub name: &'a SwfStr,
    pub params: Vec<&'a SwfStr>,
    pub actions: ActionsData<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DefineFunction2<'a> {
    pub name: &'a SwfStr,
    pub register_count: RegisterIndex,
    pub params: Vec<FunctionParam<'a>>,
    pub preload_parent: bool,
    pub preload_root: bool,
    pub suppress_super: bool,
    pub preload_super: bool,
    pub suppress_arguments: bool,
    pub preload_arguments: bool,
    pub suppress_this: bool,
    pub preload_this: bool,
    pub preload_global: bool,
    pub actions: ActionsData<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionParam<'a> {
    pub name: &'a SwfStr,
    pub register_index: Option<RegisterIndex>,
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

#[derive(Clone, Debug, PartialEq)]
pub struct GotoFrame2 {
    pub set_playing: bool,
    pub scene_offset: u16,
}

pub type InstructionOffset = i16;

#[derive(Clone, Debug, PartialEq)]
pub struct TryBlock<'a> {
    pub try_actions: ActionsData<'a>,
    pub catch: Option<(CatchVar<'a>, ActionsData<'a>)>,
    pub finally: Option<ActionsData<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatchVar<'a> {
    Var(&'a SwfStr),
    Register(RegisterIndex),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaitForFrame {
    pub frame: FrameNumber,
    pub num_actions_to_skip: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WaitForFrame2 {
    pub num_actions_to_skip: u8,
}
