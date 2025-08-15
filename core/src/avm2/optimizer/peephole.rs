use crate::avm2::op::Op;

use std::cell::Cell;
use std::collections::HashSet;

/// A peephole optimizer to run before type-aware optimizations. This should be
/// called once on the entire code slice.
pub fn preprocess_peephole(ops: &[Cell<Op<'_>>]) {
    for (i, op) in ops.iter().enumerate() {
        match op.get() {
            Op::Jump { offset } => {
                if offset == i + 1 {
                    op.set(Op::Nop);
                }
            }
            Op::IfTrue { offset } | Op::IfFalse { offset } => {
                if offset == i + 1 {
                    op.set(Op::Pop);
                }
            }
            _ => {}
        }
    }
}

/// A peephole optimizer to run after type-aware optimizations. This should be
/// called once on the entire code slice.
pub fn postprocess_peephole(
    ops: &[Cell<Op<'_>>],
    jump_targets: &HashSet<usize>,
    has_exceptions: bool,
) {
    // Gather some information...
    let mut sets_local_0 = false;

    for op in ops {
        match op.get() {
            Op::SetLocal { index }
            | Op::Kill { index }
            | Op::DecLocal { index }
            | Op::DecLocalI { index }
            | Op::IncLocal { index }
            | Op::IncLocalI { index } => {
                if index == 0 {
                    sets_local_0 = true;
                    break;
                }
            }
            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                if object_register == 0 || index_register == 0 {
                    sets_local_0 = true;
                    break;
                }
            }

            _ => {}
        }
    }

    let simple_scope_op_positions =
        simple_scope_structure(ops, jump_targets).filter(|_| !has_exceptions);

    // Now actually run the peephole optimizer.
    let mut last_op = None;

    for (i, current_op) in ops.iter().enumerate() {
        if jump_targets.contains(&i) {
            // If this op was a jump target, we don't know what the last op was
            last_op = None;
        }

        match (last_op, last_op.map(Cell::get), current_op.get()) {
            (Some(last_op), Some(Op::GetLocal { .. }), Op::Pop) => {
                // Eliminate GetLocal+Pop
                last_op.set(Op::Nop);
                current_op.set(Op::Nop);
            }
            (
                Some(last_op),
                Some(Op::CoerceB),
                Op::IfTrue { .. } | Op::IfFalse { .. } | Op::Not,
            ) => {
                // Remove CoerceB before IfTrue, IfFalse, and Not
                last_op.set(Op::Nop);
            }
            (_, _, Op::GetScopeObject { index: 0 }) => {
                // Replace `getscopeobject 0` with `getlocal 0` if possible
                if simple_scope_op_positions.is_some() && !sets_local_0 {
                    current_op.set(Op::GetLocal { index: 0 })
                }
            }
            _ => {}
        }

        // Don't set last_op to the current_op if the current op does nothing.
        // This allows us to peephole-optimize sequences such as
        // `getlocal0`-`nop`-`pop`, as when the `pop` op is being processed,
        // `last_op` will still be set to the `getlocal0`.
        if !current_op.get().is_nop() {
            last_op = Some(current_op);
        }
    }

    // Gather some more information...
    let mut uses_scope_ops = false;

    for op in ops {
        match op.get() {
            Op::GetScopeObject { .. }
            | Op::SetGlobalSlot { .. }
            | Op::FindProperty { .. }
            | Op::FindPropStrict { .. }
            | Op::NewFunction { .. }
            | Op::NewClass { .. } => {
                uses_scope_ops = true;
                break;
            }

            _ => {}
        }
    }

    // Eliminate the `getlocal0` and `pushscope` ops at the beginning of the
    // method, if possible.
    if let Some((getlocal0_pos, pushscope_pos)) = simple_scope_op_positions {
        if !uses_scope_ops {
            ops[getlocal0_pos].set(Op::Nop);
            ops[pushscope_pos].set(Op::Nop);
        }
    }
}

/// Checks if the method fits the following pattern:
///
/// ```text
/// [Debug/DebugFile/DebugLine] zero or more times
/// GetLocal { index: 0 }
/// [Debug/DebugFile/DebugLine] zero or more times
/// PushScope
/// ...
/// ```
///
/// along with the following conditions:
/// * No jumps to that initial PushScope opcode, or anything before it
/// * No additional scope-related opcodes (PushScope, PushWith, PopScope)
///
/// If all these conditions are met, the method will return the positions of
/// the GetLocal { 0 } and PushScope ops. Otherwise, it will return None.
fn simple_scope_structure(
    ops: &[Cell<Op<'_>>],
    jump_targets: &HashSet<usize>,
) -> Option<(usize, usize)> {
    let mut getlocal0_pos = None;
    for (i, op) in ops.iter().enumerate() {
        match op.get() {
            // Ignore any initial debug opcodes
            Op::Debug { .. } | Op::DebugFile { .. } | Op::DebugLine { .. } => {}
            // Look for an initial getlocal0
            Op::GetLocal { index: 0 } => {
                getlocal0_pos = Some(i);
                break;
            }
            // Anything else doesn't fit the pattern, so give up
            _ => return None,
        }
    }
    // Give up if we didn't find it
    let getlocal0_pos = getlocal0_pos?;

    let mut pushscope_pos = None;
    for (i, op) in ops.iter().enumerate().skip(getlocal0_pos + 1) {
        match op.get() {
            // Ignore any debug opcodes
            Op::Debug { .. } | Op::DebugFile { .. } | Op::DebugLine { .. } => {}
            // Look for a pushscope
            Op::PushScope => {
                pushscope_pos = Some(i);
                break;
            }
            // Anything else doesn't fit the pattern, so give up
            _ => return None,
        }
    }
    // Give up if we didn't find it
    let pushscope_pos = pushscope_pos?;

    for i in 0..=pushscope_pos {
        if jump_targets.contains(&i) {
            return None;
        }
    }

    for op in &ops[pushscope_pos + 1..] {
        match op.get() {
            Op::PushScope | Op::PushWith | Op::PopScope => {
                return None;
            }
            _ => {}
        }
    }

    Some((getlocal0_pos, pushscope_pos))
}
