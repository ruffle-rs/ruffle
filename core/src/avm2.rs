//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::value::Value;
use gc_arena::Collect;

mod value;

/// The state of an AVM2 interpreter.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm2<'gc> {
    /// Values currently present on the operand stack.
    stack: Vec<Value<'gc>>,
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }
}
