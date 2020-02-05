//! Default AVM2 object impl

use crate::avm2::names::QName;
use crate::avm2::object::TObject;
use crate::avm2::value::Value;
use gc_arena::{Collect, GcCell};
use std::collections::HashMap;
use std::fmt::Debug;

/// Default implementation of `avm2::Object`.
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ScriptObjectData<'gc> {
    /// Properties stored on this object.
    values: HashMap<QName, Value<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {}
