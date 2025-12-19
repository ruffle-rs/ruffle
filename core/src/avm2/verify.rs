use crate::avm2::class::Class;
use crate::avm2::error::{
    make_error_1011, make_error_1014, make_error_1019, make_error_1020, make_error_1021,
    make_error_1025, make_error_1026, make_error_1032, make_error_1043, make_error_1051,
    make_error_1054, make_error_1072, make_error_1078, make_error_1107, make_error_1113,
    make_error_1124, Error1014Type,
};
use crate::avm2::method::Method;
use crate::avm2::multiname::Multiname;
use crate::avm2::namespace::Namespace;
use crate::avm2::op::{LookupSwitch, Op};
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Activation, Error, QName};
use crate::string::{AvmAtom, AvmString};

use gc_arena::{Collect, Gc};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use swf::avm2::read::Reader;
use swf::avm2::types::{
    Class as AbcClass, Index, Method as AbcMethod, MethodFlags as AbcMethodFlags,
    Multiname as AbcMultiname, Namespace as AbcNamespace, Op as AbcOp,
};
use swf::error::Error as AbcReadError;

#[derive(Collect)]
#[collect(no_drop)]
pub struct VerifiedMethodInfo<'gc> {
    pub parsed_code: Vec<Op<'gc>>,

    pub exceptions: Vec<Exception<'gc>>,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct Exception<'gc> {
    pub from_offset: usize,
    pub to_offset: usize,
    pub target_offset: usize,

    pub catch_class: Option<Class<'gc>>,
    pub target_class: Option<Class<'gc>>,
}

#[derive(Clone, Copy, Debug)]
enum ByteInfo<'gc> {
    OpStart(Op<'gc>),
    OpStartNonJumpable(Op<'gc>),

    OpContinue,

    NotYetReached,
}

impl<'gc> ByteInfo<'gc> {
    fn get_op(self) -> Option<Op<'gc>> {
        match self {
            ByteInfo::OpStart(op) | ByteInfo::OpStartNonJumpable(op) => Some(op),
            _ => None,
        }
    }
}

