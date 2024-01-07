use crate::avm2::error::{make_error_1025, make_error_1054, make_error_1107, verify_error};
use crate::avm2::method::BytecodeMethod;
use crate::avm2::op::Op;
use crate::avm2::property::Property;
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Activation, Error};
use gc_arena::GcCell;
use std::collections::{HashMap, HashSet};
use swf::avm2::read::Reader;
use swf::avm2::types::{Index, MethodFlags as AbcMethodFlags, Multiname, Op as AbcOp};
use swf::error::Error as AbcReadError;

pub struct VerifiedMethodInfo {
    pub parsed_code: Vec<Op>,
    pub exceptions: Vec<Exception>,
}

pub struct Exception {
    pub from_offset: u32,
    pub to_offset: u32,
    pub target_offset: u32,

    pub variable_name: Index<Multiname>,
    pub type_name: Index<Multiname>,
}

pub fn verify_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
) -> Result<VerifiedMethodInfo, Error<'gc>> {
    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");
    let translation_unit = method.translation_unit();

    let param_count = method.method().params.len();
    let locals_count = body.num_locals;

    // Ensure there are enough local variables
    // to fit the parameters in.
    if (locals_count as usize) < param_count + 1 {
        return Err(make_error_1107(activation));
    }

    let mut new_code = Vec::new();

    let mut byte_offset_to_idx = HashMap::new();
    let mut idx_to_byte_offset = vec![0];
    byte_offset_to_idx.insert(0, 0);

    use swf::extensions::ReadSwfExt;

    if body.code.is_empty() {
        return Err(Error::AvmError(verify_error(
            activation,
            "Error #1043: Invalid code_length=0.",
            1043,
        )?));
    }

    // FIXME: This is wrong, verification/control flow handling should happen at the same
    // time as reading. A side effect of this is that avmplus allows for holes in bytecode,
    // while this implementation throws errors #1011 or #1021 in those cases.
    let mut reader = Reader::new(&body.code);
    loop {
        let op = match reader.read_op() {
            Ok(op) => op,

            Err(AbcReadError::InvalidData(_)) => {
                return Err(Error::AvmError(verify_error(
                    activation,
                    "Error #1011: Method contained illegal opcode.",
                    1011,
                )?));
            }
            Err(AbcReadError::IoError(_)) => break,
            Err(unknown) => {
                tracing::error!(
                    "Encountered unexpected error while verifying AVM2 method body: {unknown:?}"
                );
                break;
            }
        };
        let byte_offset = reader.as_slice().as_ptr() as usize - body.code.as_ptr() as usize;

        new_code.push(op);

        byte_offset_to_idx.insert(byte_offset as i32, new_code.len() as i32);
        idx_to_byte_offset.push(byte_offset as i32);
    }

    // Avoid verifying the same blocks again.
    let mut verified_blocks = Vec::new();

    verify_code_starting_from(
        activation,
        method,
        idx_to_byte_offset.as_slice(),
        &byte_offset_to_idx,
        &mut verified_blocks,
        new_code.as_slice(),
        0,
    )?;

    // Record a list of possible places the code could
    // jump to- this will be used for optimization.
    let mut potential_jump_targets = HashSet::new();

    // Handle exceptions
    let mut new_exceptions = Vec::new();
    for exception in body.exceptions.iter() {
        // FIXME: This is actually wrong, we should be using the byte offsets, not the opcode offsets.
        // avmplus allows for from/to (but not targets) that aren't on a opcode, and some obfuscated
        // SWFs have them. FFDEC handles them correctly, stepping forward byte-by-byte until it
        // reaches the next opcode. This does the same (stepping byte-by-byte), but it would be better
        // to directly use the byte offsets when handling exceptions.
        let mut from_offset = byte_offset_to_idx
            .get(&(exception.from_offset as i32))
            .copied();

        let mut offs = 0;
        while from_offset.is_none() {
            from_offset = byte_offset_to_idx
                .get(&((exception.from_offset + offs) as i32))
                .copied();
            offs += 1;
            if offs as usize >= new_code.len() {
                return Err(make_error_1054(activation));
            }
        }

        // Now for the `to_offset`.
        let mut to_offset = byte_offset_to_idx
            .get(&(exception.to_offset as i32))
            .copied();

        let mut offs = 0;
        while from_offset.is_none() {
            to_offset = byte_offset_to_idx
                .get(&((exception.to_offset + offs) as i32))
                .copied();
            if offs == 0 {
                return Err(make_error_1054(activation));
            }
            offs -= 1;
        }

        if to_offset.unwrap() < from_offset.unwrap() {
            return Err(make_error_1054(activation));
        }

        let new_from_offset = from_offset.unwrap() as u32;
        let new_to_offset = to_offset.unwrap() as u32;

        // FIXME: Use correct error instead of `.unwrap()`
        let new_target_offset = byte_offset_to_idx
            .get(&(exception.target_offset as i32))
            .copied()
            .unwrap();

        if exception.target_offset < exception.to_offset {
            return Err(make_error_1054(activation));
        }

        new_exceptions.push(Exception {
            from_offset: new_from_offset,
            to_offset: new_to_offset,
            target_offset: new_target_offset as u32,
            variable_name: exception.variable_name,
            type_name: exception.type_name,
        });

        if ops_can_throw_error(new_code.as_slice(), new_from_offset, new_to_offset) {
            verify_code_starting_from(
                activation,
                method,
                idx_to_byte_offset.as_slice(),
                &byte_offset_to_idx,
                &mut verified_blocks,
                new_code.as_slice(),
                new_target_offset,
            )?;

            potential_jump_targets.insert(new_target_offset);
        }
    }

    // Adjust jump offsets from byte-based to idx-based
    for (i, op) in new_code.iter_mut().enumerate() {
        let i = i as i32;
        let adjusted = |i, offset, one_off| {
            let byte_offset = if one_off {
                idx_to_byte_offset.get((i + 1) as usize).unwrap()
            } else {
                idx_to_byte_offset.get(i as usize).unwrap()
            };
            let new_byte_offset = byte_offset + offset;
            let new_idx = byte_offset_to_idx
                .get(&new_byte_offset)
                .copied()
                .unwrap_or(0);
            // Verification should have confirmed that this `unwrap_or` is from an unreachable instruction;
            // if it were reachable, then verification would have thrown a VerifyError

            (new_idx, new_idx - i - 1)
        };
        match op {
            AbcOp::IfEq { offset }
            | AbcOp::IfFalse { offset }
            | AbcOp::IfGe { offset }
            | AbcOp::IfGt { offset }
            | AbcOp::IfLe { offset }
            | AbcOp::IfLt { offset }
            | AbcOp::IfNe { offset }
            | AbcOp::IfNge { offset }
            | AbcOp::IfNgt { offset }
            | AbcOp::IfNle { offset }
            | AbcOp::IfNlt { offset }
            | AbcOp::IfStrictEq { offset }
            | AbcOp::IfStrictNe { offset }
            | AbcOp::IfTrue { offset }
            | AbcOp::Jump { offset } => {
                let adjusted_result = adjusted(i, *offset, true);
                *offset = adjusted_result.1;
                potential_jump_targets.insert(adjusted_result.0);
            }
            AbcOp::LookupSwitch(ref mut lookup_switch) => {
                let adjusted_default = adjusted(i, lookup_switch.default_offset, false);
                lookup_switch.default_offset = adjusted_default.1;
                potential_jump_targets.insert(adjusted_default.0);

                for case in lookup_switch.case_offsets.iter_mut() {
                    let adjusted_case = adjusted(i, *case, false);
                    *case = adjusted_case.1;
                    potential_jump_targets.insert(adjusted_case.0);
                }
            }
            _ => {}
        }
    }

    let mut verified_code = Vec::new();
    for abc_op in new_code {
        let resolved_op = resolve_op(activation, translation_unit, abc_op)?;

        verified_code.push(resolved_op);
    }

    optimize(
        activation,
        method,
        &mut verified_code,
        potential_jump_targets,
    );

    Ok(VerifiedMethodInfo {
        parsed_code: verified_code,
        exceptions: new_exceptions,
    })
}

