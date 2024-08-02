use crate::avm2::class::Class;
use crate::avm2::error::{
    make_error_1014, make_error_1021, make_error_1025, make_error_1032, make_error_1054,
    make_error_1107, verify_error,
};
use crate::avm2::method::{BytecodeMethod, ParamConfig, ResolvedParamConfig};
use crate::avm2::multiname::Multiname;
use crate::avm2::op::Op;
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Activation, Error, QName};
use crate::string::AvmAtom;

use gc_arena::{Collect, Gc};
use std::collections::{HashMap, HashSet};
use swf::avm2::read::Reader;
use swf::avm2::types::{
    Class as AbcClass, Index, MethodFlags as AbcMethodFlags, Multiname as AbcMultiname, Op as AbcOp,
};
use swf::error::Error as AbcReadError;

#[derive(Collect)]
#[collect(no_drop)]
pub struct VerifiedMethodInfo<'gc> {
    pub parsed_code: Vec<Op<'gc>>,

    pub exceptions: Vec<Exception<'gc>>,

    pub param_config: Vec<ResolvedParamConfig<'gc>>,
    pub return_type: Option<Class<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct Exception<'gc> {
    pub from_offset: u32,
    pub to_offset: u32,
    pub target_offset: u32,

    pub variable_name: Option<QName<'gc>>,
    pub target_class: Option<Class<'gc>>,
}

#[derive(Clone, Debug)]
enum ByteInfo {
    OpStart(AbcOp),
    OpContinue,

    OpStartNonJumpable(AbcOp),

    NotYetReached,
}

impl ByteInfo {
    fn get_op(&self) -> Option<&AbcOp> {
        match self {
            ByteInfo::OpStart(op) | ByteInfo::OpStartNonJumpable(op) => Some(op),
            _ => None,
        }
    }
}

pub enum JumpSource {
    JumpFrom(i32),
    ExceptionTarget,
}

