use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::multiname::Multiname;
use crate::avm2::namespace::Namespace;
use crate::avm2::optimizer::utils::SmallBitSet;
use crate::avm2::script::Script;
use crate::string::AvmAtom;

use gc_arena::{Collect, Gc};
use std::cell::Cell;

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub enum Op<'gc> {
    Add {
        /// Whether the two inputs to this op are both guaranteed to be
        /// integers. This field is used in the optimizer.
        inputs_integral: bool,
    },
    AddI,
    ApplyType {
        num_types: u32,
    },
    AsType {
        class: Class<'gc>,
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
        index: usize,
        num_args: u32,
        push_return_value: bool,
    },
    CallNative {
        #[collect(require_static)]
        method: NativeMethodImpl,
        num_args: u32,
        push_return_value: bool,
    },
    CallProperty {
        multiname: Gc<'gc, Multiname<'gc>>,

        num_args: u32,
    },
    CallPropLex {
        multiname: Gc<'gc, Multiname<'gc>>,

        num_args: u32,
    },
    CallPropVoid {
        multiname: Gc<'gc, Multiname<'gc>>,

        num_args: u32,
    },
    CallStatic {
        method: Method<'gc>,

        num_args: u32,
    },
    CallSuper {
        multiname: Gc<'gc, Multiname<'gc>>,

        num_args: u32,
    },
    CheckFilter,
    Coerce {
        class: Class<'gc>,
    },
    CoerceSwapPop {
        class: Class<'gc>,
    },
    CoerceA,
    CoerceB,
    CoerceD,
    CoerceDSwapPop,
    CoerceI,
    CoerceISwapPop,
    CoerceO,
    CoerceS,
    CoerceU,
    CoerceUSwapPop,
    Construct {
        num_args: u32,
    },
    ConstructProp {
        multiname: Gc<'gc, Multiname<'gc>>,
        num_args: u32,
    },
    ConstructSlot {
        index: usize,
        num_args: u32,
    },
    ConstructSuper {
        num_args: u32,
    },
    ConvertO,
    ConvertS,
    Debug {
        is_local_register: bool,
        register_name: AvmAtom<'gc>,
        register: u8,
    },
    DebugFile {
        file_name: AvmAtom<'gc>,
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
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    Divide,
    Dup,
    Dxns {
        string: AvmAtom<'gc>,
    },
    DxnsLate,
    Equals,
    EscXAttr,
    EscXElem,
    FindDef {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    FindProperty {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    FindPropStrict {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    GetDescendants {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    GetLocal {
        index: u32,
    },
    GetOuterScope {
        index: usize,
    },

    // The GetProperty op is specialized into three different ops depending on the multiname.
    //  - If the multiname is fully static, the verifier emits GetPropertyStatic.
    //  - If the multiname has a lazy name, a static namespace, contains the
    //    the public namespace, and is not an attribute multiname, the verifier
    //    emits GetPropertyFast.
    //  - If neither condition is met (i.e. the multiname has a lazy namespace),
    //    the verifier emits GetPropertySlow.
    GetPropertyStatic {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    GetPropertyFast {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    GetPropertySlow {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    GetScopeObject {
        index: usize,
    },
    GetScriptGlobals {
        script: Script<'gc>,
    },
    GetSlot {
        // note: 0-indexed, as opposed to FP.
        index: usize,
    },
    GetSuper {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    GreaterEquals,
    GreaterThan,
    HasNext,
    HasNext2 {
        object_register: u32,
        index_register: u32,
    },
    IfFalse {
        offset: usize,
    },
    IfTrue {
        offset: usize,
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
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    InstanceOf,
    IsType {
        class: Class<'gc>,
    },
    IsTypeLate,
    Jump {
        offset: usize,
    },
    Kill {
        index: u32,
    },
    LessEquals,
    LessThan,
    Lf32,
    Lf64,
    Li16,
    Li32,
    Li8,
    LookupSwitch(Gc<'gc, LookupSwitch>),
    LShift,
    Modulo,
    Multiply,
    MultiplyI,
    Negate,
    NegateI,
    NewActivation {
        activation_class: Class<'gc>,
    },
    NewArray {
        num_args: u32,
    },
    NewCatch {
        index: usize,
    },
    NewClass {
        class: Class<'gc>,
    },
    NewFunction {
        method: Method<'gc>,
    },
    NewObject {
        num_args: u32,
    },
    NextName,
    NextValue,
    Nop,
    Not,
    Pop,
    PopJump {
        offset: usize,
    },
    PopScope,
    PushDouble {
        value: f64,
    },
    PushFalse,
    PushInt {
        value: i32,
    },
    PushNamespace {
        namespace: Namespace<'gc>,
    },
    PushNull,
    PushScope,
    PushString {
        string: AvmAtom<'gc>,
    },
    PushTrue,
    PushUint {
        value: u32,
    },
    PushUndefined,
    PushWith,
    ReturnValue {
        return_type: Option<Class<'gc>>,
    },
    ReturnVoid {
        return_type: Option<Class<'gc>>,
    },
    RunIntInterpreter(Gc<'gc, IntInterpreterInfo>),
    RShift,
    SetGlobalSlot {
        // note: 0-indexed, as opposed to FP.
        index: usize,
    },
    SetLocal {
        index: u32,
    },

    // See the comments on the GetProperty op
    SetPropertyStatic {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    SetPropertyFast {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    SetPropertySlow {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    SetSlot {
        // note: 0-indexed, as opposed to FP.
        index: usize,
    },
    SetSlotCoerceI {
        // note: 0-indexed, as opposed to FP.
        index: usize,
    },
    SetSlotNoCoerce {
        // note: 0-indexed, as opposed to FP.
        index: usize,
    },
    SetSuper {
        multiname: Gc<'gc, Multiname<'gc>>,
    },
    Sf32,
    Sf64,
    Si16,
    Si32,
    Si8,
    StrictEquals,
    StoreLocal {
        index: u32,
    },
    Subtract {
        /// See comment on `Op::Add`
        inputs_integral: bool,
    },
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

impl Op<'_> {
    pub fn can_throw_error(&self) -> bool {
        !matches!(
            self,
            Op::AsType { .. }
                | Op::Bkpt
                | Op::BkptLine { .. }
                | Op::CoerceO
                | Op::Dup
                | Op::GetScopeObject { .. }
                | Op::GetOuterScope { .. }
                | Op::GetLocal { .. }
                | Op::IfTrue { .. }
                | Op::IfFalse { .. }
                | Op::IsType { .. }
                | Op::Jump { .. }
                | Op::Kill { .. }
                | Op::LookupSwitch { .. }
                | Op::Nop
                | Op::Not
                | Op::Pop
                | Op::PopJump { .. }
                | Op::PopScope
                | Op::PushDouble { .. }
                | Op::PushFalse
                | Op::PushInt { .. }
                | Op::PushNamespace { .. }
                | Op::PushNull
                | Op::PushString { .. }
                | Op::PushTrue
                | Op::PushUint { .. }
                | Op::PushUndefined
                | Op::SetLocal { .. }
                | Op::StrictEquals
                | Op::StoreLocal { .. }
                | Op::Swap
                | Op::Timestamp
                | Op::TypeOf
                | Op::ReturnVoid { .. }
        )
    }

    pub fn is_nop(&self) -> bool {
        if cfg!(feature = "avm_debug") {
            matches!(self, Op::Nop | Op::CoerceA)
        } else {
            matches!(
                self,
                Op::Nop
                    | Op::CoerceA
                    | Op::Debug { .. }
                    | Op::DebugFile { .. }
                    | Op::DebugLine { .. }
            )
        }
    }

    /// Whether all this op does is push a single value to the stack, possibly
    /// reading from stack or locals, but never, e.g., throwing an error or
    /// calling a method.
    pub fn is_pure_push(&self) -> bool {
        matches!(
            self,
            Op::PushTrue
                | Op::PushFalse
                | Op::PushUndefined
                | Op::PushNull
                | Op::PushDouble { .. }
                | Op::PushInt { .. }
                | Op::PushUint { .. }
                | Op::GetLocal { .. }
                | Op::Dup
        )
    }
}

// This has interior mutability so that we can rewrite switch offsets from the
// optimizer when we need to
#[derive(Collect, Debug)]
#[collect(require_static)]
pub struct LookupSwitch {
    pub default_offset: Cell<usize>,
    pub case_offsets: Box<[Cell<usize>]>,
}

#[derive(Collect, Debug)]
#[collect(require_static)]
pub struct IntInterpreterInfo {
    /// The locals that should be provided as inputs to the int interpreter.
    pub input_locals: SmallBitSet,

    /// The locals that are modified by the int interpreter.
    pub output_locals: SmallBitSet,

    /// The stack height of the int interpreter after execution ends.
    pub final_stack_height: usize,

    /// The offset in normal code that should be jumped to after execution ends.
    ///
    /// This has interior mutability so that we can rewrite the offset from the
    /// optimizer when we need to.
    pub exit_offset: Cell<usize>,

    /// The ops run by the int interpreter.
    pub ops: Vec<IntOp>,
}

/// An op used in the int interpreter.
///
/// These ops only work on `i32` values.
#[derive(Debug)]
pub enum IntOp {
    Add,
    BitAnd,
    BitNot,
    BitOr,
    BitXor,
    DecLocal { index: u32 },
    Dup,
    End,
    GetLocal { index: u32 },
    IncLocal { index: u32 },
    Nop,
    PushInt { value: i32 },
    SetLocal { index: u32 },
    StoreLocal { index: u32 },
    Subtract,
}

const _: () = assert!(std::mem::size_of::<IntOp>() == 8);
