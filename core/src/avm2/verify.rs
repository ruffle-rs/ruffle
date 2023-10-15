use crate::avm2::error::{make_error_1025, make_error_1054, make_error_1107, verify_error};
use crate::avm2::method::BytecodeMethod;
use crate::avm2::{Activation, Error};
use std::collections::HashMap;
use swf::avm2::read::Reader;
use swf::avm2::types::{MethodBody as AbcMethodBody, MethodFlags as AbcMethodFlags, Op};
use swf::error::Error as AbcReadError;

pub fn verify_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
) -> Result<AbcMethodBody, Error<'gc>> {
    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");

    let param_count = method.method().params.len();
    let locals_count = body.num_locals;

    // Ensure there are enough local variables
    // to fit the parameters in.
    if (locals_count as usize) < param_count + 1 {
        return Err(make_error_1107(activation));
    }

    let mut new_body = AbcMethodBody {
        method: body.method,
        max_stack: body.max_stack,
        num_locals: locals_count,
        init_scope_depth: body.init_scope_depth,
        max_scope_depth: body.max_scope_depth,
        code: vec![],
        parsed_code: vec![],
        exceptions: body.exceptions.clone(),
        traits: body.traits.clone(),
    };

    let new_code = &mut new_body.parsed_code;

    let mut byte_offset_to_idx = HashMap::new();
    let mut idx_to_byte_offset = vec![0];
    byte_offset_to_idx.insert(0, 0);

    use swf::extensions::ReadSwfExt;

    if body.code.len() == 0 {
        return Err(Error::AvmError(verify_error(
            activation,
            "Error #1043: Invalid code_length=0.",
            1043,
        )?));
    }

    // FIXME: This is wrong, verification should happen at the same time as reading.
    // A side effect of this is that avmplus allows for holes in bytecode.
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

    verify_block(
        activation,
        method,
        idx_to_byte_offset.as_slice(),
        &byte_offset_to_idx,
        &mut verified_blocks,
        new_code.as_slice(),
        0,
        None,
        0,
        true,
    )?;

    // Adjust exception offsets
    for exception in new_body.exceptions.iter_mut() {
        // FIXME: This is actually wrong, we should be using the byte offsets, not the opcode offsets.
        // avmplus allows for from/to (but not targets) that aren't on a opcode, and some obfuscated
        // SWFs have them. FFDEC handles them correctly, stepping forward byte-by-byte until it
        // reaches the next opcode.
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

        exception.from_offset = from_offset.unwrap() as u32;
        exception.to_offset = to_offset.unwrap() as u32;

        // FIXME: Use correct error instead of `.unwrap()`
        exception.target_offset = byte_offset_to_idx
            .get(&(exception.target_offset as i32))
            .copied()
            .unwrap() as u32;

        if exception.target_offset < exception.to_offset {
            return Err(make_error_1054(activation));
        }

        // FIXME: avmplus only verifies the exception target
        // if there's an opcode within `to` and `from` that could
        // potentially throw an error (e.g. getproperty, and findpropstrict,
        // but not getlocal0, setlocal0, and add)
        verify_block(
            activation,
            method,
            idx_to_byte_offset.as_slice(),
            &byte_offset_to_idx,
            &mut verified_blocks,
            new_code.as_slice(),
            exception.target_offset as i32,
            None,
            0,
            true,
        )?;
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

            new_idx - i - 1
        };
        match op {
            Op::IfEq { offset }
            | Op::IfFalse { offset }
            | Op::IfGe { offset }
            | Op::IfGt { offset }
            | Op::IfLe { offset }
            | Op::IfLt { offset }
            | Op::IfNe { offset }
            | Op::IfNge { offset }
            | Op::IfNgt { offset }
            | Op::IfNle { offset }
            | Op::IfNlt { offset }
            | Op::IfStrictEq { offset }
            | Op::IfStrictNe { offset }
            | Op::IfTrue { offset }
            | Op::Jump { offset } => {
                *offset = adjusted(i, *offset, true);
            }
            Op::LookupSwitch {
                default_offset,
                case_offsets,
            } => {
                *default_offset = adjusted(i, *default_offset, false);
                for case in case_offsets.iter_mut() {
                    *case = adjusted(i, *case, false);
                }
            }
            _ => {}
        }
    }

    Ok(new_body)
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
        .ok_or(Error::AvmError(verify_error(
            activation,
            "Error #1021: At least one branch target was not on a valid instruction in the method.",
            1021,
        )?))?;

    Ok(new_idx - 1)
}

