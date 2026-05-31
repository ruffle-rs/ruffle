use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::int_interpreter::{MAX_INT_INTERPRETER_FRAME, ObjectType};
use crate::avm2::method::Method;
use crate::avm2::object::TObject;
use crate::avm2::op::{IntInterpreterInfo, IntOp, Op};
use crate::avm2::optimizer::utils::SmallBitSet;

use enum_map::EnumMap;
use gc_arena::Gc;
use std::cell::Cell;
use std::collections::{BTreeMap, HashSet};
use std::ops::Range;

/// The minimum number of consecutive ops that will be run in the integer
/// interpreter. If this number is too low, the overhead of entering and exiting
/// the integer interpreter may be greater than the speedup of having faster
/// ops. On the other hand, if this number is too high, some sequences of ops
/// that would benefit from being run in the integer interpreter may end up
/// being considered too short to be run in it.
const MIN_INT_OPS_LENGTH: usize = 50;

/// The maximum number of ops in a method that can be considered for int
/// interpreter analysis.
const MAX_METHOD_OPS_LENGTH: usize = 40000;

pub fn run_analysis<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    ops: &[Cell<Op<'gc>>],
    jump_targets: &HashSet<usize>,
    empty_stack_positions: &BTreeMap<usize, SmallBitSet>,
    has_exceptions: bool,
) {
    let method_body = method
        .body()
        .expect("Cannot verify non-native method without body!");
    let max_locals = method_body.num_locals;
    let max_stack = method_body.max_stack;

    if ops.len() > MAX_METHOD_OPS_LENGTH {
        // Not worth trying to optimize a method this large
        return;
    } else if has_exceptions {
        // The analysis does not support handling exceptions
        return;
    } else if (max_locals + max_stack) as usize >= MAX_INT_INTERPRETER_FRAME {
        // The int interpreter does not support a frame size larger the max
        return;
    } else if method
        .translation_unit()
        .domain()
        .is_playerglobals_domain(activation.avm2())
    {
        // Some playerglobals code will run before domain memory is initialized,
        // so if playerglobals code attempts to access domain memory there'll
        // be a panic on startup
        return;
    }

    let top_outer_scope_class = activation
        .outer()
        .get(0)
        .and_then(|s| s.values().as_object())
        .map(|o| o.instance_class());

    // TODO we already have this information from earlier from
    // `peephole::postprocess_peephole`; figure out some way to reuse it
    let mut sets_local_0 = false;

    for op in ops {
        match op.get() {
            Op::SetLocal { index }
            | Op::Kill { index }
            | Op::DecLocal { index }
            | Op::DecLocalI { index }
            | Op::IncLocal { index }
            | Op::IncLocalI { index }
                if index == 0 =>
            {
                sets_local_0 = true;
                break;
            }
            Op::HasNext2 {
                object_register,
                index_register,
            } if object_register == 0 || index_register == 0 => {
                sets_local_0 = true;
                break;
            }

            _ => {}
        }
    }

    let receiver_class = method.bound_class().filter(|_| !sets_local_0);

    let object_classes = EnumMap::from_fn(|t| match t {
        ObjectType::TopOuterScope => top_outer_scope_class,
        ObjectType::Receiver => receiver_class,
    });

    // Keep track of the ranges that we've already created int interpreter
    // promotions for so that we don't create even more int interpreter
    // promotions within them. Because `empty_stack_positions` is a `BTreeMap`,
    // iterating over it will result in earlier positions (the ones that will
    // yield the largest promoted areas) being handled first.
    let mut already_covered_ranges: Vec<Range<usize>> = Vec::new();

    // Now we can actually run the analysis pass.
    for (start_index, locals_state) in empty_stack_positions {
        let start_index = *start_index;

        // If the start index is a jump target, do promotion anyway
        if !jump_targets.contains(&start_index) {
            if already_covered_ranges
                .iter()
                .any(|r| r.contains(&start_index))
            {
                // See above comment
                continue;
            }
        }

        let analysis_results = run_single_analysis(
            ops,
            start_index,
            *locals_state,
            &object_classes,
            sets_local_0,
        );

        if let Some((info, num_ops)) = analysis_results {
            already_covered_ranges.push(start_index..start_index + num_ops);

            let op = Op::RunIntInterpreter(Gc::new(activation.gc(), info));
            ops[start_index].set(op);
        }
    }
}

