use crate::avm2::error::verify_error;
use crate::avm2::method::{BytecodeMethod, ResolvedParamConfig};
use crate::avm2::multiname::Multiname;
use crate::avm2::object::TObject;
use crate::avm2::op::Op;
use crate::avm2::property::Property;
use crate::avm2::verify::{Exception, JumpSource};
use crate::avm2::vtable::VTable;
use crate::avm2::{Activation, Class, Error};

use gc_arena::Gc;
use std::collections::{HashMap, HashSet};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum NullState {
    NotNull,
    MaybeNull,
    IsNull,
}

#[derive(Clone, Copy)]
struct OptValue<'gc> {
    // This corresponds to the compile-time assumptions about the type:
    // - primitive types can't be undefined or null,
    // - Object (and any other non-primitive type) is non-undefined, but can be null
    // - None (the * type) can be any value,
    // - a value typed as int can be stored as a Number (and vice versa),
    //   BUT an int-typed value should always pass `is int`
    //   (say, a Value::Number above hardcoded int-range that's still representable as i32).
    // Note that `null is Object` is still `false`. So think of this type more in terms of
    // "could this value be a possible value of `var t: T`"
    pub class: Option<Class<'gc>>,

    pub vtable: Option<VTable<'gc>>,

    // true if the value is guaranteed to be Value::Integer
    // should only be set if class is numeric.
    pub contains_valid_integer: bool,
    // true if the value is guaranteed to be Value::Integer AND is >=0
    // should only be set if class is numeric.
    pub contains_valid_unsigned: bool,

    // TODO: FP actually has a separate `null` type just for this, this can be observed in VerifyErrors
    // (a separate type would also prevent accidental "null int" values)
    pub null_state: NullState,
}
impl<'gc> OptValue<'gc> {
    pub fn any() -> Self {
        Self {
            class: None,
            vtable: None,
            contains_valid_integer: false,
            contains_valid_unsigned: false,
            null_state: NullState::MaybeNull,
        }
    }

    pub fn null() -> Self {
        Self {
            class: None,
            vtable: None,
            null_state: NullState::IsNull,
            ..Self::any()
        }
    }

    pub fn of_type(class: Class<'gc>) -> Self {
        Self {
            class: Some(class),
            vtable: Some(class.vtable()),
            ..Self::any()
        }
    }

    pub fn vtable(self) -> Option<VTable<'gc>> {
        if let Some(class) = self.class {
            if class.is_interface() {
                return None;
            }
        }

        self.vtable
    }

    pub fn is_null(self) -> bool {
        matches!(self.null_state, NullState::IsNull)
    }

    pub fn not_null(self, activation: &mut Activation<'_, 'gc>) -> bool {
        if matches!(self.null_state, NullState::NotNull) {
            return true;
        }

        let class_defs = activation.avm2().class_defs();

        // Primitives are always not-null
        self.class == Some(class_defs.int)
            || self.class == Some(class_defs.uint)
            || self.class == Some(class_defs.number)
            || self.class == Some(class_defs.boolean)
            || self.class == Some(class_defs.void)
    }

    pub fn merged_with(self, other: OptValue<'gc>) -> OptValue<'gc> {
        let mut created_value = OptValue::any();

        // TODO: Also check common superclasses.
        if self.class == other.class {
            created_value.class = self.class;
        }

        if self.vtable == other.vtable {
            created_value.vtable = self.vtable;
        }

        if self.null_state == other.null_state {
            created_value.null_state = self.null_state;
        }

        if self.contains_valid_integer && other.contains_valid_integer {
            created_value.contains_valid_integer = true;
        }

        if self.contains_valid_unsigned && other.contains_valid_unsigned {
            created_value.contains_valid_unsigned = true;
        }

        created_value
    }
}

impl std::fmt::Debug for OptValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("OptValue")
            .field("class", &self.class)
            .field("contains_valid_integer", &self.contains_valid_integer)
            .field("contains_valid_unsigned", &self.contains_valid_unsigned)
            .field("null_state", &self.null_state)
            .finish()
    }
}

#[derive(Clone, Debug)]
struct Locals<'gc>(Vec<OptValue<'gc>>);

impl<'gc> Locals<'gc> {
    fn new(size: usize) -> Self {
        Self(vec![OptValue::any(); size])
    }

    fn set_any(&mut self, index: usize) {
        self.0[index] = OptValue::any();
    }

    fn set(&mut self, index: usize, value: OptValue<'gc>) {
        self.0[index] = value;
    }

    fn at(&self, index: usize) -> OptValue<'gc> {
        self.0[index]
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug)]
struct Stack<'gc>(Vec<OptValue<'gc>>, usize);

impl<'gc> Stack<'gc> {
    fn new(max_height: usize) -> Self {
        Self(Vec::new(), max_height)
    }

    fn push_class(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        class: Class<'gc>,
    ) -> Result<(), Error<'gc>> {
        self.push(activation, OptValue::of_type(class))?;

        Ok(())
    }

    fn push_class_not_null(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        class: Class<'gc>,
    ) -> Result<(), Error<'gc>> {
        let mut value = OptValue::of_type(class);
        value.null_state = NullState::NotNull;

        self.push(activation, value)?;

        Ok(())
    }

    fn push_any(&mut self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        self.push(activation, OptValue::any())?;

        Ok(())
    }

    fn push(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: OptValue<'gc>,
    ) -> Result<(), Error<'gc>> {
        if self.len() >= self.max_height() {
            return Err(Error::AvmError(verify_error(
                activation,
                "Error #1023: Stack overflow occurred.",
                1023,
            )?));
        }

        self.0.push(value);

        Ok(())
    }

    fn pop(&mut self, activation: &mut Activation<'_, 'gc>) -> Result<OptValue<'gc>, Error<'gc>> {
        if self.0.is_empty() {
            return Err(Error::AvmError(verify_error(
                activation,
                "Error #1024: Stack underflow occurred.",
                1024,
            )?));
        }

        Ok(self.0.pop().unwrap())
    }

    pub fn pop_for_multiname(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        multiname: Gc<'gc, Multiname<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if multiname.has_lazy_name() {
            self.pop(activation)?;
        }
        if multiname.has_lazy_ns() {
            self.pop(activation)?;
        }

        Ok(())
    }

    fn popn(&mut self, activation: &mut Activation<'_, 'gc>, count: u32) -> Result<(), Error<'gc>> {
        for _ in 0..count {
            self.pop(activation)?;
        }

        Ok(())
    }

    fn fill_with_any_up_to_len(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        len: usize,
    ) -> Result<(), Error<'gc>> {
        self.0.clear();
        for _ in 0..len {
            self.push(activation, OptValue::any())?;
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn max_height(&self) -> usize {
        self.1
    }

    fn replace_with_any(&mut self) {
        for item in self.0.iter_mut() {
            *item = OptValue::any();
        }
    }
}

