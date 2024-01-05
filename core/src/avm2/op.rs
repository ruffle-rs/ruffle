use swf::avm2::types::{Class, Exception, Index, LookupSwitch, Method, Multiname, Namespace};

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Add,
    AddI,
    ApplyType {
        num_types: u32,
    },
    AsType {
        type_name: Index<Multiname>,
    },
    AsTypeLate,
    BitAnd,
    BitNot,
    BitOr,
    BitXor,
    Bkpt,
    BkptLine {
        line_num: u32,
    },
    Call {
        num_args: u32,
    },
    CallMethod {
        index: Index<Method>,
        num_args: u32,
    },
    CallProperty {
        index: Index<Multiname>,
        num_args: u32,
    },
    CallPropLex {
        index: Index<Multiname>,
        num_args: u32,
    },
    CallPropVoid {
        index: Index<Multiname>,
        num_args: u32,
    },
    CallStatic {
        index: Index<Method>,
        num_args: u32,
    },
    CallSuper {
        index: Index<Multiname>,
        num_args: u32,
    },
    CallSuperVoid {
        index: Index<Multiname>,
        num_args: u32,
    },
    CheckFilter,
    Coerce {
        index: Index<Multiname>,
    },
    CoerceA,
    CoerceB,
    CoerceD,
    CoerceI,
    CoerceO,
    CoerceS,
    CoerceU,
    Construct {
        num_args: u32,
    },
    ConstructProp {
        index: Index<Multiname>,
        num_args: u32,
    },
    ConstructSuper {
        num_args: u32,
    },
    ConvertO,
    ConvertS,
    Debug {
        is_local_register: bool,
        register_name: Index<String>,
        register: u8,
    },
    DebugFile {
        file_name: Index<String>,
    },
    DebugLine {
        line_num: u32,
    },
    DecLocal {
        index: u32,
    },
    DecLocalI {
        index: u32,
    },
    Decrement,
    DecrementI,
    DeleteProperty {
        index: Index<Multiname>,
    },
    Divide,
    Dup,
    Equals,
    EscXAttr,
    EscXElem,
    FindDef {
        index: Index<Multiname>,
    },
    FindProperty {
        index: Index<Multiname>,
    },
    FindPropStrict {
        index: Index<Multiname>,
    },
    GetDescendants {
        index: Index<Multiname>,
    },
    GetGlobalScope,
    GetGlobalSlot {
        index: u32,
    },
    GetLex {
        index: Index<Multiname>,
    },
    GetLocal {
        index: u32,
    },
    GetOuterScope {
        index: u32,
    },
    GetProperty {
        index: Index<Multiname>,
    },
    GetScopeObject {
        index: u8,
    },
    GetSlot {
        index: u32,
    },
    GetSuper {
        index: Index<Multiname>,
    },
    GreaterEquals,
    GreaterThan,
    HasNext,
    HasNext2 {
        object_register: u32,
        index_register: u32,
    },
    IfEq {
        offset: i32,
    },
    IfFalse {
        offset: i32,
    },
    IfGe {
        offset: i32,
    },
    IfGt {
        offset: i32,
    },
    IfLe {
        offset: i32,
    },
    IfLt {
        offset: i32,
    },
    IfNe {
        offset: i32,
    },
    IfNge {
        offset: i32,
    },
    IfNgt {
        offset: i32,
    },
    IfNle {
        offset: i32,
    },
    IfNlt {
        offset: i32,
    },
    IfStrictEq {
        offset: i32,
    },
    IfStrictNe {
        offset: i32,
    },
    IfTrue {
        offset: i32,
    },
    In,
    IncLocal {
        index: u32,
    },
    IncLocalI {
        index: u32,
    },
    Increment,
    IncrementI,
    InitProperty {
        index: Index<Multiname>,
    },
    InstanceOf,
    IsType {
        index: Index<Multiname>,
    },
    IsTypeLate,
    Jump {
        offset: i32,
    },
    Kill {
        index: u32,
    },
    Label,
    LessEquals,
    LessThan,
    Lf32,
    Lf64,
    Li16,
    Li32,
    Li8,
    LookupSwitch(Box<LookupSwitch>),
    LShift,
    Modulo,
    Multiply,
    MultiplyI,
    Negate,
    NegateI,
    NewActivation,
    NewArray {
        num_args: u32,
    },
    NewCatch {
        index: Index<Exception>,
    },
    NewClass {
        index: Index<Class>,
    },
    NewFunction {
        index: Index<Method>,
    },
    NewObject {
        num_args: u32,
    },
    NextName,
    NextValue,
    Nop,
    Not,
    Pop,
    PopScope,
    PushByte {
        value: u8,
    },
    PushDouble {
        value: f64,
    },
    PushFalse,
    PushInt {
        value: i32,
    },
    PushNamespace {
        value: Index<Namespace>,
    },
    PushNaN,
    PushNull,
    PushScope,
    PushShort {
        value: i16,
    },
    PushString {
        value: Index<String>,
    },
    PushTrue,
    PushUint {
        value: u32,
    },
    PushUndefined,
    PushWith,
    ReturnValue,
    ReturnVoid,
    RShift,
    SetGlobalSlot {
        index: u32,
    },
    SetLocal {
        index: u32,
    },
    SetProperty {
        index: Index<Multiname>,
    },
    SetSlot {
        index: u32,
    },
    SetSuper {
        index: Index<Multiname>,
    },
    Sf32,
    Sf64,
    Si16,
    Si32,
    Si8,
    StrictEquals,
    Subtract,
    SubtractI,
    Swap,
    Sxi1,
    Sxi16,
    Sxi8,
    Throw,
    TypeOf,
    Timestamp,
    URShift,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<Op>() == 16);
