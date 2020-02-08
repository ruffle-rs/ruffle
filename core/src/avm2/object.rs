//! AVM2 objects.

use crate::avm2::function::FunctionObject;
use crate::avm2::names::QName;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::Collect;
use ruffle_macros::enum_trait_object;
use std::fmt::Debug;

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        FunctionObject(FunctionObject<'gc>)
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Retrieve a property by it's QName.
    fn get_property(
        self,
        _name: &QName,
        _avm: &mut Avm2<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        Ok(Value::Undefined.into())
    }

    /// Set a property by it's QName.
    fn set_property(
        &mut self,
        _name: &QName,
        _value: Value<'gc>,
        _avm: &mut Avm2<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Call the object.
    fn call(
        self,
        _reciever: Object<'gc>,
        _arguments: &[Value<'gc>],
        _avm: &mut Avm2<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        Err("Object is not callable".into())
    }

    /// Get a raw pointer value for this object.
    fn as_ptr(&self) -> *const ObjectPtr;
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}
