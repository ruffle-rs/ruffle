use crate::avm2::activation::Activation;
use crate::avm2::int_interpreter::MAX_INT_INTERPRETER_FRAME;
use crate::avm2::method::Method;
use crate::avm2::op::{IntInterpreterInfo, IntOp, Op};
use crate::avm2::optimizer::utils::SmallBitSet;

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
const MAX_METHOD_OPS_LENGTH: usize = 300;

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

        if let Some((info, num_ops)) = run_single_analysis(ops, start_index, locals_state) {
            covered_ranges.push(start_index..start_index + num_ops);

            let op = Op::RunIntInterpreter(Gc::new(activation.gc(), info));
            ops[start_index].set(op);
        }
    }
}

fn run_single_analysis<'gc>(
    ops: &[Cell<Op<'gc>>],
    start_index: usize,
    entry_locals_state: &SmallBitSet,
) -> Option<(IntInterpreterInfo, usize)> {
    let mut output_vec = Vec::new();

    let mut current_locals_state = entry_locals_state.clone();
    let mut stack_height = 0;

    // We run a simplified abstract interpreter pass. We know that at this point,
    // the stack is empty and locals are either integral or non-integral. Loading
    // a non-integral local results in the pass being aborted, so we can be
    // certain that these ops are only working on integers.
    let mut i = start_index;
    while i < ops.len() {
        let op = ops[i].get();

        let translated_op = match op {
            Op::AddI => {
                stack_height -= 1;

                IntOp::Add
            }
            Op::BitAnd => {
                stack_height -= 1;

                IntOp::BitAnd
            }
            Op::BitNot => IntOp::BitNot,
            Op::BitOr => {
                stack_height -= 1;

                IntOp::BitOr
            }
            Op::BitXor => {
                stack_height -= 1;

                IntOp::BitXor
            }
            Op::DecLocalI { index } => {
                if !current_locals_state.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                IntOp::DecLocal { index }
            }
            Op::Dup => {
                stack_height += 1;

                IntOp::Dup
            }
            Op::GetLocal { index } => {
                if !current_locals_state.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                stack_height += 1;

                IntOp::GetLocal { index }
            }
            Op::IncLocalI { index } => {
                if !current_locals_state.get(index as usize) {
                    // Can't access a non-int
                    break;
                }

                IntOp::IncLocal { index }
            }
            Op::Jump { offset } => IntOp::ExternalJump {
                offset: Cell::new(offset as u32),
            },
            Op::Li8 => IntOp::Li8,
            Op::Li32 => IntOp::Li32,
            Op::Nop => IntOp::Nop,
            Op::PushInt { value } => {
                stack_height += 1;

                IntOp::PushInt { value }
            }
            Op::SetLocal { index } => {
                stack_height -= 1;

                current_locals_state.set(index as usize, true);
                IntOp::SetLocal { index }
            }
            Op::Si8 => {
                stack_height -= 2;

                IntOp::Si8
            }
            Op::Si32 => {
                stack_height -= 2;

                IntOp::Si32
            }
            Op::StoreLocal { index } => {
                current_locals_state.set(index as usize, true);
                IntOp::StoreLocal { index }
            }
            Op::SubtractI => {
                stack_height -= 1;

                IntOp::Subtract
            }
            _ => {
                break;
            }
        };

        output_vec.push(translated_op);

        i += 1;
    }

    let num_ops = output_vec.len();

    // Once all ops are done executing, jump to the normal interpreter at the
    // position where we should continue.
    output_vec.push(IntOp::ExternalJump {
        offset: Cell::new((start_index + num_ops) as u32),
    });

    // Not enough ops for entering the int interpreter to be worth it
    if num_ops < MIN_INT_OPS_LENGTH {
        return None;
    }

    let info = IntInterpreterInfo {
        input_locals: entry_locals_state.clone(),
        output_locals: current_locals_state,
        final_stack_height: stack_height,
        ops: output_vec,
    };

    Some((info, num_ops))
}
