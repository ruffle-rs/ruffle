//! Object trait to expose objects to AVM

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ScriptObject, UpdateContext, Value};
use crate::display_object::DisplayObject;
use enumset::EnumSet;
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::collections::HashSet;
use std::fmt::Debug;

/// Represents an object that can be directly interacted with by the AVM
/// runtime.
#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug {
    /// Retrieve a named property from this object exclusively.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    ///
    /// This function should not inspect prototype chains. Instead, use `get`
    /// to do ordinary property look-up and resolution.
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Retrieve a named property from the object, or it's prototype.
    fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        if self.has_own_property(name) {
            self.get_local(name, avm, context, this)
        } else {
            let mut depth = 0;
            let mut proto = self.proto();

            while proto.is_some() {
                if depth == 255 {
                    return Err("Encountered an excessively deep prototype chain.".into());
                }

                if proto.unwrap().has_own_property(name) {
                    return proto.unwrap().get_local(name, avm, context, this);
                }

                proto = proto.unwrap().proto();
                depth += 1;
            }

            Ok(Value::Undefined.into())
        }
    }

    /// Set a named property on this object, or it's prototype.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error>;

    /// Call the underlying object.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Construct a host object of some kind and return it's cell.
    ///
    /// As the first step in object construction, the `new` method is called on
    /// the prototype to initialize an object. The prototype may construct any
    /// object implementation it wants, with itself as the new object's proto.
    /// Then, the constructor is `call`ed with the new object as `this` to
    /// initialize the object.
    ///
    /// The arguments passed to the constructor are provided here; however, all
    /// object construction should happen in `call`, not `new`. `new` exists
    /// purely so that host objects can be constructed by the VM.
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error>;

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool;

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>>;

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
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    );

    /// Define a virtual property onto a given object.
    ///
    /// A virtual property is a set of get/set functions that are called when a
    /// given named property is retrieved or stored on an object. These
    /// functions are then responsible for providing or accepting the value
    /// that is given to or taken from the AVM.
    ///
    /// It is not guaranteed that all objects accept virtual properties,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    );

    /// Checks if the object has a given named property.
    fn has_property(&self, name: &str) -> bool;

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    fn has_own_property(&self, name: &str) -> bool;

    /// Checks if a named property can be overwritten.
    fn is_property_overwritable(&self, name: &str) -> bool;

    /// Checks if a named property appears when enumerating the object.
    fn is_property_enumerable(&self, name: &str) -> bool;

    /// Enumerate the object.
    fn get_keys(&self) -> HashSet<String>;

    /// Coerce the object into a string.
    fn as_string(&self) -> String;

    /// Get the object's type string.
    fn type_of(&self) -> &'static str;

    /// Get the underlying script object, if it exists.
    fn as_script_object(&self) -> Option<&ScriptObject<'gc>>;

    /// Get the underlying script object, if it exists.
    fn as_script_object_mut(&mut self) -> Option<&mut ScriptObject<'gc>>;

    /// Get the underlying display node for this object, if it exists.
    fn as_display_node(&self) -> Option<DisplayObject<'gc>>;

    /// Get the underlying executable for this object, if it exists.
    fn as_executable(&self) -> Option<Executable<'gc>>;

    fn as_ptr(&self) -> *const ObjectPtr;
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}
