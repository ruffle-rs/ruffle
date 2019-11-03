//! AVM1 object type to represent objects on the stage.

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectCell, ScriptObject, Value};
use crate::context::UpdateContext;
use crate::display_object::DisplayNode;
use enumset::EnumSet;
use gc_arena;
use std::collections::HashSet;
use std::fmt;

/// A ScriptObject that is inherently tied to a display node.
#[derive(Clone)]
pub struct StageObject<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The display node this stage object
    display_node: DisplayNode<'gc>,
}

unsafe impl<'gc> gc_arena::Collect for StageObject<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.base.trace(cc);
        self.display_node.trace(cc);
    }
}

impl fmt::Debug for StageObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StageObject")
            .field("base", &self.base)
            .field("display_node", &self.display_node)
            .finish()
    }
}

impl<'gc> Object<'gc> for StageObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base.get_local(name, avm, context, this)
    }
    fn set(
        &mut self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
    ) -> Result<(), Error> {
        self.base.set(name, value, avm, context, this)
    }
    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base.call(avm, context, this, args)
    }
    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ObjectCell<'gc>, Error> {
        //TODO: Create a StageObject of some kind
        self.base.new(avm, context, this, args)
    }
    fn delete(&mut self, name: &str) -> bool {
        self.base.delete(name)
    }
    fn proto(&self) -> Option<ObjectCell<'gc>> {
        self.base.proto()
    }
    fn define_value(&mut self, name: &str, value: Value<'gc>, attributes: EnumSet<Attribute>) {
        self.base.define_value(name, value, attributes)
    }
    fn add_property(
        &mut self,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base.add_property(name, get, set, attributes)
    }

    fn has_property(&self, name: &str) -> bool {
        self.base.has_property(name)
    }
    fn has_own_property(&self, name: &str) -> bool {
        self.base.has_own_property(name)
    }
    fn is_property_enumerable(&self, name: &str) -> bool {
        self.base.is_property_enumerable(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.base.is_property_overwritable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        self.base.get_keys()
    }
    fn as_string(&self) -> String {
        self.base.as_string()
    }
    fn type_of(&self) -> &'static str {
        self.base.type_of()
    }
    fn as_script_object(&self) -> Option<&ScriptObject<'gc>> {
        Some(&self.base)
    }

    fn as_script_object_mut(&mut self) -> Option<&mut ScriptObject<'gc>> {
        Some(&mut self.base)
    }
    fn as_display_node(&self) -> Option<DisplayNode<'gc>> {
        Some(self.display_node)
    }
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }
}
