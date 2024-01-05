use crate::avm2::error::{make_error_1025, make_error_1054, make_error_1107, verify_error};
use crate::avm2::method::BytecodeMethod;
use crate::avm2::{Activation, Error};
use std::collections::HashMap;
use swf::avm2::read::Reader;
use swf::avm2::types::{Index, MethodFlags as AbcMethodFlags, Multiname, Op};
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

    verify_code_starting_from(
        activation,
        method,
        idx_to_byte_offset.as_slice(),
        &byte_offset_to_idx,
        &mut verified_blocks,
        new_code.as_slice(),
        0,
    )?;

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
            .unwrap() as u32;

        if exception.target_offset < exception.to_offset {
            return Err(make_error_1054(activation));
        }

        new_exceptions.push(Exception {
            from_offset: new_from_offset,
            to_offset: new_to_offset,
            target_offset: new_target_offset,
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
                new_target_offset as i32,
            )?;
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
            Op::LookupSwitch(ref mut lookup_switch) => {
                lookup_switch.default_offset = adjusted(i, lookup_switch.default_offset, false);
                for case in lookup_switch.case_offsets.iter_mut() {
                    *case = adjusted(i, *case, false);
                }
            }
            _ => {}
        }
    }

    Ok(VerifiedMethodInfo {
        parsed_code: new_code,
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
        .ok_or(Error::AvmError(verify_error(
            activation,
            "Error #1021: At least one branch target was not on a valid instruction in the method.",
            1021,
        )?))?;

    Ok(new_idx - 1)
}

fn verify_code_starting_from<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
    idx_to_byte_offset: &[i32],
    byte_offset_to_idx: &HashMap<i32, i32>,
    verified_blocks: &mut Vec<i32>,
    ops: &[Op],
    start_idx: i32,
) -> Result<(), Error<'gc>> {
    if verified_blocks.iter().any(|o| *o == start_idx) {
        // Already verified
        return Ok(());
    }

    verified_blocks.push(start_idx);

    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");
    let max_locals = body.num_locals;

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
                    let position = op_idx;

                    verify_code_starting_from(
                        activation,
                        method,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                        verified_blocks,
                        ops,
                        position + 1,
                    )?;

                    if matches!(op, Op::Jump { .. }) {
                        // A Jump is terminal, the code
                        // after it won't be executed
                        return Ok(());
                    }
                }
            }

            // Terminal opcodes
            Op::Throw => return Ok(()),
            Op::ReturnValue => return Ok(()),
            Op::ReturnVoid => return Ok(()),

            Op::LookupSwitch(ref lookup_switch) => {
                let default_idx = adjust_jump_offset(
                    activation,
                    i,
                    lookup_switch.default_offset,
                    false,
                    idx_to_byte_offset,
                    byte_offset_to_idx,
                )?;

                verify_code_starting_from(
                    activation,
                    method,
                    idx_to_byte_offset,
                    byte_offset_to_idx,
                    verified_blocks,
                    ops,
                    default_idx,
                )?;

                for case in lookup_switch.case_offsets.iter() {
                    let case_idx = adjust_jump_offset(
                        activation,
                        i,
                        *case,
                        false,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                    )?;

                    verify_code_starting_from(
                        activation,
                        method,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                        verified_blocks,
                        ops,
                        case_idx,
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
                if *index >= max_locals {
                    return Err(make_error_1025(activation, *index));
                }
            }

            Op::HasNext2 {
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

            _ => {}
        }

        i += 1;
    }

    Err(Error::AvmError(verify_error(
        activation,
        "Error #1020: Code cannot fall off the end of a method.",
        1020,
    )?))
}

fn ops_can_throw_error(ops: &[Op], start_idx: u32, end_idx: u32) -> bool {
    for i in start_idx..end_idx {
        let op = &ops[i as usize];
        match op {
            Op::PushByte { .. }
            | Op::PushDouble { .. }
            | Op::PushFalse
            | Op::PushInt { .. }
            | Op::PushNamespace { .. }
            | Op::PushNaN
            | Op::PushNull
            | Op::PushShort { .. }
            | Op::PushString { .. }
            | Op::PushTrue
            | Op::PushUint { .. }
            | Op::PushUndefined
            | Op::Dup
            | Op::Pop
            | Op::GetLocal { .. }
            | Op::SetLocal { .. }
            | Op::Kill { .. }
            | Op::Nop
            | Op::Not
            | Op::PopScope
            | Op::ReturnVoid => {}
            _ => return true,
        }
    }

    false
}
