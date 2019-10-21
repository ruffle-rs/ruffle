//! Return value enum

use crate::avm1::activation::Activation;
use crate::avm1::stack_continuation::StackContinuation;
use crate::avm1::{Avm1, Error, Value};
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell};
use std::fmt;

/// Represents a value which can be returned immediately or at a later time.
#[must_use = "Return values must be used"]
#[derive(Clone)]
pub enum ReturnValue<'gc> {
    /// Indicates that the return value is available immediately.
    Immediate(Value<'gc>),

    /// Indicates that the return value will be calculated on the stack.
    ResultOf(GcCell<'gc, Activation<'gc>>),

    /// Indicates that there is no value to return.
    ///
    /// This is primarily intended to signal to the AVM that a given stack
    /// frame should not cause a value to be pushed to the stack when it
    /// returns.
    NoResult,
}

unsafe impl<'gc> Collect for ReturnValue<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        use ReturnValue::*;

        match self {
            Immediate(value) => value.trace(cc),
            ResultOf(frame) => frame.trace(cc),
            NoResult => {}
        }
    }
}

impl PartialEq for ReturnValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        use ReturnValue::*;

        match (self, other) {
            (Immediate(val1), Immediate(val2)) => val1 == val2,
            (ResultOf(frame1), ResultOf(frame2)) => GcCell::ptr_eq(*frame1, *frame2),
            (NoResult, NoResult) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for ReturnValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReturnValue::*;

        match self {
            Immediate(val) => write!(f, "Immediate({:?})", val),
            ResultOf(_frame) => write!(f, "ResultOf(<activation frame>)"),
            NoResult => write!(f, "NoResult"),
        }
    }
}

impl<'gc> ReturnValue<'gc> {
    /// Run the return value through a stack continuation.
    ///
    /// If the return value is instant, we call the continuation immediately;
    /// else we schedule it on the AVM stack. We return a new return value
    /// representing the most up-to-date state of the computation in question.
    /// This means it's possible to chain `and_then` functions across multiple
    /// AVM stack frames.
    pub fn and_then<F>(
        self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        mut cont: F,
    ) -> Result<ReturnValue<'gc>, Error>
    where
        F: StackContinuation<'gc>,
    {
        use ReturnValue::*;

        match self {
            Immediate(val) => cont.returned(avm, context, val),
            ResultOf(frame) => {
                frame.write(context.gc_context).and_then(Box::new(cont));

                //WARNING: This isn't exactly chainable, only one continuation
                //can run at once.
                Ok(ResultOf(frame))
            }
            NoResult => Ok(NoResult),
        }
    }

    /// Mark a given return value as intended to be pushed onto the stack.
    ///
    /// The natural result of a stack frame retiring is to be pushed, so this
    /// only ensures that Immediate values are pushed.
    pub fn push(self, avm: &mut Avm1<'gc>) {
        use ReturnValue::*;

        match self {
            Immediate(val) => avm.push(val),
            ResultOf(_frame) => {}
            NoResult => {}
        };
    }

    /// Consumes the given return value.
    ///
    /// This exists primarily so that users of return values can indicate that
    /// they do not plan to use them.
    pub fn ignore(self) {}

    pub fn is_immediate(&self) -> bool {
        use ReturnValue::*;

        if let Immediate(_v) = self {
            true
        } else {
            false
        }
    }

    /// Panic if a value is not immediate.
    ///
    /// This should only be used in test assertions.
    pub fn unwrap_immediate(self) -> Value<'gc> {
        use ReturnValue::*;

        match self {
            Immediate(val) => val,
            _ => panic!("Unwrapped a non-immediate return value"),
        }
    }
}
