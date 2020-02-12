//! AVM2 objects.

use crate::avm2::function::FunctionObject;
use crate::avm2::names::{Multiname, QName};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use enumset::{EnumSet, EnumSetType};
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::fmt::Debug;

/// Attributes of properties in the AVM runtime.
///
/// TODO: Replace with AVM2 properties for traits
#[derive(EnumSetType, Debug)]
pub enum Attribute {
    DontEnum,
    DontDelete,
    ReadOnly,
}

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
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Set a property by it's QName.
    fn set_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Resolve a multiname into a single QName, if any of the namespaces
    /// match.
    fn resolve_multiname(self, multiname: &Multiname) -> Option<QName> {
        for ns in multiname.namespace_set() {
            let qname = QName::qualified(ns, multiname.local_name());

            if self.has_property(&qname) {
                return Some(qname);
            }
        }

        None
    }

    /// Indicates whether or not a property exists on an object.
    fn has_property(self, _name: &QName) -> bool;

    /// Indicates whether or not a property exists on an object and is not part
    /// of the prototype chain.
    fn has_own_property(self, _name: &QName) -> bool {
        false
    }

    /// Indicates whether or not a property is overwritable.
    fn is_property_overwritable(self, _name: &QName) -> bool {
        false
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, gc_context: MutationContext<'gc, '_>, multiname: &QName) -> bool {
        false
    }

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>> {
        None
    }

    /// Define a value on an object.
    ///
    /// Unlike setting a value, this function is intended to replace any
    /// existing virtual or built-in properties already installed on a given
    /// object. As such, this should not run any setters; the resulting name
    /// slot should either be completely replaced with the value or completely
    /// untouched.
    ///
    /// It is not guaranteed that all objects accept value definitions,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &QName,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
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