#[derive(Clone, Debug)]
struct ScopeStack<'gc>(Vec<(OptValue<'gc>, bool)>, usize);

impl<'gc> ScopeStack<'gc> {
    fn new(max_height: usize) -> Self {
        Self(Vec::new(), max_height)
    }

    fn push(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: OptValue<'gc>,
    ) -> Result<(), Error<'gc>> {
        if self.len() >= self.max_height() {
            return Err(Error::AvmError(verify_error(
                activation,
                "Error #1017: Scope stack overflow occurred.",
                1017,
            )?));
        }

        self.0.push((value, false));

        Ok(())
    }

    fn push_with(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: OptValue<'gc>,
    ) -> Result<(), Error<'gc>> {
        if self.len() >= self.max_height() {
            return Err(Error::AvmError(verify_error(
                activation,
                "Error #1017: Scope stack overflow occurred.",
                1017,
            )?));
        }

        self.0.push((value, true));

        Ok(())
    }

    fn pop(&mut self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        if self.0.is_empty() {
            return Err(Error::AvmError(verify_error(
                activation,
                "Error #1018: Scope stack underflow occurred.",
                1018,
            )?));
        }

        self.0.pop().unwrap();

        Ok(())
    }

    fn at(&self, index: usize) -> OptValue<'gc> {
        self.0[index].0
    }

    fn fill_with_any_up_to_len(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        len: usize,
    ) -> Result<(), Error<'gc>> {
        self.0.clear();
        for _ in 0..len {
            self.push_with(activation, OptValue::any())?;
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn max_height(&self) -> usize {
        self.1
    }

    fn replace_with_any(&mut self) {
        for item in self.0.iter_mut() {
            *item = (OptValue::any(), true);
        }
    }
}

