use crate::avm1::function::{Executable, NativeFunction};
use crate::avm1::property::{Attribute, Property};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, TObject, UpdateContext, Value};
use crate::display_object::DisplayObject;
use core::fmt;
use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

pub const TYPE_OF_OBJECT: &str = "object";
pub const TYPE_OF_FUNCTION: &str = "function";
pub const TYPE_OF_MOVIE_CLIP: &str = "movieclip";

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

pub struct ScriptObjectData<'gc> {
    prototype: Option<Object<'gc>>,
    display_object: Option<DisplayObject<'gc>>,
    values: HashMap<String, Property<'gc>>,
    function: Option<Executable<'gc>>,
    type_of: &'static str,
}

unsafe impl<'gc> Collect for ScriptObjectData<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.prototype.trace(cc);
        self.display_object.trace(cc);
        self.values.trace(cc);
        self.function.trace(cc);
    }
}

impl fmt::Debug for ScriptObjectData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Object")
            .field("prototype", &self.prototype)
            .field("display_object", &self.display_object)
            .field("values", &self.values)
            .field("function", &self.function.is_some())
            .finish()
    }
}

impl<'gc> ScriptObject<'gc> {
    pub fn object(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> ScriptObject<'gc> {
        ScriptObject(GcCell::allocate(
            gc_context,
            ScriptObjectData {
                prototype: proto,
                type_of: TYPE_OF_OBJECT,
                display_object: None,
                values: HashMap::new(),
                function: None,
            },
        ))
    }