fn adjust_jump_offset<'gc>(
    activation: &mut Activation<'_, 'gc>,
    i: i32,
    offset: i32,
    one_off: bool,
    idx_to_byte_offset: &[i32],
    byte_offset_to_idx: &HashMap<i32, i32>,
) -> Result<i32, Error<'gc>> {
    let byte_offset = if one_off {
        idx_to_byte_offset.get((i + 1) as usize).unwrap()
    } else {
        idx_to_byte_offset.get(i as usize).unwrap()
    };
    let new_byte_offset = byte_offset + offset;
    let new_idx = byte_offset_to_idx
        .get(&new_byte_offset)
        .copied()
        .ok_or_else(|| {
            Error::AvmError(verify_error(
                activation,
                "Error #1021: At least one branch target was not on a valid instruction in the method.",
                1021,
            ).expect("Error should construct"))
        })?;

    Ok(new_idx - 1)
}

fn verify_code_starting_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
    idx_to_byte_offset: &[i32],
    byte_offset_to_idx: &HashMap<i32, i32>,
    verified_blocks: &mut Vec<i32>,
    ops: &[AbcOp],
    start_idx: i32,
) -> Result<(), Error<'gc>> {
    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");
    let max_locals = body.num_locals;

    let mut worklist = Vec::new();
    worklist.push(start_idx);

    while !worklist.is_empty() {
        let mut save_current_work = false;
        let mut i = *worklist.last().unwrap();
        loop {
            if (i as usize) >= ops.len() {
                return Err(Error::AvmError(verify_error(
                    activation,
                    "Error #1020: Code cannot fall off the end of a method.",
                    1020,
                )?));
            }

            let op = &ops[i as usize];

            // Special control flow ops
            match op {
                AbcOp::IfEq { offset }
                | AbcOp::IfFalse { offset }
                | AbcOp::IfGe { offset }
                | AbcOp::IfGt { offset }
                | AbcOp::IfLe { offset }
                | AbcOp::IfLt { offset }
                | AbcOp::IfNe { offset }
                | AbcOp::IfNge { offset }
                | AbcOp::IfNgt { offset }
                | AbcOp::IfNle { offset }
                | AbcOp::IfNlt { offset }
                | AbcOp::IfStrictEq { offset }
                | AbcOp::IfStrictNe { offset }
                | AbcOp::IfTrue { offset }
                | AbcOp::Jump { offset } => {
                    let op_idx = adjust_jump_offset(
                        activation,
                        i,
                        *offset,
                        true,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                    )?;

                    if op_idx != i {
                        // Replace last entry on worklist with current i (update i)
                        worklist.pop();
                        worklist.push(i + 1);

                        if !verified_blocks.iter().any(|o| *o == op_idx + 1) {
                            if matches!(op, AbcOp::Jump { .. }) {
                                // If this is a Jump, there's no path that just ignores
                                // the jump, since it's an unconditional jump.
                                worklist.pop();
                            }
                            worklist.push(op_idx + 1);

                            verified_blocks.push(op_idx + 1);

                            save_current_work = true;
                            break;
                        } else if matches!(op, AbcOp::Jump { .. }) {
                            // The target of the jump has already been verified, and
                            // we need to avoid the ops that the Jump will jump over.
                            break;
                        }
                    }
                }

                // Terminal opcodes
                AbcOp::Throw | AbcOp::ReturnValue | AbcOp::ReturnVoid => {
                    break;
                }

                AbcOp::LookupSwitch(ref lookup_switch) => {
                    // A LookupSwitch is terminal

                    let default_idx = adjust_jump_offset(
                        activation,
                        i,
                        lookup_switch.default_offset,
                        false,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                    )?;

                    if !verified_blocks.iter().any(|o| *o == default_idx) {
                        verified_blocks.push(default_idx);

                        worklist.push(default_idx);
                    }

                    for case in lookup_switch.case_offsets.iter() {
                        let case_idx = adjust_jump_offset(
                            activation,
                            i,
                            *case,
                            false,
                            idx_to_byte_offset,
                            byte_offset_to_idx,
                        )?;

                        if !verified_blocks.iter().any(|o| *o == case_idx) {
                            verified_blocks.push(case_idx);

                            worklist.push(case_idx);
                        }
                    }

                    // A LookupSwitch is terminal
                    break;
                }

                // Verifications

                // Local register verifications
                AbcOp::GetLocal { index }
                | AbcOp::SetLocal { index }
                | AbcOp::Kill { index }
                | AbcOp::DecLocal { index }
                | AbcOp::DecLocalI { index }
                | AbcOp::IncLocal { index }
                | AbcOp::IncLocalI { index } => {
                    if *index >= max_locals {
                        return Err(make_error_1025(activation, *index));
                    }
                }

                AbcOp::HasNext2 {
                    object_register,
                    index_register,
                } => {
                    // NOTE: This is the correct order (first check object register, then check index register)
                    if *object_register >= max_locals {
                        return Err(make_error_1025(activation, *object_register));
                    } else if *index_register >= max_locals {
                        return Err(make_error_1025(activation, *index_register));
                    }
                }

                // Misc opcode verification
                AbcOp::CallMethod { index, .. } => {
                    return Err(Error::AvmError(if index.as_u30() == 0 {
                        verify_error(activation, "Error #1072: Disp_id 0 is illegal.", 1072)?
                    } else {
                        verify_error(
                            activation,
                            "Error #1051: Illegal early binding access.",
                            1051,
                        )?
                    }));
                }

                AbcOp::NewActivation => {
                    if !method
                        .method()
                        .flags
                        .contains(AbcMethodFlags::NEED_ACTIVATION)
                    {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1113: OP_newactivation used in method without NEED_ACTIVATION flag.",
                            1113,
                        )?));
                    }
                }

                AbcOp::GetLex { index } => {
                    let multiname = method
                        .translation_unit()
                        .pool_maybe_uninitialized_multiname(*index, &mut activation.context)?;

                    if multiname.has_lazy_component() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1078: Illegal opcode/multiname combination.",
                            1078,
                        )?));
                    }
                }

                AbcOp::AsType {
                    type_name: name_index,
                }
                | AbcOp::Coerce { index: name_index } => {
                    let multiname = method
                        .translation_unit()
                        .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context)
                        .unwrap();

                    activation
                        .domain()
                        .get_class(&multiname, activation.context.gc_context)
                        .ok_or_else(|| {
                            Error::AvmError(
                                verify_error(
                                    activation,
                                    &format!(
                                        "Error #1014: Class {} could not be found.",
                                        multiname.to_qualified_name(activation.context.gc_context)
                                    ),
                                    1014,
                                )
                                .expect("Error should construct"),
                            )
                        })?;
                }

                _ => {}
            }

            i += 1;
        }

        if !save_current_work {
            worklist.pop();
        }
    }

    Ok(())
}

