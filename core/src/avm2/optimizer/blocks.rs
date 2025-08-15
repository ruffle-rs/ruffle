use crate::avm2::op::Op;

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

// Represents non-overlapping slices of ops with
// a single entry point and many exit points.
// (as opposed to basic blocks, which have 1 entry and exit point)
#[derive(Debug)]
pub struct Block<'a, 'gc> {
    // The ops making up this block.
    pub ops: &'a [Cell<Op<'gc>>],

    // The index of the first op making up this Block.
    pub start_index: usize,
}

pub fn assemble_blocks<'a, 'gc>(
    code: &'a [Cell<Op<'gc>>],
    jump_targets: &HashSet<usize>,
) -> (Vec<Block<'a, 'gc>>, HashMap<usize, usize>) {
    let mut block_list = Vec::with_capacity(2);
    let mut current_block_start = 0;

    for (i, op) in code.iter().enumerate() {
        let op = op.get();
        if matches!(
            op,
            Op::Jump { .. }
                | Op::ReturnVoid { .. }
                | Op::ReturnValue { .. }
                | Op::Throw
                | Op::LookupSwitch(_)
        ) || jump_targets.contains(&(i + 1))
        // The next op is a jump target
        {
            let block = Block {
                start_index: current_block_start,
                ops: &code[current_block_start..i + 1],
            };

            block_list.push(block);

            current_block_start = i + 1;
        }
    }

    // Create a table mapping op indices to block indices.
    let mut op_index_to_block_index_table = HashMap::new();
    for (i, block) in block_list.iter().enumerate() {
        op_index_to_block_index_table.insert(block.start_index, i);
    }

    (block_list, op_index_to_block_index_table)
}
