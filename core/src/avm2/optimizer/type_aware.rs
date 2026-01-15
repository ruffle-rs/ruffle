use crate::avm2::error::{
    make_error_1013, make_error_1017, make_error_1018, make_error_1019, make_error_1023,
    make_error_1024, make_error_1026, make_error_1030, make_error_1031, make_error_1035,
    make_error_1051, make_error_1058, make_error_1068,
};
use crate::avm2::method::{Method, MethodAssociation, MethodKind, ResolvedParamConfig};
use crate::avm2::multiname::Multiname;
use crate::avm2::op::Op;
use crate::avm2::optimizer::blocks::assemble_blocks;
use crate::avm2::property::Property;
use crate::avm2::verify::Exception;
use crate::avm2::vtable::VTable;
use crate::avm2::{Activation, Class, Error};

use gc_arena::Gc;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq)]
enum ConstantValue {
    True,
    False,
    Null,
}
impl ConstantValue {
    pub fn is_truthy(self) -> bool {
        matches!(self, ConstantValue::True)
    }

    pub fn is_falsey(self) -> bool {
        matches!(self, ConstantValue::False | ConstantValue::Null)
    }
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

    // Whether this value is guaranteed to be neither `null` nor `undefined`.
    pub not_null: bool,

    // The constant that this value represents, if we know it. For example, we
    // know that a `pushtrue` call will push `true`, represented by
    // `ConstantValue::True`.
    pub constant_value: Option<ConstantValue>,
}
impl<'gc> OptValue<'gc> {
    pub fn any() -> Self {
        Self {
            class: None,
            contains_valid_integer: false,
            contains_valid_unsigned: false,
            not_null: false,
            constant_value: None,
        }
    }

