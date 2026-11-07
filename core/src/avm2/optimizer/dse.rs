use crate::avm2::activation::Activation;
use crate::avm2::method::Method;
use crate::avm2::op::Op;
use crate::avm2::optimizer::blocks::assemble_blocks;
use crate::avm2::verify::Exception;

use smallvec::SmallVec;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
struct LocalStoresInfo {
    /// The indices of the last op(s) to store to this local register. These
    /// should all be unique.
    contributing_stores: SmallVec<[usize; 2]>,
}

impl LocalStoresInfo {
    pub fn new() -> Self {
        Self {
            contributing_stores: SmallVec::new(),
        }
    }

    pub fn set_store(&mut self, store_index: usize) {
        self.contributing_stores.clear();
        self.contributing_stores.push(store_index);
    }

    pub fn merge_with(&mut self, other: &LocalStoresInfo) -> bool {
        let mut changed = false;

        for other_store_index in &other.contributing_stores {
            // If this store index already existed in this stores info,
            // we don't need to add it again.
            if !self.contributing_stores.contains(other_store_index) {
                self.contributing_stores.push(*other_store_index);
                changed = true;
            }
        }

        changed
    }

    pub fn iter_stores(&self) -> impl Iterator<Item = usize> {
        self.contributing_stores.iter().copied()
    }
}

#[derive(Clone, Debug)]
struct Locals(Vec<LocalStoresInfo>);

impl Locals {
    fn new(size: usize) -> Self {
        Self(vec![LocalStoresInfo::new(); size])
    }

    fn merge_with(&mut self, other: &Locals) -> bool {
        let mut changed = false;

        assert_eq!(self.len(), other.len());

        for i in 0..self.len() {
            let our_local = self.get_mut(i);
            let other_local = other.get(i);

            let this_local_changed = our_local.merge_with(other_local);

            if this_local_changed {
                changed = true;
            }
        }

        changed
    }