pub fn verify_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
) -> Result<VerifiedMethodInfo<'gc>, Error<'gc>> {
    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");
    let translation_unit = method.translation_unit();

    let param_count = method.method().params.len();
    let max_locals = body.num_locals;

    // Ensure there are enough local variables
    // to fit the parameters in.
    if (max_locals as usize) < 1 + param_count {
        return Err(make_error_1107(activation));
    }

    if (max_locals as usize) < 1 + param_count + if method.is_variadic() { 1 } else { 0 } {
        // This matches FP's error message
        return Err(make_error_1025(activation, 1 + param_count as u32));
    }

    use swf::extensions::ReadSwfExt;

    if body.code.is_empty() {
        return Err(Error::AvmError(verify_error(
            activation,
            "Error #1043: Invalid code_length=0.",
            1043,
        )?));
    }

    let resolved_param_config = resolve_param_config(activation, method.signature())?;
    let resolved_return_type = resolve_return_type(activation, &method.return_type)?;

    let mut seen_exception_indices = HashSet::new();

    let mut worklist = vec![0];

    let mut byte_info = vec![ByteInfo::NotYetReached; body.code.len()];
    let mut seen_targets = HashSet::new();

    let mut reader = Reader::new(&body.code);
    while let Some(i) = worklist.pop() {
        reader.seek_absolute(&body.code, i as usize);
        loop {
            let previous_position = reader.pos(&body.code) as i32;

            let op = match reader.read_op() {
                Ok(op) => op,

                Err(AbcReadError::InvalidData(_)) => {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        "Error #1011: Method contained illegal opcode.",
                        1011,
                    )?));
                }
                Err(AbcReadError::IoError(_)) => {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        "Error #1020: Code cannot fall off the end of a method.",
                        1020,
                    )?));
                }
                Err(_) => unreachable!(),
            };

            for (exception_index, exception) in body.exceptions.iter().enumerate() {
                // If this op is in the to..from and it can throw an error,
                // add the exception's target to the worklist.
                if exception.from_offset as i32 <= previous_position
                    && previous_position < exception.to_offset as i32
                    && op_can_throw_error(&op)
                {
                    if !seen_targets.contains(&(exception.target_offset as i32)) {
                        worklist.push(exception.target_offset);
                        seen_targets.insert(exception.target_offset as i32);
                    }

                    // Keep track of all the valid exceptions, and only verify
                    // them- this is more lenient than avmplus, but still safe.
                    seen_exception_indices.insert(exception_index);
                }
            }

            let new_position = reader.pos(&body.code) as i32;

            let bytes_read = new_position - previous_position;
            assert!(bytes_read > 0);

            byte_info[previous_position as usize] = ByteInfo::OpStart(op.clone());
            for j in 1..bytes_read {
                byte_info[(previous_position + j) as usize] = ByteInfo::OpContinue;
            }

            let mut check_target = |seen_targets: &HashSet<i32>, offs: i32, is_jump: bool| {
                let target_position = if is_jump {
                    offs + new_position
                } else {
                    offs + previous_position
                };

                let lookedup_target_info = byte_info.get(target_position as usize);

                if matches!(
                    lookedup_target_info,
                    Some(ByteInfo::OpContinue | ByteInfo::OpStartNonJumpable(_))
                ) {
                    return Err(make_error_1021(activation));
                }

                if target_position < 0 || target_position as usize >= body.code.len() {
                    return Err(make_error_1021(activation));
                }

                // Ensure backwards jumps to not-yet-jumped-to code target a `Label` op
                if !seen_targets.contains(&target_position) && offs < 0 {
                    reader.seek_absolute(&body.code, target_position as usize);
                    let read_op = reader.read_op();

                    // Seek back to the original position
                    reader.seek_absolute(&body.code, new_position as usize);

                    if !matches!(read_op, Ok(AbcOp::Label)) {
                        return Err(make_error_1021(activation));
                    }
                }

                Ok(())
            };

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
                    check_target(&seen_targets, offset, true)?;

                    let offset = offset + new_position;
                    if !seen_targets.contains(&offset) {
                        worklist.push(offset as u32);
                        seen_targets.insert(offset);
                    }

                    if matches!(op, AbcOp::Jump { .. }) {
                        break;
                    }
                }

                // Terminal opcodes
                AbcOp::Throw | AbcOp::ReturnValue | AbcOp::ReturnVoid => {
                    break;
                }

                AbcOp::LookupSwitch(ref lookup_switch) => {
                    check_target(&seen_targets, lookup_switch.default_offset, false)?;
                    let default_offset = lookup_switch.default_offset + previous_position;

                    if !seen_targets.contains(&default_offset) {
                        seen_targets.insert(default_offset);

                        worklist.push(default_offset as u32);
                    }

                    for case_offset in lookup_switch.case_offsets.iter() {
                        check_target(&seen_targets, *case_offset, false)?;

                        let case_offset = case_offset + previous_position;
                        if !seen_targets.contains(&case_offset) {
                            seen_targets.insert(case_offset);

                            worklist.push(case_offset as u32);
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
                    if index >= max_locals {
                        return Err(make_error_1025(activation, index));
                    }
                }

                AbcOp::HasNext2 {
                    object_register,
                    index_register,
                } => {
                    // NOTE: This is the correct order (first check object register, then check index register)
                    if object_register >= max_locals {
                        return Err(make_error_1025(activation, object_register));
                    } else if index_register >= max_locals {
                        return Err(make_error_1025(activation, index_register));
                    }
                }

                // Misc opcode verification
                AbcOp::CallMethod { index, .. } => {
                    return Err(Error::AvmError(if index == 0 {
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

                AbcOp::FindDef { index } => {
                    let multiname = method
                        .translation_unit()
                        .pool_maybe_uninitialized_multiname(index, activation.context)?;

                    if multiname.has_lazy_component() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1078: Illegal opcode/multiname combination.",
                            1078,
                        )?));
                    }
                }

                AbcOp::GetLex { index } => {
                    let multiname = method
                        .translation_unit()
                        .pool_maybe_uninitialized_multiname(index, activation.context)?;

                    if multiname.has_lazy_component() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1078: Illegal opcode/multiname combination.",
                            1078,
                        )?));
                    }

                    // Split this `GetLex` into a `FindPropStrict` and a `GetProperty`.
                    // A `GetLex` is guaranteed to take up at least 2 bytes. We need
                    // one byte for the opcode and at least one byte for the multiname
                    // index. Overwrite the op registered at the opcode byte with a
                    // `FindPropStrict` op, and register a non-jumpable `GetProperty`
                    // op at the next byte. This isn't the best way to do it, but it's
                    // simpler than actually emitting ops and rewriting the jump offsets
                    // to match.
                    assert!(bytes_read > 1);
                    byte_info[previous_position as usize] =
                        ByteInfo::OpStart(AbcOp::FindPropStrict { index });
                    byte_info[(previous_position + 1) as usize] =
                        ByteInfo::OpStartNonJumpable(AbcOp::GetProperty { index });
                }

                AbcOp::GetOuterScope { index } => {
                    if activation.outer().get(index as usize).is_none() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1019: Getscopeobject  is out of bounds.",
                            1019,
                        )?));
                    }
                }

                AbcOp::AsType {
                    type_name: name_index,
                }
                | AbcOp::IsType { index: name_index }
                | AbcOp::Coerce { index: name_index } => {
                    let multiname = method
                        .translation_unit()
                        .pool_maybe_uninitialized_multiname(name_index, activation.context)?;

                    if multiname.has_lazy_component() {
                        // This matches FP's error message
                        return Err(make_error_1014(activation, "[]".into()));
                    }

                    activation
                        .domain()
                        .get_class(activation.context, &multiname)
                        .ok_or_else(|| {
                            make_error_1014(
                                activation,
                                multiname.to_qualified_name(activation.context.gc_context),
                            )
                        })?;
                }

                _ => {}
            }
        }
    }

    let mut byte_offset_to_idx = HashMap::new();
    let mut idx_to_byte_offset = Vec::new();

    let mut new_code = Vec::new();
    for (i, info) in byte_info.iter().enumerate() {
        if let Some(op) = info.get_op() {
            byte_offset_to_idx.insert(i, new_code.len() as i32);
            new_code.push(op.clone());
            idx_to_byte_offset.push(i);
        }
    }

    // Record a target->sources mapping of all jump
    // targets- this will be used in the optimizer.
    let mut potential_jump_targets: HashMap<i32, Vec<JumpSource>> = HashMap::new();

    // Handle exceptions
    let mut new_exceptions = Vec::new();
    for (exception_index, exception) in body.exceptions.iter().enumerate() {
        // Resolve the variable name and target class.

        let target_class = if exception.type_name.0 == 0 {
            None
        } else {
            let pooled_type_name = method
                .translation_unit()
                .pool_maybe_uninitialized_multiname(exception.type_name, activation.context)?;

            if pooled_type_name.has_lazy_component() {
                // This matches FP's error message
                return Err(make_error_1014(activation, "[]".into()));
            }

            let resolved_type = activation
                .domain()
                .get_class(activation.context, &pooled_type_name)
                .ok_or_else(|| {
                    make_error_1014(
                        activation,
                        pooled_type_name.to_qualified_name(activation.context.gc_context),
                    )
                })?;

            Some(resolved_type)
        };

        let variable_name = if exception.variable_name.0 == 0 {
            None
        } else {
            let pooled_variable_name = method
                .translation_unit()
                .pool_maybe_uninitialized_multiname(exception.variable_name, activation.context)?;

            // FIXME: avmplus also seems to check the namespace(s)?
            if pooled_variable_name.has_lazy_component()
                || pooled_variable_name.is_attribute()
                || pooled_variable_name.is_any_name()
            {
                // This matches FP's error message
                return Err(make_error_1107(activation));
            }

            let namespaces = pooled_variable_name.namespace_set();

            if namespaces.is_empty() {
                // NOTE: avmplus segfaults here
                panic!("Should have at least one namespace for QName in exception variable name");
            }

            let name = pooled_variable_name.local_name().expect("Just checked");

            // avmplus uses the first namespace, regardless of how many namespaces there are.
            Some(QName::new(namespaces[0], name))
        };

        if !seen_exception_indices.contains(&exception_index) {
            // We need to push an exception because otherwise `newcatch` ops can try to
            // read it, but we can give it dummy from/to/target offsets because no code
            // can actually trigger it (and we might not even have valid offsets anyway).
            new_exceptions.push(Exception {
                from_offset: 0,
                to_offset: 0,
                target_offset: 0,
                variable_name,
                target_class,
            });
            continue;
        }

        // Now resolve the offsets.

        // NOTE: This is actually wrong, we should be using the byte offsets in
        // `Activation::handle_err`, not the opcode offsets. avmplus allows for from/to
        // (but not targets) that aren't on a opcode, and some obfuscated SWFs have them.
        // FFDEC handles them correctly, stepping forward byte-by-byte until it reaches
        // the next opcode. This does the same (stepping byte-by-byte), but it would
        // be better to directly use the byte offsets when handling exceptions.
        let mut from_offset = None;

        let mut offs = 0;
        while from_offset.is_none() {
            from_offset = byte_offset_to_idx
                .get(&((exception.from_offset + offs) as usize))
                .copied();

            offs += 1;
            if (exception.from_offset + offs) as usize >= body.code.len() {
                return Err(make_error_1054(activation));
            }
        }

        // Now for the `to_offset`.
        let mut to_offset = None;

        let mut offs = 0;
        while to_offset.is_none() {
            to_offset = byte_offset_to_idx
                .get(&((exception.to_offset + offs) as usize))
                .copied();

            offs += 1;
            if (exception.to_offset + offs) as usize >= body.code.len() {
                return Err(make_error_1054(activation));
            }
        }

        let new_from_offset = from_offset.unwrap() as u32;
        let new_to_offset = to_offset.unwrap() as u32;

        if new_to_offset < new_from_offset {
            return Err(make_error_1054(activation));
        }

        let maybe_new_target_offset = byte_offset_to_idx
            .get(&(exception.target_offset as usize))
            .copied();

        // The large "NOTE" comment below is also relevant here
        if let Some(new_target_offset) = maybe_new_target_offset {
            // If this is a reachable target offset, insert it into the list
            // of potential jump targets.
            potential_jump_targets
                .entry(new_target_offset)
                .or_default()
                .push(JumpSource::ExceptionTarget);
        }

        let new_target_offset = maybe_new_target_offset.unwrap_or(0);

        // NOTE: That `unwrap_or` is definitely reachable, e.g. in a case where
        // the target offset is unreachable (see the test "verification"), but it
        // might also be reachable in cases where the target offset will actually
        // be jumped to. Any SWF that does this is extremely cursed and should
        // VerifyError in FP (though I haven't been able to confirm that it does),
        // so we probably don't need to worry about that case.

        if exception.target_offset < exception.to_offset {
            return Err(make_error_1054(activation));
        }

        if new_target_offset as usize >= new_code.len() {
            return Err(make_error_1054(activation));
        }

        new_exceptions.push(Exception {
            from_offset: new_from_offset,
            to_offset: new_to_offset,
            target_offset: new_target_offset as u32,
            variable_name,
            target_class,
        });
    }

    let mut adjust_jump_to_idx = |i, offset, is_jump| -> Result<(i32, i32), Error<'gc>> {
        const JUMP_INSTRUCTION_LENGTH: usize = 4;

        let mut byte_offset = idx_to_byte_offset
            .get(i as usize)
            .copied()
            .ok_or_else(|| make_error_1021(activation))?; // This is still reachable with some weird bytecode, see the `verify_jump_to_middle_of_op` test

        if is_jump {
            byte_offset += JUMP_INSTRUCTION_LENGTH;
        }

        let new_byte_offset = byte_offset as i32 + offset;
        let new_idx = byte_offset_to_idx
            .get(&(new_byte_offset as usize))
            .copied()
            .ok_or_else(|| make_error_1021(activation))?; // See above comment

        Ok((new_idx, new_idx - i - 1))
    };

    // Adjust jump offsets from byte-based to idx-based
    for (i, op) in new_code.iter_mut().enumerate() {
        let i = i as i32;

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
                let adjusted_result = adjust_jump_to_idx(i, *offset, true)?;
                *offset = adjusted_result.1;

                potential_jump_targets
                    .entry(adjusted_result.0)
                    .or_default()
                    .push(JumpSource::JumpFrom(i));
            }
            AbcOp::LookupSwitch(ref mut lookup_switch) => {
                let adjusted_default = adjust_jump_to_idx(i, lookup_switch.default_offset, false)?;
                lookup_switch.default_offset = adjusted_default.1;

                potential_jump_targets
                    .entry(adjusted_default.0)
                    .or_default()
                    .push(JumpSource::JumpFrom(i));

                for case in lookup_switch.case_offsets.iter_mut() {
                    let adjusted_case = adjust_jump_to_idx(i, *case, false)?;
                    *case = adjusted_case.1;

                    potential_jump_targets
                        .entry(adjusted_case.0)
                        .or_default()
                        .push(JumpSource::JumpFrom(i));
                }
            }
            _ => {}
        }
    }

    let mut verified_code = Vec::new();
    for abc_op in new_code {
        let resolved_op = resolve_op(activation, translation_unit, abc_op.clone())?;

        verified_code.push(resolved_op);
    }

    if activation.avm2().optimizer_enabled() {
        crate::avm2::optimize::optimize(
            activation,
            method,
            &mut verified_code,
            &resolved_param_config,
            resolved_return_type,
            potential_jump_targets,
        );
    }

    Ok(VerifiedMethodInfo {
        parsed_code: verified_code,
        exceptions: new_exceptions,
        param_config: resolved_param_config,
        return_type: resolved_return_type,
    })
}