pub fn verify_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
) -> Result<VerifiedMethodInfo<'gc>, Error<'gc>> {
    let mc = activation.gc();
    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");

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
        return Err(make_error_1043(activation));
    }

    let activation_class = create_activation_class(activation, method)?;

    let resolved_return_type = method.resolved_return_type();
    let resolved_param_config = method.resolved_param_config();

    let mut seen_exception_indices = HashSet::new();

    let mut worklist = vec![0];

    let mut byte_info = vec![ByteInfo::NotYetReached; body.code.len()];
    let mut seen_targets = HashSet::new();

    let mut reader = Reader::new(&body.code);
    while let Some(i) = worklist.pop() {
        reader.seek_absolute(&body.code, i as usize);
        loop {
            let previous_position = reader.pos(&body.code) as i32;

            // We've already verified this chunk of code, let's not run the logic again
            if matches!(
                byte_info.get(previous_position as usize),
                Some(ByteInfo::OpStart(_))
            ) {
                break;
            }

            let op = match reader.read_op() {
                Ok(op) => op,

                Err(AbcReadError::InvalidData(_)) => {
                    // Invalid opcode
                    return Err(make_error_1011(activation));
                }
                Err(AbcReadError::IoError(_)) => {
                    // Code flow continued past end of method
                    return Err(make_error_1020(activation));
                }
                Err(_) => unreachable!(),
            };

            if op_can_throw_error(&op) {
                for (exception_index, exception) in body.exceptions.iter().enumerate() {
                    // If this op is in the to..from and it can throw an error,
                    // add the exception's target to the worklist.
                    if exception.from_offset as i32 <= previous_position
                        && previous_position < exception.to_offset as i32
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
            }

            let new_position = reader.pos(&body.code) as i32;

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

            // Special control flow ops: handle the worklist
            match &op {
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
                    check_target(&seen_targets, *offset, true)?;

                    let offset = offset + new_position;
                    if !seen_targets.contains(&offset) {
                        worklist.push(offset as u32);
                        seen_targets.insert(offset);
                    }
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
                }

                _ => {}
            }

            let is_terminator_op = matches!(
                op,
                AbcOp::Jump { .. }
                    | AbcOp::LookupSwitch(_)
                    | AbcOp::Throw
                    | AbcOp::ReturnValue
                    | AbcOp::ReturnVoid
            );

            // Actually translate the AbcOp into an Op

            let bytes_read = new_position - previous_position;
            assert!(bytes_read > 0);

            for j in 0..bytes_read {
                byte_info[(previous_position + j) as usize] = ByteInfo::OpContinue;
            }

            let translated_ops = translate_op(
                activation,
                method,
                max_locals,
                activation_class,
                resolved_return_type,
                op,
            )?;

            byte_info[previous_position as usize] = ByteInfo::OpStart(translated_ops.0);
            if let Some(second_op) = translated_ops.1 {
                assert!(bytes_read > 1);

                // Split this op into two.
                // This op must be guaranteed to take up at least 2 bytes. We
                // simply register a non-jumpable second op at the next byte.
                // This isn't the best way to do it, but it's simpler than
                // actually emitting ops and rewriting the jump offsets to match.
                byte_info[previous_position as usize + 1] = ByteInfo::OpStartNonJumpable(second_op);
            }

            if is_terminator_op {
                break;
            }
        }
    }

    let mut byte_offset_to_idx = HashMap::new();
    let mut idx_to_byte_offset = Vec::new();

    let mut verified_code = Vec::new();
    for (i, info) in byte_info.iter().enumerate() {
        if let Some(op) = info.get_op() {
            byte_offset_to_idx.insert(i, verified_code.len());
            verified_code.push(op);
            idx_to_byte_offset.push(i);
        }
    }

    // Record a target->sources mapping of all jump
    // targets- this will be used in the optimizer.
    let mut jump_targets = HashSet::new();

    // Handle exceptions
    let mut new_exceptions = Vec::new();
    for (exception_index, exception) in body.exceptions.iter().enumerate() {
        // Resolve the variable name and target class.

        let target_class = if exception.type_name.0 == 0 {
            None
        } else {
            let pooled_type_name = method
                .translation_unit()
                .pool_maybe_uninitialized_multiname(activation, exception.type_name)?;

            if pooled_type_name.has_lazy_component() {
                // This matches FP's error message
                return Err(make_error_1014(
                    activation,
                    Error1014Type::VerifyError,
                    AvmString::new_utf8(mc, "[]"),
                ));
            }

            let resolved_type = activation
                .domain()
                .get_class(activation.context, &pooled_type_name)
                .ok_or_else(|| {
                    make_error_1014(
                        activation,
                        Error1014Type::VerifyError,
                        pooled_type_name.to_qualified_name(mc),
                    )
                })?;

            Some(resolved_type)
        };

        let catch_class = if exception.variable_name.0 == 0 {
            None
        } else {
            let pooled_variable_name = method
                .translation_unit()
                .pool_maybe_uninitialized_multiname(activation, exception.variable_name)?;

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
            let variable_name = QName::new(namespaces[0], name);

            Some(Class::for_catch(activation, variable_name)?)
        };

        if !seen_exception_indices.contains(&exception_index) {
            // We need to push an exception because `newcatch` ops can try to read
            // it, but we can give it dummy from/to/target offsets because no code
            // can actually trigger it (and we might not even have valid offsets anyway).
            new_exceptions.push(Exception {
                from_offset: 0,
                to_offset: 0,
                target_offset: 0,
                catch_class,
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

        let new_from_offset = from_offset.unwrap();
        let new_to_offset = to_offset.unwrap();

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
            jump_targets.insert(new_target_offset);
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

        if new_target_offset >= verified_code.len() {
            return Err(make_error_1054(activation));
        }

        new_exceptions.push(Exception {
            from_offset: new_from_offset,
            to_offset: new_to_offset,
            target_offset: new_target_offset,
            catch_class,
            target_class,
        });
    }

    // We have to deal with AbcOp storing branch offsets as i32 offsets, while Op
    // stores them as usize absolute positions. When initially converting AbcOps
    // to Ops, we convert the values without processing them at all. Now we
    // convert them back, and get the correct absolute positions.
    let mut adjust_jump_to_idx = |i, offset, is_jump| -> Result<usize, Error<'gc>> {
        const JUMP_INSTRUCTION_LENGTH: usize = 4;

        let mut byte_offset = idx_to_byte_offset
            .get(i)
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

        Ok(new_idx)
    };

    // Adjust jump offsets from byte-based to idx-based
    for (i, op) in verified_code.iter_mut().enumerate() {
        match op {
            Op::IfFalse { offset } | Op::IfTrue { offset } | Op::Jump { offset } => {
                let adjusted_result = adjust_jump_to_idx(i, *offset as i32, true)?;
                *offset = adjusted_result;

                jump_targets.insert(adjusted_result);
            }
            Op::LookupSwitch(lookup_switch) => {
                for target in lookup_switch
                    .case_offsets
                    .iter()
                    .chain(std::slice::from_ref(&lookup_switch.default_offset))
                {
                    let adjusted_target = adjust_jump_to_idx(i, target.get() as i32, false)?;
                    target.set(adjusted_target);

                    jump_targets.insert(adjusted_target);
                }
            }
            _ => {}
        }
    }

    crate::avm2::optimizer::optimize(
        activation,
        method,
        &mut verified_code,
        &mut new_exceptions,
        resolved_param_config,
        jump_targets,
    )?;

    Ok(VerifiedMethodInfo {
        parsed_code: verified_code,
        exceptions: new_exceptions,
    })
}

fn create_activation_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
) -> Result<Option<Class<'gc>>, Error<'gc>> {
    let translation_unit = method.translation_unit();
    let abc_method = method.method();
    let body = method
        .body()
        .expect("Cannot verify non-native method without body!");

    if abc_method.flags.contains(AbcMethodFlags::NEED_ACTIVATION) {
        let activation_class =
            Class::for_activation(activation, translation_unit, abc_method, body)?;

        Ok(Some(activation_class))
    } else {
        Ok(None)
    }
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
    // `Multiname::from_abc_index` will do constant pool range checks anyway, so
    // don't perform an extra one here
    translation_unit.pool_maybe_uninitialized_multiname(activation, index)
}