    fn get(&self, index: usize) -> &LocalStoresInfo {
        &self.0[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut LocalStoresInfo {
        &mut self.0[index]
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn eliminate_dead_stores<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    code_slice: &[Cell<Op<'gc>>],
    method_exceptions: &[Exception<'gc>],
    jump_targets: &HashSet<usize>,
) {
    if !activation.avm2().optimizer_settings().dse_enabled {
        return;
    }

    if !method_exceptions.is_empty() {
        // If the method has exceptions, control flow is very unpredictable.
        // It's usually not worth it doing analysis then.
        return;
    }

    // NOTE: We assemble the list of blocks once already in the type-aware
    // optimizer, but we do it again here. This is because the type-aware
    // optimizer simplifies branches when possible, which can modify the list
    // of blocks.
    let (block_list, op_index_to_block_index_table) = assemble_blocks(code_slice, jump_targets);

    let method_body = method
        .body()
        .expect("Cannot verify non-native method without body!");

    let entry_state = Locals::new(method_body.num_locals as usize);

    // A map associating the op index of a store to whether that store is known
    // to be live.
    let mut stores_map = HashMap::new();

    // A list of current abstract states, indexed by block index
    let mut abstract_states: Vec<Option<Locals>> = vec![None; block_list.len()];

    abstract_states[0] = Some(entry_state);

    // Block #0 is the entry block
    let mut worklist = vec![0];
    while let Some(block_idx) = worklist.pop() {
        let block = &block_list[block_idx];

        let block_entry_state = abstract_states[block_idx]
            .clone()
            .expect("Entry state not found");

        run_dse(
            block.start_index,
            block.ops,
            block_entry_state,
            &mut stores_map,
            &mut abstract_states,
            &op_index_to_block_index_table,
            &mut worklist,
            false,
        );
    }

    // Now run through the ops, filling out `stores_map`, a HashMap that
    // includes every store and whether that store is live or dead.
    for (i, block) in block_list.iter().enumerate() {
        let block_entry_state = abstract_states[i].clone();

        // The type-aware optimizer may perform optimizations in such a way that
        // some blocks become unreachable. Therefore, there may be blocks that
        // were never visited (`block_entry_state` is None). Ignore such blocks.
        if let Some(block_entry_state) = block_entry_state {
            run_dse(
                block.start_index,
                block.ops,
                block_entry_state,
                &mut stores_map,
                &mut abstract_states,
                &op_index_to_block_index_table,
                &mut worklist,
                true,
            );
        }
    }

    // Finally, use `stores_map` to eliminate stores that are dead.
    for (store_index, is_live) in stores_map {
        if !is_live {
            let op = &code_slice[store_index];

            match op.get() {
                Op::SetLocal { .. } => {
                    op.set(Op::Pop);
                }
                Op::StoreLocal { .. } | Op::Kill { .. } => {
                    op.set(Op::Nop);
                }
                _ => {
                    unreachable!(
                        "Dead store analysis only considers SetLocal, StoreLocal, and Kill"
                    );
                }
            }
        }
    }
}

fn process_jump(
    target: usize,
    abstract_states: &mut [Option<Locals>],
    current_state: &Locals,
    op_index_to_block_index_table: &HashMap<usize, usize>,
    worklist: &mut Vec<usize>,
    do_optimize: bool,
) {
    if do_optimize {
        return;
    }

    let target_block_id = *op_index_to_block_index_table
        .get(&target)
        .expect("unexpected jump target");
    if let Some(target_state) = &mut abstract_states[target_block_id] {
        let state_changed = target_state.merge_with(current_state);
        if !state_changed {
            return;
        }
    } else {
        abstract_states[target_block_id] = Some(current_state.clone());
    };

    if !worklist.contains(&target_block_id) {
        worklist.push(target_block_id);
    }
}

#[expect(clippy::too_many_arguments)]
fn run_dse<'gc>(
    start_index: usize,
    ops: &[Cell<Op<'gc>>],
    initial_state: Locals,
    stores_map: &mut HashMap<usize, bool>,
    abstract_states: &mut [Option<Locals>],
    op_index_to_block_index_table: &HashMap<usize, usize>,
    worklist: &mut Vec<usize>,
    do_optimize: bool,
) {
    let mut locals = initial_state;

    let load_local = |local: &mut LocalStoresInfo, stores_map: &mut HashMap<usize, bool>| {
        // Mark the previous stores to the register as live
        if do_optimize {
            for store_index in local.iter_stores() {
                stores_map.insert(store_index, true);
            }
        }
    };

    let store_local = |local: &mut LocalStoresInfo, i: usize| {
        // Mark this location as the location that the local was last stored to at
        local.set_store(start_index + i);
    };

    for (i, op) in ops.iter().enumerate() {
        match op.get() {
            Op::GetLocal { index } => {
                let local = locals.get_mut(index as usize);

                load_local(local, stores_map);
            }
            Op::SetLocal { index } | Op::StoreLocal { index } | Op::Kill { index } => {
                let local = locals.get_mut(index as usize);
                store_local(local, i);

                if do_optimize {
                    // If there was no store marked at this position, insert
                    // one. We can't do the insertion unconditionally, as that
                    // may cause us to overwrite an existing store that is known
                    // to be live.
                    if stores_map.get(&(start_index + i)).is_none() {
                        stores_map.insert(start_index + i, false);
                    }
                }
            }
            Op::DecLocal { index }
            | Op::DecLocalI { index }
            | Op::IncLocal { index }
            | Op::IncLocalI { index } => {
                // The referenced local register is both loaded from and stored
                // into.
                let local = locals.get_mut(index as usize);
                load_local(local, stores_map);
                store_local(local, i);

                if do_optimize {
                    // This store isn't actually known to be live yet, but these
                    // ops have side effects, so we can never eliminate them.
                    stores_map.insert(start_index + i, true);
                }
            }
            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                // Both registers are both loaded from and stored into.
                let object_local = locals.get_mut(object_register as usize);
                load_local(object_local, stores_map);
                store_local(object_local, i);

                let index_local = locals.get_mut(index_register as usize);
                load_local(index_local, stores_map);
                store_local(index_local, i);

                if do_optimize {
                    // This store isn't actually known to be live yet, but the
                    // hasnext2 op has side effects, so we can never eliminate it.
                    stores_map.insert(start_index + i, true);
                }
            }

            // Control flow ops
            Op::ReturnVoid { .. } | Op::ReturnValue { .. } | Op::Throw => {
                return;
            }
            Op::Jump { offset } | Op::PopJump { offset } => {
                process_jump(
                    offset,
                    abstract_states,
                    &locals,
                    op_index_to_block_index_table,
                    worklist,
                    do_optimize,
                );
                return;
            }
            Op::IfTrue { offset } | Op::IfFalse { offset } => {
                process_jump(
                    offset,
                    abstract_states,
                    &locals,
                    op_index_to_block_index_table,
                    worklist,
                    do_optimize,
                );
            }
            Op::LookupSwitch(lookup_switch) => {
                for target in lookup_switch
                    .case_offsets
                    .iter()
                    .chain(std::slice::from_ref(&lookup_switch.default_offset))
                {
                    process_jump(
                        target.get(),
                        abstract_states,
                        &locals,
                        op_index_to_block_index_table,
                        worklist,
                        do_optimize,
                    );
                }
                return;
            }

            _ => {}
        }
    }

    // if we didn't return earlier, we must jump to the next block.
    let target = start_index + ops.len();
    process_jump(
        target,
        abstract_states,
        &locals,
        op_index_to_block_index_table,
        worklist,
        do_optimize,
    );
}