pub fn resolve_param_config<'gc>(
    activation: &mut Activation<'_, 'gc>,
    param_config: &[ParamConfig<'gc>],
) -> Result<Vec<ResolvedParamConfig<'gc>>, Error<'gc>> {
    let mut resolved_param_config = Vec::new();

    for param in param_config {
        if param.param_type_name.has_lazy_component() {
            return Err(make_error_1014(activation, "[]".into()));
        }

        let resolved_class = if param.param_type_name.is_any_name() {
            None
        } else {
            let lookedup_class = activation
                .domain()
                .get_class(activation.context, &param.param_type_name)
                .ok_or_else(|| {
                    make_error_1014(
                        activation,
                        param
                            .param_type_name
                            .to_qualified_name(activation.context.gc_context),
                    )
                })?;

            Some(lookedup_class)
        };

        resolved_param_config.push(ResolvedParamConfig {
            param_name: param.param_name,
            param_type: resolved_class,
            default_value: param.default_value,
        });
    }

    Ok(resolved_param_config)
}

fn resolve_return_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    return_type: &Multiname<'gc>,
) -> Result<Option<Class<'gc>>, Error<'gc>> {
    if return_type.has_lazy_component() {
        return Err(make_error_1014(activation, "[]".into()));
    }

    if return_type.is_any_name() {
        return Ok(None);
    }

    Ok(Some(
        activation
            .domain()
            .get_class(activation.context, return_type)
            .ok_or_else(|| {
                make_error_1014(
                    activation,
                    return_type.to_qualified_name(activation.context.gc_context),
                )
            })?,
    ))
}

