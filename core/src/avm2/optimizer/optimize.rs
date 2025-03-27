use crate::avm2::error::verify_error;
use crate::avm2::method::{BytecodeMethod, ResolvedParamConfig};
use crate::avm2::multiname::Multiname;
use crate::avm2::op::Op;
use crate::avm2::optimizer::blocks::{assemble_blocks, BlockExit};
use crate::avm2::property::Property;
use crate::avm2::verify::Exception;
use crate::avm2::vtable::VTable;
use crate::avm2::{Activation, Class, Error};

use gc_arena::Gc;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum NullState {
    NotNull,
    MaybeNull,
    IsNull,
}

#[derive(Clone, Copy, PartialEq)]
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
            contains_valid_integer: false,
            contains_valid_unsigned: false,
            null_state: NullState::MaybeNull,
        }
    }

    pub fn null() -> Self {
        Self {
            class: None,
            null_state: NullState::IsNull,
            ..Self::any()
        }
    }

    pub fn of_type(class: Class<'gc>) -> Self {
        Self {
            class: Some(class),
            ..Self::any()
        }
    }

    pub fn vtable(self) -> Option<VTable<'gc>> {
        self.class.filter(|c| !c.is_interface()).map(|c| c.vtable())
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

    pub fn merged_with(
        self,
        activation: &mut Activation<'_, 'gc>,
        other: OptValue<'gc>,
    ) -> OptValue<'gc> {
        let mut created_value = OptValue::any();

        let class_defs = activation.avm2().class_defs();

        // TODO: Also check common superclasses.
        if self.class == other.class {
            created_value.class = self.class;
        } else if matches!(other.null_state, NullState::IsNull) {
            // If the other value is guaranteed to be null, we can just use our class.
            // Unless it's a primitive class.
            if let Some(self_class) = self.class {
                if self_class != class_defs.int
                    && self_class != class_defs.uint
                    && self_class != class_defs.number
                    && self_class != class_defs.boolean
                    && self_class != class_defs.void
                {
                    created_value.class = self.class;
                }
            }
        } else if matches!(self.null_state, NullState::IsNull) {
            // And vice-versa.
            if let Some(other_class) = other.class {
                if other_class != class_defs.int
                    && other_class != class_defs.uint
                    && other_class != class_defs.number
                    && other_class != class_defs.boolean
                    && other_class != class_defs.void
                {
                    created_value.class = other.class;
                }
            }
        }

        if self.contains_valid_integer && other.contains_valid_integer {
            created_value.contains_valid_integer = true;
        }

        if self.contains_valid_unsigned && other.contains_valid_unsigned {
            created_value.contains_valid_unsigned = true;
        }

        if self.null_state == other.null_state {
            created_value.null_state = self.null_state;
        }

        created_value
    }

    // Check whether if this OptValue were stored in a slot of type `checked_type`,
    // whether it could be represented as-is, without any coercion.
    pub fn matches_type(
        self,
        activation: &mut Activation<'_, 'gc>,
        checked_type: Option<Class<'gc>>,
    ) -> bool {
        // This makes the code less readable
        #![allow(clippy::if_same_then_else)]

        let class_defs = activation.avm2().class_defs();

        if let Some(checked_class) = checked_type {
            if let Some(own_class) = self.class {
                // Check if the checked class is a superclass of our class, or if
                // the checked class is `Number` and our class is `int` or `uint`
                if own_class.has_class_in_chain(checked_class) {
                    return true;
                } else if (own_class == class_defs.int || own_class == class_defs.uint)
                    && checked_class == class_defs.number
                {
                    return true;
                }
            }

            if checked_class == class_defs.int && self.contains_valid_integer {
                true
            } else if checked_class == class_defs.uint && self.contains_valid_unsigned {
                true
            } else {
                let is_not_primitive_class = checked_class != class_defs.int
                    && checked_class != class_defs.uint
                    && checked_class != class_defs.number
                    && checked_class != class_defs.boolean
                    && checked_class != class_defs.void;

                // Null matches every class except the primitive classes
                matches!(self.null_state, NullState::IsNull) && is_not_primitive_class
            }
        } else {
            // All values match the Any type
            true
        }
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

#[derive(Clone, Debug, PartialEq)]
struct Locals<'gc>(Vec<OptValue<'gc>>);

