use crate::avm2::op::Op;
use crate::avm2::verify::Exception;

use smallvec::SmallVec;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

// The indices of the successors of a block.
#[derive(Clone, Copy, Debug)]
pub enum BlockExit {
    Goto(usize),
    GotoException(usize),
    Return,
}

#[derive(Debug)]
pub struct BasicBlock<'a, 'gc> {
    // The ops making up this block.
    pub ops: &'a [Cell<Op<'gc>>],

    // The index of the first op making up this BasicBlock.
    pub start_index: usize,

    // Successors of this block. NOTE: The order matters because we want our errors
    // to match avmplus
    pub exits: SmallVec<[BlockExit; 2]>,
}

pub fn assemble_blocks<'a, 'gc>(
    code: &'a [Cell<Op<'gc>>],
    method_exceptions: &[Exception<'gc>],
    jump_targets: &HashSet<usize>,
) -> Vec<BasicBlock<'a, 'gc>> {
    let mut block_list = Vec::with_capacity(2);
    let mut current_block_start = 0;

    for (i, op) in code.iter().enumerate() {
        let op = op.get();
        match op {
            Op::IfFalse { offset } | Op::IfTrue { offset } => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                    exits: smallvec![BlockExit::Goto(i + 1), BlockExit::Goto(offset)],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            Op::Jump { offset } => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                    exits: smallvec![BlockExit::Goto(offset)],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            Op::LookupSwitch(lookup_switch) => {
                let mut target_list = SmallVec::new();

                for offset in &lookup_switch.case_offsets {
                    target_list.push(BlockExit::Goto(*offset));
                }

                target_list.push(BlockExit::Goto(lookup_switch.default_offset));

                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                    exits: target_list,
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            Op::ReturnVoid { .. } => {
                let block = BasicBlock {
                    start_index: current_block_start,
                    ops: &code[current_block_start..i + 1],
                    exits: smallvec![BlockExit::Return],
                };

                block_list.push(block);

                current_block_start = i + 1;
            }
            _ => {
                let mut could_exception_branch = false;
                let mut target_list = SmallVec::new();

                // `Throw` and `ReturnValue` will never go to the next op. The
                // possibility of them doing an immediate return is addressed at
                // the end of the `if op.can_throw_error()` block: they are grouped
                // in with all other ops that can throw an error and return.
                if !matches!(op, Op::Throw | Op::ReturnValue { .. }) {
                    target_list.push(BlockExit::Goto(i + 1));
                }

                if op.can_throw_error() {
                    for exception in method_exceptions {
                        if i >= exception.from_offset && i < exception.to_offset {
                            // This op is a branch to the exception target block.
                            could_exception_branch = true;

                            target_list.push(BlockExit::GotoException(exception.target_offset));
                        }
                    }

                    target_list.push(BlockExit::Return);
                }

                // There are several ways other ops can terminate a block:
                // 1. This op could throw an exception, which means it could
                //     immediately stop execution of the method or do a goto to
                //     an exception target
                // 2. This op is Throw or ReturnValue, both of which can throw
                //     an exception (see above) or return immediately
                // 3. The next op is a jump target
                if could_exception_branch
                    || matches!(op, Op::Throw | Op::ReturnValue { .. })
                    || jump_targets.contains(&(i + 1))
                {
                    let block = BasicBlock {
                        start_index: current_block_start,
                        ops: &code[current_block_start..i + 1],
                        exits: target_list,
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

    // Now convert the op indices mentioned in BlockExits to BB indices.
    for block in block_list.iter_mut() {
        for exit in block.exits.iter_mut() {
            match exit {
                BlockExit::Goto(ref mut offset) | BlockExit::GotoException(ref mut offset) => {
                    *offset = *op_index_to_block_index_table
                        .get(offset)
                        .expect("Op index should map to valid block index");
                }
                _ => {}
            }
        }
    }

    block_list
}
