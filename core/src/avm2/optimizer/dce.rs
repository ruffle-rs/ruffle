use crate::avm2::op::Op;

use std::cell::Cell;
use std::collections::HashSet;

pub fn eliminate_dead_code(ops: &[Cell<Op<'_>>], jump_targets: &HashSet<usize>) {
    // Remove jumps and the code that they jump over when the code inside the
    // jump is unreachable
    for (i, op) in ops.iter().enumerate() {
        match op.get() {
            Op::Jump { offset } => {
                let is_reachable = (i..offset).any(|pos| jump_targets.contains(&pos));

                if !is_reachable && i < offset {
                    for op in &ops[i..offset] {
                        // Set all unreachable ops to Nop, including the Jump op
                        op.set(Op::Nop);
                    }
                }
            }
            Op::PopJump { offset } => {
                // PopJump works exactly like `Jump`, but it sets the jumping op
                // (`PopJump` in this case) to `Pop` instead of `Nop`.
                let is_reachable = (i..offset).any(|pos| jump_targets.contains(&pos));

                if !is_reachable && i < offset {
                    for op in &ops[i..offset] {
                        // Set all unreachable ops to Nop, including the Jump op
                        op.set(Op::Nop);
                    }
                    ops[i].set(Op::Pop);
                }
            }
            _ => {}
        }
    }
}
