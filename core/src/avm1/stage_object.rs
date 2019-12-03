//! AVM1 object type to represent objects on the stage.

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TDisplayObject, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;
use enumset::EnumSet;
use gc_arena::{Collect, MutationContext};
use std::collections::HashSet;
use std::fmt;

/// The type string for MovieClip objects.
pub const TYPE_OF_MOVIE_CLIP: &str = "movieclip";

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
    fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        // Property search order for DisplayObjects:
        if self.has_own_property(name) {
            // 1) Actual properties on the underlying object
            self.get_local(name, avm, context, (*self).into())
        } else if let Some(child) = self.display_object.get_child_by_name(name) {
            // 2) Child display objects with the given instance name
            Ok(child.object().into())
        } else {
            // 3) Prototype
            crate::avm1::object::search_prototype(self.proto(), name, avm, context, (*self).into())
        }
        // 4) TODO: __resolve?
    }

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
    ) -> Result<(), Error> {
        self.base.set(name, value, avm, context)
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
        if self.base.has_property(name) {
            return true;
        }

        if self.display_object.get_child_by_name(name).is_some() {
            return true;
        }

        false
    }

    fn has_own_property(&self, name: &str) -> bool {
        // Note that `hasOwnProperty` does NOT return true for child display objects.
        self.base.has_own_property(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.base.is_property_enumerable(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.base.is_property_overwritable(name)
    }

    fn get_keys(&self) -> HashSet<String> {
        // Keys from the underlying object are listed first, followed by
        // child display objects in order from highest depth to lowest depth.
        // TODO: It's possible to have multiple instances with the same name,
        // and that name will be returned multiple times in the key list for a `for..in` loop.
        let mut keys = self.base.get_keys();
        for child in self.display_object.children() {
            keys.insert(child.name().to_string());
        }
        keys
    }

    fn as_string(&self) -> String {
        self.display_object.path()
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