// Taken from avmplus's opcodes.tbl
fn op_can_throw_error(op: &AbcOp) -> bool {
    !matches!(
        op,
        AbcOp::Bkpt
            | AbcOp::BkptLine { .. }
            | AbcOp::Timestamp
            | AbcOp::PushByte { .. }
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
            | AbcOp::Swap
            | AbcOp::Pop
            | AbcOp::TypeOf
            | AbcOp::GetGlobalScope
            | AbcOp::GetScopeObject { .. }
            | AbcOp::GetOuterScope { .. }
            | AbcOp::GetGlobalSlot { .. }
            | AbcOp::GetLocal { .. }
            | AbcOp::SetLocal { .. }
            | AbcOp::Kill { .. }
            | AbcOp::Label
            | AbcOp::Jump { .. }
            | AbcOp::IfTrue { .. }
            | AbcOp::IfFalse { .. }
            | AbcOp::IfStrictEq { .. }
            | AbcOp::IfStrictNe { .. }
            | AbcOp::LookupSwitch { .. }
            | AbcOp::Nop
            | AbcOp::Not
            | AbcOp::PopScope
            | AbcOp::ReturnVoid
    )
}

fn pool_int<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<i32>,
) -> Result<i32, Error<'gc>> {
    if index.0 == 0 {
        return Err(make_error_1032(activation, 0));
    }

    translation_unit
        .abc()
        .constant_pool
        .ints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| make_error_1032(activation, index.0))
}

