use crate::string::SwfStr;
use bitflags::bitflags;
use serde::Serialize;
use std::num::NonZeroU8;

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConstantPool<'a> {
    pub strings: Vec<&'a SwfStr>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DefineFunction<'a> {
    pub name: &'a SwfStr,
    pub params: Vec<&'a SwfStr>,
    pub actions: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DefineFunction2<'a> {
    pub name: &'a SwfStr,
    pub register_count: u8,
    pub params: Vec<FunctionParam<'a>>,
    pub flags: FunctionFlags,
    pub actions: &'a [u8],
}

impl<'a> From<DefineFunction<'a>> for DefineFunction2<'a> {
    #[inline]
    fn from(function: DefineFunction<'a>) -> Self {
        let params = function
            .params
            .into_iter()
            .map(|param| FunctionParam {
                name: param,
                register_index: None,
            })
            .collect();
        Self {
            name: function.name,
            register_count: 0,
            params,
            flags: FunctionFlags::empty(),
            actions: function.actions,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FunctionParam<'a> {
    pub name: &'a SwfStr,
    pub register_index: Option<NonZeroU8>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GetUrl<'a> {
    pub url: &'a SwfStr,
    pub target: &'a SwfStr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GetUrl2(pub(crate) GetUrlFlags);

impl GetUrl2 {
    /// Returns the flags for an AVM1 `loadMovie` call.
    #[inline]
    pub fn for_load_movie(method: SendVarsMethod) -> Self {
        let mut flags = Self(GetUrlFlags::LOAD_TARGET);
        flags.set_send_vars_method(method);
        flags
    }

    /// Returns the flags for an AVM1 `getURL` call.
    #[inline]
    pub fn for_get_url(method: SendVarsMethod) -> Self {
        let mut flags = Self(GetUrlFlags::empty());
        flags.set_send_vars_method(method);
        flags
    }

    /// Returns the flags for an AVM1 `loadVariables` or `LoadVars.load` call.
    #[inline]
    pub fn for_load_vars(method: SendVarsMethod) -> Self {
        let mut flags = Self(GetUrlFlags::LOAD_VARIABLES);
        flags.set_send_vars_method(method);
        flags
    }

    /// The HTTP method used for sending data.
    #[inline]
    pub fn send_vars_method(&self) -> SendVarsMethod {
        match self.0 & GetUrlFlags::METHOD_MASK {
            GetUrlFlags::METHOD_NONE => SendVarsMethod::None,
            GetUrlFlags::METHOD_GET => SendVarsMethod::Get,
            GetUrlFlags::METHOD_POST => SendVarsMethod::Post,
            _ => unreachable!(),
        }
    }

    /// Sets the HTTP method used for sending data.
    #[inline]
    pub fn set_send_vars_method(&mut self, method: SendVarsMethod) {
        self.0 -= GetUrlFlags::METHOD_MASK;
        self.0 |= GetUrlFlags::from_bits(method as u8).unwrap();
    }

    /// Whether this action will load a movie or image into a display object.
    #[inline]
    pub fn is_target_sprite(&self) -> bool {
        self.0.contains(GetUrlFlags::LOAD_TARGET)
    }

    /// Whether this action will load variables into an ActionScript object.
    #[inline]
    pub fn is_load_vars(&self) -> bool {
        self.0.contains(GetUrlFlags::LOAD_VARIABLES)
    }
}

bitflags! {
    // NOTE: The GetURL2 flag layout is listed backwards in the SWF19 specs.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
    pub(crate) struct GetUrlFlags: u8 {
        const METHOD_NONE = 0;
        const METHOD_GET = 1;
        const METHOD_POST = 2;
        const METHOD_MASK = 3;

        const LOAD_TARGET = 1 << 6;
        const LOAD_VARIABLES = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SendVarsMethod {
    None = 0,
    Get = 1,
    Post = 2,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GotoFrame {
    pub frame: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GotoFrame2 {
    pub set_playing: bool,
    pub scene_offset: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GotoLabel<'a> {
    pub label: &'a SwfStr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct If {
    pub offset: i16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Jump {
    pub offset: i16,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Push<'a> {
    pub values: Vec<Value<'a>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SetTarget<'a> {
    pub target: &'a SwfStr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct StoreRegister {
    pub register: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Try<'a> {
    pub try_body: &'a [u8],
    pub catch_body: Option<(CatchVar<'a>, &'a [u8])>,
    pub finally_body: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WaitForFrame {
    pub frame: u16,
    pub num_actions_to_skip: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WaitForFrame2 {
    pub num_actions_to_skip: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct With<'a> {
    pub actions: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Unknown<'a> {
    pub opcode: u8,
    pub data: &'a [u8],
}

impl Action<'_> {
    pub fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}