impl<'gc> Locals<'gc> {
    fn empty() -> Self {
        Self(Vec::new())
    }

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

#[derive(Clone, Debug, PartialEq)]
struct Stack<'gc>(Vec<OptValue<'gc>>, usize);

impl<'gc> Stack<'gc> {
    fn empty() -> Self {
        Self(Vec::new(), 0)
    }

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

    fn set(&mut self, index: usize, value: OptValue<'gc>) {
        self.0[index] = value;
    }

    fn at(&self, index: usize) -> OptValue<'gc> {
        self.0[index]
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn max_height(&self) -> usize {
        self.1
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ScopeStack<'gc>(Vec<(OptValue<'gc>, bool)>, usize);

impl<'gc> ScopeStack<'gc> {
    fn empty() -> Self {
        Self(Vec::new(), 0)
    }

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

    fn at(&self, index: usize) -> (OptValue<'gc>, bool) {
        self.0[index]
    }

    fn set(&mut self, index: usize, value: OptValue<'gc>, is_with: bool) {
        self.0[index] = (value, is_with);
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
}

#[derive(Clone, Debug, PartialEq)]
struct AbstractState<'gc> {
    locals: Locals<'gc>,
    stack: Stack<'gc>,
    scope_stack: ScopeStack<'gc>,
}

impl<'gc> AbstractState<'gc> {
    #[inline(never)]
    fn merged_with(
        self,
        activation: &mut Activation<'_, 'gc>,
        other: &AbstractState<'gc>,
    ) -> Result<AbstractState<'gc>, Error<'gc>> {
        // Merge locals
        assert!(self.locals.len() == other.locals.len());

        let mut new_locals = self.locals;
        for i in 0..new_locals.len() {
            let our_local = new_locals.at(i);
            let other_local = other.locals.at(i);

            new_locals.set(i, our_local.merged_with(activation, other_local));
        }

        // Merge stack
        if self.stack.len() != other.stack.len() {
            return Err(Error::AvmError(verify_error(
                activation,
                &format!(
                    "Error #1030: Stack depth is unbalanced. {} != {}.",
                    self.stack.len(),
                    other.stack.len(),
                ),
                1030,
            )?));
        }

        let mut new_stack = self.stack;
        for i in 0..new_stack.len() {
            let our_entry = new_stack.at(i);
            let other_entry = other.stack.at(i);

            new_stack.set(i, our_entry.merged_with(activation, other_entry));
        }

        // Merge scope stack
        if self.scope_stack.len() != other.scope_stack.len() {
            return Err(Error::AvmError(verify_error(
                activation,
                &format!(
                    "Error #1031: Scope depth is unbalanced. {} != {}.",
                    self.scope_stack.len(),
                    other.scope_stack.len(),
                ),
                1031,
            )?));
        }

        let mut new_scope_stack = self.scope_stack;
        for i in 0..new_scope_stack.len() {
            let our_scope = new_scope_stack.at(i);
            let other_scope = other.scope_stack.at(i);

            if our_scope.1 != other_scope.1 {
                return Err(Error::AvmError(verify_error(
                    activation,
                    "Error #1068: Scope values cannot be reconciled.",
                    1068,
                )?));
            }

            new_scope_stack.set(
                i,
                our_scope.0.merged_with(activation, other_scope.0),
                our_scope.1,
            );
        }

        Ok(AbstractState {
            locals: new_locals,
            stack: new_stack,
            scope_stack: new_scope_stack,
        })
    }
}

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