fn pool_uint<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<u32>,
) -> Result<u32, Error<'gc>> {
    if index.0 == 0 {
        return Err(make_error_1032(activation, 0));
    }

    translation_unit
        .abc()
        .constant_pool
        .uints
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| make_error_1032(activation, index.0))
}

fn pool_double<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<f64>,
) -> Result<f64, Error<'gc>> {
    if index.0 == 0 {
        return Err(make_error_1032(activation, 0));
    }

    translation_unit
        .abc()
        .constant_pool
        .doubles
        .get(index.0 as usize - 1)
        .cloned()
        .ok_or_else(|| make_error_1032(activation, index.0))
}

fn pool_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<AbcMultiname>,
) -> Result<Gc<'gc, Multiname<'gc>>, Error<'gc>> {
    if index.0 == 0 {
        return Err(make_error_1032(activation, 0));
    }

    translation_unit.pool_maybe_uninitialized_multiname(index, activation.context)
}

fn pool_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<String>,
) -> Result<AvmAtom<'gc>, Error<'gc>> {
    if index.0 == 0 {
        return Err(make_error_1032(activation, 0));
    }

    translation_unit.pool_string(index.0, &mut activation.borrow_gc())
}

fn pool_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<AbcClass>,
) -> Result<Class<'gc>, Error<'gc>> {
    translation_unit.load_class(index.0, activation)
}