fn run_single_analysis<'gc>(
    ops: &[Cell<Op<'gc>>],
    start_index: usize,
    integral_locals: SmallBitSet,
    object_classes: &EnumMap<ObjectType, Option<Class<'gc>>>,
    sets_local_0: bool,
) -> Option<(IntInterpreterInfo, usize)> {
    let mut output_vec = Vec::new();
    let mut used_locals = SmallBitSet::new(integral_locals.len());

    let mut stack = Stack::new();

    // We run a simplified abstract interpreter pass. We know that at this point,
    // the stack is empty and locals are either integral or non-integral. Loading
    // a non-integral local results in the pass being aborted, so we can be
    // certain that these ops are only working on integers.
    let mut i = start_index;
    while i < ops.len() {
        let op = ops[i].get();

        let translated_op = match op {
            Op::AddI => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::Add
            }
            Op::BitAnd => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::BitAnd
            }
            Op::BitNot => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.push_int();

                IntOp::BitNot
            }
            Op::BitOr => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::BitOr
            }
            Op::BitXor => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::BitXor
            }
            Op::DecLocalI { index } => {
                if !integral_locals.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                used_locals.set(index as usize);

                IntOp::DecLocal { index }
            }
            Op::Dup => {
                let value = stack.pop();
                stack.push(value);
                stack.push(value);

                IntOp::Dup
            }
            Op::EqualsIntegral => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::Equals
            }
            Op::GetLocal { index } => {
                // Accessing the receiver and accessing an integer local are
                // two completely different things, so handle them separately

                if !sets_local_0 && index == 0 {
                    stack.push_object(ObjectType::Receiver);

                    IntOp::PushObject {
                        value: ObjectType::Receiver,
                    }
                } else {
                    if !integral_locals.get(index as usize) {
                        // Can't access a non-int
                        break;
                    }

                    used_locals.set(index as usize);

                    // Only integers can be stored in locals
                    stack.push_int();

                    IntOp::GetLocal { index }
                }
            }
            Op::GetOuterScope { index: 0 } => {
                stack.push_object(ObjectType::TopOuterScope);

                IntOp::PushObject {
                    value: ObjectType::TopOuterScope,
                }
            }
            Op::GetSlot { index } => {
                let Some(object_type) = stack.pop_object() else {
                    // Getslot on a primitive is impossible thanks to the
                    // verifier
                    unreachable!()
                };

                let Some(class) = object_classes[object_type] else {
                    // We couldn't conclude the class from the simplified
                    // analysis that we perform. This is a very rare case.

                    // We need to restore the stack to how it was before this
                    // op was considered.
                    stack.push_object(object_type);

                    break;
                };

                let vtable = class.vtable();

                let value_class = vtable
                    .slot_class(index)
                    .expect("Guaranteed by verifier")
                    .get_existing_class();

                if value_class.is_none_or(|c| !c.is_builtin_int()) {
                    // Can't access a non-integral slot from an object.

                    // We need to restore the stack to how it was before this
                    // op was considered.
                    stack.push_object(object_type);

                    break;
                }

                stack.push_int();

                IntOp::GetSlot {
                    index: index as u32,
                }
            }
            Op::GreaterEqualsIntegral => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::GreaterEquals
            }
            Op::GreaterThanIntegral => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::GreaterThan
            }
            Op::IfFalse { offset } => {
                if !stack.expect(ValueType::Bool, 1) {
                    // Can't do this operation on non-bools
                    break;
                }

                if stack.len() != 1 {
                    // It's fairly rare for items to be on the stack during
                    // merging, and this allows us to avoid dealing with
                    // state merging or tracking to know the correct height of
                    // the stack at a certain point. However, this does mean
                    // all ternary expressions disable int interpreter
                    // promotion. TODO support this case

                    // (note: this is a != 1 check rather than an is_empty
                    // check, as the one value on the stack will be popped by
                    // this op)
                    break;
                }

                stack.pop();

                IntOp::IfFalseExternal {
                    offset: Cell::new(offset as u32),

                    // The frame size of the abstract interpreter should fit in
                    // a u8
                    final_stack_height: stack.len() as u8,
                }
            }
            Op::IfTrue { offset } => {
                if !stack.expect(ValueType::Bool, 1) {
                    // Can't do this operation on non-bools
                    break;
                }

                if stack.len() != 1 {
                    // See comment on `Op::IfFalse`
                    break;
                }

                stack.pop();

                IntOp::IfTrueExternal {
                    offset: Cell::new(offset as u32),

                    // See comment on `Op::IfFalse`
                    final_stack_height: stack.len() as u8,
                }
            }
            Op::Jump { offset } => {
                if !stack.is_empty() {
                    // See comment on `Op::IfFalse`
                    break;
                }

                IntOp::JumpExternal {
                    offset: Cell::new(offset as u32),

                    // See comment on `Op::IfFalse`
                    final_stack_height: stack.len() as u8,
                }
            }
            Op::IncLocalI { index } => {
                if !integral_locals.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                used_locals.set(index as usize);

                IntOp::IncLocal { index }
            }
            Op::LessEqualsIntegral => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::LessEquals
            }
            Op::LessThanIntegral => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::LessThan
            }
            Op::Li8 => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.push_int();

                IntOp::Li8
            }
            Op::Li16 => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.push_int();

                IntOp::Li16
            }
            Op::Li32 => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.push_int();

                IntOp::Li32
            }
            Op::LShift => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::LShift
            }
            Op::MultiplyIntegralI => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::MultiplyNumbers
            }
            Op::Nop => IntOp::Nop,
            Op::Not => {
                if !stack.expect(ValueType::Bool, 1) {
                    // Can't do this operation on non-bools
                    break;
                }

                stack.pop();
                stack.push_bool();

                IntOp::Not
            }
            Op::Pop => {
                stack.pop();

                IntOp::Pop
            }
            Op::PushInt { value } => {
                stack.push_int();

                IntOp::PushInt { value }
            }
            Op::SetLocal { index } => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't store a non-integer into a local
                    break;
                }

                if !integral_locals.get(index as usize) {
                    // Can't access a non-int

                    // i.e. we cannot set a non-int to an int, which would
                    // break certain guarantees that simplify handling of
                    // branching. For example, if the program proceeded for a
                    // while assuming that local #X was non-integral, then
                    // local #X were to be set to be integral in a branch, then
                    // we would have to implement proper state merging to
                    // determine that local #X was non-integral.
                    break;
                }

                used_locals.set(index as usize);

                stack.pop();

                IntOp::SetLocal { index }
            }
            Op::SetSlotNoCoerce { index } => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't store a non-integer into a slot
                    break;
                }

                // The verifier guarantees that the receiver value will be an
                // object, as primitives have no slots

                stack.pop();
                stack.pop();

                // We just guaranteed that the stack has an integer on it and
                // that the value was about to be stored without coercion
                // (this is a SetSlotNoCoerce op), so we know that this is an
                // integer store

                IntOp::SetSlot {
                    index: index as u32,
                }
            }
            Op::RShift => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::RShift
            }
            Op::Si8 => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();

                IntOp::Si8
            }
            Op::Si16 => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();

                IntOp::Si16
            }
            Op::Si32 => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();

                IntOp::Si32
            }
            Op::StoreLocal { index } => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't store a non-integer into a local
                    break;
                }

                if !integral_locals.get(index as usize) {
                    // Can't access a non-int

                    // See comment on `Op::SetLocal`
                    break;
                }

                used_locals.set(index as usize);

                // No need to change the stack

                IntOp::StoreLocal { index }
            }
            Op::SubtractI => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::Subtract
            }
            Op::Swap => {
                let first = stack.pop();
                let second = stack.pop();
                stack.push(first);
                stack.push(second);

                IntOp::Swap
            }
            Op::Sxi16 => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.push_int();

                IntOp::Sxi16
            }
            Op::Sxi8 => {
                if !stack.expect(ValueType::Int, 1) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.push_int();

                IntOp::Sxi8
            }
            Op::URShiftI => {
                if !stack.expect(ValueType::Int, 2) {
                    // Can't do this operation on non-ints
                    break;
                }

                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::URShift
            }
            _ => {
                break;
            }
        };

        output_vec.push(translated_op);

        i += 1;
    }

    // A single `pushobject` at the end of the output vec can lead to the pass
    // failing. So if there is one, let's remove it now.
    if matches!(output_vec.last(), Some(IntOp::PushObject { .. })) {
        output_vec.pop();

        // Also remove the object from the stack
        stack.pop();
    }

    let num_ops = output_vec.len();

    let mut has_backwards_branch = false;

    // Right now all the branches/jumps are external, i.e. they will exit the int
    // interpreter into normal code. This is technically correct, but let's see
    // which of the branches we can make internal branches (branches that
    // remain in the int interpreter). We want to stay in the int interpreter
    // as much as possible.
    for (i, op) in output_vec.iter_mut().enumerate() {
        match op {
            IntOp::IfFalseExternal { offset, .. } => {
                let offset = offset.get() as usize;
                if (start_index..start_index + num_ops).contains(&offset) {
                    let new_offset = offset - start_index;

                    *op = IntOp::IfFalse {
                        offset: new_offset as u32,
                    };

                    if new_offset < i {
                        has_backwards_branch = true;
                    }
                }
            }
            IntOp::IfTrueExternal { offset, .. } => {
                let offset = offset.get() as usize;
                if (start_index..start_index + num_ops).contains(&offset) {
                    let new_offset = offset - start_index;

                    *op = IntOp::IfTrue {
                        offset: new_offset as u32,
                    };

                    if new_offset < i {
                        has_backwards_branch = true;
                    }
                }
            }
            IntOp::JumpExternal { offset, .. } => {
                let offset = offset.get() as usize;
                if (start_index..start_index + num_ops).contains(&offset) {
                    let new_offset = offset - start_index;

                    *op = IntOp::Jump {
                        offset: new_offset as u32,
                    };

                    if new_offset < i {
                        has_backwards_branch = true;
                    }
                }
            }
            _ => {}
        }
    }

    // Not enough ops for entering the int interpreter to be worth it
    // (unless there's a backwards branch within the int interpreter; in that
    // case it's likely that this is a hot loop)
    if num_ops < MIN_INT_OPS_LENGTH && !has_backwards_branch {
        return None;
    }

    // This massively complicates synchronization of locals between the normal
    // interpreter and the int interpreter, so we don't support it
    if !stack.is_entirely_ints() {
        return None;
    }

    // Once all ops are done executing, jump to the normal interpreter at the
    // position where we should continue. Unless we got to the end of the
    // method, in which case don't add this last op, as it'll confuse the nop
    // remover.
    if start_index + num_ops != ops.len() {
        output_vec.push(IntOp::JumpExternal {
            offset: Cell::new((start_index + num_ops) as u32),
            final_stack_height: stack.len() as u8,
        });
    }

    let info = IntInterpreterInfo {
        synchronize_locals: used_locals,
        ops: output_vec,
    };

    Some((info, num_ops))
}