fn verify_block<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
    idx_to_byte_offset: &[i32],
    byte_offset_to_idx: &HashMap<i32, i32>,
    verified_blocks: &mut Vec<(i32, Option<i32>)>,
    ops: &[Op],
    start_idx: i32,
    end_idx: Option<i32>,
    mut current_scope_depth: u32,
    top_level: bool,
) -> Result<(), Error<'gc>> {
    if verified_blocks.iter().any(|o| *o == (start_idx, end_idx)) {
        return Ok(());
    }

    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");

    verified_blocks.push((start_idx, end_idx));

    let initial_scope_depth = current_scope_depth;
    let max_scope_depth = body.max_scope_depth - body.init_scope_depth;

    let mut i = start_idx;
    while (i as usize) < ops.len() {
        let op = &ops[i as usize];

        // Special control flow ops
        match op {
            Op::IfEq { offset }
            | Op::IfFalse { offset }
            | Op::IfGe { offset }
            | Op::IfGt { offset }
            | Op::IfLe { offset }
            | Op::IfLt { offset }
            | Op::IfNe { offset }
            | Op::IfNge { offset }
            | Op::IfNgt { offset }
            | Op::IfNle { offset }
            | Op::IfNlt { offset }
            | Op::IfStrictEq { offset }
            | Op::IfStrictNe { offset }
            | Op::IfTrue { offset }
            | Op::Jump { offset } => {
                let op_idx = adjust_jump_offset(
                    activation,
                    i,
                    *offset,
                    true,
                    idx_to_byte_offset,
                    byte_offset_to_idx,
                )?;
                if op_idx != i {
                    let mut start = i + 1;
                    let mut end = op_idx;
                    if start > end {
                        // Switch start and end
                        let temp = start - 1;
                        start = end + 1;
                        end = temp;
                    }

                    if matches!(op, Op::Jump { .. }) {
                        verify_block(
                            activation,
                            method,
                            idx_to_byte_offset,
                            byte_offset_to_idx,
                            verified_blocks,
                            ops,
                            end + 1,
                            None,
                            current_scope_depth,
                            false,
                        )?;

                        // A Jump is terminal
                        return Ok(());
                    } else {
                        verify_block(
                            activation,
                            method,
                            idx_to_byte_offset,
                            byte_offset_to_idx,
                            verified_blocks,
                            ops,
                            start,
                            Some(end),
                            current_scope_depth,
                            false,
                        )?;
                        if op_idx > i {
                            i = op_idx;
                        }
                    }
                }
            }

            // Terminal opcodes
            Op::Throw => return Ok(()),
            Op::ReturnValue => return Ok(()),
            Op::ReturnVoid => return Ok(()),

            Op::LookupSwitch {
                default_offset,
                case_offsets,
            } => {
                let default_idx = adjust_jump_offset(
                    activation,
                    i,
                    *default_offset,
                    false,
                    idx_to_byte_offset,
                    byte_offset_to_idx,
                )?;

                verify_block(
                    activation,
                    method,
                    idx_to_byte_offset,
                    byte_offset_to_idx,
                    verified_blocks,
                    ops,
                    default_idx,
                    None,
                    current_scope_depth,
                    false,
                )?;
                for case in case_offsets.iter() {
                    let case_idx = adjust_jump_offset(
                        activation,
                        i,
                        *case,
                        false,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                    )?;

                    verify_block(
                        activation,
                        method,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                        verified_blocks,
                        ops,
                        case_idx,
                        None,
                        current_scope_depth,
                        false,
                    )?;
                }

                // A LookupSwitch is terminal
                return Ok(());
            }

            // Verifications

            // Local register verifications
            Op::GetLocal { index }
            | Op::SetLocal { index }
            | Op::Kill { index }
            | Op::DecLocal { index }
            | Op::DecLocalI { index }
            | Op::IncLocal { index }
            | Op::IncLocalI { index } => {
                let max = body.num_locals;
                if *index >= max {
                    return Err(make_error_1025(activation, *index));
                }
            }

            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                let max = body.num_locals;

                // NOTE: This is the correct order (first check object register, then check index register)
                if *object_register >= max {
                    return Err(make_error_1025(activation, *object_register));
                } else if *index_register >= max {
                    return Err(make_error_1025(activation, *index_register));
                }
            }

            Op::Debug {
                is_local_register,
                register,
                ..
            } => {
                if *is_local_register {
                    let max = body.num_locals;
                    if *register as u32 >= max {
                        return Err(make_error_1025(activation, *register as u32));
                    }
                }
            }

            // Scope stack-related verifications
            Op::PushWith | Op::PushScope => {
                current_scope_depth += 1;
                if current_scope_depth > max_scope_depth {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        "Error #1017: Scope stack overflow occurred.",
                        1018,
                    )?));
                }
            }

            Op::PopScope => {
                if current_scope_depth == 0 {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        "Error #1018: Scope stack underflow occurred.",
                        1018,
                    )?));
                }
                current_scope_depth -= 1;
            }

            Op::GetScopeObject { index } => {
                if (index + 1) as u32 > current_scope_depth {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        &format!("Error #1019: Getscopeobject {} is out of bounds.", index),
                        1019,
                    )?));
                }
            }

            // Ensure the global scope exists for these opcodes
            Op::FindProperty { .. } | Op::FindPropStrict { .. } => {
                // FP checks the scope that the function was defined in
                // for freestanding functions. We can't do that easily,
                // so just avoid this verification step for them.
                if !method.is_function {
                    if body.init_scope_depth + current_scope_depth == 0 {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1013: Cannot call OP_findproperty when scopeDepth is 0.",
                            1013,
                        )?));
                    }
                }
            }

            // Misc opcode verification
            Op::CallMethod { index, .. } => {
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

            Op::NewActivation => {
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

            Op::GetLex { index } => {
                // See comment for FindProperty/FindPropStrict.
                if !method.is_function {
                    if body.init_scope_depth + current_scope_depth == 0 {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1013: Cannot call OP_findproperty when scopeDepth is 0.",
                            1013,
                        )?));
                    }
                }

                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*index, &mut activation.borrow_gc())?;

                if multiname.has_lazy_component() {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        "Error #1078: Illegal opcode/multiname combination.",
                        1078,
                    )?));
                }
            }

            _ => {}
        }
        if let Some(end_idx) = end_idx {
            if i >= end_idx {
                if !top_level && current_scope_depth != initial_scope_depth {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        &format!(
                            "Error #1031: Scope depth unbalanced. {} != {}.",
                            current_scope_depth, initial_scope_depth
                        ),
                        1031,
                    )?));
                }
                return Ok(());
            }
        }

        i += 1;
    }

    Err(Error::AvmError(verify_error(
        activation,
        "Error #1020: Code cannot fall off the end of a method.",
        1020,
    )?))
}
