//! Internal representation of the AVM2 value stack

use crate::avm2::function::FunctionArgs;
use crate::avm2::method::Method;
use crate::avm2::value::Value;

use gc_arena::barrier::Write;
use gc_arena::collect::Trace;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::Cell;

const PREALLOCATED_STACK_SIZE: usize = 200000;

// This struct is needed for two reasons: first, [T] does not implement Collect,
// so we need to provide a manual implementation; and second, we need to
// construct a custom DST of type Box<Lock<[Value]>>, which is impossible without
// this intermediate struct.
#[repr(transparent)]
struct RawStack<T: ?Sized> {
    data: Lock<T>,
}

unsafe impl<'gc, T: ?Sized> Collect<'gc> for RawStack<T> {
    // SAFETY: AvmStack only calls Collect on RawStack when stack_pointer is 0,
    // which means that the RawStack won't have any reachable values stored
    // in it
    fn trace<C: Trace<'gc>>(&self, _cc: &mut C) {}
}

/// The global, preallocated value stack. This has little use directly due to
/// the way AVM2 works; to do anything with it, an AvmStackFrame must be
/// obtained first through the `get_stack_slice` method.
///
/// We use this instead of a `Vec<Value>` to allow for obtaining "mutable" (via
/// interior mutability) references to the stack without requiring a mutable
/// borrow on the stack or the `Avm2`.
#[derive(Clone, Copy)]
pub struct AvmStack<'gc> {
    /// Stack data
    stack: Gc<'gc, Box<RawStack<[Value<'gc>]>>>,

    /// The number of values currently stored on the stack
    stack_pointer: Gc<'gc, Cell<usize>>,
}

impl<'gc> AvmStack<'gc> {
    pub fn new(mc: &Mutation<'gc>) -> Self {
        let values_vec = vec![Value::Undefined; PREALLOCATED_STACK_SIZE];
        let values = values_vec.into_boxed_slice();

        // We can't use `cast` for this operation because it only works on sized pointers
        let value_ptr = Box::into_raw(values) as *mut RawStack<[Value]>;

        // SAFETY: RawStack<T> is guaranteed to have the same representation
        // as T because RawStack, Lock, and Cell are all `#[repr(transparent)]`
        let boxed_raw_stack = unsafe { Box::from_raw(value_ptr) };

        AvmStack {
            stack: Gc::new(mc, boxed_raw_stack),
            stack_pointer: Gc::new(mc, Cell::new(0)),
        }
    }

    /// Returns a slice of stack data for the specified method, starting at the
    /// current stack pointer.
    pub fn get_stack_frame(
        &self,
        mc: &Mutation<'gc>,
        method: Method<'gc>,
    ) -> AvmStackFrame<'_, 'gc> {
        // First calculate the frame size
        let body = method
            .body()
            .expect("Cannot execute non-native method without body");
        let frame_size = body.max_stack as usize + body.num_locals as usize;

        // Then actually create the stack frame
        let stack_pointer = self.stack_pointer;

        Gc::write(mc, self.stack);
        // SAFETY: We just triggered a write barrier on the Gc.
        let inner = unsafe { Write::assume(&self.stack.data) };

        let entire_slice = inner.unlock().as_slice_of_cells();
        let subslice = &entire_slice[stack_pointer.get()..stack_pointer.get() + frame_size];

        stack_pointer.set(stack_pointer.get() + frame_size);

        // Ensure the AvmStackFrame returned to the caller does not have any
        // old values on it, as these may contain Gc pointers that have already
        // been collected.
        for value in subslice {
            value.set(Value::Undefined);
        }

        AvmStackFrame::for_data(subslice)
    }

    pub fn dispose_stack_frame(&self, stack_frame: &AvmStackFrame<'_, 'gc>) {
        self.stack_pointer
            .set(self.stack_pointer.get() - stack_frame.data.len());
    }
}

unsafe impl<'gc> Collect<'gc> for AvmStack<'gc> {
    // SAFETY: All values stored on `stack` are unreachable when stack_pointer is 0
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        // There should be no values on the value stack when collection is triggered
        assert!(self.stack_pointer.get() == 0);

        self.stack.trace(cc);
        self.stack_pointer.trace(cc);
    }
}

/// A stack frame for a particular method. Despite its name, this stores both
/// method locals and method stack.
///
/// NOTE: Clones of this struct will share the same slice of values on the AVM stack
#[derive(Clone)]
pub struct AvmStackFrame<'a, 'gc> {
    data: &'a [Cell<Value<'gc>>],
    stack_pointer: Cell<usize>,
}

impl<'a, 'gc> AvmStackFrame<'a, 'gc> {
    pub fn for_data(data: &'a [Cell<Value<'gc>>]) -> AvmStackFrame<'a, 'gc> {
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
        let value = self.data[self.stack_pointer.get()].get();

        value
    }

    /// Peek the n-th value from the end of the operand stack.
    #[inline(always)]
    pub fn peek(&self, index: usize) -> Value<'gc> {
        self.data[self.stack_pointer.get() - index - 1].get()
    }

    #[inline(always)]
    pub fn stack_top(&self) -> &Cell<Value<'gc>> {
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

    pub fn truncate(&self, size: usize) {
        self.stack_pointer.set(size);
    }

    /// Get the number of entries currently on the stack.
    pub fn len(&self) -> usize {
        self.stack_pointer.get()
    }
}

unsafe impl<'a, 'gc> Collect<'gc> for AvmStackFrame<'a, 'gc> {
    // SAFETY: There are no values on the frame
    fn trace<C: Trace<'gc>>(&self, _cc: &mut C) {
        // The only stack frame that is ever traced is the dummy stack frame.
        // The dummy stack frame has no values on it, so this is sound.
        assert!(self.data.is_empty());
    }
}
