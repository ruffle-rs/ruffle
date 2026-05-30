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
use std::collections::BTreeMap;
use std::ops::Range;

/// The minimum number of consecutive ops that will be run in the integer
/// interpreter. If this number is too low, the overhead of entering and exiting
/// the integer interpreter may be greater than the speedup of having faster
/// ops. On the other hand, if this number is too high, some sequences of ops
/// that would benefit from being run in the integer interpreter may end up
/// being considered too short to be run in it.
const MIN_INT_OPS_LENGTH: usize = 30;

/// The maximum number of ops in a method that can be considered for int
/// interpreter analysis.
const MAX_METHOD_OPS_LENGTH: usize = 600;

pub fn run_analysis<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    ops: &[Cell<Op<'gc>>],
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

    let object_classes = EnumMap::from_fn(|t| match t {
        ObjectType::TopOuterScope => top_outer_scope_class,
    });

    // Keep track of the ranges that we've already created int interpreter
    // promotions for so that we don't create even more int interpreter
    // promotions within them. Because `empty_stack_positions` is a `BTreeMap`,
    // iterating over it will result in earlier positions (the ones that will
    // yield the largest promoted areas) being handled first.
    let mut covered_ranges: Vec<Range<usize>> = Vec::new();

    // Now we can actually run the analysis pass.
    for (start_index, locals_state) in empty_stack_positions {
        let start_index = *start_index;

        if covered_ranges.iter().any(|r| r.contains(&start_index)) {
            // See above comment
            continue;
        }

        let analysis_results =
            run_single_analysis(ops, start_index, *locals_state, &object_classes);

        if let Some((info, num_ops)) = analysis_results {
            covered_ranges.push(start_index..start_index + num_ops);

            let op = Op::RunIntInterpreter(Gc::new(activation.gc(), info));
            ops[start_index].set(op);
        }
    }
}

fn run_single_analysis<'gc>(
    ops: &[Cell<Op<'gc>>],
    start_index: usize,
    used_locals: SmallBitSet,
    object_classes: &EnumMap<ObjectType, Option<Class<'gc>>>,
) -> Option<(IntInterpreterInfo, usize)> {
    let mut output_vec = Vec::new();

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
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::Add
            }
            Op::BitAnd => {
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::BitAnd
            }
            Op::BitNot => {
                stack.pop();
                stack.push_int();

                IntOp::BitNot
            }
            Op::BitOr => {
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::BitOr
            }
            Op::BitXor => {
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::BitXor
            }
            Op::DecLocalI { index } => {
                if !used_locals.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                IntOp::DecLocal { index }
            }
            Op::Dup => {
                let value = stack.pop();
                stack.push(value);
                stack.push(value);

                IntOp::Dup
            }
            Op::EqualsIntegral => {
                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::Equals
            }
            Op::GetLocal { index } => {
                if !used_locals.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                // Only integers can be stored in locals
                stack.push_int();

                IntOp::GetLocal { index }
            }
            Op::GetOuterScope { index: 0 } => {
                stack.push_object(ObjectType::TopOuterScope);

                IntOp::PushObject {
                    value: ObjectType::TopOuterScope,
                }
            }
            Op::GetSlot { index } => {
                let Some(object_type) = stack.pop_expecting_object() else {
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
                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::GreaterEquals
            }
            Op::GreaterThanIntegral => {
                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::GreaterThan
            }
            Op::IfFalse { offset } => {
                stack.pop();

                if !stack.is_empty() {
                    // It's fairly rare for items to be on the stack during
                    // merging, and this allows us to avoid dealing with
                    // state merging or tracking to know the correct height of
                    // the stack at a certain point. However, this does mean
                    // all ternary expressions disable int interpreter
                    // promotion. TODO support this case
                    break;
                }

                IntOp::IfFalseExternal {
                    offset: Cell::new(offset as u32),

                    // The frame size of the abstract interpreter should fit in
                    // a u8
                    final_stack_height: stack.len() as u8,
                }
            }
            Op::IfTrue { offset } => {
                stack.pop();

                if !stack.is_empty() {
                    // See comment on `Op::IfFalse`
                    break;
                }

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
                if !used_locals.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                IntOp::IncLocal { index }
            }
            Op::LessEqualsIntegral => {
                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::LessEquals
            }
            Op::LessThanIntegral => {
                stack.pop();
                stack.pop();
                stack.push_bool();

                IntOp::LessThan
            }
            Op::Li8 => {
                stack.pop();
                stack.push_int();

                IntOp::Li8
            }
            Op::Li16 => {
                stack.pop();
                stack.push_int();

                IntOp::Li16
            }
            Op::Li32 => {
                stack.pop();
                stack.push_int();

                IntOp::Li32
            }
            Op::LShift => {
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::LShift
            }
            Op::Nop => IntOp::Nop,
            Op::Not => {
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
                if !used_locals.get(index as usize) {
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

                if !stack.pop_expecting_int() {
                    // Can't store a non-integer into a local
                    break;
                }

                IntOp::SetLocal { index }
            }
            Op::RShift => {
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::RShift
            }
            Op::Si8 => {
                stack.pop();
                stack.pop();

                IntOp::Si8
            }
            Op::Si16 => {
                stack.pop();
                stack.pop();

                IntOp::Si16
            }
            Op::Si32 => {
                stack.pop();
                stack.pop();

                IntOp::Si32
            }
            Op::StoreLocal { index } => {
                if !used_locals.get(index as usize) {
                    // Can't access a non-int

                    // See comment on `Op::SetLocal`
                    break;
                }

                if !stack.pop_expecting_int() {
                    // Can't store a non-integer into a local
                    break;
                }
                stack.push_int();

                IntOp::StoreLocal { index }
            }
            Op::SubtractI => {
                stack.pop();
                stack.pop();
                stack.push_int();

                IntOp::Subtract
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

    // Not enough ops for entering the int interpreter to be worth it
    if num_ops < MIN_INT_OPS_LENGTH {
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

    // Right now all the branches/jumps are external, i.e. they will exit the int
    // interpreter into normal code. This is technically correct, but let's see
    // which of the branches we can make internal branches (branches that
    // remain in the int interpreter). We want to stay in the int interpreter
    // as much as possible.
    for op in &mut output_vec {
        match op {
            IntOp::IfFalseExternal { offset, .. } => {
                let offset = offset.get() as usize;
                if (start_index..start_index + num_ops).contains(&offset) {
                    *op = IntOp::IfFalse {
                        offset: (offset - start_index) as u32,
                    };
                }
            }
            IntOp::IfTrueExternal { offset, .. } => {
                let offset = offset.get() as usize;
                if (start_index..start_index + num_ops).contains(&offset) {
                    *op = IntOp::IfTrue {
                        offset: (offset - start_index) as u32,
                    };
                }
            }
            IntOp::JumpExternal { offset, .. } => {
                let offset = offset.get() as usize;
                if (start_index..start_index + num_ops).contains(&offset) {
                    *op = IntOp::Jump {
                        offset: (offset - start_index) as u32,
                    };
                }
            }
            _ => {}
        }
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
#[derive(Clone, Copy)]
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

    pub fn pop_expecting_int(&mut self) -> bool {
        matches!(self.pop(), ValueType::Int)
    }

    pub fn pop_expecting_object(&mut self) -> Option<ObjectType> {
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
