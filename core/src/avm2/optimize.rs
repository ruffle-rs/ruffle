use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::BytecodeMethod;
use crate::avm2::multiname::Multiname;
use crate::avm2::object::ClassObject;
use crate::avm2::op::Op;
use crate::avm2::property::Property;

use gc_arena::{Gc, GcCell};
use std::collections::HashSet;
use swf::avm2::types::Index;

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
            .class_object()
            .map(ValueType::Class)
            .unwrap_or(ValueType::Any);
    }

    fn set_any(&mut self, index: usize) {
        self.0[index] = ValueType::Any;
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
                .class_object()
                .map(ValueType::Class)
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

    pub fn pop_for_multiname(&mut self, multiname: Gc<'gc, Multiname<'gc>>) {
        if multiname.has_lazy_name() {
            self.0.pop();
        }
        if multiname.has_lazy_ns() {
            self.0.pop();
        }
    }

    fn popn(&mut self, count: u32) {
        for _ in 0..count {
            self.pop();
        }
    }

    fn clear(&mut self) {
        self.0 = Vec::new();
    }
}

pub fn optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: &BytecodeMethod<'gc>,
    code: &mut Vec<Op>,
    jump_targets: HashSet<i32>,
) {
    // These make the code less readable
    #![allow(clippy::manual_filter)]
    #![allow(clippy::single_match)]

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

    // TODO: Store these argument types somewhere on the function so they don't
    // have to be re-resolved every function call
    let mut argument_types = Vec::new();
    for argument in &method.signature {
        let type_name = &argument.param_type_name;

        let argument_type = if !type_name.has_lazy_component() {
            activation
                .domain()
                .get_class(type_name, activation.context.gc_context)
        } else {
            None
        };

        argument_types.push(argument_type);
    }

    // Initial set of local types
    let mut initial_local_types = Locals::new(method_body.num_locals as usize);
    if let Some(this_class) = this_class {
        initial_local_types.set_class_object(0, this_class);
    }

    let mut i = 1;
    for argument_type in argument_types {
        if let Some(argument_type) = argument_type {
            initial_local_types.set_class(i, argument_type);
        }
        i += 1;
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

    macro_rules! stack_pop_multiname {
        ($index: expr) => {{
            let multiname = method
                .translation_unit()
                // note: ideally this should be a VerifyError here or earlier
                .pool_maybe_uninitialized_multiname(*$index, &mut activation.context);

            if let Ok(multiname) = multiname {
                stack.pop_for_multiname(multiname);
                Some(multiname)
            } else {
                None
            }
        }};
    }

    let mut scope_stack = Stack::new();
    let mut local_types = initial_local_types.clone();

    for (i, op) in code.iter_mut().enumerate() {
        if jump_targets.contains(&(i as i32)) {
            stack.clear();
            scope_stack.clear();
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
            Op::Equals
            | Op::StrictEquals
            | Op::LessEquals
            | Op::LessThan
            | Op::GreaterThan
            | Op::GreaterEquals => {
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
            Op::LShift => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::RShift => {
                stack.pop();
                stack.pop();
                stack.push_any();
            }
            Op::URShift => {
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
                stack.popn(*num_args);

                stack.push_class_object(activation.avm2().classes().array);
            }
            Op::NewObject { num_args } => {
                stack.popn(*num_args * 2);

                stack.push_class_object(activation.avm2().classes().object);
            }
            Op::NewFunction { .. } => {
                stack.push_class_object(activation.avm2().classes().function);
            }
            Op::NewClass { .. } => {
                stack.push_class_object(activation.avm2().classes().class);
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
                stack.popn(*num_types);

                stack.pop();

                stack.push_any();
            }
            Op::AsTypeLate => {
                stack.pop();
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
                if resolved_type.is_some() && matches!(stack_value, Some(ValueType::Null)) {
                    *op = Op::Nop;
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
                let stack_value = stack.pop();
                if let Some(value) = stack_value {
                    scope_stack.push(value);
                }
            }
            Op::PushWith => {
                // TODO: Some way to mark scopes as with-scope vs normal-scope?
                let stack_value = stack.pop();
                if let Some(value) = stack_value {
                    scope_stack.push(value);
                }
            }
            Op::PopScope => {
                scope_stack.pop();
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
            Op::Kill { index } => {
                if (*index as usize) < local_types.len() {
                    local_types.set_any(*index as usize);
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

                let multiname = stack_pop_multiname!(name_index);
                let stack_value = stack.pop();

                if let Some(multiname) = multiname {
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
                    }
                    // `stack_pop_multiname` handled lazy
                }

                if !stack_push_done {
                    stack.push_any();
                }
            }
            Op::GetSlot { .. } => {
                stack.pop();

                // Avoid handling type for now
                stack.push_any();
            }
            Op::InitProperty { index: name_index } => {
                stack.pop();

                let multiname = stack_pop_multiname!(name_index);
                let stack_value = stack.pop();

                if let Some(multiname) = multiname {
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
                    }
                    // `stack_pop_multiname` handled lazy
                }
            }
            Op::SetProperty { index: name_index } => {
                stack.pop();

                let multiname = stack_pop_multiname!(name_index);
                let stack_value = stack.pop();

                if let Some(multiname) = multiname {
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
                    }
                    // `stack_pop_multiname` handled lazy
                }
            }
            Op::Construct { num_args } => {
                // Arguments
                stack.popn(*num_args);

                stack.pop();

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::ConstructSuper { num_args } => {
                // Arguments
                stack.popn(*num_args);

                // Then receiver.
                stack.pop();
            }
            Op::ConstructProp {
                index: name_index,
                num_args,
            } => {
                // Arguments
                stack.popn(*num_args);

                stack_pop_multiname!(name_index);

                // Then receiver.
                stack.pop();

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::CallProperty {
                index: name_index,
                num_args,
            } => {
                // Arguments
                stack.popn(*num_args);

                let multiname = stack_pop_multiname!(name_index);

                // Then receiver.
                let stack_value = stack.pop();

                if let Some(multiname) = multiname {
                    if !multiname.has_lazy_component() {
                        if let Some(ValueType::Class(class)) = stack_value {
                            if !class.inner_class_definition().read().is_interface() {
                                match class.instance_vtable().get_trait(&multiname) {
                                    Some(Property::Method { disp_id }) => {
                                        *op = Op::CallMethod {
                                            num_args: *num_args,
                                            index: Index::new(disp_id),
                                        };
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    // `stack_pop_multiname` handled lazy
                }

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::CallPropVoid { .. } => {
                // Avoid handling for now
                stack.clear();
            }
            Op::Call { num_args } => {
                // Arguments
                stack.popn(*num_args);

                stack.pop();

                // Avoid checking return value for now
                stack.push_any();
            }
            Op::GetGlobalScope => {
                // Avoid handling for now
                stack.push_any();
            }
            Op::NewActivation => {
                // Avoid handling for now
                stack.push_any();
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
            Op::ReturnVoid | Op::ReturnValue | Op::Jump { .. } | Op::LookupSwitch(_) => {
                stack.clear();
                scope_stack.clear();
                local_types = initial_local_types.clone();
            }
            _ => {
                stack.clear();
                scope_stack.clear();
                local_types = initial_local_types.clone();
            }
        }
    }
}
