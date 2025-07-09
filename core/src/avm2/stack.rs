//! Internal representation of the AVM2 value stack

use crate::avm2::function::FunctionArgs;
use crate::avm2::method::Method;
use crate::avm2::value::Value;

use gc_arena::collect::Trace;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::Cell;

const PREALLOCATED_STACK_SIZE: usize = 200000;

/// The global, preallocated value stack. This has little use directly due to
/// the way AVM2 works; to do anything with it, an StackFrame must be
/// obtained first through the `get_stack_slice` method.
///
/// We use this instead of a `Vec<Value>` to allow for obtaining "mutable" (via
/// interior mutability) references to the stack without requiring a mutable
/// borrow on the stack or the `Avm2`.
#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct Stack<'gc>(Gc<'gc, StackData<'gc>>);

struct StackData<'gc> {
    /// Stack data
    stack: Box<[Cell<Value<'gc>>]>,

    /// The number of values currently stored on the stack
    stack_pointer: Cell<usize>,
}

impl<'gc> Stack<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        let stack = (0..PREALLOCATED_STACK_SIZE)
            .map(|_| Cell::new(Value::Undefined))
            .collect::<Box<[_]>>();

        Stack(Gc::new(
            mc,
            StackData {
                stack,
                stack_pointer: Cell::new(0),
            },
        ))
    }

    /// Returns a slice of stack data for the specified method, starting at the
    /// current stack pointer. Stack frames obtained from this method must be
    /// properly disposed of by using the `dispose_stack_frame` method.
    pub fn get_stack_frame(&self, method: Method<'gc>) -> StackFrame<'_, 'gc> {
        // First calculate the frame size
        let body = method
            .body()
            .expect("Cannot execute non-native method without body");
        let frame_size = body.max_stack as usize + body.num_locals as usize;

        // Then actually create the stack frame
        let stack_data = &self.0.stack;
        let stack_pointer = &self.0.stack_pointer;

        let subslice = &stack_data[stack_pointer.get()..stack_pointer.get() + frame_size];

        stack_pointer.set(stack_pointer.get() + frame_size);

        // Ensure the StackFrame returned to the caller does not have any
        // old values on it, as these may contain Gc pointers that have already
        // been collected.
        for value in subslice {
            value.set(Value::Undefined);
        }

        StackFrame::for_data(subslice)
    }

    pub fn dispose_stack_frame(&self, stack_frame: StackFrame<'_, 'gc>) {
        self.0
            .stack_pointer
            .set(self.0.stack_pointer.get() - stack_frame.data.len());
    }
}

unsafe impl<'gc> Collect<'gc> for StackData<'gc> {
    // SAFETY: The only way to access values on the stack is by using StackFrames
    // obtained from get_stack_frame. StackFrame doesn't implement Collect, so
    // StackFrames from before a collection can't be accessed after the collection.
    fn trace<C: Trace<'gc>>(&self, _cc: &mut C) {
        // There should be no values on the value stack when collection is triggered
        assert!(self.stack_pointer.get() == 0);
    }
}

/// A stack frame for a particular method. Despite its name, this stores both
/// method locals and method stack.
pub struct StackFrame<'a, 'gc> {
    data: &'a [Cell<Value<'gc>>],
    stack_pointer: Cell<usize>,
}

impl<'a, 'gc> StackFrame<'a, 'gc> {
    pub fn empty() -> StackFrame<'a, 'gc> {
        Self {
            data: &[],
            stack_pointer: Cell::new(0),
        }
    }

    fn for_data(data: &'a [Cell<Value<'gc>>]) -> StackFrame<'a, 'gc> {
        Self {
            data,
            stack_pointer: Cell::new(0),
        }
    }

    /// Push a value onto the operand stack.
    #[inline(always)]
    pub fn push(&self, value: Value<'gc>) {
        self.data[self.stack_pointer.get()].set(value);
        self.stack_pointer.set(self.stack_pointer.get() + 1);
    }

    /// Retrieve the top-most value on the operand stack.
    #[inline(always)]
    pub fn pop(&self) -> Value<'gc> {
        self.stack_pointer.set(self.stack_pointer.get() - 1);

        self.data[self.stack_pointer.get()].get()
    }

    /// Peek the n-th value from the end of the operand stack.
    #[inline(always)]
    pub fn peek(&self, index: usize) -> Value<'gc> {
        self.data[self.stack_pointer.get() - index - 1].get()
    }

    #[inline(always)]
    pub fn stack_top(&self) -> &'a Cell<Value<'gc>> {
        &self.data[self.stack_pointer.get() - 1]
    }

    #[inline(always)]
    pub fn value_at(&self, index: usize) -> Value<'gc> {
        self.data[index].get()
    }

    #[inline(always)]
    pub fn set_value_at(&self, index: usize, value: Value<'gc>) {
        self.data[index].set(value);
    }

    #[inline(always)]
    pub fn get_args(&self, num_args: usize) -> FunctionArgs<'a, 'gc> {
        let base = self.stack_pointer.get() - num_args;

        self.stack_pointer.set(base);

        FunctionArgs::AsCellArgSlice {
            arguments: &self.data[base..base + num_args],
        }
    }

    pub fn pop_args(&self, arg_count: u32) -> Vec<Value<'gc>> {
        let mut args = vec![Value::Undefined; arg_count as usize];
        for arg in args.iter_mut().rev() {
            *arg = self.pop();
        }
        args
    }

    pub fn set_stack_pointer(&self, size: usize) {
        self.stack_pointer.set(size);
    }

    /// Get the number of entries currently on the stack.
    pub fn len(&self) -> usize {
        self.stack_pointer.get()
    }

    /// Move out of this StackFrame, leaving it empty and creating a new StackFrame
    /// that points to the same data as this one.
    pub fn take(&mut self) -> Self {
        let new_frame = Self {
            data: self.data,
            stack_pointer: self.stack_pointer.clone(),
        };

        self.data = &[];
        self.stack_pointer = Cell::new(0);

        new_frame
    }
}
