use crate::avm2::op::Op;
use crate::avm2::optimizer::int_interpretation::IntAnalysisInfo;

use std::cell::Cell;
use std::collections::HashSet;

/// A peephole optimizer to run before type-aware optimizations. This should be
/// called once on the entire code slice.
pub fn preprocess_peephole(ops: &[Cell<Op<'_>>]) {
    for (i, op) in ops.iter().enumerate() {
        match op.get() {
            Op::Jump { offset } if offset == i + 1 => {
                op.set(Op::Nop);
            }
            Op::IfTrue { offset } | Op::IfFalse { offset } if offset == i + 1 => {
                op.set(Op::Pop);
            }
            _ => {}
        }
    }
}

/// A peephole optimizer to run after type-aware optimizations. This should be
/// called once on the entire code slice.
pub fn postprocess_peephole<'a>(
    ops: &'a [Cell<Op<'_>>],
    jump_targets: &HashSet<usize>,
    int_analysis_info: &mut IntAnalysisInfo,
) {
    // Determine if this method ever depends on the state of the scope stack.
    let mut uses_scope_ops = false;

    for op in ops {
        match op.get() {
            Op::GetScopeObject { .. }
            | Op::SetGlobalSlot { .. }
            | Op::FindProperty { .. }
            | Op::FindPropStrict { .. }
            | Op::NewFunction { .. }
            | Op::NewClass { .. }
            | Op::PopScope => {
                uses_scope_ops = true;
                break;
            }

            _ => {}
        }
    }

    // If the scope stack is never read from, only written to, we can replace
    // `PushScope` ops with `Pop`. This allows the peephole optimizer to emit
    // slightly better code.
    for current_op in ops {
        match current_op.get() {
            Op::PushScope {
                input_not_null: true,
            } if !uses_scope_ops => {
                current_op.set(Op::Pop);
            }
            _ => {}
        }
    }

    // Now actually run the main peephole optimizer.
    let mut last_op: Option<&'a Cell<Op<'_>>> = None;

    for (i, current_op) in ops.iter().enumerate() {
        if jump_targets.contains(&i) {
            // If this op was a jump target, we don't know what the last op was
            last_op = None;
        }

        // NOTE: If a peephole optimization changes the stack from being empty
        // to being non-empty at a certain position, it MUST make sure to
        // invalidate that position in `empty_stack_positions`.

        if let Some(last_op) = last_op {
            // Optimizations on both the current and the last op
            match (last_op.get(), current_op.get()) {
                (push_op, Op::Pop) if push_op.is_pure_push() => {
                    // Eliminate PushXXX+Pop and GetLocal+Pop
                    last_op.set(Op::Nop);
                    current_op.set(Op::Nop);
                }
                (push_op, Op::PopJump { offset }) if push_op.is_pure_push() => {
                    // PushXXX+PopJump becomes Nop+Jump
                    last_op.set(Op::Nop);
                    current_op.set(Op::Jump { offset });
                }
                (Op::CoerceB, Op::IfTrue { .. } | Op::IfFalse { .. } | Op::Not) => {
                    // Remove CoerceB before IfTrue, IfFalse, and Not
                    last_op.set(Op::Nop);
                }
                (Op::Dup, Op::SetLocal { index }) => {
                    // Dup+SetLocal becomes Nop+StoreLocal
                    last_op.set(Op::Nop);
                    current_op.set(Op::StoreLocal { index });
                }
                (Op::SetLocal { index: index1 }, Op::GetLocal { index: index2 })
                    if index1 == index2 =>
                {
                    // SetLocal+GetLocal becomes Nop+StoreLocal
                    last_op.set(Op::Nop);
                    current_op.set(Op::StoreLocal { index: index1 });

                    // It's possible that before this peephole optimization, the
                    // stack was empty at the `GetLocal`'s position. However,
                    // after this optimization, it is guaranteed that the stack
                    // is no longer empty at the `GetLocal`'s position, as the
                    // `StoreLocal` keeps one entry on the stack.
                    int_analysis_info.remove_empty_stack_position(i);
                }
                (
                    Op::Add {
                        inputs_integral: true,
                    },
                    Op::CoerceI,
                ) => {
                    // An integral addition that yields Number on overflow
                    // followed by coerce-to-integer is equivalent to wrapping
                    // integral addition
                    last_op.set(Op::AddI);
                    current_op.set(Op::Nop);
                }
                (
                    Op::Subtract {
                        inputs_integral: true,
                    },
                    Op::CoerceI,
                ) => {
                    // The same is true for subtraction
                    last_op.set(Op::SubtractI);
                    current_op.set(Op::Nop);
                }
                (
                    Op::Add {
                        inputs_integral: true,
                    },
                    Op::Li8 | Op::Li16 | Op::Li32 | Op::Si8 | Op::Si16 | Op::Si32,
                ) => {
                    // See comments above
                    last_op.set(Op::AddI);
                }
                (
                    Op::Subtract {
                        inputs_integral: true,
                    },
                    Op::Li8 | Op::Li16 | Op::Li32 | Op::Si8 | Op::Si16 | Op::Si32,
                ) => {
                    // The same is true for subtraction
                    last_op.set(Op::SubtractI);
                }
                (
                    Op::Add {
                        inputs_integral: true,
                    },
                    Op::SetSlotCoerceI { index },
                ) => {
                    // See comments above
                    last_op.set(Op::AddI);
                    current_op.set(Op::SetSlotNoCoerce { index });
                }
                (
                    Op::Subtract {
                        inputs_integral: true,
                    },
                    Op::SetSlotCoerceI { index },
                ) => {
                    // The same is true for subtraction
                    last_op.set(Op::SubtractI);
                    current_op.set(Op::SetSlotNoCoerce { index });
                }
                _ => {}
            }
        }

        // Don't set last_op to the current_op if the current op does nothing.
        // This allows us to peephole-optimize sequences such as
        // `getlocal0`-`nop`-`pop`, as when the `pop` op is being processed,
        // `last_op` will still be set to the `getlocal0`.
        if !current_op.get().is_nop() {
            last_op = Some(current_op);
        }
    }
}