/// Checks if the method fits the following pattern:
///
/// ```text
/// [Debug/DebugFile/DebugLine] zero or more times
/// GetLocal { index: 0 }
/// [Debug/DebugFile/DebugLine] zero or more times
/// PushScope
/// ...
/// ```
///
/// along with the following conditions:
/// * No jumps to that initial PushScope opcode, or anything before it
/// * No additional scope-related opcodes (PushScope, PushWith, PopScope)
/// * No catch blocks (MethodBody.exceptions is empty)
///
/// If all of these conditions are fulfilled, then the optimizer will predict the type of
/// `FindPropStrict/FindProperty` opcodes.
fn has_simple_scope_structure(
    code: &[Op],
    jump_targets: &HashMap<i32, Vec<JumpSource>>,
    method_exceptions: &[Exception<'_>],
) -> bool {
    if !method_exceptions.is_empty() {
        return false;
    }

    let mut getlocal0_pos = None;
    for (i, op) in code.iter().enumerate() {
        match op {
            // Ignore any initial debug opcodes
            Op::Debug { .. } | Op::DebugFile { .. } | Op::DebugLine { .. } => {}
            // Look for an initial getlocal0
            Op::GetLocal { index: 0 } => {
                getlocal0_pos = Some(i);
                break;
            }
            // Anything else doesn't fit the pattern, so give up
            _ => return false,
        }
    }
    // Give up if we didn't find it
    let Some(getlocal0_pos) = getlocal0_pos else {
        return false;
    };

    let mut pushscope_pos = None;
    for (i, op) in code.iter().enumerate().skip(getlocal0_pos + 1) {
        match op {
            // Ignore any debug opcodes
            Op::Debug { .. } | Op::DebugFile { .. } | Op::DebugLine { .. } => {}
            // Look for a pushscope
            Op::PushScope => {
                pushscope_pos = Some(i);
                break;
            }
            // Anything else doesn't fit the pattern, so give up
            _ => return false,
        }
    }
    // Give up if we didn't find it
    let Some(pushscope_pos) = pushscope_pos else {
        return false;
    };

    for i in 0..=pushscope_pos {
        if jump_targets.contains_key(&(i as i32)) {
            return false;
        }
    }

    for op in &code[pushscope_pos + 1..] {
        match op {
            Op::PushScope | Op::PushWith | Op::PopScope => {
                return false;
            }
            _ => {}
        }
    }
    true
}

pub fn optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Gc<'gc, BytecodeMethod<'gc>>,
    code: &mut Vec<Op<'gc>>,
    resolved_parameters: &[ResolvedParamConfig<'gc>],
    return_type: Option<Class<'gc>>,
    method_exceptions: &[Exception<'gc>],
    jump_targets: HashMap<i32, Vec<JumpSource>>,
) -> Result<(), Error<'gc>> {
    // These make the code less readable
    #![allow(clippy::collapsible_if)]
    #![allow(clippy::manual_filter)]
    #![allow(clippy::single_match)]

    // this is unfortunate, but way more convenient than grabbing types from Activation
    struct Types<'gc> {
        pub object: Class<'gc>,
        pub int: Class<'gc>,
        pub uint: Class<'gc>,
        pub number: Class<'gc>,
        pub boolean: Class<'gc>,
        pub string: Class<'gc>,
        pub array: Class<'gc>,
        pub function: Class<'gc>,
        pub void: Class<'gc>,
        pub namespace: Class<'gc>,
    }
    let types = Types {
        object: activation.avm2().class_defs().object,
        int: activation.avm2().class_defs().int,
        uint: activation.avm2().class_defs().uint,
        number: activation.avm2().class_defs().number,
        boolean: activation.avm2().class_defs().boolean,
        string: activation.avm2().class_defs().string,
        array: activation.avm2().class_defs().array,
        function: activation.avm2().class_defs().function,
        void: activation.avm2().class_defs().void,
        namespace: activation.avm2().class_defs().namespace,
    };

    let method_body = method
        .body()
        .expect("Cannot verify non-native method without body!");

    // This can probably be done better by recording the receiver in `Activation`,
    // but this works since it's guaranteed to be set in `Activation::from_method`.
    let this_value = activation.local_register(0);

    let this_class = if let Some(this_class) = activation.bound_class() {
        if this_value.is_of_type(activation, this_class) {
            Some(this_class)
        } else {
            None
        }
    } else {
        None
    };

    let this_value = OptValue {
        class: this_class,
        vtable: this_class.map(|cls| cls.vtable()),
        contains_valid_integer: false,
        contains_valid_unsigned: false,
        null_state: NullState::NotNull,
    };

    let argument_types = resolved_parameters
        .iter()
        .map(|arg| arg.param_type)
        .collect::<Vec<_>>();

    // Initial set of local types
    let mut initial_local_types = Locals::new(method_body.num_locals as usize);
    initial_local_types.set(0, this_value);

    for (i, argument_type) in argument_types.iter().enumerate() {
        if let Some(argument_type) = argument_type {
            initial_local_types.set(i + 1, OptValue::of_type(*argument_type));
            // `i + 1` because the receiver takes up local #0
        }
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
                initial_local_types.set_any(*index as usize);
            }
            Op::HasNext2 {
                object_register,
                index_register,
            } => {
                initial_local_types.set_any(*object_register as usize);
                initial_local_types.set_any(*index_register as usize);
            }
            _ => {}
        }
    }

    let has_simple_scoping = has_simple_scope_structure(code, &jump_targets, method_exceptions);

    // Map of op index -> local types + stack heights
    let mut state_map: HashMap<i32, (Locals<'gc>, usize, usize)> = HashMap::new();
    let mut expected_stack_heights: HashMap<i32, usize> = HashMap::new();
    let mut expected_scope_stack_heights: HashMap<i32, usize> = HashMap::new();

    let mut stack = Stack::new(method_body.max_stack as usize);
    let mut scope_stack =
        ScopeStack::new((method_body.max_scope_depth - method_body.init_scope_depth) as usize);
    let mut local_types = initial_local_types.clone();
    let mut last_block_op_was_block_terminating = false;
    let mut current_op_reached_linearly;

    let mut worklist = vec![0];

    let mut seen_targets = HashSet::new();

    while let Some(mut i) = worklist.pop() {
        current_op_reached_linearly = false;
        loop {
            let last_op_was_block_terminating = if i != 0 {
                code.get(i as usize - 1)
                    .map_or(false, |o| o.is_block_terminating())
            } else {
                false
            };

            let op = &mut code[i as usize];

            if op.can_throw_error() {
                for exception in method_exceptions.iter() {
                    // If this op is in the to..from and it can throw an error,
                    // add the exception's target to the worklist.
                    if exception.from_offset as i32 <= i && i < exception.to_offset as i32 {
                        // Upon jumping to an exception target, both stacks are cleared,
                        // and the error object is pushed, so state that the stack
                        // height is 1 and that the scope stack height is 0.
                        check_target(
                            activation,
                            &mut seen_targets,
                            &mut worklist,
                            &mut expected_stack_heights,
                            &mut expected_scope_stack_heights,
                            exception.target_offset as i32,
                            1,
                            0,
                        )?;
                    }
                }
            }

            if let Some(jump_sources) = jump_targets.get(&i) {
                // Avoid handling multiple sources for now
                if jump_sources.len() == 1 {
                    if let JumpSource::JumpFrom(source_i) = jump_sources[0] {
                        // We can merge the locals easily, now
                        if let Some(source_state) = state_map.get(&source_i) {
                            let mut merged_types = initial_local_types.clone();
                            assert_eq!(source_state.0.len(), local_types.len());

                            if last_op_was_block_terminating {
                                // If the last op (index-based) was a block-terminating op,
                                // the only possible way this is reachable is from the jump.
                                // Just set the types to the types at the jump.
                                merged_types = source_state.0.clone();
                            } else if current_op_reached_linearly {
                                // Only if the current op was reached by incrementing i can we
                                // rely on the current state as the state before this op.
                                for (i, target_local) in local_types.0.iter().enumerate() {
                                    let source_local = source_state.0.at(i);

                                    merged_types.set(i, source_local.merged_with(*target_local));
                                }
                            }

                            local_types = merged_types;
                        } else {
                            local_types = initial_local_types.clone();
                        }
                    } else {
                        local_types = initial_local_types.clone();
                    }
                } else {
                    local_types = initial_local_types.clone();
                }

                if last_block_op_was_block_terminating {
                    // The last block-op was block terminating, so we can't rely on the state.
                    // However, this means that the op was jumped to, which means we know the
                    // expected stack height. Use it.
                    stack.fill_with_any_up_to_len(
                        activation,
                        *expected_stack_heights
                            .get(&i)
                            .expect("check_target always fills in the expected stack height"),
                    )?;

                    scope_stack.fill_with_any_up_to_len(
                        activation,
                        *expected_scope_stack_heights
                            .get(&i)
                            .expect("check_target always fills in the expected scope stack height"),
                    )?;
                } else {
                    stack.replace_with_any();
                    scope_stack.replace_with_any();
                }
            }

            last_block_op_was_block_terminating = false;
            current_op_reached_linearly = true;

            if let Some(expected_stack_height) = expected_stack_heights.get(&i) {
                if stack.len() != *expected_stack_height {
                    // Stack height was not what it was expected to be
                    return Err(Error::AvmError(verify_error(
                        activation,
                        &format!(
                            "Error #1030: Stack depth is unbalanced. {} != {}.",
                            stack.len(),
                            *expected_stack_height,
                        ),
                        1030,
                    )?));
                }
            }

            if let Some(expected_scope_stack_height) = expected_scope_stack_heights.get(&i) {
                if scope_stack.len() != *expected_scope_stack_height {
                    // Scope stack height was not what it was expected to be
                    return Err(Error::AvmError(verify_error(
                        activation,
                        &format!(
                            "Error #1031: Scope depth is unbalanced. {} != {}.",
                            scope_stack.len(),
                            *expected_scope_stack_height,
                        ),
                        1031,
                    )?));
                }
            }

            match op {
                Op::CoerceA => {
                    // This does actually inhibit optimizations in FP
                    stack.pop(activation)?;
                    stack.push_any(activation)?;
                }
                Op::CoerceB => {
                    let stack_value = stack.pop(activation)?;
                    if stack_value.class == Some(types.boolean) {
                        *op = Op::Nop;
                    }
                    stack.push_class(activation, types.boolean)?;
                }
                Op::CoerceD => {
                    let stack_value = stack.pop(activation)?;
                    if stack_value.class == Some(types.number)
                        || stack_value.class == Some(types.int)
                        || stack_value.class == Some(types.uint)
                    {
                        *op = Op::Nop;
                    }
                    stack.push_class(activation, types.number)?;
                }
                Op::CoerceI => {
                    let stack_value = stack.pop(activation)?;
                    if stack_value.class == Some(types.int) || stack_value.contains_valid_integer {
                        *op = Op::Nop;
                    }
                    stack.push_class(activation, types.int)?;
                }
                Op::CoerceO => {
                    stack.pop(activation)?;

                    stack.push_class(activation, types.object)?;
                }
                Op::ConvertO => {
                    let value = stack.pop(activation)?;
                    stack.push(activation, value)?;
                }
                Op::CoerceS => {
                    let stack_value = stack.pop(activation)?;
                    if stack_value.is_null() {
                        *op = Op::Nop;
                    }
                    stack.push_class(activation, types.string)?;
                }
                Op::ConvertS => {
                    stack.pop(activation)?;
                    stack.push_class_not_null(activation, types.string)?;
                }
                Op::CoerceU => {
                    let stack_value = stack.pop(activation)?;
                    if stack_value.class == Some(types.uint) || stack_value.contains_valid_unsigned
                    {
                        *op = Op::Nop;
                    }
                    stack.push_class(activation, types.uint)?;
                }
                Op::Equals
                | Op::StrictEquals
                | Op::LessEquals
                | Op::LessThan
                | Op::GreaterThan
                | Op::GreaterEquals => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.boolean)?;
                }
                Op::Not => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.boolean)?;
                }
                Op::PushTrue | Op::PushFalse => {
                    stack.push_class(activation, types.boolean)?;
                }
                Op::PushNull => {
                    // TODO: we should push null type here
                    stack.push(activation, OptValue::null())?;
                }
                Op::PushUndefined => {
                    stack.push_class(activation, types.void)?;
                }
                Op::PushNaN => {
                    stack.push_class(activation, types.number)?;
                }
                Op::PushByte { value } => {
                    let mut new_value = OptValue::of_type(types.int);
                    new_value.contains_valid_integer = true;
                    if *value >= 0 {
                        new_value.contains_valid_unsigned = true;
                    }
                    stack.push(activation, new_value)?;
                }
                Op::PushShort { value } => {
                    let mut new_value = OptValue::of_type(types.int);
                    new_value.contains_valid_integer = true;
                    if *value >= 0 {
                        new_value.contains_valid_unsigned = true;
                    }
                    stack.push(activation, new_value)?;
                }
                Op::PushInt { value } => {
                    let mut new_value = OptValue::of_type(types.int);
                    if *value < -(1 << 28) || *value >= (1 << 28) {
                        // will be coerced to Number
                    } else {
                        new_value.contains_valid_integer = true;
                        if *value >= 0 {
                            new_value.contains_valid_unsigned = true;
                        }
                    }
                    stack.push(activation, new_value)?;
                }
                Op::PushUint { value } => {
                    let mut new_value = OptValue::of_type(types.uint);
                    if *value < (1 << 28) {
                        new_value.contains_valid_integer = true;
                        new_value.contains_valid_unsigned = true;
                    }
                    stack.push(activation, new_value)?;
                }
                Op::DecrementI => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::IncrementI => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::DecLocalI { index } => {
                    local_types.set(*index as usize, OptValue::of_type(types.int));
                }
                Op::DecLocal { index } => {
                    local_types.set(*index as usize, OptValue::of_type(types.number));
                }
                Op::IncLocalI { index } => {
                    local_types.set(*index as usize, OptValue::of_type(types.int));
                }
                Op::IncLocal { index } => {
                    local_types.set(*index as usize, OptValue::of_type(types.number));
                }
                Op::Increment => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::Decrement => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::Negate => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::AddI => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::SubtractI => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::MultiplyI => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::NegateI => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::Add => {
                    let value2 = stack.pop(activation)?;
                    let value1 = stack.pop(activation)?;
                    if (value1.class == Some(types.int)
                        || value1.class == Some(types.uint)
                        || value1.class == Some(types.number))
                        && (value2.class == Some(types.int)
                            || value2.class == Some(types.uint)
                            || value2.class == Some(types.number))
                    {
                        stack.push_class(activation, types.number)?;
                    } else if (value1.class == Some(types.string) && value1.not_null(activation))
                        || (value2.class == Some(types.string) && value2.not_null(activation))
                    {
                        stack.push_class_not_null(activation, types.string)?;
                    } else {
                        stack.push_any(activation)?;
                    }
                }
                Op::Subtract => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::Multiply => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::Divide => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::Modulo => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::BitNot => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::BitAnd => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::BitOr => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::BitXor => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::LShift => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::RShift => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::URShift => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::PushDouble { .. } => {
                    stack.push_class(activation, types.number)?;
                }
                Op::PushNamespace { .. } => {
                    stack.push_class_not_null(activation, types.namespace)?;
                }
                Op::PushString { .. } => {
                    stack.push_class_not_null(activation, types.string)?;
                }
                Op::NewArray { num_args } => {
                    stack.popn(activation, *num_args)?;

                    stack.push_class_not_null(activation, types.array)?;
                }
                Op::NewObject { num_args } => {
                    stack.popn(activation, *num_args * 2)?;

                    stack.push_class_not_null(activation, types.object)?;
                }
                Op::NewFunction { .. } => {
                    stack.push_class_not_null(activation, types.function)?;
                }
                Op::NewClass { class } => {
                    let c_class = class.c_class().expect("NewClass holds an i_class");
                    stack.pop(activation)?;
                    stack.push_class_not_null(activation, c_class)?;
                }
                Op::NewCatch { .. } => {
                    // Avoid handling for now
                    stack.push_any(activation)?;
                }
                Op::IsType { .. } => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.boolean)?;
                }
                Op::IsTypeLate => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.boolean)?;
                }
                Op::InstanceOf => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.boolean)?;
                }
                Op::TypeOf => {
                    stack.pop(activation)?;
                    stack.push_class_not_null(activation, types.string)?;
                }
                Op::ApplyType { num_types } => {
                    stack.popn(activation, *num_types)?;

                    stack.pop(activation)?;

                    stack.push_any(activation)?;
                }
                Op::CheckFilter => {
                    stack.pop(activation)?;
                    stack.push_any(activation)?;
                }
                Op::Dxns { .. } => {
                    // Dxns doesn't change stack or locals
                }
                Op::DxnsLate => {
                    stack.pop(activation)?;
                }
                Op::EscXAttr | Op::EscXElem => {
                    stack.pop(activation)?;
                    stack.push_class_not_null(activation, types.string)?;
                }
                Op::GetDescendants { multiname } => {
                    stack.pop_for_multiname(activation, *multiname)?;

                    stack.pop(activation)?;

                    stack.push_any(activation)?;
                }
                Op::AsTypeLate => {
                    let type_c_class = stack.pop(activation)?;
                    stack.pop(activation)?;

                    let mut new_value = OptValue::any();

                    if let Some(class) = type_c_class.class.and_then(|c| c.i_class()) {
                        let class_is_primitive = class == types.int
                            || class == types.uint
                            || class == types.number
                            || class == types.boolean
                            || class == types.void;

                        if !class_is_primitive {
                            // If the type on the stack was a c_class with a non-primitive
                            // i_class, we can use the type
                            new_value = OptValue::of_type(class);
                        }
                    }

                    stack.push(activation, new_value)?;
                }
                Op::AsType { class } => {
                    let stack_value = stack.pop(activation)?;

                    let class_is_primitive = *class == types.int
                        || *class == types.uint
                        || *class == types.number
                        || *class == types.boolean
                        || *class == types.void;

                    let mut new_value = OptValue::any();
                    if !class_is_primitive {
                        // if T is non-nullable, we can assume the result is typed T
                        new_value = OptValue::of_type(*class);
                    }
                    if let Some(stack_class) = stack_value.class {
                        if *class == stack_class {
                            // If type check is guaranteed, preserve original type
                            // TODO: there are more cases when this can succeed,
                            // like inheritance and numbers (`x: Number = 1; x as int;`)
                            new_value = stack_value;
                        }
                    }
                    if stack_value.is_null() {
                        // null always turns into null
                        *op = Op::Nop;
                        new_value.null_state = NullState::IsNull;
                    }
                    stack.push(activation, new_value)?;
                }
                Op::Coerce { class } => {
                    let stack_value = stack.pop(activation)?;
                    let mut new_value = OptValue::of_type(*class);

                    if stack_value.is_null() {
                        // Coercing null to a non-primitive or void is a noop.
                        if *class != types.int
                            && *class != types.uint
                            && *class != types.number
                            && *class != types.boolean
                            && *class != types.void
                        {
                            *op = Op::Nop;
                            new_value.null_state = NullState::IsNull;
                        }
                    } else if let Some(stack_class) = stack_value.class {
                        // TODO: this could check for inheritance
                        if *class == stack_class {
                            *op = Op::Nop;
                            new_value.null_state = stack_value.null_state;
                        }
                    }

                    stack.push(activation, new_value)?;
                }
                Op::PushScope => {
                    let stack_value = stack.pop(activation)?;
                    scope_stack.push(activation, stack_value)?;
                }
                Op::PushWith => {
                    let stack_value = stack.pop(activation)?;
                    scope_stack.push_with(activation, stack_value)?;
                }
                Op::PopScope => {
                    scope_stack.pop(activation)?;
                }
                Op::GetScopeObject { index } => {
                    let index = *index as usize;

                    if index >= scope_stack.len() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1019: Getscopeobject  is out of bounds.",
                            1019,
                        )?));
                    }

                    if has_simple_scoping && index == 0 {
                        stack.push(activation, this_value)?;
                    } else {
                        stack.push(activation, scope_stack.at(index))?;
                    }
                }
                Op::GetOuterScope { index } => {
                    let class = activation
                        .outer()
                        .get_unchecked(*index as usize)
                        .values()
                        .instance_class();
                    stack.push_class(activation, class)?;
                }
                Op::Pop => {
                    stack.pop(activation)?;
                }
                Op::Dup => {
                    let stack_value = stack.pop(activation)?;
                    stack.push(activation, stack_value)?;
                    stack.push(activation, stack_value)?;
                }
                Op::Swap => {
                    let first = stack.pop(activation)?;
                    let second = stack.pop(activation)?;
                    stack.push(activation, first)?;
                    stack.push(activation, second)?;
                }
                Op::Kill { index } => {
                    local_types.set_any(*index as usize);
                }
                Op::SetLocal { index } => {
                    let stack_value = stack.pop(activation)?;
                    local_types.set(*index as usize, stack_value);
                }
                Op::GetLocal { index } => {
                    let local_type = local_types.at(*index as usize);
                    stack.push(activation, local_type)?;
                }
                Op::FindPropStrict { multiname } | Op::FindProperty { multiname } => {
                    let multiname = *multiname;
                    let mut stack_push_done = false;
                    stack.pop_for_multiname(activation, multiname)?;

                    if !multiname.has_lazy_component() && has_simple_scoping {
                        let outer_scope = activation.outer();
                        if !outer_scope.is_empty() {
                            if let Some(this_vtable) = this_class.map(|cls| cls.vtable()) {
                                if this_vtable.has_trait(&multiname) {
                                    *op = Op::GetScopeObject { index: 0 };

                                    stack_push_done = true;
                                    stack.push(activation, this_value)?;
                                }
                            } else {
                                stack_push_done = true;
                                stack.push_any(activation)?;
                            }
                        }

                        if !stack_push_done {
                            if let Some(info) = outer_scope.get_entry_for_multiname(&multiname) {
                                if let Some((class, index)) = info {
                                    *op = Op::GetOuterScope { index };

                                    stack_push_done = true;
                                    stack.push_class(activation, class)?;
                                } else {
                                    // If `get_entry_for_multiname` returned `Some(None)`, there was
                                    // a `with` scope in the outer ScopeChain- abort optimization.
                                    stack_push_done = true;
                                    stack.push_any(activation)?;
                                }
                            }
                        }

                        if !stack_push_done {
                            if let Ok(Some((_, script))) =
                                outer_scope.domain().get_defining_script(&multiname)
                            {
                                // NOTE: avmplus rewrites this into a FindDef, and it caches
                                // the results of that FindDef at runtime, rather than caching
                                // the lookup here, in the verifier. However, this discrepancy
                                // is unlikely to cause any real problems with SWFs.
                                *op = Op::GetScriptGlobals { script };

                                if script.traits_loaded() {
                                    stack_push_done = true;
                                    stack.push_class_not_null(activation, script.global_class())?;
                                }
                            }
                        }

                        // Ignore global scope for now
                    }

                    if !stack_push_done {
                        stack.push_any(activation)?;
                    }
                }
                Op::FindDef { .. } => {
                    // Avoid handling for now
                    stack.push_any(activation)?;
                }
                Op::In => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.boolean)?;
                }
                Op::NextName => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_any(activation)?;
                }
                Op::NextValue => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_any(activation)?;
                }
                Op::HasNext => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_any(activation)?;
                }
                Op::HasNext2 {
                    index_register,
                    object_register,
                } => {
                    stack.push_class(activation, types.boolean)?;
                    local_types.set_any(*index_register as usize);
                    local_types.set_any(*object_register as usize);
                }
                Op::GetSlot { index: slot_id } => {
                    let mut stack_push_done = false;
                    let stack_value = stack.pop(activation)?;

                    if let Some(vtable) = stack_value.vtable() {
                        let slot_classes = vtable.slot_classes();
                        let value_class = slot_classes.get(*slot_id as usize).copied();
                        if let Some(mut value_class) = value_class {
                            let resolved_value_class = value_class.get_class(activation);
                            if let Ok(class) = resolved_value_class {
                                stack_push_done = true;

                                if let Some(class) = class {
                                    stack.push_class(activation, class)?;
                                } else {
                                    stack.push_any(activation)?;
                                }
                            }

                            drop(slot_classes);
                            vtable.set_slot_class(
                                activation.context.gc_context,
                                *slot_id as usize,
                                value_class,
                            );
                        }
                    }

                    if !stack_push_done {
                        stack.push_any(activation)?;
                    }
                }
                Op::SetSlot { .. } => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                }
                Op::GetProperty { multiname } => {
                    let mut stack_push_done = false;
                    stack.pop_for_multiname(activation, *multiname)?;
                    let stack_value = stack.pop(activation)?;

                    if !multiname.has_lazy_component() {
                        if let Some(vtable) = stack_value.vtable() {
                            match vtable.get_trait(multiname) {
                                Some(Property::Slot { slot_id })
                                | Some(Property::ConstSlot { slot_id }) => {
                                    *op = Op::GetSlot { index: slot_id };

                                    let mut value_class = vtable.slot_classes()[slot_id as usize];
                                    let resolved_value_class = value_class.get_class(activation);
                                    if let Ok(class) = resolved_value_class {
                                        stack_push_done = true;

                                        if let Some(class) = class {
                                            stack.push_class(activation, class)?;
                                        } else {
                                            stack.push_any(activation)?;
                                        }
                                    }

                                    vtable.set_slot_class(
                                        activation.context.gc_context,
                                        slot_id as usize,
                                        value_class,
                                    );
                                }
                                Some(Property::Virtual {
                                    get: Some(disp_id), ..
                                }) => {
                                    *op = Op::CallMethod {
                                        num_args: 0,
                                        index: disp_id,
                                        push_return_value: true,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    // `stack_pop_multiname` handled lazy

                    if !stack_push_done {
                        stack.push_any(activation)?;
                    }
                }
                Op::InitProperty { multiname } => {
                    let set_value = stack.pop(activation)?;

                    stack.pop_for_multiname(activation, *multiname)?;
                    let stack_value = stack.pop(activation)?;
                    if !multiname.has_lazy_component() {
                        if let Some(vtable) = stack_value.vtable() {
                            match vtable.get_trait(multiname) {
                                Some(Property::Slot { slot_id })
                                | Some(Property::ConstSlot { slot_id }) => {
                                    *op = Op::SetSlot { index: slot_id };

                                    // If the set value's type is the same as the type of the slot,
                                    // a SetSlotNoCoerce can be emitted.
                                    let mut value_class = vtable.slot_classes()[slot_id as usize];
                                    let resolved_value_class = value_class.get_class(activation);

                                    if let Ok(slot_class) = resolved_value_class {
                                        if let Some(slot_class) = slot_class {
                                            if let Some(set_value_class) = set_value.class {
                                                if set_value_class == slot_class {
                                                    *op = Op::SetSlotNoCoerce { index: slot_id };
                                                }
                                            }
                                        } else {
                                            // Slot type was Any, no coercion will be done anyways
                                            *op = Op::SetSlotNoCoerce { index: slot_id };
                                        }
                                    }
                                }
                                Some(Property::Virtual {
                                    set: Some(disp_id), ..
                                }) => {
                                    *op = Op::CallMethod {
                                        num_args: 1,
                                        index: disp_id,
                                        push_return_value: false,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    // `stack_pop_multiname` handled lazy
                }
                Op::SetProperty { multiname } => {
                    let set_value = stack.pop(activation)?;

                    stack.pop_for_multiname(activation, *multiname)?;
                    let stack_value = stack.pop(activation)?;
                    if !multiname.has_lazy_component() {
                        if let Some(vtable) = stack_value.vtable() {
                            match vtable.get_trait(multiname) {
                                Some(Property::Slot { slot_id }) => {
                                    *op = Op::SetSlot { index: slot_id };

                                    // If the set value's type is the same as the type of the slot,
                                    // a SetSlotNoCoerce can be emitted.
                                    let mut value_class = vtable.slot_classes()[slot_id as usize];
                                    let resolved_value_class = value_class.get_class(activation);

                                    if let Ok(slot_class) = resolved_value_class {
                                        if let Some(slot_class) = slot_class {
                                            if let Some(set_value_class) = set_value.class {
                                                if set_value_class == slot_class {
                                                    *op = Op::SetSlotNoCoerce { index: slot_id };
                                                }
                                            }
                                        } else {
                                            // Slot type was Any, no coercion will be done anyways
                                            *op = Op::SetSlotNoCoerce { index: slot_id };
                                        }
                                    }
                                }
                                Some(Property::Virtual {
                                    set: Some(disp_id), ..
                                }) => {
                                    *op = Op::CallMethod {
                                        num_args: 1,
                                        index: disp_id,
                                        push_return_value: false,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    // `stack_pop_multiname` handled lazy
                }
                Op::DeleteProperty { multiname } => {
                    stack.pop_for_multiname(activation, *multiname)?;

                    stack.pop(activation)?;

                    stack.push_class(activation, types.boolean)?;
                }
                Op::Construct { num_args } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop(activation)?;

                    // Avoid checking return value for now
                    stack.push_any(activation)?;
                }
                Op::ConstructSuper { num_args } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    // Then receiver.
                    stack.pop(activation)?;
                }
                Op::ConstructProp {
                    multiname,
                    num_args,
                } => {
                    let mut stack_push_done = false;

                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Then receiver.
                    let stack_value = stack.pop(activation)?;

                    if !multiname.has_lazy_component() {
                        if let Some(vtable) = stack_value.vtable() {
                            match vtable.get_trait(multiname) {
                                Some(Property::Slot { slot_id })
                                | Some(Property::ConstSlot { slot_id }) => {
                                    let mut value_class = vtable.slot_classes()[slot_id as usize];
                                    let resolved_value_class = value_class.get_class(activation);

                                    if let Ok(Some(slot_class)) = resolved_value_class {
                                        if let Some(instance_class) = slot_class.i_class() {
                                            // ConstructProp on a c_class will construct its i_class
                                            stack_push_done = true;
                                            stack.push_class(activation, instance_class)?;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if !stack_push_done {
                        stack.push_any(activation)?;
                    }
                }
                Op::Call { num_args } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop(activation)?;

                    stack.pop(activation)?;

                    // Avoid checking return value for now
                    stack.push_any(activation)?;
                }
                Op::CallPropLex {
                    multiname,
                    num_args,
                } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Then receiver.
                    stack.pop(activation)?;

                    // Avoid checking return value for now
                    stack.push_any(activation)?;
                }
                Op::CallStatic { num_args, .. } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    // Then receiver.
                    stack.pop(activation)?;

                    // Avoid checking return value for now
                    stack.push_any(activation)?;
                }
                Op::CallProperty {
                    multiname,
                    num_args,
                } => {
                    let mut stack_push_done = false;

                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Then receiver.
                    let stack_value = stack.pop(activation)?;

                    if !multiname.has_lazy_component() {
                        if let Some(vtable) = stack_value.vtable() {
                            match vtable.get_trait(multiname) {
                                Some(Property::Method { disp_id }) => {
                                    *op = Op::CallMethod {
                                        num_args: *num_args,
                                        index: disp_id,
                                        push_return_value: true,
                                    };
                                }
                                Some(Property::Slot { slot_id })
                                | Some(Property::ConstSlot { slot_id }) => {
                                    if stack_value.not_null(activation) {
                                        if *num_args == 1 {
                                            let mut value_class =
                                                vtable.slot_classes()[slot_id as usize];
                                            let resolved_value_class =
                                                value_class.get_class(activation);

                                            if let Ok(Some(slot_class)) = resolved_value_class {
                                                if let Some(called_class) = slot_class.i_class() {
                                                    // Calling a c_class will perform a simple coercion to the class
                                                    if called_class.call_handler().is_none() {
                                                        *op = Op::CoerceSwapPop {
                                                            class: called_class,
                                                        };

                                                        stack_push_done = true;
                                                        stack
                                                            .push_class(activation, called_class)?;
                                                    } else if called_class == types.int {
                                                        *op = Op::CoerceISwapPop;

                                                        stack_push_done = true;
                                                        stack.push_class(activation, types.int)?;
                                                    } else if called_class == types.uint {
                                                        *op = Op::CoerceUSwapPop;

                                                        stack_push_done = true;
                                                        stack.push_class(activation, types.uint)?;
                                                    } else if called_class == types.number {
                                                        *op = Op::CoerceDSwapPop;

                                                        stack_push_done = true;
                                                        stack
                                                            .push_class(activation, types.number)?;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    // `stack_pop_multiname` handled lazy

                    // Avoid checking return value for now
                    if !stack_push_done {
                        stack.push_any(activation)?;
                    }
                }
                Op::CallPropVoid {
                    multiname,
                    num_args,
                } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Then receiver.
                    let stack_value = stack.pop(activation)?;

                    if !multiname.has_lazy_component() {
                        if let Some(vtable) = stack_value.vtable() {
                            match vtable.get_trait(multiname) {
                                Some(Property::Method { disp_id }) => {
                                    *op = Op::CallMethod {
                                        num_args: *num_args,
                                        index: disp_id,
                                        push_return_value: false,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    // `stack_pop_multiname` handled lazy
                }
                Op::GetSuper { multiname } => {
                    stack.pop_for_multiname(activation, *multiname)?;

                    // Receiver
                    stack.pop(activation)?;

                    // Avoid checking return value for now
                    stack.push_any(activation)?;
                }
                Op::SetSuper { multiname } => {
                    stack.pop(activation)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Receiver
                    stack.pop(activation)?;
                }
                Op::CallSuper {
                    multiname,
                    num_args,
                } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Then receiver.
                    stack.pop(activation)?;

                    // Avoid checking return value for now
                    stack.push_any(activation)?;
                }
                Op::CallSuperVoid {
                    multiname,
                    num_args,
                } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    stack.pop_for_multiname(activation, *multiname)?;

                    // Then receiver.
                    stack.pop(activation)?;
                }
                Op::GetGlobalScope => {
                    let outer_scope = activation.outer();
                    if !outer_scope.is_empty() {
                        let global_scope = outer_scope.get_unchecked(0);

                        stack.push_class(activation, global_scope.values().instance_class())?;
                    } else if has_simple_scoping {
                        stack.push(activation, this_value)?;
                    } else {
                        if scope_stack.is_empty() {
                            return Err(Error::AvmError(verify_error(
                                activation,
                                "Error #1019: Getscopeobject  is out of bounds.",
                                1019,
                            )?));
                        }

                        stack.push_any(activation)?;
                    }
                }
                Op::GetGlobalSlot { index: slot_id } => {
                    let outer_scope = activation.outer();
                    if outer_scope.is_empty() && scope_stack.is_empty() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1019: Getscopeobject  is out of bounds.",
                            1019,
                        )?));
                    }

                    let mut stack_push_done = false;

                    if !outer_scope.is_empty() {
                        let global_scope = outer_scope.get_unchecked(0);

                        let class = global_scope.values().instance_class();
                        let mut value_class = class.vtable().slot_classes()[*slot_id as usize];
                        let resolved_value_class = value_class.get_class(activation);
                        if let Ok(class) = resolved_value_class {
                            stack_push_done = true;

                            if let Some(class) = class {
                                stack.push_class(activation, class)?;
                            } else {
                                stack.push_any(activation)?;
                            }
                        }

                        class.vtable().set_slot_class(
                            activation.context.gc_context,
                            *slot_id as usize,
                            value_class,
                        );
                    }

                    if !stack_push_done {
                        stack.push_any(activation)?;
                    }
                }
                Op::SetGlobalSlot { .. } => {
                    let outer_scope = activation.outer();
                    if outer_scope.is_empty() && scope_stack.is_empty() {
                        return Err(Error::AvmError(verify_error(
                            activation,
                            "Error #1019: Getscopeobject  is out of bounds.",
                            1019,
                        )?));
                    }

                    // Avoid handling for now
                    stack.pop(activation)?;
                }
                Op::NewActivation => {
                    // Avoid handling for now
                    stack.push_any(activation)?;
                }
                Op::Nop => {}
                Op::DebugFile { .. }
                | Op::DebugLine { .. }
                | Op::Debug { .. }
                | Op::Bkpt
                | Op::BkptLine { .. }
                | Op::Timestamp => {}
                Op::IfTrue { offset } | Op::IfFalse { offset } => {
                    stack.pop(activation)?;
                    state_map.insert(i, (local_types.clone(), stack.len(), scope_stack.len()));

                    check_target(
                        activation,
                        &mut seen_targets,
                        &mut worklist,
                        &mut expected_stack_heights,
                        &mut expected_scope_stack_heights,
                        *offset + i + 1,
                        stack.len(),
                        scope_stack.len(),
                    )?;
                }
                Op::IfStrictEq { offset }
                | Op::IfStrictNe { offset }
                | Op::IfEq { offset }
                | Op::IfNe { offset }
                | Op::IfGe { offset }
                | Op::IfGt { offset }
                | Op::IfLe { offset }
                | Op::IfLt { offset }
                | Op::IfNge { offset }
                | Op::IfNgt { offset }
                | Op::IfNle { offset }
                | Op::IfNlt { offset } => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    state_map.insert(i, (local_types.clone(), stack.len(), scope_stack.len()));

                    check_target(
                        activation,
                        &mut seen_targets,
                        &mut worklist,
                        &mut expected_stack_heights,
                        &mut expected_scope_stack_heights,
                        *offset + i + 1,
                        stack.len(),
                        scope_stack.len(),
                    )?;
                }
                Op::Si8 | Op::Si16 | Op::Si32 => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                }
                Op::Li8 | Op::Li16 => {
                    stack.pop(activation)?;
                    let mut value = OptValue::of_type(types.int);
                    value.contains_valid_integer = true;
                    stack.push(activation, value)?;
                }
                Op::Li32 => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::Sxi1 | Op::Sxi8 | Op::Sxi16 => {
                    stack.pop(activation)?;
                    let mut value = OptValue::of_type(types.int);
                    value.contains_valid_integer = true;
                    stack.push(activation, value)?;
                }
                Op::Sf32 | Op::Sf64 => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                }
                Op::Lf32 | Op::Lf64 => {
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::ReturnVoid | Op::Throw => {
                    // End of block
                    last_block_op_was_block_terminating = true;
                    break;
                }
                Op::LookupSwitch(ref lookup_switch) => {
                    stack.pop(activation)?;
                    state_map.insert(i, (local_types.clone(), stack.len(), scope_stack.len()));

                    check_target(
                        activation,
                        &mut seen_targets,
                        &mut worklist,
                        &mut expected_stack_heights,
                        &mut expected_scope_stack_heights,
                        lookup_switch.default_offset + i + 1,
                        stack.len(),
                        scope_stack.len(),
                    )?;

                    for case_offset in lookup_switch.case_offsets.iter() {
                        check_target(
                            activation,
                            &mut seen_targets,
                            &mut worklist,
                            &mut expected_stack_heights,
                            &mut expected_scope_stack_heights,
                            *case_offset + i + 1,
                            stack.len(),
                            scope_stack.len(),
                        )?;
                    }

                    // End of block
                    last_block_op_was_block_terminating = true;
                    break;
                }
                Op::ReturnValue => {
                    let stack_value = stack.pop(activation)?;

                    if let Some(return_type) = return_type {
                        let return_type_is_primitive = return_type == types.int
                            || return_type == types.uint
                            || return_type == types.number
                            || return_type == types.boolean
                            || return_type == types.void;

                        if let Some(stack_value_class) = stack_value.class {
                            if stack_value_class == return_type {
                                *op = Op::ReturnValueNoCoerce;
                            }
                        }

                        if !return_type_is_primitive {
                            if stack_value.is_null() {
                                *op = Op::ReturnValueNoCoerce;
                            }
                        }
                    } else {
                        // Return type was Any, no coercion will be done anyways
                        *op = Op::ReturnValueNoCoerce;
                    }

                    // End of block
                    last_block_op_was_block_terminating = true;
                    break;
                }
                Op::Jump { offset } => {
                    state_map.insert(i, (local_types.clone(), stack.len(), scope_stack.len()));

                    check_target(
                        activation,
                        &mut seen_targets,
                        &mut worklist,
                        &mut expected_stack_heights,
                        &mut expected_scope_stack_heights,
                        *offset + i + 1,
                        stack.len(),
                        scope_stack.len(),
                    )?;
                    // End of block
                    last_block_op_was_block_terminating = true;
                    break;
                }

                Op::CallMethod {
                    num_args,
                    push_return_value,
                    ..
                } => {
                    // Arguments
                    stack.popn(activation, *num_args)?;

                    // Receiver
                    stack.pop(activation)?;

                    if *push_return_value {
                        // Avoid checking return type for now
                        stack.push_any(activation)?;
                    }
                }
                Op::CoerceSwapPop { class } => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, *class)?;
                }
                Op::CoerceDSwapPop => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.number)?;
                }
                Op::CoerceISwapPop => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.int)?;
                }
                Op::CoerceUSwapPop => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                    stack.push_class(activation, types.uint)?;
                }
                Op::GetScriptGlobals { script } => {
                    stack.push_class_not_null(activation, script.global_class())?;
                }
                Op::ReturnValueNoCoerce => {
                    last_block_op_was_block_terminating = true;
                    break;
                }
                Op::SetSlotNoCoerce { .. } => {
                    stack.pop(activation)?;
                    stack.pop(activation)?;
                }
            }

            i += 1;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_target<'gc>(
    activation: &mut Activation<'_, 'gc>,
    seen_targets: &mut HashSet<i32>,
    worklist: &mut Vec<i32>,
    expected_stack_heights: &mut HashMap<i32, usize>,
    expected_scope_stack_heights: &mut HashMap<i32, usize>,
    target: i32,
    stack_height: usize,
    scope_stack_height: usize,
) -> Result<(), Error<'gc>> {
    if !seen_targets.contains(&target) {
        seen_targets.insert(target);

        worklist.push(target);
    }

    if let Some(height) = expected_stack_heights.get(&target) {
        if stack_height != *height {
            // Two jumps to target with conflicting stack heights
            return Err(Error::AvmError(verify_error(
                activation,
                &format!(
                    "Error #1030: Stack depth is unbalanced. {} != {}.",
                    stack_height, *height,
                ),
                1030,
            )?));
        }
    } else {
        expected_stack_heights.insert(target, stack_height);
    }

    if let Some(height) = expected_scope_stack_heights.get(&target) {
        if scope_stack_height != *height {
            // Two jumps to target with conflicting scope stack heights
            return Err(Error::AvmError(verify_error(
                activation,
                &format!(
                    "Error #1031: Scope depth is unbalanced. {} != {}.",
                    scope_stack_height, *height,
                ),
                1031,
            )?));
        }
    } else {
        expected_scope_stack_heights.insert(target, scope_stack_height);
    }

    Ok(())
}
