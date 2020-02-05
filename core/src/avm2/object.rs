//! AVM2 objects.

use crate::avm2::names::QName;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use gc_arena::Collect;
use ruffle_macros::enum_trait_object;
use std::fmt::Debug;

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>)
    }
)]

pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Retrieve a property by it's QName.
    fn get_property(self, _name: &QName) -> Value<'gc> {
        Value::Undefined
    }

    /// Set a property by it's QName.
    fn set_property(self, _name: &QName, _value: Value<'gc>) {}
}
