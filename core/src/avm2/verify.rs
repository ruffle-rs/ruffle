use crate::avm2::class::Class;
use crate::avm2::error::{
    make_error_1025, make_error_1032, make_error_1054, make_error_1107, verify_error,
};
use crate::avm2::method::BytecodeMethod;
use crate::avm2::object::ClassObject;
use crate::avm2::op::Op;
use crate::avm2::property::Property;
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Activation, Error};
use gc_arena::GcCell;
use std::collections::{HashMap, HashSet};
use swf::avm2::read::Reader;
use swf::avm2::types::{
    Index, MethodFlags as AbcMethodFlags, Multiname as AbcMultiname, Op as AbcOp,
};
use swf::error::Error as AbcReadError;

pub struct VerifiedMethodInfo {
    pub parsed_code: Vec<Op>,
    pub exceptions: Vec<Exception>,
}

pub struct Exception {
    pub from_offset: u32,
    pub to_offset: u32,
    pub target_offset: u32,

    pub variable_name: Index<AbcMultiname>,
    pub type_name: Index<AbcMultiname>,
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
        // FIXME: This is actually wrong, we should be using the byte offsets in
        // `Activation::handle_err`, not the opcode offsets. avmplus allows for from/to
        // (but not targets) that aren't on a opcode, and some obfuscated SWFs have them.
        // FFDEC handles them correctly, stepping forward byte-by-byte until it reaches
        // the next opcode. This does the same (stepping byte-by-byte), but it would
        // be better to directly use the byte offsets when handling exceptions.
        let mut from_offset = None;

        let mut offs = 0;
        while from_offset.is_none() {
            from_offset = byte_offset_to_idx
                .get(&((exception.from_offset + offs) as i32))
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
                .get(&((exception.to_offset + offs) as i32))
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

        // FIXME: Use correct error instead of `.unwrap()`
        let new_target_offset = byte_offset_to_idx
            .get(&(exception.target_offset as i32))
            .copied()
            .unwrap();

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
            variable_name: exception.variable_name,
            type_name: exception.type_name,
        });

        if ops_can_throw_error(&new_code[new_from_offset as usize..new_to_offset as usize]) {
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

fn resolve_jump_target<'gc>(
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

    Ok(new_idx)
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

    while let Some(mut i) = worklist.pop() {
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
                    let op_idx = resolve_jump_target(
                        activation,
                        i,
                        *offset,
                        true,
                        idx_to_byte_offset,
                        byte_offset_to_idx,
                    )?;

                    if !verified_blocks.iter().any(|o| *o == op_idx) {
                        worklist.push(op_idx);
                        verified_blocks.push(op_idx);
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
                    // A LookupSwitch is terminal
                    let default_idx = resolve_jump_target(
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
                        let case_idx = resolve_jump_target(
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
                        .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);

                    if let Ok(multiname) = multiname {
                        if multiname.has_lazy_component() {
                            // This matches FP's error message
                            return Err(Error::AvmError(verify_error(
                                activation,
                                "Error #1014: Class [] could not be found.",
                                1014,
                            )?));
                        }

                        activation
                            .domain()
                            .get_class(&multiname, activation.context.gc_context)
                            .ok_or_else(|| {
                                Error::AvmError(
                                    verify_error(
                                        activation,
                                        &format!(
                                            "Error #1014: Class {} could not be found.",
                                            multiname
                                                .to_qualified_name(activation.context.gc_context)
                                        ),
                                        1014,
                                    )
                                    .expect("Error should construct"),
                                )
                            })?;
                    } else {
                        return Err(make_error_1032(activation, name_index.0));
                    }
                }

                _ => {}
            }

            i += 1;
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum ValueType<'gc> {
    // Either a class, or null.
    Class(ClassObject<'gc>),
    Int,
    Uint,
    Number,
    Boolean,
    Null,
    Any,
}

#[derive(Clone, Debug)]
struct Locals<'gc>(Vec<ValueType<'gc>>);

impl<'gc> Locals<'gc> {
    fn new(size: usize) -> Self {
        Self(vec![ValueType::Any; size])
    }

    fn set_class_object(&mut self, index: usize, class: ClassObject<'gc>) {
        self.0[index] = ValueType::Class(class);
    }

    fn set_class(&mut self, index: usize, class: GcCell<'gc, Class<'gc>>) {
        // FIXME: Getting the ClassObject this way should be unnecessary
        // after the ClassObject refactor
        self.0[index] = class
            .read()
            .class_objects()
            .get(0)
            .map(|c| ValueType::Class(*c))
            .unwrap_or(ValueType::Any);
    }

    fn set_int(&mut self, index: usize) {
        self.0[index] = ValueType::Int;
    }

    fn set_uint(&mut self, index: usize) {
        self.0[index] = ValueType::Uint;
    }

    fn set_number(&mut self, index: usize) {
        self.0[index] = ValueType::Number;
    }

    fn set_boolean(&mut self, index: usize) {
        self.0[index] = ValueType::Boolean;
    }

    fn set_any(&mut self, index: usize) {
        self.0[index] = ValueType::Any;
    }

    fn set_null(&mut self, index: usize) {
        self.0[index] = ValueType::Null;
    }

    fn set(&mut self, index: usize, value: ValueType<'gc>) {
        self.0[index] = value;
    }

    fn at(&self, index: usize) -> Option<ValueType<'gc>> {
        self.0.get(index).copied()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug)]
