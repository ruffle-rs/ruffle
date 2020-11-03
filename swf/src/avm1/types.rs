use smallvec::SmallVec;

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
    ConstantPool(Vec<&'a str>),
    Decrement,
    DefineFunction {
        name: &'a str,
        params: Vec<&'a str>,
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
        url: &'a str,
        target: &'a str,
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
    GotoLabel(&'a str),
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
    Push(SmallVec<[Value<'a>; 4]>),
    PushDuplicate,
    RandomNumber,
    RemoveSprite,
    Return,
    SetMember,
    SetProperty,
    SetTarget(&'a str),
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
    Str(&'a str),
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
    pub name: &'a str,
    pub register_count: u8,
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
    pub actions: &'a [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionParam<'a> {
    pub name: &'a str,
    pub register_index: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TryBlock<'a> {
    pub try_actions: &'a [u8],
    pub catch: Option<(CatchVar<'a>, &'a [u8])>,
    pub finally: Option<&'a [u8]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatchVar<'a> {
    Var(&'a str),
    Register(u8),
}