    /// Constructs and allocates an empty but normal object in one go.
    pub fn object_cell(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            gc_context,
            ScriptObjectData {
                prototype: proto,
                type_of: TYPE_OF_OBJECT,
                display_object: None,
                values: HashMap::new(),
                function: None,
            },
        ))
        .into()
    }

    /// Constructs an object with no values, not even builtins.
    ///
    /// Intended for constructing scope chains, since they exclusively use the
    /// object values, but can't just have a hashmap because of `with` and
    /// friends.
    pub fn bare_object(gc_context: MutationContext<'gc, '_>) -> Self {
        ScriptObject(GcCell::allocate(
            gc_context,
            ScriptObjectData {
                prototype: None,
                type_of: TYPE_OF_OBJECT,
                display_object: None,
                values: HashMap::new(),
                function: None,
            },
        ))
    }

    /// Construct a function sans prototype.
    pub fn bare_function(
        gc_context: MutationContext<'gc, '_>,
        function: impl Into<Executable<'gc>>,
        fn_proto: Option<Object<'gc>>,
    ) -> Self {
        ScriptObject(GcCell::allocate(
            gc_context,
            ScriptObjectData {
                prototype: fn_proto,
                type_of: TYPE_OF_FUNCTION,
                function: Some(function.into()),
                display_object: None,
                values: HashMap::new(),
            },
        ))
    }

    /// Construct a function from an executable and associated protos.
    ///
    /// Since prototypes need to link back to themselves, this function builds
    /// both objects itself and returns the function to you, fully allocated.
    ///
    /// `fn_proto` refers to the implicit proto of the function object, and the
    /// `prototype` refers to the explicit prototype of the function. If
    /// provided, the function and it's prototype will be linked to each other.
    pub fn function(
        gc_context: MutationContext<'gc, '_>,
        function: impl Into<Executable<'gc>>,
        fn_proto: Option<Object<'gc>>,
        prototype: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let function = Self::bare_function(gc_context, function, fn_proto).into();

        //TODO: Can we make these proper sets or no?
        if let Some(p) = prototype {
            p.define_value(
                gc_context,
                "constructor",
                Value::Object(function),
                Attribute::DontEnum.into(),
            );
            function.define_value(gc_context, "prototype", p.into(), EnumSet::empty());
        }

        function
    }

    pub fn set_display_object(
        self,
        gc_context: MutationContext<'gc, '_>,
        display_object: DisplayObject<'gc>,
    ) {
        self.0.write(gc_context).display_object = Some(display_object);
    }

    #[allow(dead_code)]
    pub fn display_object(self) -> Option<DisplayObject<'gc>> {
        self.0.read().display_object
    }

    /// Declare a native function on the current object.
    ///
    /// This is intended for use with defining host object prototypes. Notably,
    /// this creates a function object without an explicit `prototype`, which
    /// is only possible when defining host functions. User-defined functions
    /// always get a fresh explicit prototype, so you should never force set a
    /// user-defined function.
    pub fn force_set_function<A>(
        &mut self,
        name: &str,
        function: NativeFunction<'gc>,
        gc_context: MutationContext<'gc, '_>,
        attributes: A,
        fn_proto: Option<Object<'gc>>,
    ) where
        A: Into<EnumSet<Attribute>>,
    {
        self.define_value(
            gc_context,
            name,
            Value::Object(ScriptObject::function(gc_context, function, fn_proto, None)),
            attributes.into(),
        )
    }

    pub fn set_prototype(&mut self, gc_context: MutationContext<'gc, '_>, prototype: Object<'gc>) {
        self.0.write(gc_context).prototype = Some(prototype);
    }

    pub fn set_type_of(&mut self, gc_context: MutationContext<'gc, '_>, type_of: &'static str) {
        self.0.write(gc_context).type_of = type_of;
    }
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    /// Get the value of a particular property on this object.
    ///
    /// The `avm`, `context`, and `this` parameters exist so that this object
    /// can call virtual properties. Furthermore, since some virtual properties
    /// may resolve on the AVM stack, this function may return `None` instead
    /// of a `Value`. *This is not equivalent to `undefined`.* Instead, it is a
    /// signal that your value will be returned on the ActionScript stack, and
    /// that you should register a stack continuation in order to get it.
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        if name == "__proto__" {
            return Ok(self.proto().map_or(Value::Undefined, Value::Object).into());
        }

        if let Some(value) = self.0.read().values.get(name) {
            return value.get(avm, context, this);
        }

        Ok(Value::Undefined.into())
    }

    /// Set a named property on the object.
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
    ) -> Result<(), Error> {
        if name == "__proto__" {
            self.0.write(context.gc_context).prototype = value.as_object().ok();
        } else {
            match self
                .0
                .write(context.gc_context)
                .values
                .entry(name.to_owned())
            {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().set(avm, context, (*self).into(), value)?;
                }
                Entry::Vacant(entry) => {
                    entry.insert(Property::Stored {
                        value,
                        attributes: Default::default(),
                    });
                }
            }
        }

        Ok(())
    }

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
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Some(function) = &self.0.read().function {
            function.exec(avm, context, this, args)
        } else {
            Ok(Value::Undefined.into())
        }
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        _avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Ok(ScriptObject::object(context.gc_context, Some(this)).into())
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        let mut object = self.0.write(gc_context);
        if let Some(prop) = object.values.get(name) {
            if prop.can_delete() {
                object.values.remove(name);
                return true;
            }
        }

        false
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0.write(gc_context).values.insert(
            name.to_owned(),
            Property::Virtual {
                get,
                set,
                attributes,
            },
        );
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .write(gc_context)
            .values
            .insert(name.to_string(), Property::Stored { value, attributes });
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().prototype
    }

    /// Checks if the object has a given named property.
    fn has_property(&self, name: &str) -> bool {
        self.has_own_property(name)
            || self
                .proto()
                .as_ref()
                .map_or(false, |p| p.has_property(name))
    }

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    fn has_own_property(&self, name: &str) -> bool {
        if name == "__proto__" {
            return true;
        }
        self.0.read().values.contains_key(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.0
            .read()
            .values
            .get(name)
            .map(|p| p.is_overwritable())
            .unwrap_or(false)
    }

    /// Checks if a named property appears when enumerating the object.
    fn is_property_enumerable(&self, name: &str) -> bool {
        if let Some(prop) = self.0.read().values.get(name) {
            prop.is_enumerable()
        } else {
            false
        }
    }

    /// Enumerate the object.
    fn get_keys(&self) -> HashSet<String> {
        let mut result = self.proto().map_or_else(HashSet::new, |p| p.get_keys());

        self.0
            .read()
            .values
            .iter()
            .filter_map(|(k, p)| {
                if p.is_enumerable() {
                    Some(k.to_string())
                } else {
                    None
                }
            })
            .for_each(|k| {
                result.insert(k);
            });

        result
    }

    fn as_string(&self) -> String {
        if self.0.read().function.is_some() {
            "[type Function]".to_string()
        } else {
            "[object Object]".to_string()
        }
    }

    fn type_of(&self) -> &'static str {
        self.0.read().type_of
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(*self)
    }
    /// Get the underlying display node for this object, if it exists.
    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        self.0.read().display_object
    }

    /// Returns a copy of a given function.
    ///
    /// TODO: We have to clone here because of how executables are stored on
    /// objects directly. This might not be a good idea for performance.
    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.0.read().function.clone()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::avm1::activation::Activation;
    use crate::avm1::property::Attribute::*;
    use crate::backend::audio::NullAudioBackend;
    use crate::backend::navigator::NullNavigatorBackend;
    use crate::backend::render::NullRenderer;
    use crate::display_object::MovieClip;
    use crate::library::Library;
    use crate::prelude::*;
    use gc_arena::rootless_arena;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::sync::Arc;

    fn with_object<F, R>(swf_version: u8, test: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut Avm1<'gc>, &mut UpdateContext<'a, 'gc, '_>, Object<'gc>) -> R,
    {
        rootless_arena(|gc_context| {
            let mut avm = Avm1::new(gc_context, swf_version);
            let root = MovieClip::new(swf_version, gc_context).into();
            let mut context = UpdateContext {
                gc_context,
                global_time: 0,
                player_version: 32,
                swf_version,
                root,
                start_clip: root,
                active_clip: root,
                target_clip: Some(root),
                target_path: Value::Undefined,
                rng: &mut SmallRng::from_seed([0u8; 16]),
                action_queue: &mut crate::context::ActionQueue::new(),
                audio: &mut NullAudioBackend::new(),
                background_color: &mut Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
                library: &mut Library::new(),
                navigator: &mut NullNavigatorBackend::new(),
                renderer: &mut NullRenderer::new(),
                swf_data: &mut Arc::new(vec![]),
                system_prototypes: avm.prototypes().clone(),
                mouse_hovered_object: None,
            };

            let object = ScriptObject::object(gc_context, Some(avm.prototypes().object)).into();

            let globals = avm.global_object_cell();
            avm.insert_stack_frame(GcCell::allocate(
                gc_context,
                Activation::from_nothing(swf_version, globals, gc_context),
            ));

            test(&mut avm, &mut context, object)
        })
    }

    #[test]
    fn test_get_undefined() {
        with_object(0, |avm, context, object| {
            assert_eq!(
                object.get("not_defined", avm, context).unwrap(),
                ReturnValue::Immediate(Value::Undefined)
            );
        })
    }

    #[test]
    fn test_set_get() {
        with_object(0, |avm, context, object| {
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "forced",
                "forced".into(),
                EnumSet::empty(),
            );
            object
                .set("natural", "natural".into(), avm, context)
                .unwrap();

            assert_eq!(
                object.get("forced", avm, context).unwrap(),
                ReturnValue::Immediate("forced".into())
            );
            assert_eq!(
                object.get("natural", avm, context).unwrap(),
                ReturnValue::Immediate("natural".into())
            );
        })
    }

    #[test]
    fn test_set_readonly() {
        with_object(0, |avm, context, object| {
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "normal",
                "initial".into(),
                EnumSet::empty(),
            );
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "readonly",
                "initial".into(),
                ReadOnly.into(),
            );

            object
                .set("normal", "replaced".into(), avm, context)
                .unwrap();
            object
                .set("readonly", "replaced".into(), avm, context)
                .unwrap();

            assert_eq!(
                object.get("normal", avm, context).unwrap(),
                ReturnValue::Immediate("replaced".into())
            );
            assert_eq!(
                object.get("readonly", avm, context).unwrap(),
                ReturnValue::Immediate("initial".into())
            );
        })
    }

    #[test]
    fn test_deletable_not_readonly() {
        with_object(0, |avm, context, object| {
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "test",
                "initial".into(),
                DontDelete.into(),
            );

            assert_eq!(object.delete(context.gc_context, "test"), false);
            assert_eq!(
                object.get("test", avm, context).unwrap(),
                ReturnValue::Immediate("initial".into())
            );

            object
                .as_script_object()
                .unwrap()
                .set("test", "replaced".into(), avm, context)
                .unwrap();

            assert_eq!(object.delete(context.gc_context, "test"), false);
            assert_eq!(
                object.get("test", avm, context).unwrap(),
                ReturnValue::Immediate("replaced".into())
            );
        })
    }

    #[test]
    fn test_virtual_get() {
        with_object(0, |avm, context, object| {
            let getter = Executable::Native(|_avm, _context, _this, _args| {
                Ok(ReturnValue::Immediate("Virtual!".into()))
            });

            object.as_script_object().unwrap().add_property(
                context.gc_context,
                "test",
                getter,
                None,
                EnumSet::empty(),
            );

            assert_eq!(
                object.get("test", avm, context).unwrap(),
                ReturnValue::Immediate("Virtual!".into())
            );

            // This set should do nothing
            object.set("test", "Ignored!".into(), avm, context).unwrap();
            assert_eq!(
                object.get("test", avm, context).unwrap(),
                ReturnValue::Immediate("Virtual!".into())
            );
        })
    }

    #[test]
    fn test_delete() {
        with_object(0, |avm, context, object| {
            let getter = Executable::Native(|_avm, _context, _this, _args| {
                Ok(ReturnValue::Immediate("Virtual!".into()))
            });

            object.as_script_object().unwrap().add_property(
                context.gc_context,
                "virtual",
                getter.clone(),
                None,
                EnumSet::empty(),
            );
            object.as_script_object().unwrap().add_property(
                context.gc_context,
                "virtual_un",
                getter,
                None,
                DontDelete.into(),
            );
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "stored",
                "Stored!".into(),
                EnumSet::empty(),
            );
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "stored_un",
                "Stored!".into(),
                DontDelete.into(),
            );

            assert_eq!(object.delete(context.gc_context, "virtual"), true);
            assert_eq!(object.delete(context.gc_context, "virtual_un"), false);
            assert_eq!(object.delete(context.gc_context, "stored"), true);
            assert_eq!(object.delete(context.gc_context, "stored_un"), false);
            assert_eq!(object.delete(context.gc_context, "non_existent"), false);

            assert_eq!(
                object.get("virtual", avm, context).unwrap(),
                ReturnValue::Immediate(Value::Undefined)
            );
            assert_eq!(
                object.get("virtual_un", avm, context).unwrap(),
                ReturnValue::Immediate("Virtual!".into())
            );
            assert_eq!(
                object.get("stored", avm, context).unwrap(),
                ReturnValue::Immediate(Value::Undefined)
            );
            assert_eq!(
                object.get("stored_un", avm, context).unwrap(),
                ReturnValue::Immediate("Stored!".into())
            );
        })
    }

    #[test]
    fn test_iter_values() {
        with_object(0, |_avm, context, object| {
            let getter = Executable::Native(|_avm, _context, _this, _args| {
                Ok(ReturnValue::Immediate(Value::Null))
            });

            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "stored",
                Value::Null,
                EnumSet::empty(),
            );
            object.as_script_object().unwrap().define_value(
                context.gc_context,
                "stored_hidden",
                Value::Null,
                DontEnum.into(),
            );
            object.as_script_object().unwrap().add_property(
                context.gc_context,
                "virtual",
                getter.clone(),
                None,
                EnumSet::empty(),
            );
            object.as_script_object().unwrap().add_property(
                context.gc_context,
                "virtual_hidden",
                getter,
                None,
                DontEnum.into(),
            );

            let keys = object.get_keys();
            assert_eq!(keys.len(), 2);
            assert_eq!(keys.contains(&"stored".to_string()), true);
            assert_eq!(keys.contains(&"stored_hidden".to_string()), false);
            assert_eq!(keys.contains(&"virtual".to_string()), true);
            assert_eq!(keys.contains(&"virtual_hidden".to_string()), false);
        })
    }
}