fn optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
    code: &mut Vec<Op>,
    jump_targets: HashSet<i32>,
) {
    // These make the code less readable
    #![allow(clippy::manual_filter)]
    #![allow(clippy::single_match)]

    // This can probably be done better by recording the receiver in `Activation`,
    // but this works since it's guaranteed to be set in `Activation::from_method`.
    let this_value = activation.local_register(0);

    let this_class = if let Some(this_class) = activation.subclass_object() {
        if this_value.is_of_type(activation, this_class.inner_class_definition()) {
            Some(this_class)
        } else {
            None
        }
    } else {
        None
    };

    // Initial set of local types
    let mut local_types = vec![None; method.body().unwrap().num_locals as usize];
    local_types[0] = this_class;

    // Invalidate local types if they should be invalidated
    for op in &*code {
        match op {
            Op::SetLocal { index }
            | Op::Kill { index }
            | Op::IncLocal { index }
            | Op::IncLocalI { index }
            | Op::DecLocal { index }
            | Op::DecLocalI { index } => {
                if (*index as usize) < local_types.len() {
                    local_types[*index as usize] = None;
                }
            }
            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                if (*object_register as usize) < local_types.len() {
                    local_types[*object_register as usize] = None;
                }
                if (*index_register as usize) < local_types.len() {
                    local_types[*index_register as usize] = None;
                }
            }
            _ => {}
        }
    }

    let mut previous_op = None;
    for (i, op) in code.iter_mut().enumerate() {
        if let Some(previous_op_some) = previous_op {
            if !jump_targets.contains(&(i as i32)) {
                match op {
                    Op::CoerceB => match previous_op_some {
                        Op::CoerceB
                        | Op::Equals
                        | Op::GreaterEquals
                        | Op::GreaterThan
                        | Op::LessEquals
                        | Op::LessThan
                        | Op::PushTrue
                        | Op::PushFalse
                        | Op::Not
                        | Op::IsType { .. }
                        | Op::IsTypeLate => {
                            previous_op = Some(op.clone());
                            *op = Op::Nop;
                            continue;
                        }
                        _ => {}
                    },
                    Op::CoerceD => match previous_op_some {
                        Op::CoerceD
                        | Op::PushDouble { .. }
                        | Op::Multiply
                        | Op::Divide
                        | Op::Modulo
                        | Op::Increment
                        | Op::IncLocal { .. }
                        | Op::Decrement
                        | Op::DecLocal { .. }
                        | Op::Negate => {
                            previous_op = Some(op.clone());
                            *op = Op::Nop;
                            continue;
                        }
                        _ => {}
                    },
                    Op::CoerceI => match previous_op_some {
                        Op::CoerceI | Op::PushByte { .. } | Op::PushShort { .. } => {
                            previous_op = Some(op.clone());
                            *op = Op::Nop;
                            continue;
                        }
                        Op::PushInt { value } => {
                            if value >= -(1 << 28) && value < (1 << 28) {
                                previous_op = Some(op.clone());
                                *op = Op::Nop;
                                continue;
                            }
                        }
                        _ => {}
                    },
                    Op::CoerceU => match previous_op_some {
                        Op::CoerceU => {
                            previous_op = Some(op.clone());
                            *op = Op::Nop;
                            continue;
                        }
                        Op::PushByte { value } => {
                            if (value as i8) >= 0 {
                                previous_op = Some(op.clone());
                                *op = Op::Nop;
                                continue;
                            }
                        }
                        Op::PushShort { value } => {
                            if value >= 0 {
                                previous_op = Some(op.clone());
                                *op = Op::Nop;
                                continue;
                            }
                        }
                        Op::PushInt { value } => {
                            if value >= 0 && value < (1 << 28) {
                                previous_op = Some(op.clone());
                                *op = Op::Nop;
                                continue;
                            }
                        }
                        _ => {}
                    },
                    Op::GetProperty { index: name_index } => match previous_op_some {
                        Op::GetLocal { index: local_index } => {
                            let class = local_types[local_index as usize];
                            if let Some(class) = class {
                                let multiname = method
                                    .translation_unit()
                                    .pool_maybe_uninitialized_multiname(
                                        *name_index,
                                        &mut activation.context,
                                    )
                                    .unwrap();

                                if !multiname.has_lazy_component() {
                                    match class.instance_vtable().get_trait(&multiname) {
                                        Some(Property::Slot { slot_id })
                                        | Some(Property::ConstSlot { slot_id }) => {
                                            previous_op = Some(op.clone());
                                            *op = Op::GetSlot { index: slot_id };
                                            continue;
                                        }
                                        Some(Property::Virtual { get: Some(get), .. }) => {
                                            previous_op = Some(op.clone());
                                            *op = Op::CallMethod {
                                                num_args: 0,
                                                index: Index::new(get),
                                            };
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Op::AsType {
                        type_name: name_index,
                    } => {
                        let multiname = method
                            .translation_unit()
                            .pool_maybe_uninitialized_multiname(
                                *name_index,
                                &mut activation.context,
                            )
                            .unwrap();

                        let resolved_type = activation
                            .domain()
                            .get_class(&multiname, activation.context.gc_context);

                        if resolved_type.is_some() {
                            match previous_op_some {
                                Op::PushNull => {
                                    previous_op = Some(op.clone());
                                    *op = Op::Nop;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    Op::Coerce { index: name_index } => {
                        let multiname = method
                            .translation_unit()
                            .pool_maybe_uninitialized_multiname(
                                *name_index,
                                &mut activation.context,
                            )
                            .unwrap();

                        let resolved_type = activation
                            .domain()
                            .get_class(&multiname, activation.context.gc_context);

                        if let Some(class) = resolved_type {
                            match previous_op_some {
                                Op::PushNull => {
                                    if !GcCell::ptr_eq(
                                        class,
                                        activation.avm2().classes().int.inner_class_definition(),
                                    ) && !GcCell::ptr_eq(
                                        class,
                                        activation.avm2().classes().uint.inner_class_definition(),
                                    ) && !GcCell::ptr_eq(
                                        class,
                                        activation.avm2().classes().number.inner_class_definition(),
                                    ) && !GcCell::ptr_eq(
                                        class,
                                        activation
                                            .avm2()
                                            .classes()
                                            .boolean
                                            .inner_class_definition(),
                                    ) && !GcCell::ptr_eq(
                                        class,
                                        activation.avm2().classes().void.inner_class_definition(),
                                    ) && !GcCell::ptr_eq(
                                        class,
                                        activation.avm2().classes().string.inner_class_definition(),
                                    ) {
                                        previous_op = Some(op.clone());
                                        *op = Op::Nop;
                                        continue;
                                    }
                                }
                                Op::Coerce {
                                    index: previous_name_index,
                                } => {
                                    if name_index.as_u30() == previous_name_index.as_u30() {
                                        previous_op = Some(op.clone());
                                        *op = Op::Nop;
                                        continue;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        previous_op = Some(op.clone());
    }
}

fn ops_can_throw_error(ops: &[AbcOp], start_idx: u32, end_idx: u32) -> bool {
    for i in start_idx..end_idx {
        let op = &ops[i as usize];
        match op {
            AbcOp::PushByte { .. }
            | AbcOp::PushDouble { .. }
            | AbcOp::PushFalse
            | AbcOp::PushInt { .. }
            | AbcOp::PushNamespace { .. }
            | AbcOp::PushNaN
            | AbcOp::PushNull
            | AbcOp::PushShort { .. }
            | AbcOp::PushString { .. }
            | AbcOp::PushTrue
            | AbcOp::PushUint { .. }
            | AbcOp::PushUndefined
            | AbcOp::Dup
            | AbcOp::Pop
            | AbcOp::GetLocal { .. }
            | AbcOp::SetLocal { .. }
            | AbcOp::Kill { .. }
            | AbcOp::Nop
            | AbcOp::Not
            | AbcOp::PopScope
            | AbcOp::ReturnVoid => {}
            _ => return true,
        }
    }

    false
}

fn pool_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<i32>,
) -> Result<i32, Error<'gc>> {
    if index.0 == 0 {
        return Err(Error::AvmError(
            verify_error(
                activation,
                "Error #1032: Cpool index 0 is out of range.",
                1032,
            )
            .expect("Error should construct"),
        ));
    }

    translation_unit
        .abc()
        .constant_pool
        .ints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| {
            Error::AvmError(
                verify_error(
                    activation,
                    &format!("Error #1032: Cpool index {} is out of range.", index.0),
                    1032,
                )
                .expect("Error should construct"),
            )
        })
}

fn pool_uint<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<u32>,
) -> Result<u32, Error<'gc>> {
    if index.0 == 0 {
        return Err(Error::AvmError(
            verify_error(
                activation,
                "Error #1032: Cpool index 0 is out of range.",
                1032,
            )
            .expect("Error should construct"),
        ));
    }

    translation_unit
        .abc()
        .constant_pool
        .uints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| {
            Error::AvmError(
                verify_error(
                    activation,
                    &format!("Error #1032: Cpool index {} is out of range.", index.0),
                    1032,
                )
                .expect("Error should construct"),
            )
        })
}

fn pool_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<f64>,
) -> Result<f64, Error<'gc>> {
    if index.0 == 0 {
        return Err(Error::AvmError(
            verify_error(
                activation,
                "Error #1032: Cpool index 0 is out of range.",
                1032,
            )
            .expect("Error should construct"),
        ));
    }

    translation_unit
        .abc()
        .constant_pool
        .doubles
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| {
            Error::AvmError(
                verify_error(
                    activation,
                    &format!("Error #1032: Cpool index {} is out of range.", index.0),
                    1032,
                )
                .expect("Error should construct"),
            )
        })
}

fn resolve_op<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    op: AbcOp,
) -> Result<Op, Error<'gc>> {
    Ok(match op {
        AbcOp::PushByte { value } => Op::PushByte { value },
        AbcOp::PushDouble { value } => {
            let value = pool_double(activation, translation_unit, value)?;

            Op::PushDouble { value }
        }
        AbcOp::PushFalse => Op::PushFalse,
        AbcOp::PushInt { value } => {
            let value = pool_int(activation, translation_unit, value)?;

            Op::PushInt { value }
        }
        AbcOp::PushNamespace { value } => Op::PushNamespace { value },
        AbcOp::PushNaN => Op::PushNaN,
        AbcOp::PushNull => Op::PushNull,
        AbcOp::PushShort { value } => Op::PushShort { value },
        AbcOp::PushString { value } => Op::PushString { value },
        AbcOp::PushTrue => Op::PushTrue,
        AbcOp::PushUint { value } => {
            let value = pool_uint(activation, translation_unit, value)?;

            Op::PushUint { value }
        }
        AbcOp::PushUndefined => Op::PushUndefined,
        AbcOp::Pop => Op::Pop,
        AbcOp::Dup => Op::Dup,
        AbcOp::GetLocal { index } => Op::GetLocal { index },
        AbcOp::SetLocal { index } => Op::SetLocal { index },
        AbcOp::Kill { index } => Op::Kill { index },
        AbcOp::Call { num_args } => Op::Call { num_args },
        AbcOp::CallMethod { index, num_args } => Op::CallMethod { index, num_args },
        AbcOp::CallProperty { index, num_args } => Op::CallProperty { index, num_args },
        AbcOp::CallPropLex { index, num_args } => Op::CallPropLex { index, num_args },
        AbcOp::CallPropVoid { index, num_args } => Op::CallPropVoid { index, num_args },
        AbcOp::CallStatic { index, num_args } => Op::CallStatic { index, num_args },
        AbcOp::CallSuper { index, num_args } => Op::CallSuper { index, num_args },
        AbcOp::CallSuperVoid { index, num_args } => Op::CallSuperVoid { index, num_args },
        AbcOp::ReturnValue => Op::ReturnValue,
        AbcOp::ReturnVoid => Op::ReturnVoid,
        AbcOp::GetProperty { index } => Op::GetProperty { index },
        AbcOp::SetProperty { index } => Op::SetProperty { index },
        AbcOp::InitProperty { index } => Op::InitProperty { index },
        AbcOp::DeleteProperty { index } => Op::DeleteProperty { index },
        AbcOp::GetSuper { index } => Op::GetSuper { index },
        AbcOp::SetSuper { index } => Op::SetSuper { index },
        AbcOp::In => Op::In,
        AbcOp::PushScope => Op::PushScope,
        AbcOp::NewCatch { index } => Op::NewCatch { index },
        AbcOp::PushWith => Op::PushWith,
        AbcOp::PopScope => Op::PopScope,
        AbcOp::GetOuterScope { index } => Op::GetOuterScope { index },
        AbcOp::GetScopeObject { index } => Op::GetScopeObject { index },
        AbcOp::GetGlobalScope => Op::GetGlobalScope,
        AbcOp::FindDef { index } => Op::FindDef { index },
        AbcOp::FindProperty { index } => Op::FindProperty { index },
        AbcOp::FindPropStrict { index } => Op::FindPropStrict { index },
        AbcOp::GetLex { index } => Op::GetLex { index },
        AbcOp::GetDescendants { index } => Op::GetDescendants { index },
        AbcOp::GetSlot { index } => Op::GetSlot { index },
        AbcOp::SetSlot { index } => Op::SetSlot { index },
        AbcOp::GetGlobalSlot { index } => Op::GetGlobalSlot { index },
        AbcOp::SetGlobalSlot { index } => Op::SetGlobalSlot { index },
        AbcOp::Construct { num_args } => Op::Construct { num_args },
        AbcOp::ConstructProp { index, num_args } => Op::ConstructProp { index, num_args },
        AbcOp::ConstructSuper { num_args } => Op::ConstructSuper { num_args },
        AbcOp::NewActivation => Op::NewActivation,
        AbcOp::NewObject { num_args } => Op::NewObject { num_args },
        AbcOp::NewFunction { index } => Op::NewFunction { index },
        AbcOp::NewClass { index } => Op::NewClass { index },
        AbcOp::ApplyType { num_types } => Op::ApplyType { num_types },
        AbcOp::NewArray { num_args } => Op::NewArray { num_args },
        AbcOp::CoerceA => Op::CoerceA,
        AbcOp::CoerceO => Op::CoerceO,
        AbcOp::CoerceS => Op::CoerceS,
        AbcOp::CoerceB | AbcOp::ConvertB => Op::CoerceB,
        AbcOp::CoerceD | AbcOp::ConvertD => Op::CoerceD,
        AbcOp::CoerceI | AbcOp::ConvertI => Op::CoerceI,
        AbcOp::CoerceU | AbcOp::ConvertU => Op::CoerceU,
        AbcOp::ConvertO => Op::ConvertO,
        AbcOp::ConvertS => Op::ConvertS,
        AbcOp::Add => Op::Add,
        AbcOp::AddI => Op::AddI,
        AbcOp::BitAnd => Op::BitAnd,
        AbcOp::BitNot => Op::BitNot,
        AbcOp::BitOr => Op::BitOr,
        AbcOp::BitXor => Op::BitXor,
        AbcOp::DecLocal { index } => Op::DecLocal { index },
        AbcOp::DecLocalI { index } => Op::DecLocalI { index },
        AbcOp::Decrement => Op::Decrement,
        AbcOp::DecrementI => Op::DecrementI,
        AbcOp::Divide => Op::Divide,
        AbcOp::IncLocal { index } => Op::IncLocal { index },
        AbcOp::IncLocalI { index } => Op::IncLocalI { index },
        AbcOp::Increment => Op::Increment,
        AbcOp::IncrementI => Op::IncrementI,
        AbcOp::LShift => Op::LShift,
        AbcOp::Modulo => Op::Modulo,
        AbcOp::Multiply => Op::Multiply,
        AbcOp::MultiplyI => Op::MultiplyI,
        AbcOp::Negate => Op::Negate,
        AbcOp::NegateI => Op::NegateI,
        AbcOp::RShift => Op::RShift,
        AbcOp::Subtract => Op::Subtract,
        AbcOp::SubtractI => Op::SubtractI,
        AbcOp::Swap => Op::Swap,
        AbcOp::URShift => Op::URShift,
        AbcOp::Jump { offset } => Op::Jump { offset },
        AbcOp::IfTrue { offset } => Op::IfTrue { offset },
        AbcOp::IfFalse { offset } => Op::IfFalse { offset },
        AbcOp::IfStrictEq { offset } => Op::IfStrictEq { offset },
        AbcOp::IfStrictNe { offset } => Op::IfStrictNe { offset },
        AbcOp::IfEq { offset } => Op::IfEq { offset },
        AbcOp::IfNe { offset } => Op::IfNe { offset },
        AbcOp::IfGe { offset } => Op::IfGe { offset },
        AbcOp::IfGt { offset } => Op::IfGt { offset },
        AbcOp::IfLe { offset } => Op::IfLe { offset },
        AbcOp::IfLt { offset } => Op::IfLt { offset },
        AbcOp::IfNge { offset } => Op::IfNge { offset },
        AbcOp::IfNgt { offset } => Op::IfNgt { offset },
        AbcOp::IfNle { offset } => Op::IfNle { offset },
        AbcOp::IfNlt { offset } => Op::IfNlt { offset },
        AbcOp::StrictEquals => Op::StrictEquals,
        AbcOp::Equals => Op::Equals,
        AbcOp::GreaterEquals => Op::GreaterEquals,
        AbcOp::GreaterThan => Op::GreaterThan,
        AbcOp::LessEquals => Op::LessEquals,
        AbcOp::LessThan => Op::LessThan,
        AbcOp::Nop => Op::Nop,
        AbcOp::Not => Op::Not,
        AbcOp::HasNext => Op::HasNext,
        AbcOp::HasNext2 {
            object_register,
            index_register,
        } => Op::HasNext2 {
            object_register,
            index_register,
        },
        AbcOp::NextName => Op::NextName,
        AbcOp::NextValue => Op::NextValue,
        AbcOp::IsType { index } => Op::IsType { index },
        AbcOp::IsTypeLate => Op::IsTypeLate,
        AbcOp::AsType { type_name } => Op::AsType { type_name },
        AbcOp::AsTypeLate => Op::AsTypeLate,
        AbcOp::InstanceOf => Op::InstanceOf,
        AbcOp::Label => Op::Nop,
        AbcOp::Debug {
            is_local_register,
            register_name,
            register,
        } => Op::Debug {
            is_local_register,
            register_name,
            register,
        },
        AbcOp::DebugFile { file_name } => Op::DebugFile { file_name },
        AbcOp::DebugLine { line_num } => Op::DebugLine { line_num },
        AbcOp::Bkpt => Op::Bkpt,
        AbcOp::BkptLine { line_num } => Op::BkptLine { line_num },
        AbcOp::Timestamp => Op::Timestamp,
        AbcOp::TypeOf => Op::TypeOf,
        AbcOp::EscXAttr => Op::EscXAttr,
        AbcOp::EscXElem => Op::EscXElem,
        AbcOp::Dxns { index } => Op::Dxns { index },
        AbcOp::DxnsLate => Op::DxnsLate,
        AbcOp::LookupSwitch(lookup_switch) => Op::LookupSwitch(lookup_switch),
        AbcOp::Coerce { index } => Op::Coerce { index },
        AbcOp::CheckFilter => Op::CheckFilter,
        AbcOp::Si8 => Op::Si8,
        AbcOp::Si16 => Op::Si16,
        AbcOp::Si32 => Op::Si32,
        AbcOp::Sf32 => Op::Sf32,
        AbcOp::Sf64 => Op::Sf64,
        AbcOp::Li8 => Op::Li8,
        AbcOp::Li16 => Op::Li16,
        AbcOp::Li32 => Op::Li32,
        AbcOp::Lf32 => Op::Lf32,
        AbcOp::Lf64 => Op::Lf64,
        AbcOp::Sxi1 => Op::Sxi1,
        AbcOp::Sxi8 => Op::Sxi8,
        AbcOp::Sxi16 => Op::Sxi16,
        AbcOp::Throw => Op::Throw,
        _ => {
            tracing::error!("Unimplemented AVM2 op {:?} found during verification", op);

            return Err(Error::AvmError(verify_error(
                activation,
                "Error #1011: Method contained illegal opcode.",
                1011,
            )?));
        }
    })
}
