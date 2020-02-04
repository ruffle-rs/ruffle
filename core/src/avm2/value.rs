//! AVM2 values

use gc_arena::{Collect, Gc};

/// An AVM2 value.
///
/// TODO: AVM2 also needs Object, Scope, Namespace, and XML values.
#[derive(Collect)]
#[collect(no_drop)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(Gc<'gc, Value<'gc>>),
}