    pub fn null() -> Self {
        Self {
            class: None,
            not_null: false,
            constant_value: Some(ConstantValue::Null),
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

    pub fn vtable_and_class(self) -> Option<(VTable<'gc>, Class<'gc>)> {
        self.class
            .filter(|c| !c.is_interface())
            .map(|c| (c.vtable(), c))
    }

    pub fn is_null(self) -> bool {
        matches!(self.constant_value, Some(ConstantValue::Null))
    }

    // Returns true if the value is guaranteed to be neither Null nor Undefined.
    pub fn not_null(self) -> bool {
        if self.not_null {
            true
        } else if let Some(class) = self.class {
            // Primitives are always not-null. Note that we can't use
            // `Class::is_builtin_non_null`, as that will return true for the
            // `void` class.
            class.is_builtin_int()
                || class.is_builtin_uint()
                || class.is_builtin_number()
                || class.is_builtin_boolean()
        } else {
            false
        }
    }

    pub fn merged_with(self, other: OptValue<'gc>) -> OptValue<'gc> {
        let mut created_value = OptValue::any();

        if self.class == other.class {
            created_value.class = self.class;
        } else if other.is_null() {
            // If the other value is guaranteed to be null, we can just use our class.
            // Unless it's a non-null class.
            if let Some(self_class) = self.class {
                if !self_class.is_builtin_non_null() {
                    created_value.class = self.class;
                }
            }
        } else if self.is_null() {
            // And vice-versa.
            if let Some(other_class) = other.class {
                if !other_class.is_builtin_non_null() {
                    created_value.class = other.class;
                }
            }
        } else if let (Some(self_class), Some(other_class)) = (self.class, other.class) {
            // Check for a common superclass.
            // FIXME: Make this faster?
            let mut other_class = Some(other_class);
            'outer: while let Some(current_other_class) = other_class {
                let mut self_class = Some(self_class);
                while let Some(current_self_class) = self_class {
                    if current_other_class == current_self_class {
                        // Found a common superclass; we're done
                        created_value.class = Some(current_self_class);
                        break 'outer;
                    }

                    self_class = current_self_class.super_class();
                }

                other_class = current_other_class.super_class();
            }
        }

        if self.contains_valid_integer && other.contains_valid_integer {
            created_value.contains_valid_integer = true;
        }

        if self.contains_valid_unsigned && other.contains_valid_unsigned {
            created_value.contains_valid_unsigned = true;
        }

        if self.not_null == other.not_null {
            created_value.not_null = self.not_null;
        }

        if self.constant_value == other.constant_value {
            created_value.constant_value = self.constant_value;
        }

        created_value
    }

    // Check whether if this OptValue were stored in a slot of type `checked_type`,
    // whether it could be represented as-is, without any coercion.
    pub fn matches_type(self, checked_type: Option<Class<'gc>>) -> bool {
        // This makes the code less readable
        #![allow(clippy::if_same_then_else)]

        if let Some(checked_class) = checked_type {
            if let Some(own_class) = self.class {
                // Check if the checked class is a superclass of our class, or if
                // the checked class is `Number` and our class is `int` or `uint`
                if own_class.has_class_in_chain(checked_class) {
                    return true;
                } else if (own_class.is_builtin_int() || own_class.is_builtin_uint())
                    && checked_class.is_builtin_number()
                {
                    return true;
                }
            }

            if checked_class.is_builtin_int() && self.contains_valid_integer {
                true
            } else if checked_class.is_builtin_uint() && self.contains_valid_unsigned {
                true
            } else {
                let is_not_primitive_class = !checked_class.is_builtin_non_null();

                // Null matches every class except the primitive classes
                self.is_null() && is_not_primitive_class
            }
        } else {
            // All values match the Any type
            true
        }
    }

    // Whether this value is known not to represent a primitive type (Boolean,
    // String, int, uint, Object, void, or String)
    pub fn is_not_primitive(self) -> bool {
        self.class.is_some_and(|c| {
            !c.is_builtin_boolean()
                && !c.is_builtin_string()
                && !c.is_builtin_int()
                && !c.is_builtin_uint()
                && !c.is_builtin_object()
                && !c.is_builtin_void()
                && !c.is_builtin_string()
        })
    }

    // Whether this value is known to be truthy.
    pub fn known_truthy(self) -> bool {
        if self.constant_value.is_some_and(|v| v.is_truthy()) {
            // If this value is known to be a constant value, and that value is
            // truthy, then return true
            true
        } else if self.is_not_primitive() && self.not_null {
            // Otherwise, if this value is an object type that isn't null, also
            // return true
            true
        } else {
            false
        }
    }

    // Whether this value is known to be falsey.
    pub fn known_falsey(self) -> bool {
        if self.constant_value.is_some_and(|v| v.is_falsey()) {
            // If this value is known to be a constant value, and that value is
            // truthy, then return true
            // NOTE: This condition is also met if this value is known to be
            // `null`, since we represent that using `ConstantValue::Null`
            true
        } else {
            false
        }
    }
}

// This type is used throughout the abstract interpreter, make sure it doesn't
// grow too much
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<OptValue>() == 16);

impl std::fmt::Debug for OptValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("OptValue")
            .field("class", &self.class)
            .field("contains_valid_integer", &self.contains_valid_integer)
            .field("contains_valid_unsigned", &self.contains_valid_unsigned)
            .field("not_null", &self.not_null)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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
        value.not_null = true;

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
            return Err(make_error_1023(activation));
        }

        self.0.push(value);

        Ok(())
    }

    fn pop(&mut self, activation: &mut Activation<'_, 'gc>) -> Result<OptValue<'gc>, Error<'gc>> {
        if self.0.is_empty() {
            return Err(make_error_1024(activation));
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

    fn popn(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        count: u32,
    ) -> Result<Vec<OptValue<'gc>>, Error<'gc>> {
        let mut vec = vec![OptValue::any(); count as usize];
        for item in vec.iter_mut().rev() {
            *item = self.pop(activation)?;
        }

        Ok(vec)
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
    fn new(max_height: usize) -> Self {
        Self(Vec::new(), max_height)
    }

    fn push(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        value: OptValue<'gc>,
    ) -> Result<(), Error<'gc>> {
        if self.len() >= self.max_height() {
            return Err(make_error_1017(activation));
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
            return Err(make_error_1017(activation));
        }

        self.0.push((value, true));

        Ok(())
    }

    fn pop(&mut self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        if self.0.is_empty() {
            return Err(make_error_1018(activation));
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

#[derive(Clone, Debug)]
struct AbstractState<'gc> {
    locals: Locals<'gc>,
    stack: Stack<'gc>,
    scope_stack: ScopeStack<'gc>,
}

struct AbstractStateRef<'a, 'gc> {
    locals: &'a Locals<'gc>,
    stack: &'a Stack<'gc>,
    scope_stack: &'a ScopeStack<'gc>,
}

impl<'gc> AbstractStateRef<'_, 'gc> {
    fn to_owned(&self) -> AbstractState<'gc> {
        AbstractState {
            locals: self.locals.clone(),
            stack: self.stack.clone(),
            scope_stack: self.scope_stack.clone(),
        }
    }
}

impl<'gc> AbstractState<'gc> {
    #[inline(never)]
    fn merge_with(
        &mut self,
        activation: &mut Activation<'_, 'gc>,
        other: &AbstractStateRef<'_, 'gc>,
    ) -> Result<bool, Error<'gc>> {
        let mut changed = false;

        // Merge locals
        assert!(self.locals.len() == other.locals.len());

        for i in 0..self.locals.len() {
            let our_local = self.locals.at(i);
            let other_local = other.locals.at(i);

            let merged = our_local.merged_with(other_local);
            self.locals.set(i, merged);
            if merged != our_local {
                changed = true;
            }
        }

        // Merge stack
        if self.stack.len() != other.stack.len() {
            return Err(make_error_1030(
                activation,
                other.stack.len(),
                self.stack.len(),
            ));
        }

        for i in 0..self.stack.len() {
            let our_entry = self.stack.at(i);
            let other_entry = other.stack.at(i);

            let merged = our_entry.merged_with(other_entry);
            self.stack.set(i, merged);
            if merged != our_entry {
                changed = true;
            }
        }

        // Merge scope stack
        if self.scope_stack.len() != other.scope_stack.len() {
            return Err(make_error_1031(
                activation,
                other.scope_stack.len(),
                self.scope_stack.len(),
            ));
        }

        for i in 0..self.scope_stack.len() {
            let our_scope = self.scope_stack.at(i);
            let other_scope = other.scope_stack.at(i);

            if our_scope.1 != other_scope.1 {
                return Err(make_error_1068(activation));
            }

            let merged = our_scope.0.merged_with(other_scope.0);
            self.scope_stack.set(i, merged, our_scope.1);
            if merged != our_scope.0 {
                changed = true;
            }
        }
        Ok(changed)
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

/// Run the type-aware optimizer pass on the ops. This optimizer pass also
/// performs type verification, so it must be run regardless of whether the
/// "disable AVM2 optimizer" player option is on or not. If the "disable AVM2
// optimizer" player option is on, this method will not actually optimize the
// resulting code.
pub fn type_aware_optimize<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    code_slice: &[Cell<Op<'gc>>],
    method_exceptions: &mut [Exception<'gc>],
    resolved_parameters: &[ResolvedParamConfig<'gc>],
    jump_targets: &mut HashSet<usize>,
) -> Result<(), Error<'gc>> {
    let (block_list, op_index_to_block_index_table) = assemble_blocks(code_slice, jump_targets);

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

    let this_class = method.bound_class();

    let mut this_value = OptValue::any();
    this_value.class = this_class;
    this_value.not_null = true;

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

    if method.is_variadic() {
        // Set the local variable holding restargs/arguments to the `Array` type
        let mut array_type = OptValue::of_type(types.array);
        array_type.not_null = true;

        initial_local_types.set(resolved_parameters.len() + 1, array_type);
    }

    let empty_stack = Stack::new(method_body.max_stack as usize);
    let empty_scope_stack =
        ScopeStack::new((method_body.max_scope_depth - method_body.init_scope_depth) as usize);

    let entry_state = AbstractState {
        locals: initial_local_types,
        stack: empty_stack,
        scope_stack: empty_scope_stack,
    };

    // A Vec holding lists of possible abstract states, indexed by block index
    let mut abstract_states: Vec<Option<AbstractState<'gc>>> = vec![None; block_list.len()];

    abstract_states[0] = Some(entry_state);

    // Block #0 is the entry block
    let mut worklist = vec![0];
    while let Some(block_idx) = worklist.pop() {
        let block = &block_list[block_idx];

        let block_entry_state = abstract_states[block_idx]
            .clone()
            .expect("Entry state not found");

        abstract_interpret_ops(
            activation,
            block.start_index,
            block.ops,
            block_entry_state,
            &mut abstract_states,
            &types,
            &op_index_to_block_index_table,
            method_exceptions,
            &mut worklist,
            false,
        )?;
    }

    if activation.avm2().optimizer_enabled() {
        // Now run through the ops and actually optimize them
        for (i, block) in block_list.iter().enumerate() {
            // todo: don't need to clone here
            let block_entry_state = abstract_states[i].clone().expect("Entry state not found");
            abstract_interpret_ops(
                activation,
                block.start_index,
                block.ops,
                block_entry_state,
                &mut abstract_states,
                &types,
                &op_index_to_block_index_table,
                method_exceptions,
                &mut worklist,
                true,
            )?;
        }
    }

    // It's possible that an optimization removed some jumps, so let's
    // recalculate the jump targets. This makes some later optimizations, such
    // as dead code elimination, more likely to happen.
    recalculate_jump_targets(code_slice, method_exceptions, jump_targets);

    Ok(())
}

fn process_jump<'gc>(
    activation: &mut Activation<'_, 'gc>,
    target: usize,
    abstract_states: &mut [Option<AbstractState<'gc>>],
    current_state: &AbstractStateRef<'_, 'gc>,
    op_index_to_block_index_table: &HashMap<usize, usize>,
    worklist: &mut Vec<usize>,
    do_optimize: bool,
) -> Result<(), Error<'gc>> {
    if do_optimize {
        return Ok(());
    }

    let target_block_id = *op_index_to_block_index_table
        .get(&target)
        .expect("unexpected jump target");
    if let Some(target_state) = &mut abstract_states[target_block_id] {
        let state_changed = target_state.merge_with(activation, current_state)?;
        if !state_changed {
            // We've already verified that this state works, no need to run it again
            return Ok(());
        }
    } else {
        // We don't have any state in the state list yet, so we use the provided one
        abstract_states[target_block_id] = Some(current_state.to_owned());
    };

    // FP reschedules blocks to the front of queue (for us, it'd be back of the vec).
    // I don't know if there's any good reason for that, but not doing it is faster.
    if !worklist.contains(&target_block_id) {
        worklist.push(target_block_id);
    }

    Ok(())
}

#[expect(clippy::too_many_arguments)]
fn abstract_interpret_ops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    start_index: usize,
    ops: &[Cell<Op<'gc>>],
    initial_state: AbstractState<'gc>,
    abstract_states: &mut [Option<AbstractState<'gc>>],
    types: &Types<'gc>,
    op_index_to_block_index_table: &HashMap<usize, usize>,
    method_exceptions: &[Exception<'gc>],
    worklist: &mut Vec<usize>,
    do_optimize: bool,
) -> Result<(), Error<'gc>> {
    let mut locals = initial_state.locals;
    let mut stack = initial_state.stack;
    let mut scope_stack = initial_state.scope_stack;

    for (i, op) in ops.iter().enumerate() {
        if op.get().can_throw_error() {
            let current_idx = start_index + i;
            for exception in method_exceptions {
                if current_idx >= exception.from_offset && current_idx < exception.to_offset {
                    // When branching as a result of an exception in a catch block,
                    // the exception target will be run starting with an empty
                    // scope stack and a stack with a single entry on it.

                    // The abstract_states[0] access is here to copy
                    // the empty stacks, without repeating code to create them from scratch.
                    let mut exception_stack = abstract_states[0].as_ref().unwrap().stack.clone();
                    exception_stack.push_any(activation)?;
                    let empty_scope_stack =
                        abstract_states[0].as_ref().unwrap().scope_stack.clone();

                    let current_state = AbstractStateRef {
                        locals: &locals,
                        stack: &exception_stack,
                        scope_stack: &empty_scope_stack,
                    };
                    process_jump(
                        activation,
                        exception.target_offset,
                        abstract_states,
                        &current_state,
                        op_index_to_block_index_table,
                        worklist,
                        do_optimize,
                    )?;
                }
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

                let mut value = OptValue::of_type(types.boolean);

                // If we know whether the value on the stack was truthy or
                // falsey, we know what this `CoerceB` will push
                if stack_value.known_truthy() {
                    value.constant_value = Some(ConstantValue::True);
                } else if stack_value.known_falsey() {
                    value.constant_value = Some(ConstantValue::False);
                }

                stack.push(activation, value)?;
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
            Op::PushTrue => {
                let mut new_value = OptValue::of_type(types.boolean);
                new_value.constant_value = Some(ConstantValue::True);
                stack.push(activation, new_value)?;
            }
            Op::PushFalse => {
                let mut new_value = OptValue::of_type(types.boolean);
                new_value.constant_value = Some(ConstantValue::False);
                stack.push(activation, new_value)?;
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
                } else if (value1.class == Some(types.string) && value1.not_null())
                    || (value2.class == Some(types.string) && value2.not_null())
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
            Op::NewFunction { method } => {
                // The method must be suitable for use as a freestanding method.
                method.associate(activation, MethodAssociation::freestanding())?;

                stack.push_class_not_null(activation, types.function)?;
            }
            Op::NewClass { class } => {
                let c_class = class.c_class().expect("NewClass holds an i_class");
                stack.pop(activation)?;
                stack.push_class_not_null(activation, c_class)?;
            }
            Op::NewCatch { index } => {
                let catch_class = &method_exceptions[index].catch_class;

                if let Some(catch_class) = catch_class {
                    stack.push_class_not_null(activation, *catch_class)?;
                } else {
                    stack.push_class_not_null(activation, types.object)?;
                }
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
                    if !class.is_builtin_non_null() {
                        // If the type on the stack was a c_class with a non-primitive
                        // i_class, we can use the type
                        new_value = OptValue::of_type(class);
                    }
                }

                stack.push(activation, new_value)?;
            }
            Op::AsType { class } => {
                let stack_value = stack.pop(activation)?;

                let mut new_value = OptValue::any();
                if !class.is_builtin_non_null() {
                    // if T is not non-nullable, we can assume the result is typed T
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
                    new_value.constant_value = Some(ConstantValue::Null);
                }
                stack.push(activation, new_value)?;
            }
            Op::Coerce { class } => {
                let stack_value = stack.pop(activation)?;
                let mut new_value = OptValue::of_type(class);

                if stack_value.is_null() {
                    // Coercing null to a class is a noop, as long as that class
                    // isn't one of the special non-null classes.
                    if !class.is_builtin_non_null() {
                        optimize_op_to!(Op::Nop);
                        new_value.constant_value = Some(ConstantValue::Null);
                    }
                } else if let Some(stack_class) = stack_value.class {
                    // TODO: this could check for inheritance
                    if class == stack_class {
                        optimize_op_to!(Op::Nop);
                        new_value.not_null = stack_value.not_null;
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
                    return Err(make_error_1019(activation, Some(index)));
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
            Op::StoreLocal { .. } => {
                unreachable!("Only the peephole optimizer emits StoreLocal")
            }
            Op::FindPropStrict { multiname } | Op::FindProperty { multiname } => {
                let outer_scope = activation.outer();
                if outer_scope.is_empty() && scope_stack.is_empty() {
                    return Err(make_error_1013(activation));
                }

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
            Op::FindDef { multiname } => {
                let domain = activation.domain();

                if let Some((_, script)) = domain.get_defining_script(&multiname) {
                    // See comment above for GetScriptGlobals optimization
                    optimize_op_to!(Op::GetScriptGlobals { script });

                    stack.push_class_not_null(activation, script.global_class())?;
                } else {
                    stack.push_any(activation)?;
                }
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
                let stack_value = stack.pop(activation)?;

                // The value must have a vtable
                let Some((vtable, class)) = stack_value.vtable_and_class() else {
                    return Err(make_error_1051(activation));
                };

                // The slot must be a valid slot
                let Some(mut value_class) = vtable.slot_class(slot_id) else {
                    // We store slots 0-indexed, but FP stores them 1-indexed; add
                    // 1 to the slot id to make the error message match FP
                    return Err(make_error_1026(
                        activation,
                        slot_id + 1,
                        Some(vtable.default_slots().len()),
                        Some(class),
                    ));
                };

                let resolved_value_class = value_class.get_class(activation)?;

                vtable.set_slot_class(activation.gc(), slot_id, value_class);

                if let Some(class) = resolved_value_class {
                    stack.push_class(activation, class)?;
                } else {
                    stack.push_any(activation)?;
                }
            }
            Op::SetSlot { index: slot_id } => {
                let set_value = stack.pop(activation)?;
                let stack_value = stack.pop(activation)?;

                // The value must have a vtable
                let Some((vtable, class)) = stack_value.vtable_and_class() else {
                    return Err(make_error_1051(activation));
                };

                // The slot must be a valid slot
                let Some(mut value_class) = vtable.slot_class(slot_id) else {
                    // We store slots 0-indexed, but FP stores them 1-indexed; add
                    // 1 to the slot id to make the error message match FP
                    return Err(make_error_1026(
                        activation,
                        slot_id + 1,
                        Some(vtable.default_slots().len()),
                        Some(class),
                    ));
                };

                let resolved_value_class = value_class.get_class(activation)?;

                vtable.set_slot_class(activation.gc(), slot_id, value_class);

                // Skip the coercion when possible
                if set_value.matches_type(resolved_value_class) {
                    optimize_op_to!(Op::SetSlotNoCoerce { index: slot_id });
                }
            }
            Op::GetPropertyStatic { multiname } => {
                // Verifier only emits this op when the multiname is static
                assert!(!multiname.has_lazy_component());

                let stack_value = stack.pop(activation)?;
                let opt_result = optimize_get_property(activation, multiname, stack_value)?;

                if let Some((new_op, return_type)) = opt_result {
                    optimize_op_to!(new_op);

                    if let Some(return_type) = return_type {
                        stack.push_class(activation, return_type)?;
                    } else {
                        stack.push_any(activation)?;
                    }
                } else {
                    stack.push_any(activation)?;
                }
            }
            Op::GetPropertyFast { multiname } => {
                // Verifier only emits this op when the multiname is lazy
                assert!(multiname.has_lazy_name() && !multiname.has_lazy_ns());

                let mut stack_push_done = false;

                let index = stack.pop(activation)?;
                let stack_value = stack.pop(activation)?;

                if let Some(param) = stack_value.class.and_then(|c| c.param()) {
                    // This is a Vector class, if our index is numeric and the
                    // multiname is valid for indexing then we know the type of
                    // the result

                    let index_numeric = index.class.is_some_and(|c| c.is_builtin_numeric());

                    // In non-JIT mode, GetPropertyFast can be emitted even when
                    // the multiname isn't valid for indexing (see comment in
                    // `verify::translate_op`), so we need to check here again
                    let multiname_valid = multiname.valid_dynamic_name();

                    if index_numeric && multiname_valid {
                        if let Some(param) = param {
                            // NOTE this is a bug in FP, in SWFv10 indexing
                            // vectors with a numeric index is not actually
                            // guaranteed to produce a result of the correct
                            // type. For some reason, this special-case of
                            // int/uint/number isn't version-gated.
                            if param.is_builtin_int()
                                || param.is_builtin_uint()
                                || param.is_builtin_number()
                            {
                                stack_push_done = true;
                                stack.push_class(activation, param)?;
                            } else if activation.caller_movie_or_root().version() >= 14 {
                                // The general case, meanwhile, *is* correctly
                                // version-gated.
                                stack_push_done = true;
                                stack.push_class(activation, param)?;
                            }
                        }
                    }
                }

                if !stack_push_done {
                    stack.push_any(activation)?;
                }
            }
            Op::GetPropertySlow { multiname } => {
                // Verifier only emits this op when the multiname is lazy
                assert!(multiname.has_lazy_component());

                stack.pop_for_multiname(activation, multiname)?;

                let _stack_value = stack.pop(activation)?;

                // `stack_pop_multiname` handled lazy

                stack.push_any(activation)?;
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
                                let mut value_class =
                                    vtable.slot_class(slot_id).expect("Slot should exist");
                                let resolved_value_class = value_class.get_class(activation)?;

                                vtable.set_slot_class(activation.gc(), slot_id, value_class);

                                if set_value.matches_type(resolved_value_class) {
                                    optimize_op_to!(Op::SetSlotNoCoerce { index: slot_id });
                                } else {
                                    optimize_op_to!(Op::SetSlot { index: slot_id });
                                }
                            }
                            Some(Property::Virtual {
                                set: Some(disp_id), ..
                            }) => {
                                let method =
                                    vtable.get_method(disp_id).expect("Method should exist");

                                let mut result_op = Op::CallMethod {
                                    num_args: 1,
                                    index: disp_id,
                                    push_return_value: false,
                                };

                                // We can further optimize calling FastCall setters into a
                                // static native method call
                                maybe_optimize_static_call(
                                    activation,
                                    &mut result_op,
                                    method,
                                    stack_value,
                                    &[set_value], // passed args
                                    false,        // push_return_value
                                )?;

                                optimize_op_to!(result_op);
                            }
                            _ => {}
                        }
                    }
                }
                // `stack_pop_multiname` handled lazy
            }
            Op::SetPropertyStatic { multiname } => {
                // Verifier only emits this op when the multiname is static
                assert!(!multiname.has_lazy_component());

                let set_value = stack.pop(activation)?;

                let stack_value = stack.pop(activation)?;
                if let Some(vtable) = stack_value.vtable() {
                    match vtable.get_trait(&multiname) {
                        Some(Property::Slot { slot_id }) => {
                            // If the set value's type is the same as the type of the slot,
                            // a SetSlotNoCoerce can be emitted. Otherwise, emit a SetSlot.
                            let mut value_class =
                                vtable.slot_class(slot_id).expect("Slot should exist");
                            let resolved_value_class = value_class.get_class(activation)?;

                            vtable.set_slot_class(activation.gc(), slot_id, value_class);

                            if set_value.matches_type(resolved_value_class) {
                                optimize_op_to!(Op::SetSlotNoCoerce { index: slot_id });
                            } else {
                                optimize_op_to!(Op::SetSlot { index: slot_id });
                            }
                        }
                        Some(Property::Virtual {
                            set: Some(disp_id), ..
                        }) => {
                            let method = vtable.get_method(disp_id).expect("Method should exist");

                            let mut result_op = Op::CallMethod {
                                num_args: 1,
                                index: disp_id,
                                push_return_value: false,
                            };

                            // We can further optimize calling FastCall setters into a
                            // static native method call
                            maybe_optimize_static_call(
                                activation,
                                &mut result_op,
                                method,
                                stack_value,
                                &[set_value], // passed args
                                false,        // push_return_value
                            )?;

                            optimize_op_to!(result_op);
                        }
                        _ => {}
                    }
                }
                // `stack_pop_multiname` handled lazy
            }
            Op::SetPropertyFast { multiname } | Op::SetPropertySlow { multiname } => {
                // Verifier only emits these ops when the multiname is lazy
                assert!(multiname.has_lazy_component());
                let _set_value = stack.pop(activation)?;

                stack.pop_for_multiname(activation, multiname)?;

                let _stack_value = stack.pop(activation)?;

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

                let constructed_value = stack.pop(activation)?;
                if let Some(instance_class) = constructed_value.class.and_then(|c| c.i_class()) {
                    // ConstructProp on a c_class will construct its i_class
                    stack.push_class_not_null(activation, instance_class)?;
                } else {
                    stack.push_any(activation)?;
                }
            }
            Op::ConstructSuper { num_args } => {
                let Some(bound_superclass_object) = activation.bound_superclass_object() else {
                    return Err(make_error_1035(activation));
                };

                // Arguments
                stack.popn(activation, num_args)?;

                // Then receiver.
                let receiver = stack.pop(activation)?;

                // Remove `super()` calls in classes that extend Object, since they
                // are noops anyway.
                if num_args == 0 {
                    let object_class = activation.avm2().class_defs().object;
                    if bound_superclass_object.inner_class_definition() == object_class {
                        // When the receiver is null, this op can still throw an
                        // error, so let's ensure it's guaranteed nonnull before
                        // optimizing it
                        if receiver.not_null() {
                            optimize_op_to!(Op::Pop);
                        }
                    }
                }
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
                                let mut value_class =
                                    vtable.slot_class(slot_id).expect("Slot should exist");
                                let resolved_value_class = value_class.get_class(activation)?;

                                if let Some(slot_class) = resolved_value_class {
                                    if let Some(instance_class) = slot_class.i_class() {
                                        optimize_op_to!(Op::ConstructSlot {
                                            index: slot_id,
                                            num_args
                                        });

                                        // ConstructProp on a c_class will construct its i_class
                                        stack_push_done = true;
                                        stack.push_class_not_null(activation, instance_class)?;
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

                stack.push_any(activation)?;
            }
            Op::CallStatic { method, num_args } => {
                // The method must be already be a class-bound method.
                method.check_classbound(activation)?;

                // Arguments
                stack.popn(activation, num_args)?;

                // Then receiver.
                stack.pop(activation)?;

                method.resolve_info(activation)?;
                let return_type = method.resolved_return_type();

                if let Some(return_type) = return_type {
                    stack.push_class(activation, return_type)?;
                } else {
                    stack.push_any(activation)?;
                }
            }
            Op::CallProperty {
                multiname,
                num_args,
            }
            | Op::CallPropLex {
                multiname,
                num_args,
            } => {
                // Arguments
                let args = stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                let stack_value = stack.pop(activation)?;

                if !multiname.has_lazy_component() {
                    let opt_result = optimize_call_property(
                        activation,
                        types,
                        multiname,
                        stack_value,
                        &args,
                        true,
                    )?;

                    if let Some((new_op, return_type)) = opt_result {
                        optimize_op_to!(new_op);

                        if let Some(return_type) = return_type {
                            stack.push_class(activation, return_type)?;
                        } else {
                            stack.push_any(activation)?;
                        }
                    } else {
                        stack.push_any(activation)?;
                    }
                } else {
                    // `stack_pop_multiname` handled lazy

                    stack.push_any(activation)?;
                }
            }
            Op::CallPropVoid {
                multiname,
                num_args,
            } => {
                // Arguments
                let args = stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                let stack_value = stack.pop(activation)?;

                if !multiname.has_lazy_component() {
                    let opt_result = optimize_call_property(
                        activation,
                        types,
                        multiname,
                        stack_value,
                        &args,
                        false,
                    )?;

                    if let Some((new_op, _)) = opt_result {
                        optimize_op_to!(new_op);
                    }
                }
            }
            Op::GetSuper { multiname } => {
                let Some(bound_superclass_object) = activation.bound_superclass_object() else {
                    return Err(make_error_1035(activation));
                };

                stack.pop_for_multiname(activation, multiname)?;

                // Receiver
                stack.pop(activation)?;

                // Push return value to the stack
                if !multiname.has_lazy_component() {
                    let vtable = bound_superclass_object.instance_vtable();
                    match vtable.get_trait(&multiname) {
                        Some(Property::Slot { slot_id })
                        | Some(Property::ConstSlot { slot_id }) => {
                            let mut value_class =
                                vtable.slot_class(slot_id).expect("Slot should exist");
                            let resolved_value_class = value_class.get_class(activation)?;

                            vtable.set_slot_class(activation.gc(), slot_id, value_class);

                            // TODO: We can optimize the op to GetSlot here

                            if let Some(resolved_value_class) = resolved_value_class {
                                stack.push_class(activation, resolved_value_class)?;
                            } else {
                                stack.push_any(activation)?;
                            }
                        }
                        Some(Property::Virtual {
                            get: Some(disp_id), ..
                        }) => {
                            let method = vtable.get_method(disp_id).expect("Method should exist");

                            method.resolve_info(activation)?;

                            // TODO: Use `maybe_optimize_static_call` here

                            let return_type = method.resolved_return_type();
                            if let Some(return_type) = return_type {
                                stack.push_class(activation, return_type)?;
                            } else {
                                stack.push_any(activation)?;
                            }
                        }
                        _ => {
                            stack.push_any(activation)?;
                        }
                    }
                } else {
                    stack.push_any(activation)?;
                }
            }
            Op::SetSuper { multiname } => {
                if activation.bound_superclass_object().is_none() {
                    return Err(make_error_1035(activation));
                }

                stack.pop(activation)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Receiver
                stack.pop(activation)?;

                // TODO: Optimize the op when possible
            }
            Op::CallSuper {
                multiname,
                num_args,
            } => {
                let Some(bound_superclass_object) = activation.bound_superclass_object() else {
                    return Err(make_error_1035(activation));
                };

                // Arguments
                stack.popn(activation, num_args)?;

                stack.pop_for_multiname(activation, multiname)?;

                // Then receiver.
                stack.pop(activation)?;

                // Push return value to the stack
                if !multiname.has_lazy_component() {
                    let vtable = bound_superclass_object.instance_vtable();
                    match vtable.get_trait(&multiname) {
                        Some(Property::Method { disp_id }) => {
                            let method = vtable.get_method(disp_id).expect("Method should exist");

                            method.resolve_info(activation)?;

                            // TODO: Use `maybe_optimize_static_call` here

                            let return_type = method.resolved_return_type();
                            if let Some(return_type) = return_type {
                                stack.push_class(activation, return_type)?;
                            } else {
                                stack.push_any(activation)?;
                            }
                        }
                        _ => {
                            stack.push_any(activation)?;
                        }
                    }
                } else {
                    stack.push_any(activation)?;
                }
            }
            Op::SetGlobalSlot { .. } => {
                let outer_scope = activation.outer();
                if outer_scope.is_empty() && scope_stack.is_empty() {
                    return Err(make_error_1019(activation, Some(0)));
                }

                stack.pop(activation)?;
            }
            Op::NewActivation { activation_class } => {
                stack.push_class_not_null(activation, activation_class)?;
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
            Op::ReturnVoid { .. } => {
                return Ok(());
            }
            Op::ReturnValue { return_type } => {
                let stack_value = stack.pop(activation)?;

                if stack_value.matches_type(return_type) {
                    optimize_op_to!(Op::ReturnValue { return_type: None });
                }
                return Ok(());
            }
            Op::Jump { offset } => {
                let current_state = AbstractStateRef {
                    locals: &locals,
                    stack: &stack,
                    scope_stack: &scope_stack,
                };
                process_jump(
                    activation,
                    offset,
                    abstract_states,
                    &current_state,
                    op_index_to_block_index_table,
                    worklist,
                    do_optimize,
                )?;
                return Ok(());
            }
            Op::IfTrue { offset } => {
                let stack_value = stack.pop(activation)?;

                if stack_value.known_falsey() {
                    // We know the branch will never be taken
                    optimize_op_to!(Op::Pop);
                } else if stack_value.known_truthy() {
                    // We know the branch will always be taken
                    optimize_op_to!(Op::PopJump { offset });
                }

                let current_state = AbstractStateRef {
                    locals: &locals,
                    stack: &stack,
                    scope_stack: &scope_stack,
                };
                process_jump(
                    activation,
                    offset,
                    abstract_states,
                    &current_state,
                    op_index_to_block_index_table,
                    worklist,
                    do_optimize,
                )?;
            }
            Op::IfFalse { offset } => {
                let stack_value = stack.pop(activation)?;

                if stack_value.known_truthy() {
                    // We know the branch will never be taken
                    optimize_op_to!(Op::Pop);
                } else if stack_value.known_falsey() {
                    // We know the branch will always be taken
                    optimize_op_to!(Op::PopJump { offset });
                }

                let current_state = AbstractStateRef {
                    locals: &locals,
                    stack: &stack,
                    scope_stack: &scope_stack,
                };
                process_jump(
                    activation,
                    offset,
                    abstract_states,
                    &current_state,
                    op_index_to_block_index_table,
                    worklist,
                    do_optimize,
                )?;
            }
            Op::LookupSwitch(lookup_switch) => {
                let value = stack.pop(activation)?;

                if value.class.is_none_or(|c| !c.is_builtin_int()) {
                    // LookupSwitch expects an int
                    return Err(make_error_1058(activation));
                }

                let current_state = AbstractStateRef {
                    locals: &locals,
                    stack: &stack,
                    scope_stack: &scope_stack,
                };
                for target in lookup_switch
                    .case_offsets
                    .iter()
                    .chain(std::slice::from_ref(&lookup_switch.default_offset))
                {
                    process_jump(
                        activation,
                        target.get(),
                        abstract_states,
                        &current_state,
                        op_index_to_block_index_table,
                        worklist,
                        do_optimize,
                    )?;
                }
                return Ok(());
            }
            Op::Throw => {
                stack.pop(activation)?;
                return Ok(());
            }

            Op::CallMethod { .. }
            | Op::CallNative { .. }
            | Op::CoerceSwapPop { .. }
            | Op::CoerceDSwapPop
            | Op::CoerceISwapPop
            | Op::CoerceUSwapPop
            | Op::ConstructSlot { .. }
            | Op::GetScriptGlobals { .. }
            | Op::PopJump { .. }
            | Op::SetSlotNoCoerce { .. } => unreachable!("Custom ops should not be encountered"),
        }
    }

    // if we didn't return earlier, we must jump to the next block.
    let target = start_index + ops.len();
    let current_state = AbstractStateRef {
        locals: &locals,
        stack: &stack,
        scope_stack: &scope_stack,
    };
    process_jump(
        activation,
        target,
        abstract_states,
        &current_state,
        op_index_to_block_index_table,
        worklist,
        do_optimize,
    )?;
    Ok(())
}

// Optimize a dynamic-dispatch call (`callmethod`) to a static-dispatch call
// (`callnative`, etc) if possible.
fn maybe_optimize_static_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    result_op: &mut Op<'gc>,
    speculated_method: Method<'gc>,
    receiver: OptValue<'gc>,
    passed_args: &[OptValue<'gc>],
    push_return_value: bool,
) -> Result<(), Error<'gc>> {
    speculated_method.resolve_info(activation)?;

    let declared_params = speculated_method.resolved_param_config();

    if receiver.class.is_some_and(|c| c.is_final()) {
        if let MethodKind::Native {
            native_method,
            fast_call: true,
        } = speculated_method.method_kind()
        {
            if declared_params.len() == passed_args.len() {
                let mut all_matches = true;
                for (i, passed_arg) in passed_args.iter().enumerate() {
                    let declared_param = &declared_params[i];
                    if !passed_arg.matches_type(declared_param.param_type) {
                        all_matches = false;
                    }
                }

                if all_matches {
                    *result_op = Op::CallNative {
                        method: *native_method,
                        num_args: passed_args.len() as u32,
                        push_return_value,
                    };
                }
            }
        }
    }

    Ok(())
}

// Optimize a `getproperty` of the given `multiname` on a value `stack_value`.
// If optimization succeeds, the optimized version of the op and the value to
// push to the stack will be returned; if it fails, the function returns `None`.
fn optimize_get_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    multiname: Gc<'gc, Multiname<'gc>>,
    stack_value: OptValue<'gc>,
) -> Result<Option<(Op<'gc>, Option<Class<'gc>>)>, Error<'gc>> {
    // Makes the code less readable
    #![allow(clippy::collapsible_if)]

    if let Some(vtable) = stack_value.vtable() {
        match vtable.get_trait(&multiname) {
            Some(Property::Slot { slot_id }) | Some(Property::ConstSlot { slot_id }) => {
                let mut value_class = vtable.slot_class(slot_id).expect("Slot should exist");
                let resolved_value_class = value_class.get_class(activation)?;

                vtable.set_slot_class(activation.gc(), slot_id, value_class);

                return Ok(Some((Op::GetSlot { index: slot_id }, resolved_value_class)));
            }
            Some(Property::Virtual {
                get: Some(disp_id), ..
            }) => {
                let method = vtable.get_method(disp_id).expect("Method should exist");

                let mut result_op = Op::CallMethod {
                    num_args: 0,
                    index: disp_id,
                    push_return_value: true,
                };

                // We can further optimize calling FastCall getters into a
                // static native method call
                maybe_optimize_static_call(
                    activation,
                    &mut result_op,
                    method,
                    stack_value,
                    &[],  // passed args
                    true, // push_return_value
                )?;

                let return_type = method.resolved_return_type();
                return Ok(Some((result_op, return_type)));
            }
            _ => {}
        }
    }

    Ok(None)
}

// Optimize a `callproperty` of the given `multiname` on a value `stack_value`,
// with the method being passed the parameters `passed_args`.
// If optimization succeeds, the optimized version of the op and the value to
// push to the stack will be returned; if it fails, the function returns `None`.
fn optimize_call_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    types: &Types<'gc>,
    multiname: Gc<'gc, Multiname<'gc>>,
    stack_value: OptValue<'gc>,
    passed_args: &[OptValue<'gc>],
    push_return_value: bool,
) -> Result<Option<(Op<'gc>, Option<Class<'gc>>)>, Error<'gc>> {
    // Makes the code less readable
    #![allow(clippy::collapsible_if)]

    let num_args = passed_args.len() as u32;

    if let Some(vtable) = stack_value.vtable() {
        match vtable.get_trait(&multiname) {
            Some(Property::Method { disp_id }) => {
                let method = vtable.get_method(disp_id).expect("Method should exist");

                let mut result_op = Op::CallMethod {
                    num_args,
                    index: disp_id,
                    push_return_value,
                };

                // If this is calling a FastCall native method on a final class,
                // and the provided arguments match the method's signature, we
                // can further optimize the call to a static native method call
                // and skip the argument coercion.
                maybe_optimize_static_call(
                    activation,
                    &mut result_op,
                    method,
                    stack_value,
                    passed_args,
                    push_return_value,
                )?;

                let return_type = method.resolved_return_type();
                return Ok(Some((result_op, return_type)));
            }
            Some(Property::Slot { slot_id }) | Some(Property::ConstSlot { slot_id }) => {
                // Don't optimize this for `callpropvoid`
                if push_return_value {
                    if stack_value.not_null() {
                        if num_args == 1 {
                            let mut value_class =
                                vtable.slot_class(slot_id).expect("Slot should exist");
                            let resolved_value_class = value_class.get_class(activation)?;

                            if let Some(slot_class) = resolved_value_class {
                                if let Some(called_class) = slot_class.i_class() {
                                    // Calling a c_class will perform a simple coercion to the class
                                    let result = if called_class.call_handler().is_none() {
                                        Some((
                                            Op::CoerceSwapPop {
                                                class: called_class,
                                            },
                                            called_class,
                                        ))
                                    } else if called_class == types.int {
                                        Some((Op::CoerceISwapPop, types.int))
                                    } else if called_class == types.uint {
                                        Some((Op::CoerceUSwapPop, types.uint))
                                    } else if called_class == types.number {
                                        Some((Op::CoerceDSwapPop, types.number))
                                    } else {
                                        None
                                    };

                                    if let Some((new_op, return_type)) = result {
                                        return Ok(Some((new_op, Some(return_type))));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

fn recalculate_jump_targets<'gc>(
    ops: &[Cell<Op<'gc>>],
    exceptions: &[Exception<'gc>],
    jump_targets: &mut HashSet<usize>,
) {
    *jump_targets = HashSet::with_capacity(jump_targets.len());

    for exception in exceptions {
        jump_targets.insert(exception.target_offset);
    }

    for op in ops {
        match op.get() {
            Op::IfFalse { offset }
            | Op::IfTrue { offset }
            | Op::Jump { offset }
            | Op::PopJump { offset } => {
                jump_targets.insert(offset);
            }
            Op::LookupSwitch(lookup_switch) => {
                for target in lookup_switch
                    .case_offsets
                    .iter()
                    .chain(std::slice::from_ref(&lookup_switch.default_offset))
                {
                    jump_targets.insert(target.get());
                }
            }
            _ => {}
        }
    }
}