fn pool_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<String>,
) -> Result<AvmAtom<'gc>, Error<'gc>> {
    if index.0 == 0 {
        return Err(make_error_1032(activation, 0));
    }

    translation_unit.pool_string(index.0, activation.strings())
}

fn pool_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<AbcClass>,
) -> Result<Class<'gc>, Error<'gc>> {
    translation_unit.load_class(index.0, activation)
}

fn pool_namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<AbcNamespace>,
) -> Result<Namespace<'gc>, Error<'gc>> {
    translation_unit.pool_namespace(activation, index)
}

fn pool_method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    index: Index<AbcMethod>,
    is_function: bool,
) -> Result<Method<'gc>, Error<'gc>> {
    translation_unit.load_method(index, is_function, activation)
}

fn lookup_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    multiname_index: Index<AbcMultiname>,
) -> Result<Class<'gc>, Error<'gc>> {
    let multiname = pool_multiname(activation, translation_unit, multiname_index)?;

    if multiname.has_lazy_component() {
        // This matches FP's error message
        return Err(make_error_1014(
            activation,
            Error1014Type::VerifyError,
            AvmString::new_utf8(activation.gc(), "[]"),
        ));
    }

    activation
        .domain()
        .get_class(activation.context, &multiname)
        .ok_or_else(|| {
            make_error_1014(
                activation,
                Error1014Type::VerifyError,
                multiname.to_qualified_name(activation.gc()),
            )
        })
}

