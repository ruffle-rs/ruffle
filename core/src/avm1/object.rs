//! Object trait to expose objects to AVM

use crate::avm1::function::Executable;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ScriptObject, UpdateContext, Value};
use crate::display_object::DisplayNode;
use gc_arena::{Collect, GcCell};
use std::fmt::Debug;

pub type ObjectCell<'gc> = GcCell<'gc, Box<dyn Object<'gc> + 'gc>>;

/// Represents an object that can be directly interacted with by the AVM
/// runtime.
pub trait Object<'gc>: 'gc + Collect + Debug {
    /// Retrieve a named property from the object.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Set a named property on the object.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    fn set(
        &mut self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
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
        this: ObjectCell<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Construct a host object of some kind and return it's cell.
    ///
    /// This is called on constructor functions to obtain a given host object.
    /// The returned object will then be passed to `call` in order to run
    /// initialization code. This function cannot be user-defined (and thus,
    /// does not yield a ReturnValue).
    ///
    /// The arguments passed to the constructor are provided here; however, all
    /// object construction should happen in `call`, not `new`. `new` exists
    /// purely so that host objects can be constructed by the VM.
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ObjectCell<'gc>, Error>;

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&mut self, name: &str) -> bool;

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
    fn get_keys(&self) -> Vec<String>;

    /// Coerce the object into a string.
    fn as_string(&self) -> String;

    /// Get the object's type string.
    fn type_of(&self) -> &'static str;

    /// Get the underlying script object, if it exists.
    fn as_script_object(&self) -> Option<&ScriptObject<'gc>>;

    /// Get the underlying script object, if it exists.
    fn as_script_object_mut(&mut self) -> Option<&mut ScriptObject<'gc>>;

    /// Get the underlying display node for this object, if it exists.
    fn as_display_node(&self) -> Option<DisplayNode<'gc>>;

    /// Get the underlying executable for this object, if it exists.
    fn as_executable(&self) -> Option<Executable<'gc>>;
}
