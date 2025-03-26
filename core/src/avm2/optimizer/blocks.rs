use crate::avm2::op::Op;
use crate::avm2::verify::Exception;

use smallvec::SmallVec;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct BasicBlock<'a, 'gc> {
    // The ops making up this block.
    pub ops: &'a [Cell<Op<'gc>>],

    // The index of the first op making up this BasicBlock.
    pub start_index: usize,
}

pub fn assemble_blocks<'a, 'gc>(
    code: &'a [Cell<Op<'gc>>],
    method_exceptions: &[Exception<'gc>],
    jump_targets: &HashSet<i32>,
) -> (Vec<BasicBlock<'a, 'gc>>, HashMap<usize, usize>) {
    let mut block_list = Vec::with_capacity(2);
    let mut current_block_start = 0;

    for (i, op) in code.iter().enumerate() {
        let op = op.get();
        match op {
            Op::IfFalse { .. } | Op::IfTrue { .. } => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            Op::Jump { .. } => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            Op::LookupSwitch(_) => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            Op::ReturnVoid | Op::ReturnValue { .. } | Op::Throw => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            _ => {
                // The next op is a jump target
                if jump_targets.contains(&(i as i32 + 1))
                {
                    let block = BasicBlock {
                        start_index: current_block_start,
                        ops: &code[current_block_start..i + 1],
                    };

                    block_list.push(block);

                    current_block_start = i + 1;
                }
            }
        }
    }

    // Create a table mapping op indices to block indicies.
    let mut op_index_to_block_index_table = HashMap::new();
    for (i, block) in block_list.iter().enumerate() {
        op_index_to_block_index_table.insert(block.start_index, i);
    }

    (block_list, op_index_to_block_index_table)
}
