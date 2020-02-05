//! AVM2 values

use crate::avm2::object::Object;
use gc_arena::Collect;

/// An AVM2 value.
///
/// TODO: AVM2 also needs Scope, Namespace, and XML values.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(Object<'gc>),
}