fn resolve_op<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    op: AbcOp,
) -> Result<Op<'gc>, Error<'gc>> {
    Ok(match op {
        AbcOp::PushByte { value } => Op::PushByte { value: value as i8 },
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
        AbcOp::PushString { value } => {
            let string = pool_string(activation, translation_unit, value)?;

            Op::PushString { string }
        }
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
        AbcOp::CallMethod { index, num_args } => Op::CallMethod {
            index,
            num_args,
            push_return_value: true,
        },
        AbcOp::CallProperty { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::CallProperty {
                multiname,
                num_args,
            }
        }
        AbcOp::CallPropLex { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::CallPropLex {
                multiname,
                num_args,
            }
        }
        AbcOp::CallPropVoid { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::CallPropVoid {
                multiname,
                num_args,
            }
        }
        AbcOp::CallStatic { index, num_args } => Op::CallStatic { index, num_args },
        AbcOp::CallSuper { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::CallSuper {
                multiname,
                num_args,
            }
        }
        AbcOp::CallSuperVoid { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::CallSuperVoid {
                multiname,
                num_args,
            }
        }
        AbcOp::ReturnValue => Op::ReturnValue,
        AbcOp::ReturnVoid => Op::ReturnVoid,
        AbcOp::GetProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::GetProperty { multiname }
        }
        AbcOp::SetProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::SetProperty { multiname }
        }
        AbcOp::InitProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::InitProperty { multiname }
        }
        AbcOp::DeleteProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::DeleteProperty { multiname }
        }
        AbcOp::GetSuper { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::GetSuper { multiname }
        }
        AbcOp::SetSuper { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::SetSuper { multiname }
        }
        AbcOp::In => Op::In,
        AbcOp::PushScope => Op::PushScope,
        AbcOp::NewCatch { index } => Op::NewCatch { index },
        AbcOp::PushWith => Op::PushWith,
        AbcOp::PopScope => Op::PopScope,
        AbcOp::GetOuterScope { index } => Op::GetOuterScope { index },
        AbcOp::GetScopeObject { index } => Op::GetScopeObject { index },
        AbcOp::GetGlobalScope => Op::GetGlobalScope,
        AbcOp::FindDef { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;
            // Verifier guarantees that multiname was non-lazy

            Op::FindDef { multiname }
        }
        AbcOp::FindProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::FindProperty { multiname }
        }
        AbcOp::FindPropStrict { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::FindPropStrict { multiname }
        }
        AbcOp::GetLex { .. } => {
            unreachable!("Verifier emits FindPropStrict and GetProperty instead of GetLex")
        }
        AbcOp::GetDescendants { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::GetDescendants { multiname }
        }
        // Turn 1-based representation into 0-based representation
        AbcOp::GetSlot { index } => Op::GetSlot { index: index - 1 },
        AbcOp::SetSlot { index } => Op::SetSlot { index: index - 1 },
        AbcOp::GetGlobalSlot { index } => Op::GetGlobalSlot { index: index - 1 },
        AbcOp::SetGlobalSlot { index } => Op::SetGlobalSlot { index: index - 1 },

        AbcOp::Construct { num_args } => Op::Construct { num_args },
        AbcOp::ConstructProp { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;
            // Verifier guarantees that multiname was non-lazy

            Op::ConstructProp {
                multiname,
                num_args,
            }
        }
        AbcOp::ConstructSuper { num_args } => Op::ConstructSuper { num_args },
        AbcOp::NewActivation => Op::NewActivation,
        AbcOp::NewObject { num_args } => Op::NewObject { num_args },
        AbcOp::NewFunction { index } => Op::NewFunction { index },
        AbcOp::NewClass { index } => {
            let class = pool_class(activation, translation_unit, index)?;
            Op::NewClass { class }
        }
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
        AbcOp::IsType { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;
            // Verifier guarantees that multiname was non-lazy

            let class = activation
                .domain()
                .get_class(activation.context, &multiname)
                .unwrap();
            // Verifier guarantees that class exists

            Op::IsType { class }
        }
        AbcOp::IsTypeLate => Op::IsTypeLate,
        AbcOp::AsType { type_name } => {
            let multiname = pool_multiname(activation, translation_unit, type_name)?;
            // Verifier guarantees that multiname was non-lazy

            let class = activation
                .domain()
                .get_class(activation.context, &multiname)
                .unwrap();
            // Verifier guarantees that class exists

            Op::AsType { class }
        }
        AbcOp::AsTypeLate => Op::AsTypeLate,
        AbcOp::InstanceOf => Op::InstanceOf,
        AbcOp::Label => Op::Nop,
        AbcOp::Debug {
            is_local_register,
            register_name,
            register,
        } => {
            let register_name = pool_string(activation, translation_unit, register_name)?;

            Op::Debug {
                is_local_register,
                register_name,
                register,
            }
        }
        AbcOp::DebugFile { file_name } => {
            let file_name = pool_string(activation, translation_unit, file_name)?;

            Op::DebugFile { file_name }
        }
        AbcOp::DebugLine { line_num } => Op::DebugLine { line_num },
        AbcOp::Bkpt => Op::Bkpt,
        AbcOp::BkptLine { line_num } => Op::BkptLine { line_num },
        AbcOp::Timestamp => Op::Timestamp,
        AbcOp::TypeOf => Op::TypeOf,
        AbcOp::EscXAttr => Op::EscXAttr,
        AbcOp::EscXElem => Op::EscXElem,
        AbcOp::Dxns { index } => {
            let string = pool_string(activation, translation_unit, index)?;

            Op::Dxns { string }
        }
        AbcOp::DxnsLate => Op::DxnsLate,
        AbcOp::LookupSwitch(lookup_switch) => Op::LookupSwitch(lookup_switch),
        AbcOp::Coerce { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;
            // Verifier guarantees that multiname was non-lazy

            let class = activation
                .domain()
                .get_class(activation.context, &multiname)
                .unwrap();
            // Verifier guarantees that class exists

            Op::Coerce { class }
        }
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
    })
}
