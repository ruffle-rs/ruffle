//! Return value enum

use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::{Error, Value};
use std::fmt;

/// Represents the return value of a function call that has not yet executed.
///
/// It is a panicking logic error to attempt to run AVM2 code while any
/// reachable object is in a locked state. Ergo, it is sometimes necessary to
/// be able to return what *should* be called rather than actually running the
/// code on the current Rust stack. This type exists to force deferred
/// execution of some child AVM2 frame.
///
/// It is also possible to stuff a regular `Value` in here - this is provided
/// for the convenience of functions that may be able to resolve a value
/// without needing a free stack. `ReturnValue` should not be used as a generic
/// wrapper for `Value`, as it can also defer actual execution, and it should
/// be resolved at the earliest safe opportunity.
///
/// It is `must_use` - failing to resolve a return value is a compiler warning.
#[must_use = "Return values must be used"]
pub enum ReturnValue<'gc> {
    /// ReturnValue has already been computed.
    ///
    /// This exists primarily for functions that don't necessarily need to
    /// always defer code execution - say, if they already have the result and
    /// do not need a free stack frame to run an activation on.
    Immediate(Value<'gc>),

    /// ReturnValue has not yet been computed.
    ///
    /// This exists for functions that do need to reference the result of user
    /// code in order to produce their result.
    ///
    /// The properties of this enum struct will be passed onto the callee's
    /// executable at resolution time.
    ResultOf {
        callee: Object<'gc>,
        unbound_reciever: Option<Object<'gc>>,
        arguments: Vec<Value<'gc>>,
        subclass_object: Option<Object<'gc>>,
    },
}

impl fmt::Debug for ReturnValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Immediate(v) => f.debug_tuple("ReturnValue::Immediate").field(v).finish(),
            Self::ResultOf {
                callee,
                unbound_reciever,
                arguments,
                subclass_object,
            } => f
                .debug_struct("ReturnValue")
                .field("callee", callee)
                .field("unbound_reciever", unbound_reciever)
                .field("arguments", arguments)
                .field("subclass_object", subclass_object)
                .finish(),
        }
    }
}

impl<'gc> ReturnValue<'gc> {
    /// Construct a new return value.
    pub fn defer_execution(
        callee: Object<'gc>,
        unbound_reciever: Option<Object<'gc>>,
        arguments: Vec<Value<'gc>>,
        subclass_object: Option<Object<'gc>>,
    ) -> Self {
        Self::ResultOf {
            callee,
            unbound_reciever,
            arguments,
            subclass_object,
        }
    }

    /// Resolve the underlying deferred execution.
    ///
    /// All return values must eventually resolved - it is a compile error to
    /// fail to do so.
    pub fn resolve(self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Value<'gc>, Error> {
        match self {
            Self::Immediate(v) => Ok(v),
            Self::ResultOf {
                callee,
                unbound_reciever,
                arguments,
                subclass_object,
            } => callee.as_executable().unwrap().exec(
                unbound_reciever,
                &arguments,
                activation,
                subclass_object,
                callee,
                false,
            ),
        }
    }
}

impl<'gc> From<Value<'gc>> for ReturnValue<'gc> {
    fn from(v: Value<'gc>) -> Self {
        Self::Immediate(v)
    }
}
