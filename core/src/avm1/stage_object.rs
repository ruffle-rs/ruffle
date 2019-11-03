//! AVM1 object type to represent objects on the stage.

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::script_object::TYPE_OF_MOVIE_CLIP;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use enumset::EnumSet;
use gc_arena::{Collect, MutationContext};
use std::collections::HashSet;
use std::fmt;

/// A ScriptObject that is inherently tied to a display node.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct StageObject<'gc> {
    /// The underlying script object.
    ///
    /// This is used to handle "expando properties" on AVM1 display nodes, as
    /// well as the underlying prototype chain.
    base: ScriptObject<'gc>,

    /// The display node this stage object
    display_object: DisplayObject<'gc>,
}

impl<'gc> StageObject<'gc> {
    /// Create a stage object for a given display node.
    pub fn for_display_object(
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
        proto: Option<Object<'gc>>,
    ) -> Self {
        let mut base = ScriptObject::object(gc_context, proto);

        //TODO: Do other display node objects have different typestrings?
        base.set_type_of(gc_context, TYPE_OF_MOVIE_CLIP);

        Self {
            base,
            display_object,
        }
    }
}

impl fmt::Debug for StageObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StageObject")
            .field("base", &self.base)
            .field("display_object", &self.display_object)
            .finish()
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base.get_local(name, avm, context, this)
    }
    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error> {
        self.base.set(name, value, avm, context, this)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.base.call(avm, context, this, args)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        //TODO: Create a StageObject of some kind
        self.base.new(avm, context, this, args)
    }
    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        self.base.delete(gc_context, name)
    }
    fn proto(&self) -> Option<Object<'gc>> {
        self.base.proto()
    }
    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base.define_value(gc_context, name, value, attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.base
            .add_property(gc_context, name, get, set, attributes)
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
    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(self.base)
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        Some(self.display_object)
    }
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.base.as_ptr() as *const ObjectPtr
    }
}