fn translate_op<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    max_locals: u32,
    activation_class: Option<Class<'gc>>,
    resolved_return_type: Option<Class<'gc>>,
    op: AbcOp,
) -> Result<(Op<'gc>, Option<Op<'gc>>), Error<'gc>> {
    let translation_unit = method.translation_unit();

    // Some quick verifications
    match op {
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
            } else if index_register == object_register {
                return Err(make_error_1124(activation));
            }
        }

        // Misc opcode verification
        AbcOp::CallMethod { index, .. } => {
            return Err(if index == 0 {
                make_error_1072(activation)
            } else {
                make_error_1051(activation)
            });
        }

        AbcOp::FindDef { index } | AbcOp::GetLex { index } => {
            let multiname =
                translation_unit.pool_maybe_uninitialized_multiname(activation, index)?;

            if multiname.has_lazy_component() {
                return Err(make_error_1078(activation));
            }
        }

        AbcOp::GetOuterScope { index } => {
            if activation.outer().get(index as usize).is_none() {
                return Err(make_error_1019(activation, None));
            }
        }

        AbcOp::GetSlot { index }
        | AbcOp::SetSlot { index }
        | AbcOp::GetGlobalSlot { index }
        | AbcOp::SetGlobalSlot { index } => {
            if index == 0 {
                return Err(make_error_1026(activation, 0, None, None));
            }
        }

        _ => {}
    }

    let op = match op {
        AbcOp::PushByte { value } => Op::PushShort {
            value: value as i8 as i16,
        },
        AbcOp::PushDouble { value } => {
            let value = pool_double(activation, translation_unit, value)?;

            Op::PushDouble { value }
        }
        AbcOp::PushFalse => Op::PushFalse,
        AbcOp::PushInt { value } => {
            let value = pool_int(activation, translation_unit, value)?;

            Op::PushInt { value }
        }
        AbcOp::PushNamespace { value } => {
            let namespace = pool_namespace(activation, translation_unit, value)?;

            Op::PushNamespace { namespace }
        }
        AbcOp::PushNaN => Op::PushDouble { value: f64::NAN },
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
        AbcOp::CallStatic { index, num_args } => {
            let method = pool_method(activation, translation_unit, index, false)?;

            Op::CallStatic { method, num_args }
        }
        AbcOp::CallSuper { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::CallSuper {
                multiname,
                num_args,
            }
        }
        AbcOp::CallSuperVoid { index, num_args } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            // CallSuperVoid is split into two ops
            return Ok((
                Op::CallSuper {
                    multiname,
                    num_args,
                },
                Some(Op::Pop),
            ));
        }
        AbcOp::ReturnValue => Op::ReturnValue {
            return_type: resolved_return_type,
        },
        AbcOp::ReturnVoid => Op::ReturnVoid {
            return_type: resolved_return_type,
        },
        AbcOp::GetProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            if !multiname.has_lazy_component() {
                Op::GetPropertyStatic { multiname }
            } else if multiname.has_lazy_name() && !multiname.has_lazy_ns() {
                // The fast-path path usually doesn't activate when the public
                // namespace isn't included in the multiname's namespace set.
                // However, it does still activate when this Activation is
                // running in interpreter mode
                if multiname.valid_dynamic_name() || activation.is_interpreter() {
                    Op::GetPropertyFast { multiname }
                } else {
                    Op::GetPropertySlow { multiname }
                }
            } else {
                Op::GetPropertySlow { multiname }
            }
        }
        AbcOp::SetProperty { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            if !multiname.has_lazy_component() {
                Op::SetPropertyStatic { multiname }
            } else if multiname.has_lazy_name() && !multiname.has_lazy_ns() {
                // The fast-path path usually doesn't activate when the public
                // namespace isn't included in the multiname's namespace set.
                // However, it does still activate when this Activation is
                // running in interpreter mode
                if multiname.valid_dynamic_name() || activation.is_interpreter() {
                    Op::SetPropertyFast { multiname }
                } else {
                    Op::SetPropertySlow { multiname }
                }
            } else {
                Op::SetPropertySlow { multiname }
            }
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
        AbcOp::NewCatch { index } => Op::NewCatch {
            index: index.0 as usize,
        },
        AbcOp::PushWith => Op::PushWith,
        AbcOp::PopScope => Op::PopScope,
        AbcOp::GetOuterScope { index } => Op::GetOuterScope {
            index: index as usize,
        },
        AbcOp::GetScopeObject { index } => Op::GetScopeObject {
            index: index as usize,
        },
        AbcOp::GetGlobalScope => {
            // GetGlobalScope is equivalent to either GetScopeObject or GetOuterScope,
            // depending on the outer scope stack size. Do this check here in the
            // verifier instead of doing it at runtime.

            if activation.outer().is_empty() {
                Op::GetScopeObject { index: 0 }
            } else {
                Op::GetOuterScope { index: 0 }
            }
        }
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
        AbcOp::GetLex { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            // GetLex is split into two ops; multiname is guaranteed static
            return Ok((
                Op::FindPropStrict { multiname },
                Some(Op::GetPropertyStatic { multiname }),
            ));
        }
        AbcOp::GetDescendants { index } => {
            let multiname = pool_multiname(activation, translation_unit, index)?;

            Op::GetDescendants { multiname }
        }
        // Turn 1-based representation into 0-based representation
        AbcOp::GetSlot { index } => Op::GetSlot { index: index - 1 },
        AbcOp::SetSlot { index } => Op::SetSlot { index: index - 1 },
        AbcOp::GetGlobalSlot { index } => {
            let first_op = if activation.outer().is_empty() {
                Op::GetScopeObject { index: 0 }
            } else {
                Op::GetOuterScope { index: 0 }
            };

            // GetGlobalSlot is split into two ops
            return Ok((first_op, Some(Op::GetSlot { index })));
        }
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
        AbcOp::NewActivation => {
            if let Some(activation_class) = activation_class {
                Op::NewActivation { activation_class }
            } else {
                // When a method's flags don't include NEED_ACTIVATION, we
                // purposefully don't construct an `activation_class` in
                // `create_activation_class`, which results in the
                // `activation_class` being passed to `translate_op` being None.
                // This results in this VerifyError being thrown upon
                // encountering any `newactivation` op in the bytecode.

                return Err(make_error_1113(activation));
            }
        }
        AbcOp::NewObject { num_args } => Op::NewObject { num_args },
        AbcOp::NewFunction { index } => {
            let method = pool_method(activation, translation_unit, index, true)?;

            Op::NewFunction { method }
        }
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
        AbcOp::Jump { offset } => Op::Jump {
            offset: offset as usize,
        },
        AbcOp::IfTrue { offset } => Op::IfTrue {
            offset: offset as usize,
        },
        AbcOp::IfFalse { offset } => Op::IfFalse {
            offset: offset as usize,
        },
        AbcOp::IfStrictEq { offset } => {
            return Ok((
                Op::StrictEquals,
                Some(Op::IfTrue {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfStrictNe { offset } => {
            return Ok((
                Op::StrictEquals,
                Some(Op::IfFalse {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfEq { offset } => {
            return Ok((
                Op::Equals,
                Some(Op::IfTrue {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfNe { offset } => {
            return Ok((
                Op::Equals,
                Some(Op::IfFalse {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfGe { offset } => {
            return Ok((
                Op::GreaterEquals,
                Some(Op::IfTrue {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfGt { offset } => {
            return Ok((
                Op::GreaterThan,
                Some(Op::IfTrue {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfLe { offset } => {
            return Ok((
                Op::LessEquals,
                Some(Op::IfTrue {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfLt { offset } => {
            return Ok((
                Op::LessThan,
                Some(Op::IfTrue {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfNge { offset } => {
            return Ok((
                Op::GreaterEquals,
                Some(Op::IfFalse {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfNgt { offset } => {
            return Ok((
                Op::GreaterThan,
                Some(Op::IfFalse {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfNle { offset } => {
            return Ok((
                Op::LessEquals,
                Some(Op::IfFalse {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
        AbcOp::IfNlt { offset } => {
            return Ok((
                Op::LessThan,
                Some(Op::IfFalse {
                    offset: (offset - 1) as usize,
                }),
            ));
        }
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
            let class = lookup_class(activation, translation_unit, index)?;

            Op::IsType { class }
        }
        AbcOp::IsTypeLate => Op::IsTypeLate,
        AbcOp::AsType { type_name } => {
            let class = lookup_class(activation, translation_unit, type_name)?;

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
        AbcOp::LookupSwitch(lookup_switch) => {
            let created_lookup_switch = LookupSwitch {
                default_offset: Cell::new(lookup_switch.default_offset as usize),
                case_offsets: lookup_switch
                    .case_offsets
                    .iter()
                    .map(|o| Cell::new(*o as usize))
                    .collect(),
            };

            Op::LookupSwitch(Gc::new(activation.gc(), created_lookup_switch))
        }
        AbcOp::Coerce { index } => {
            let class = lookup_class(activation, translation_unit, index)?;

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
    };

    Ok((op, None))
}
