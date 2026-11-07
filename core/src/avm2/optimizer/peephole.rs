use crate::avm2::op::Op;

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
/// called on the entire code slice.
///
/// This pass removes `pushscope` ops if the method never reads from the scope
/// stack.
pub fn push_scope_removal_peephole<'gc>(ops: &[Cell<Op<'gc>>]) {
    // Determine if this method ever depends on the state of the scope stack.
    for op in ops {
        match op.get() {
            Op::GetScopeObject { .. }
            | Op::SetGlobalSlot { .. }
            | Op::FindProperty { .. }
            | Op::FindPropStrict { .. }
            | Op::NewFunction { .. }
            | Op::NewClass { .. }
            | Op::PopScope => {
                // If it does, we can't perform this optimization.
                return;
            }

            _ => {}
        }
    }

    // If the scope stack is never read from, only written to, we can replace
    // `PushScope` ops with `Pop`.
    for current_op in ops {
        if matches!(
            current_op.get(),
            Op::PushScope {
                input_not_null: true
            }
        ) {
            current_op.set(Op::Pop);
        }
    }
}

/// A peephole optimizer to run after type-aware optimizations, and after the
/// scope removal peephole pass. This should be called on the entire code slice.
///
/// This pass removes redundant push+pop pairs, converts arithmetic ops into
/// their integral versions if possible, and performs other miscellaneous
/// optimizations.
pub fn postprocess_peephole<'gc>(ops: &[Cell<Op<'gc>>], jump_targets: &HashSet<usize>) {
    let mut last_op: Option<&Cell<Op<'gc>>> = None;

    for (i, current_op) in ops.iter().enumerate() {
        if jump_targets.contains(&i) {
            // If this op was a jump target, we don't know what the last op was
            last_op = None;
        }

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
