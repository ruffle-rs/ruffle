use bitflags::bitflags;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct AbcFile {
    pub major_version: u16,
    pub minor_version: u16,
    pub constant_pool: ConstantPool,
    pub methods: Vec<Method>,
    pub metadata: Vec<Metadata>,
    pub instances: Vec<Instance>,
    pub classes: Vec<Class>,
    pub scripts: Vec<Script>,
    pub method_bodies: Vec<MethodBody>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstantPool {
    pub ints: Vec<i32>,
    pub uints: Vec<u32>,
    pub doubles: Vec<f64>,
    pub strings: Vec<Vec<u8>>,
    pub namespaces: Vec<Namespace>,
    pub namespace_sets: Vec<NamespaceSet>,
    pub multinames: Vec<Multiname>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index<T>(pub u32, pub PhantomData<T>);

// see: https://github.com/rust-lang/rust/issues/26925
impl<T: Clone> Copy for Index<T> {}

impl<T> Index<T> {
    pub fn new(i: u32) -> Index<T> {
        Index(i, PhantomData)
    }

    pub fn as_u30(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Namespace {
    Namespace(Index<String>),
    Package(Index<String>),
    PackageInternal(Index<String>),
    Protected(Index<String>),
    Explicit(Index<String>),
    StaticProtected(Index<String>),
    Private(Index<String>),
}

pub type NamespaceSet = Vec<Index<Namespace>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Multiname {
    QName {
        namespace: Index<Namespace>,
        name: Index<String>,
    },
    QNameA {
        namespace: Index<Namespace>,
        name: Index<String>,
    },
    RTQName {
        name: Index<String>,
    },
    RTQNameA {
        name: Index<String>,
    },
    RTQNameL,
    RTQNameLA,
    Multiname {
        namespace_set: Index<NamespaceSet>,
        name: Index<String>,
    },
    MultinameA {
        namespace_set: Index<NamespaceSet>,
        name: Index<String>,
    },
    MultinameL {
        namespace_set: Index<NamespaceSet>,
    },
    MultinameLA {
        namespace_set: Index<NamespaceSet>,
    },
    TypeName {
        base_type: Index<Multiname>,
        parameters: Vec<Index<Multiname>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
    pub name: Index<String>,
    pub params: Vec<MethodParam>,
    pub return_type: Index<Multiname>,
    pub flags: MethodFlags,
    // not an ABC MethodInfo property; bound when parsing MethodBodies
    pub body: Option<Index<MethodBody>>,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MethodFlags: u8 {
        const NEED_ARGUMENTS  = 1 << 0;
        const NEED_ACTIVATION = 1 << 1;
        const NEED_REST       = 1 << 2;
        const HAS_OPTIONAL    = 1 << 3;
        const IGNORE_REST     = 1 << 4;
        const NATIVE          = 1 << 5;
        const SET_DXNS        = 1 << 6;
        const HAS_PARAM_NAMES = 1 << 7;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodParam {
    pub name: Option<Index<String>>,
    pub kind: Index<Multiname>,
    pub default_value: Option<DefaultValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MethodBody {
    pub method: Index<Method>,
    pub max_stack: u32,
    pub num_locals: u32,
    pub init_scope_depth: u32,
    pub max_scope_depth: u32,
    pub code: Vec<u8>,
    pub exceptions: Vec<Exception>,
    pub traits: Vec<Trait>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Exception {
    pub from_offset: u32,
    pub to_offset: u32,
    pub target_offset: u32,
    // Both of the following fields are actually `Multiname` indexes
    // despite AVM 2 description saying they are `String` ones.
    pub variable_name: Index<Multiname>,
    pub type_name: Index<Multiname>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Opcode;

#[derive(Clone, Debug, PartialEq)]
pub enum DefaultValue {
    Int(Index<i32>),
    Uint(Index<u32>),
    Double(Index<f64>),
    String(Index<String>),
    True,
    False,
    Null,
    Undefined,
    Namespace(Index<Namespace>),
    Package(Index<Namespace>),
    PackageInternal(Index<Namespace>),
    Protected(Index<Namespace>),
    Explicit(Index<Namespace>),
    StaticProtected(Index<Namespace>),
    Private(Index<Namespace>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    pub name: Index<String>,
    pub items: Vec<MetadataItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataItem {
    pub key: Index<String>,
    pub value: Index<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instance {
    pub name: Index<Multiname>,
    pub super_name: Index<Multiname>,
    pub is_sealed: bool,
    pub is_final: bool,
    pub is_interface: bool,
    pub protected_namespace: Option<Index<Namespace>>,
    pub interfaces: Vec<Index<Multiname>>,
    pub init_method: Index<Method>,
    pub traits: Vec<Trait>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Trait {
    pub name: Index<Multiname>,
    pub kind: TraitKind,
    pub metadata: Vec<Index<Metadata>>,
    pub is_final: bool,
    pub is_override: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TraitKind {
    Slot {
        slot_id: u32,
        type_name: Index<Multiname>,
        value: Option<DefaultValue>,
    },
    Method {
        disp_id: u32,
        method: Index<Method>,
    },
    Getter {
        disp_id: u32,
        method: Index<Method>,
    },
    Setter {
        disp_id: u32,
        method: Index<Method>,
    },
    Class {
        slot_id: u32,
        class: Index<Class>,
    },
    Function {
        slot_id: u32,
        function: Index<Method>,
    },
    Const {
        slot_id: u32,
        type_name: Index<Multiname>,
        value: Option<DefaultValue>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub init_method: Index<Method>,
    pub traits: Vec<Trait>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Script {
    pub init_method: Index<Method>,
    pub traits: Vec<Trait>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LookupSwitch {
    pub default_offset: i32,
    pub case_offsets: Box<[i32]>,
}

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
        index: u32,
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
    ConvertB,
    ConvertD,
    ConvertI,
    ConvertO,
    ConvertS,
    ConvertU,
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
    Dxns {
        index: Index<String>,
    },
    DxnsLate,
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
        value: Index<f64>,
    },
    PushFalse,
    PushInt {
        value: Index<i32>,
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
        value: Index<u32>,
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
