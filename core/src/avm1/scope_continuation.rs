//! GC-compatible scope continuations

use crate::avm1::{Avm1, Error, Object, Value};
use crate::context::UpdateContext;
use gc_arena::GcCell;

pub type ScopeContinuation<'gc> = fn(
    &mut Avm1<'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    GcCell<'gc, Object<'gc>>,
    Value<'gc>,
) -> Result<Value<'gc>, Error>;