struct Stack<'gc>(Vec<ValueType<'gc>>);

impl<'gc> Stack<'gc> {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push_class_object(&mut self, class: ClassObject<'gc>) {
        self.0.push(ValueType::Class(class));
    }

    fn push_class(&mut self, class: GcCell<'gc, Class<'gc>>) {
        // FIXME: Getting the ClassObject this way should be unnecessary
        // after the ClassObject refactor
        self.0.push(
            class
                .read()
                .class_objects()
                .get(0)
                .map(|c| ValueType::Class(*c))
                .unwrap_or(ValueType::Any),
        );
    }

    fn push_int(&mut self) {
        self.0.push(ValueType::Int);
    }

    fn push_uint(&mut self) {
        self.0.push(ValueType::Uint);
    }

    fn push_number(&mut self) {
        self.0.push(ValueType::Number);
    }

    fn push_boolean(&mut self) {
        self.0.push(ValueType::Boolean);
    }

    fn push_any(&mut self) {
        self.0.push(ValueType::Any);
    }

    fn push_null(&mut self) {
        self.0.push(ValueType::Null);
    }

    fn push(&mut self, value: ValueType<'gc>) {
        self.0.push(value);
    }

    fn pop(&mut self) -> Option<ValueType<'gc>> {
        self.0.pop()
    }

    fn clear(&mut self) {
        self.0 = Vec::new();
    }
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

    let mut output = crate::string::WString::new();
    activation
        .avm2()
        .call_stack()
        .read()
        .clone()
        .display(&mut output);
    println!("beginning optimizing, call stack: {}", output);

    let method_body = method
        .body()
        .expect("Cannot verify non-native method without body!");

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
    let mut initial_local_types = Locals::new(method_body.num_locals as usize);
    if let Some(this_class) = this_class {
        initial_local_types.set_class_object(0, this_class);
    }

    // Logic to only allow for type-based optimizations on types that
    // we're absolutely sure about- invalidate the local register's
    // known type if any other register-modifying opcodes mention them
    // anywhere else in the function.
    for op in &*code {
        match op {
            Op::SetLocal { index }
            | Op::Kill { index }
            | Op::IncLocal { index }
            | Op::IncLocalI { index }
            | Op::DecLocal { index }
            | Op::DecLocalI { index } => {
                if (*index as usize) < initial_local_types.len() {
                    initial_local_types.set_any(*index as usize);
                }
            }
            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                if (*object_register as usize) < initial_local_types.len() {
                    initial_local_types.set_any(*object_register as usize);
                }
                if (*index_register as usize) < initial_local_types.len() {
                    initial_local_types.set_any(*index_register as usize);
                }
            }
            _ => {}
        }
    }

    let mut stack = Stack::new();
    let mut local_types = initial_local_types.clone();

    for (i, op) in code.iter_mut().enumerate() {
        if jump_targets.contains(&(i as i32)) {
            stack.clear();
            local_types = initial_local_types.clone();
        }

        match op {
            Op::CoerceB => {
                let stack_value = stack.pop();
                if matches!(stack_value, Some(ValueType::Boolean)) {
                    *op = Op::Nop;
                }
                stack.push_boolean();
            }
            Op::CoerceD => {
                let stack_value = stack.pop();
                if matches!(stack_value, Some(ValueType::Number)) {
                    *op = Op::Nop;
                }
                stack.push_number();
            }
            Op::CoerceI => {
                let stack_value = stack.pop();
                if matches!(stack_value, Some(ValueType::Int))
                    || matches!(stack_value, Some(ValueType::Uint))
                {
                    *op = Op::Nop;
                }
                stack.push_int();
            }
            Op::CoerceU => {
                let stack_value = stack.pop();
                if matches!(stack_value, Some(ValueType::Uint)) {
                    *op = Op::Nop;
                }
                stack.push_uint();
            }
            Op::CoerceA => {
                stack.pop();
                stack.push_any();
            }
            Op::CoerceS => {
                let stack_value = stack.pop();
                if matches!(stack_value, Some(ValueType::Null)) {
                    *op = Op::Nop;
                }
                stack.push_class_object(activation.avm2().classes().string);
            }
            Op::Equals | Op::LessEquals | Op::LessThan | Op::GreaterThan | Op::GreaterEquals => {
                stack.pop();
                stack.pop();
                stack.push_boolean();
            }
            Op::Not => {
                stack.pop();
                stack.push_boolean();
            }
            Op::PushTrue | Op::PushFalse => {
                stack.push_boolean();
            }
            Op::PushNull => {
                stack.push_null();
            }
            Op::PushUndefined => {
                stack.push_any();
            }
            Op::PushNaN => {
                stack.push_number();
            }
            Op::PushByte { value } => {
                if *value >= 0 {
                    stack.push_uint();
                } else {
                    stack.push_int();
                }
            }
            Op::PushShort { value } => {
                if *value >= 0 {
                    stack.push_uint();
                } else {
                    stack.push_int();
                }
            }
            Op::PushInt { value } => {
                if *value < -(1 << 28) || *value >= (1 << 28) {
                    stack.push_number();
                } else if *value >= 0 {
                    stack.push_uint();
                } else {
                    stack.push_int();
                }
            }
            Op::DecrementI => {
                // This doesn't give any Number-int guarantees
                stack.pop();
                stack.push_any();
            }
            Op::IncrementI => {
                // This doesn't give any Number-int guarantees
                stack.pop();
                stack.push_any();
            }
            Op::DecLocalI { index } => {
                if (*index as usize) < local_types.len() {
                    // This doesn't give any Number-int guarantees
                    local_types.set_any(*index as usize);
                }
            }
            Op::IncLocalI { index } => {
                if (*index as usize) < local_types.len() {
                    // This doesn't give any Number-int guarantees
                    local_types.set_any(*index as usize);
                }
            }
            Op::Increment => {
                stack.pop();
                stack.push_number();
            }
            Op::Decrement => {
                stack.pop();
                stack.push_number();
            }
            Op::Negate => {
                stack.pop();
                stack.push_number();
            }
            Op::AddI => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::SubtractI => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::MultiplyI => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::Add => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::Subtract => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::Multiply => {
                stack.pop();
                stack.pop();

                // NOTE: In our current implementation, this is guaranteed,
                // but it may not be after correctness fixes to match avmplus
                stack.push_number();
            }
            Op::Divide => {
                stack.pop();
                stack.pop();

                // NOTE: In our current implementation, this is guaranteed,
                // but it may not be after correctness fixes to match avmplus
                stack.push_number();
            }
            Op::Modulo => {
                stack.pop();
                stack.pop();

                // NOTE: In our current implementation, this is guaranteed,
                // but it may not be after correctness fixes to match avmplus
                stack.push_number();
            }
            Op::BitNot => {
                stack.pop();
                stack.push_any();
            }
            Op::BitAnd => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::BitOr => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::BitXor => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::PushDouble { .. } => {
                stack.push_number();
            }
            Op::PushString { .. } => {
                stack.push_class_object(activation.avm2().classes().string);
            }
            Op::NewArray { num_args } => {
                for _ in 0..num_args {
                    stack.pop();
                }

                stack.push_class_object(activation.avm2().classes().array);
            }
            Op::NewFunction { .. } => {
                stack.push_class_object(activation.avm2().classes().function);
            }
            Op::IsType { .. } => {
                stack.pop();
                stack.push_boolean();
            }
            Op::IsTypeLate => {
                stack.pop();
                stack.pop();
                stack.push_boolean();
            }
            Op::ApplyType { num_types } => {
                for _ in 0..*num_types {
                    stack.pop();
                }

                stack.pop();

                stack.push_any();
            }
            Op::AsType {
                type_name: name_index,
            } => {
                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);

                let resolved_type = if let Ok(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        activation
                            .domain()
                            .get_class(&multiname, activation.context.gc_context)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let stack_value = stack.pop();
                if resolved_type.is_some() {
                    if matches!(stack_value, Some(ValueType::Null)) {
                        *op = Op::Nop;
                    }
                }

                if let Some(resolved_type) = resolved_type {
                    stack.push_class(resolved_type);
                } else {
                    stack.push_any();
                }
            }
            Op::Coerce { index: name_index } => {
                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);

                let resolved_type = if let Ok(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        activation
                            .domain()
                            .get_class(&multiname, activation.context.gc_context)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let stack_value = stack.pop();
                if let Some(resolved_type) = resolved_type {
                    if matches!(stack_value, Some(ValueType::Null)) {
                        // As long as this Coerce isn't coercing to one
                        // of these special classes, we can remove it.
                        if !GcCell::ptr_eq(
                            resolved_type,
                            activation.avm2().classes().int.inner_class_definition(),
                        ) && !GcCell::ptr_eq(
                            resolved_type,
                            activation.avm2().classes().uint.inner_class_definition(),
                        ) && !GcCell::ptr_eq(
                            resolved_type,
                            activation.avm2().classes().number.inner_class_definition(),
                        ) && !GcCell::ptr_eq(
                            resolved_type,
                            activation.avm2().classes().boolean.inner_class_definition(),
                        ) && !GcCell::ptr_eq(
                            resolved_type,
                            activation.avm2().classes().void.inner_class_definition(),
                        ) {
                            *op = Op::Nop;
                        }
                    } else if let Some(ValueType::Class(class_object)) = stack_value {
                        if GcCell::ptr_eq(resolved_type, class_object.inner_class_definition()) {
                            *op = Op::Nop;
                        }
                    }

                    stack.push_class(resolved_type);
                } else {
                    stack.push_any();
                }
            }
            Op::PushScope => {
                stack.pop();
            }
            Op::Pop => {
                stack.pop();
            }
            Op::Dup => {
                let stack_value = stack.pop();
                if let Some(stack_value) = stack_value {
                    stack.push(stack_value);
                    stack.push(stack_value);
                }
            }
            Op::SetLocal { index } => {
                let stack_value = stack.pop();
                if (*index as usize) < local_types.len() {
                    if let Some(stack_value) = stack_value {
                        local_types.set(*index as usize, stack_value);
                    } else {
                        local_types.set_any(*index as usize);
                    }
                }
            }
            Op::GetLocal { index } => {
                let local_type = local_types.at(*index as usize);
                if let Some(local_type) = local_type {
                    stack.push(local_type);
                } else {
                    stack.push_any();
                }
            }
            Op::GetLex { .. } => {
                stack.push_any();
            }
            Op::FindPropStrict { index: name_index } => {
                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);

                if let Ok(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        stack.push_any();
                    } else {
                        // Avoid handling lazy for now
                        stack.clear();
                    }
                }
            }
            Op::FindProperty { .. } => {
                // Avoid handling for now
                stack.clear();
            }
            Op::GetProperty { index: name_index } => {
                let mut stack_push_done = false;
                let stack_value = stack.pop();

                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);
                if let Ok(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        if let Some(ValueType::Class(class)) = stack_value {
                            if !class.inner_class_definition().read().is_interface() {
                                match class.instance_vtable().get_trait(&multiname) {
                                    Some(Property::Slot { slot_id })
                                    | Some(Property::ConstSlot { slot_id }) => {
                                        *op = Op::GetSlot { index: slot_id };

                                        let mut value_class =
                                            class.instance_vtable().slot_classes()
                                                [slot_id as usize];
                                        let resolved_value_class =
                                            value_class.get_class(activation);
                                        if let Ok(class) = resolved_value_class {
                                            stack_push_done = true;

                                            if let Some(class) = class {
                                                stack.push_class(class);
                                            } else {
                                                stack.push_any();
                                            }
                                        }

                                        class.instance_vtable().set_slot_class(
                                            activation.context.gc_context,
                                            slot_id as usize,
                                            value_class,
                                        );
                                    }
                                    Some(Property::Virtual { get: Some(get), .. }) => {
                                        *op = Op::CallMethod {
                                            num_args: 0,
                                            index: Index::new(get),
                                        };
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        // Avoid handling lazy for now
                        stack.clear();
                    }
                }

                if !stack_push_done {
                    stack.push_any();
                }
            }
            Op::InitProperty { index: name_index } => {
                stack.pop();
                let stack_value = stack.pop();

                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);
                if let Ok(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        if let Some(ValueType::Class(class)) = stack_value {
                            if !class.inner_class_definition().read().is_interface() {
                                match class.instance_vtable().get_trait(&multiname) {
                                    Some(Property::Slot { slot_id })
                                    | Some(Property::ConstSlot { slot_id }) => {
                                        *op = Op::SetSlot { index: slot_id };
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        // Avoid handling lazy for now
                        stack.clear();
                    }
                }
            }
            Op::SetProperty { index: name_index } => {
                stack.pop();
                let stack_value = stack.pop();

                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);
                if let Ok(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        if let Some(ValueType::Class(class)) = stack_value {
                            if !class.inner_class_definition().read().is_interface() {
                                match class.instance_vtable().get_trait(&multiname) {
                                    Some(Property::Slot { slot_id }) => {
                                        *op = Op::SetSlot { index: slot_id };
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        // Avoid handling lazy for now
                        stack.clear();
                    }
                }
            }
            Op::Construct { num_args } => {
                // Arguments
                for _ in 0..*num_args {
                    stack.pop();
                }

                // Then receiver.
                stack.pop();

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::ConstructSuper { num_args } => {
                // Arguments
                for _ in 0..*num_args {
                    stack.pop();
                }

                // Then receiver.
                stack.pop();
            }
            Op::ConstructProp {
                index: name_index,
                num_args,
            } => {
                // Arguments
                for _ in 0..*num_args {
                    stack.pop();
                }

                // Then receiver.
                stack.pop();

                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);
                if let Ok(multiname) = multiname {
                    if multiname.has_lazy_component() {
                        // Avoid handling lazy for now
                        stack.clear();
                    }
                }

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::CallProperty {
                index: name_index,
                num_args,
            } => {
                // Arguments
                for _ in 0..*num_args {
                    stack.pop();
                }

                // Then receiver.
                stack.pop();

                let multiname = method
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(*name_index, &mut activation.context);
                if let Ok(multiname) = multiname {
                    if multiname.has_lazy_component() {
                        // Avoid handling lazy for now
                        stack.clear();
                    }
                }

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::CallPropVoid { .. } => {
                // Avoid handling for now
                stack.clear();
            }
            Op::Nop => {}
            Op::DebugFile { .. } => {}
            Op::DebugLine { .. } => {}
            Op::Debug { .. } => {}
            Op::IfTrue { .. } | Op::IfFalse { .. } => {
                stack.pop();
            }
            Op::IfStrictEq { .. }
            | Op::IfStrictNe { .. }
            | Op::IfEq { .. }
            | Op::IfNe { .. }
            | Op::IfGe { .. }
            | Op::IfGt { .. }
            | Op::IfLe { .. }
            | Op::IfLt { .. }
            | Op::IfNge { .. }
            | Op::IfNgt { .. }
            | Op::IfNle { .. }
            | Op::IfNlt { .. } => {
                stack.pop();
                stack.pop();
            }
            Op::Si8 => {
                stack.pop();
                stack.pop();
            }
            Op::Li8 => {
                stack.pop();
                stack.push_int();
            }
            _ => {
                stack.clear();
                local_types = initial_local_types.clone();
            }
        }
    }
}

fn ops_can_throw_error(ops: &[AbcOp]) -> bool {
    for op in ops {
        match op {
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

fn resolve_op<'gc>(
    activation: &mut Activation<'_, 'gc>,
    translation_unit: TranslationUnit<'gc>,
    op: AbcOp,
) -> Result<Op, Error<'gc>> {
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
    })
}