pub fn optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Gc<'gc, BytecodeMethod<'gc>>,
    code: &mut Vec<Op<'gc>>,
    resolved_parameters: &[ResolvedParamConfig<'gc>],
    method_exceptions: &[Exception<'gc>],
    jump_targets: &HashSet<usize>,
) -> Result<(), Error<'gc>> {
    // These make the code less readable
    #![allow(clippy::collapsible_if)]
    #![allow(clippy::manual_filter)]
    #![allow(clippy::single_match)]

    let code_slice = Cell::from_mut(code.as_mut_slice());
    let code_slice = code_slice.as_slice_of_cells();

    let block_list = assemble_blocks(code_slice, method_exceptions, jump_targets);

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

    let this_class = activation.bound_class();

    let this_value = OptValue {
        class: this_class,
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

    let empty_stack = Stack::new(method_body.max_stack as usize);
    let empty_scope_stack =
        ScopeStack::new((method_body.max_scope_depth - method_body.init_scope_depth) as usize);

    let entry_state = AbstractState {
        locals: initial_local_types,
        stack: empty_stack.clone(),
        scope_stack: empty_scope_stack.clone(),
    };

    // A Vec holding lists of possible abstract states, indexed by block index
    let mut abstract_states: Vec<Option<AbstractState<'gc>>> = vec![None; block_list.len()];

    // Block #0 is the entry block
    let mut worklist = Vec::new();
    worklist.push((0, entry_state.clone()));
    while let Some((block_idx, provided_abstract_state)) = worklist.pop() {
        let block = &block_list[block_idx];

        let known_abstract_state = &mut abstract_states[block_idx];

        let used_entry_state = if let Some(known_abstract_state) = known_abstract_state {
            let merged_state =
                provided_abstract_state.merged_with(activation, known_abstract_state)?;
            if merged_state.eq(known_abstract_state) {
                // We've already verified that this state works, no need to run it again
                continue;
            } else {
                merged_state
            }
        } else {
            // We don't have any state in the state list yet, so we use the provided one
            provided_abstract_state
        };

        *known_abstract_state = Some(used_entry_state.clone());

        let resulting_state =
            abstract_interpret_ops(activation, block.ops, used_entry_state, None, &types, false)?;

        for exit in &block.exits {
            match exit {
                BlockExit::Goto(target_block) => {
                    worklist.push((*target_block, resulting_state.clone()));
                }
                BlockExit::GotoException(target_block) => {
                    // When branching as a result of an exception in a catch block,
                    // the exception target will be run starting with an empty
                    // scope stack and a stack with a single entry on it.
                    let mut exception_stack = empty_stack.clone();
                    exception_stack.push_any(activation)?;

                    let exception_state = AbstractState {
                        locals: resulting_state.locals.clone(),
                        stack: exception_stack,
                        scope_stack: empty_scope_stack.clone(),
                    };

                    worklist.push((*target_block, exception_state));
                }
                BlockExit::Return => {}
            }
        }
    }

    // At this point we know the guaranteed state at every block start
    let mut replacement_states = HashMap::with_capacity(block_list.len());
    for (i, abstract_state) in abstract_states.into_iter().enumerate() {
        let start_index = block_list[i].start_index;

        let abstract_state = abstract_state.expect("Every block should be visited");

        replacement_states.insert(start_index, abstract_state);
    }

    if activation.avm2().optimizer_enabled() {
        // Now run through the ops and actually optimize them
        abstract_interpret_ops(
            activation,
            code_slice,
            entry_state,
            Some(replacement_states),
            &types,
            true,
        )?;
    }

    Ok(())
}

fn abstract_interpret_ops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ops: &[Cell<Op<'gc>>],
    initial_state: AbstractState<'gc>,
    mut replacement_states: Option<HashMap<usize, AbstractState<'gc>>>,
    types: &Types<'gc>,
    do_optimize: bool,
) -> Result<AbstractState<'gc>, Error<'gc>> {
    // These make the code less readable
    #![allow(clippy::collapsible_if)]
    #![allow(clippy::single_match)]

    let mut locals = initial_state.locals;
    let mut stack = initial_state.stack;
    let mut scope_stack = initial_state.scope_stack;

    for (i, op) in ops.iter().enumerate() {
        if let Some(ref mut replacement_states) = replacement_states {
            if let Some(new_state) = replacement_states.get_mut(&i) {
                // This means we just hit a new block: update type information
                // from the AbstractState provided to us. We won't use it again,
                // so we don't need to clone it the info.
                locals = std::mem::replace(&mut new_state.locals, Locals::empty());
                stack = std::mem::replace(&mut new_state.stack, Stack::empty());
                scope_stack = std::mem::replace(&mut new_state.scope_stack, ScopeStack::empty());
            }
        }

        macro_rules! optimize_op_to {
            ($replacement_op:expr) => {
                if do_optimize {
                    op.set($replacement_op);
                }
            };
        }

        match op.get() {
            Op::CoerceA => {
                // This does actually inhibit optimizations in FP
                stack.pop(activation)?;
                stack.push_any(activation)?;
            }
            Op::CoerceB => {
                let stack_value = stack.pop(activation)?;
                if stack_value.class == Some(types.boolean) {
                    optimize_op_to!(Op::Nop);
                }
                stack.push_class(activation, types.boolean)?;
            }
            Op::CoerceD => {
                let stack_value = stack.pop(activation)?;
                if stack_value.class == Some(types.number)
                    || stack_value.class == Some(types.int)
                    || stack_value.class == Some(types.uint)
                {
                    optimize_op_to!(Op::Nop);
                }
                stack.push_class(activation, types.number)?;
            }
            Op::CoerceI => {
                let stack_value = stack.pop(activation)?;
                if stack_value.class == Some(types.int) || stack_value.contains_valid_integer {
                    optimize_op_to!(Op::Nop);
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
                    optimize_op_to!(Op::Nop);
                }
                stack.push_class(activation, types.string)?;
            }
            Op::ConvertS => {
                stack.pop(activation)?;
                stack.push_class_not_null(activation, types.string)?;
            }
            Op::CoerceU => {
                let stack_value = stack.pop(activation)?;
                if stack_value.class == Some(types.uint) || stack_value.contains_valid_unsigned {
                    optimize_op_to!(Op::Nop);
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
            Op::PushShort { value } => {
                let mut new_value = OptValue::of_type(types.int);
                new_value.contains_valid_integer = true;
                if value >= 0 {
                    new_value.contains_valid_unsigned = true;
                }
                stack.push(activation, new_value)?;
            }
            Op::PushInt { value } => {
                let mut new_value = OptValue::of_type(types.int);
                if value < -(1 << 28) || value >= (1 << 28) {
                    // will be coerced to Number
                } else {
                    new_value.contains_valid_integer = true;
                    if value >= 0 {
                        new_value.contains_valid_unsigned = true;
                    }
                }
                stack.push(activation, new_value)?;
            }
            Op::PushUint { value } => {
                let mut new_value = OptValue::of_type(types.uint);
                if value < (1 << 28) {
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
                locals.set(index as usize, OptValue::of_type(types.int));
            }
            Op::DecLocal { index } => {
                locals.set(index as usize, OptValue::of_type(types.number));
            }
            Op::IncLocalI { index } => {
                locals.set(index as usize, OptValue::of_type(types.int));
            }
            Op::IncLocal { index } => {
                locals.set(index as usize, OptValue::of_type(types.number));
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
                stack.push_class(activation, types.uint)?;
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
                stack.popn(activation, num_args)?;

                stack.push_class_not_null(activation, types.array)?;
            }
            Op::NewObject { num_args } => {
                stack.popn(activation, num_args * 2)?;

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
                stack.popn(activation, num_types)?;

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
                stack.pop_for_multiname(activation, multiname)?;

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

                let class_is_primitive = class == types.int
                    || class == types.uint
                    || class == types.number
                    || class == types.boolean
                    || class == types.void;

                let mut new_value = OptValue::any();
                if !class_is_primitive {
                    // if T is non-nullable, we can assume the result is typed T
                    new_value = OptValue::of_type(class);
                }
                if let Some(stack_class) = stack_value.class {
                    if class == stack_class {
                        // If type check is guaranteed, preserve original type
                        // TODO: there are more cases when this can succeed,
                        // like inheritance and numbers (`x: Number = 1; x as int;`)
                        new_value = stack_value;
                    }
                }
                if stack_value.is_null() {
                    // null always turns into null
                    optimize_op_to!(Op::Nop);
                    new_value.null_state = NullState::IsNull;
                }
                stack.push(activation, new_value)?;
            }
            Op::Coerce { class } => {
                let stack_value = stack.pop(activation)?;
                let mut new_value = OptValue::of_type(class);

                if stack_value.is_null() {
                    // Coercing null to a non-primitive or void is a noop.
                    if class != types.int
                        && class != types.uint
                        && class != types.number
                        && class != types.boolean
                        && class != types.void
                    {
                        optimize_op_to!(Op::Nop);
                        new_value.null_state = NullState::IsNull;
                    }
                } else if let Some(stack_class) = stack_value.class {
                    // TODO: this could check for inheritance
                    if class == stack_class {
                        optimize_op_to!(Op::Nop);
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
                if index >= scope_stack.len() {
                    return Err(Error::AvmError(verify_error(
                        activation,
                        "Error #1019: Getscopeobject  is out of bounds.",
                        1019,
                    )?));
                }

                stack.push(activation, scope_stack.at(index).0)?;
            }
            Op::GetOuterScope { index } => {
                let class = activation
                    .outer()
                    .get_unchecked(index)
                    .values()
                    .instance_class(activation);
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
                let value = OptValue::of_type(types.void);
                locals.set(index as usize, value);
            }
            Op::SetLocal { index } => {
                let stack_value = stack.pop(activation)?;
                locals.set(index as usize, stack_value);
            }
            Op::GetLocal { index } => {
                let local_type = locals.at(index as usize);
                stack.push(activation, local_type)?;
            }
            Op::FindPropStrict { multiname } | Op::FindProperty { multiname } => {
                let mut stack_push_done = false;
                stack.pop_for_multiname(activation, multiname)?;

                if !multiname.has_lazy_component() {
                    let outer_scope = activation.outer();

                    // First check the local scope stack
                    let mut i = scope_stack.len();
                    while i > 0 {
                        i -= 1;

                        if i == 0 && outer_scope.is_empty() {
                            // Global scope works differently, see `Activation::find_definition`
                            break;
                        }

                        let checked_scope = scope_stack.at(i);

                        // This was a `with` scope; we don't know what could be on it
                        // and we should stop looking now
                        if checked_scope.1 {
                            stack_push_done = true;
                            stack.push_any(activation)?;
                            break;
                        } else if let Some(vtable) = checked_scope.0.vtable() {
                            // NOTE: There is a subtle issue with this logic;
                            // if pushing an object of type `Subclass` that was
                            // declared to be of type `Superclass` with a coerce,
                            // the scope optimizer may "skip" traits that were on
                            // `Subclass` when it assumes the value is of type
                            // `Superclass`. However, this matches avmplus's
                            // behavior- see the test `avm2/scope_optimizations`.
                            if vtable.has_trait(&multiname) {
                                optimize_op_to!(Op::GetScopeObject { index: i });

                                stack_push_done = true;
                                stack.push(activation, checked_scope.0)?;
                                break;
                            }
                        } else {
                            // We don't know the class...but to match avmplus,
                            // we keep descending the scope stack, assuming that
                            // the trait wasn't found on this scope.
                        }
                    }

                    // Then the outer scope stack
                    if !stack_push_done {
                        if let Some(info) =
                            outer_scope.get_entry_for_multiname(activation, &multiname)
                        {
                            if let Some((class, index)) = info {
                                optimize_op_to!(Op::GetOuterScope { index });

                                stack_push_done = true;
                                stack.push_class_not_null(activation, class)?;
                            } else {
                                // If `get_entry_for_multiname` returned `Some(None)`, there was
                                // a `with` scope in the outer ScopeChain- abort optimization.
                                stack_push_done = true;
                                stack.push_any(activation)?;
                            }
                        }
                    }

                    // Then check the domain
                    if !stack_push_done {
                        if let Some((_, script)) =
                            outer_scope.domain().get_defining_script(&multiname)
                        {
                            // NOTE: avmplus rewrites this into a FindDef, and it caches
                            // the results of that FindDef at runtime, rather than caching
                            // the lookup here, in the verifier. However, this discrepancy
                            // is unlikely to cause any real problems with SWFs.
                            optimize_op_to!(Op::GetScriptGlobals { script });

                            stack_push_done = true;
                            stack.push_class_not_null(activation, script.global_class())?;
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

                // FIXME this should push `int` instead of `number`, but we have
                // to fix TObject::get_next_enumerant to return i32 for that
                stack.push_class(activation, types.number)?;
            }
            Op::HasNext2 {
                index_register,
                object_register,
            } => {
                stack.push_class(activation, types.boolean)?;

                // FIXME this should set the local to `int` instead of `number`, but
                // we have to fix TObject::get_next_enumerant to return i32 for that
                locals.set(index_register as usize, OptValue::of_type(types.number));
                locals.set_any(object_register as usize);
            }
            Op::GetSlot { index: slot_id } => {
                let slot_id = slot_id as usize;

                let mut stack_push_done = false;
                let stack_value = stack.pop(activation)?;

                if let Some(vtable) = stack_value.vtable() {
                    let slot_classes = vtable.slot_classes();
                    let value_class = slot_classes.get(slot_id).copied();
                    if let Some(mut value_class) = value_class {
                        stack_push_done = true;

                        let resolved_value_class = value_class.get_class(activation)?;

                        if let Some(class) = resolved_value_class {
                            stack.push_class(activation, class)?;
                        } else {
                            stack.push_any(activation)?;
                        }

                        drop(slot_classes);
                        vtable.set_slot_class(activation.gc(), slot_id, value_class);
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
                stack.pop_for_multiname(activation, multiname)?;
                let stack_value = stack.pop(activation)?;

                if !multiname.has_lazy_component() {
                    if let Some(vtable) = stack_value.vtable() {
                        match vtable.get_trait(&multiname) {
                            Some(Property::Slot { slot_id })
                            | Some(Property::ConstSlot { slot_id }) => {
                                optimize_op_to!(Op::GetSlot { index: slot_id });

                                stack_push_done = true;

                                let mut value_class = vtable.slot_classes()[slot_id as usize];
                                let resolved_value_class = value_class.get_class(activation)?;

                                if let Some(class) = resolved_value_class {
                                    stack.push_class(activation, class)?;
                                } else {
                                    stack.push_any(activation)?;
                                }

                                vtable.set_slot_class(
                                    activation.gc(),
                                    slot_id as usize,
                                    value_class,
                                );
                            }
                            Some(Property::Virtual {
                                get: Some(disp_id), ..
                            }) => {
                                optimize_op_to!(Op::CallMethod {
                                    num_args: 0,
                                    index: disp_id,
                                    push_return_value: true,
                                });
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

                stack.pop_for_multiname(activation, multiname)?;
                let stack_value = stack.pop(activation)?;
                if !multiname.has_lazy_component() {
                    if let Some(vtable) = stack_value.vtable() {
                        match vtable.get_trait(&multiname) {
                            Some(Property::Slot { slot_id })
                            | Some(Property::ConstSlot { slot_id }) => {
                                // If the set value's type is the same as the type of the slot,
                                // a SetSlotNoCoerce can be emitted. Otherwise, emit a SetSlot.
                                let mut value_class = vtable.slot_classes()[slot_id as usize];
                                let resolved_value_class = value_class.get_class(activation)?;

                                if set_value.matches_type(activation, resolved_value_class) {
                                    optimize_op_to!(Op::SetSlotNoCoerce { index: slot_id });
                                } else {
                                    optimize_op_to!(Op::SetSlot { index: slot_id });
                                }
                            }
                            Some(Property::Virtual {
                                set: Some(disp_id), ..
                            }) => {
                                optimize_op_to!(Op::CallMethod {
                                    num_args: 1,
                                    index: disp_id,
                                    push_return_value: false,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                // `stack_pop_multiname` handled lazy
            }
            Op::SetProperty { multiname } => {
                let set_value = stack.pop(activation)?;

                stack.pop_for_multiname(activation, multiname)?;
                let stack_value = stack.pop(activation)?;
                if !multiname.has_lazy_component() {
                    if let Some(vtable) = stack_value.vtable() {
                        match vtable.get_trait(&multiname) {
                            Some(Property::Slot { slot_id }) => {
                                // If the set value's type is the same as the type of the slot,
                                // a SetSlotNoCoerce can be emitted. Otherwise, emit a SetSlot.
                                let mut value_class = vtable.slot_classes()[slot_id as usize];
                                let resolved_value_class = value_class.get_class(activation)?;

                                if set_value.matches_type(activation, resolved_value_class) {
                                    optimize_op_to!(Op::SetSlotNoCoerce { index: slot_id });
                                } else {
                                    optimize_op_to!(Op::SetSlot { index: slot_id });
                                }
                            }
                            Some(Property::Virtual {
                                set: Some(disp_id), ..
                            }) => {
                                optimize_op_to!(Op::CallMethod {
                                    num_args: 1,
                                    index: disp_id,
                                    push_return_value: false,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                // `stack_pop_multiname` handled lazy
            }
            Op::DeleteProperty { multiname } => {
                stack.pop_for_multiname(activation, multiname)?;

                stack.pop(activation)?;

                stack.push_class(activation, types.boolean)?;
            }
            Op::Construct { num_args } => {
                // Arguments
                stack.popn(activation, num_args)?;

                stack.pop(activation)?;

                // Avoid checking return value for now
                stack.push_any(activation)?;
            }
            Op::ConstructSuper { num_args } => {
                // Arguments
                stack.popn(activation, num_args)?;

                // Then receiver.
                stack.pop(activation)?;
            }
            Op::ConstructProp {
                multiname,
                num_args,
            } => {
                let mut stack_push_done = false;

                // Arguments
                stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                let stack_value = stack.pop(activation)?;

                if !multiname.has_lazy_component() {
                    if let Some(vtable) = stack_value.vtable() {
                        match vtable.get_trait(&multiname) {
                            Some(Property::Slot { slot_id })
                            | Some(Property::ConstSlot { slot_id }) => {
                                optimize_op_to!(Op::ConstructSlot {
                                    index: slot_id,
                                    num_args
                                });

                                let mut value_class = vtable.slot_classes()[slot_id as usize];
                                let resolved_value_class = value_class.get_class(activation)?;

                                if let Some(slot_class) = resolved_value_class {
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
                stack.popn(activation, num_args)?;

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
                stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                stack.pop(activation)?;

                // Avoid checking return value for now
                stack.push_any(activation)?;
            }
            Op::CallStatic { num_args, .. } => {
                // Arguments
                stack.popn(activation, num_args)?;

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
                stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                let stack_value = stack.pop(activation)?;

                if !multiname.has_lazy_component() {
                    if let Some(vtable) = stack_value.vtable() {
                        match vtable.get_trait(&multiname) {
                            Some(Property::Method { disp_id }) => {
                                optimize_op_to!(Op::CallMethod {
                                    num_args,
                                    index: disp_id,
                                    push_return_value: true,
                                });
                            }
                            Some(Property::Slot { slot_id })
                            | Some(Property::ConstSlot { slot_id }) => {
                                if stack_value.not_null(activation) {
                                    if num_args == 1 {
                                        let mut value_class =
                                            vtable.slot_classes()[slot_id as usize];
                                        let resolved_value_class =
                                            value_class.get_class(activation)?;

                                        if let Some(slot_class) = resolved_value_class {
                                            if let Some(called_class) = slot_class.i_class() {
                                                // Calling a c_class will perform a simple coercion to the class
                                                if called_class.call_handler().is_none() {
                                                    optimize_op_to!(Op::CoerceSwapPop {
                                                        class: called_class,
                                                    });

                                                    stack_push_done = true;
                                                    stack.push_class(activation, called_class)?;
                                                } else if called_class == types.int {
                                                    optimize_op_to!(Op::CoerceISwapPop);

                                                    stack_push_done = true;
                                                    stack.push_class(activation, types.int)?;
                                                } else if called_class == types.uint {
                                                    optimize_op_to!(Op::CoerceUSwapPop);

                                                    stack_push_done = true;
                                                    stack.push_class(activation, types.uint)?;
                                                } else if called_class == types.number {
                                                    optimize_op_to!(Op::CoerceDSwapPop);

                                                    stack_push_done = true;
                                                    stack.push_class(activation, types.number)?;
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
                stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                let stack_value = stack.pop(activation)?;

                if !multiname.has_lazy_component() {
                    if let Some(vtable) = stack_value.vtable() {
                        match vtable.get_trait(&multiname) {
                            Some(Property::Method { disp_id }) => {
                                optimize_op_to!(Op::CallMethod {
                                    num_args,
                                    index: disp_id,
                                    push_return_value: false,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                // `stack_pop_multiname` handled lazy
            }
            Op::GetSuper { multiname } => {
                stack.pop_for_multiname(activation, multiname)?;

                // Receiver
                stack.pop(activation)?;

                // Avoid checking return value for now
                stack.push_any(activation)?;
            }
            Op::SetSuper { multiname } => {
                stack.pop(activation)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Receiver
                stack.pop(activation)?;
            }
            Op::CallSuper {
                multiname,
                num_args,
            } => {
                // Arguments
                stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                stack.pop(activation)?;

                // Avoid checking return value for now
                stack.push_any(activation)?;
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
                let activation_class = activation.activation_class();

                stack.push_class_not_null(activation, activation_class.inner_class_definition())?;
            }
            Op::Nop => {}
            Op::DebugFile { .. }
            | Op::DebugLine { .. }
            | Op::Debug { .. }
            | Op::Bkpt
            | Op::BkptLine { .. }
            | Op::Timestamp => {}
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

            // Control flow ops
            Op::ReturnValue { return_type } => {
                let stack_value = stack.pop(activation)?;

                if stack_value.matches_type(activation, return_type) {
                    optimize_op_to!(Op::ReturnValue { return_type: None });
                }
            }

            Op::LookupSwitch(_) | Op::IfTrue { .. } | Op::IfFalse { .. } | Op::Throw => {
                stack.pop(activation)?;
            }

            // These ops don't change the state
            Op::ReturnVoid { .. } | Op::Jump { .. } => {}

            Op::CallMethod { .. }
            | Op::CoerceSwapPop { .. }
            | Op::CoerceDSwapPop
            | Op::CoerceISwapPop
            | Op::CoerceUSwapPop
            | Op::ConstructSlot { .. }
            | Op::GetScriptGlobals { .. }
            | Op::SetSlotNoCoerce { .. } => unreachable!("Custom ops should not be encountered"),
        }
    }

    Ok(AbstractState {
        locals,
        stack,
        scope_stack,
    })
}