/// A value in the int interpreter.
///
/// Currently the int interpreter supports dealing with integers, booleans, and
/// some objects. However, non-integers are limited- they cannot be stored in
/// locals, and they cannot exist on the stack when the code ends.
///
/// Note that when coercing booleans to integers, AVM2 coerces `false` to `0`,
/// and `true` to `1`. This is advantageous to us because we represent those two
/// boolean values as those two integer values at runtime, which means we don't
/// need to ensure that e.g. `add_i` is always being passed two integers- even
/// if it's being passed two booleans, it'll still return the correct result at
/// runtime. The same applies to the rest of the integral ops.
#[derive(Clone, Copy, PartialEq)]
enum ValueType {
    Bool,
    Int,
    Object(ObjectType),
}

/// The stack of the abstract interpreter.
///
/// As the abstract interpreter only supports up to MAX_INT_INTERPRETER_FRAME
/// slots, this will never have a length greater than MAX_INT_INTERPRETER_FRAME.
struct Stack(Vec<ValueType>);

impl Stack {
    pub fn new() -> Self {
        Stack(Vec::with_capacity(MAX_INT_INTERPRETER_FRAME - 1))
    }

    /// Expect the topmost `count` items of the stack to be of type `expected`.
    pub fn expect(&mut self, expected: ValueType, count: usize) -> bool {
        for item in self.0.iter().rev().take(count) {
            if *item != expected {
                return false;
            }
        }

        return true;
    }

    pub fn push(&mut self, value: ValueType) {
        self.0.push(value);
    }

    pub fn push_bool(&mut self) {
        self.0.push(ValueType::Bool);
    }

    pub fn push_int(&mut self) {
        self.0.push(ValueType::Int);
    }

    pub fn push_object(&mut self, object_type: ObjectType) {
        self.0.push(ValueType::Object(object_type));
    }

    pub fn pop(&mut self) -> ValueType {
        self.0.pop().expect("Guaranteed by verifier previously")
    }

    pub fn pop_object(&mut self) -> Option<ObjectType> {
        match self.pop() {
            ValueType::Object(object_type) => Some(object_type),
            _ => None,
        }
    }

    pub fn is_entirely_ints(&self) -> bool {
        self.0.iter().all(|v| matches!(v, ValueType::Int))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
