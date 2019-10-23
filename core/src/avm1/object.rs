use crate::avm1::function::{Executable, NativeFunction};
use crate::avm1::property::{Attribute, Property};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, UpdateContext, Value};
use crate::display_object::DisplayNode;
use core::fmt;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub const TYPE_OF_OBJECT: &str = "object";
pub const TYPE_OF_FUNCTION: &str = "function";
pub const TYPE_OF_MOVIE_CLIP: &str = "movieclip";

#[derive(Clone)]
pub struct Object<'gc> {
    prototype: Option<GcCell<'gc, Object<'gc>>>,
    display_node: Option<DisplayNode<'gc>>,
    values: HashMap<String, Property<'gc>>,
    function: Option<Executable<'gc>>,
    type_of: &'static str,
}

unsafe impl<'gc> gc_arena::Collect for Object<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.prototype.trace(cc);
        self.display_node.trace(cc);
        self.values.trace(cc);
    }
}

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Object")
            .field("prototype", &self.prototype)
            .field("display_node", &self.display_node)
            .field("values", &self.values)
            .field("function", &self.function.is_some())
            .finish()
    }
}

impl<'gc> Object<'gc> {
    pub fn object(
        _gc_context: MutationContext<'gc, '_>,
        proto: Option<GcCell<'gc, Object<'gc>>>,
    ) -> Self {
        Self {
            prototype: proto, //TODO: Should be Object
            type_of: TYPE_OF_OBJECT,
            display_node: None,
            values: HashMap::new(),
            function: None,
        }
    }

    /// Constructs an object with no values, not even builtins.
    ///
    /// Intended for constructing scope chains, since they exclusively use the
    /// object values, but can't just have a hashmap because of `with` and
    /// friends.
    pub fn bare_object() -> Self {
        Self {
            prototype: None,
            type_of: TYPE_OF_OBJECT,
            display_node: None,
            values: HashMap::new(),
            function: None,
        }
    }

    /// Construct a function sans prototype.
    pub fn bare_function(
        function: impl Into<Executable<'gc>>,
        fn_proto: Option<GcCell<'gc, Object<'gc>>>,
    ) -> Self {
        Self {
            prototype: fn_proto,
            type_of: TYPE_OF_FUNCTION,
            function: Some(function.into()),
            display_node: None,
            values: HashMap::new(),
        }
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
        fn_proto: Option<GcCell<'gc, Object<'gc>>>,
        prototype: Option<GcCell<'gc, Object<'gc>>>,
    ) -> GcCell<'gc, Object<'gc>> {
        let function = GcCell::allocate(gc_context, Self::bare_function(function, fn_proto));

        if let Some(p) = prototype {
            p.write(gc_context)
                .force_set("constructor", function, EnumSet::empty());
            function
                .write(gc_context)
                .force_set("prototype", p, EnumSet::empty());
        }

        function
    }

    pub fn set_display_node(&mut self, display_node: DisplayNode<'gc>) {
        self.display_node = Some(display_node);
    }

    pub fn display_node(&self) -> Option<DisplayNode<'gc>> {
        self.display_node
    }

    pub fn set(
        &mut self,
        name: &str,
        value: impl Into<Value<'gc>>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
    ) -> Result<(), Error> {
        if name == "__proto__" {
            self.prototype = value.into().as_object().ok().to_owned();

            Ok(())
        } else {
            match self.values.entry(name.to_owned()) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().set(avm, context, this, value)?;
                    Ok(())
                }
                Entry::Vacant(entry) => {
                    entry.insert(Property::Stored {
                        value: value.into(),
                        attributes: Default::default(),
                    });
                    Ok(())
                }
            }
        }
    }

    pub fn force_set_virtual<A>(
        &mut self,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: A,
    ) where
        A: Into<EnumSet<Attribute>>,
    {
        self.values.insert(
            name.to_owned(),
            Property::Virtual {
                get,
                set,
                attributes: attributes.into(),
            },
        );
    }

    pub fn force_set<A>(&mut self, name: &str, value: impl Into<Value<'gc>>, attributes: A)
    where
        A: Into<EnumSet<Attribute>>,
    {
        self.values.insert(
            name.to_string(),
            Property::Stored {
                value: value.into(),
                attributes: attributes.into(),
            },
        );
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
        fn_proto: Option<GcCell<'gc, Object<'gc>>>,
    ) where
        A: Into<EnumSet<Attribute>>,
    {
        self.force_set(
            name,
            Value::Object(Object::function(gc_context, function, fn_proto, None)),
            attributes,
        )
    }

    /// Get the value of a particular property on this object.
    ///
    /// The `avm`, `context`, and `this` parameters exist so that this object
    /// can call virtual properties. Likewise, this function returns a
    /// `ReturnValue` which allows pulling data from the return values of user
    /// functions.
    pub fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
    ) -> Result<ReturnValue<'gc>, Error> {
        if name == "__proto__" {
            return Ok(self
                .prototype
                .map_or(Value::Undefined, Value::Object)
                .into());
        }

        if let Some(value) = self.values.get(name) {
            return value.get(avm, context, this);
        }

        self.prototype
            .as_ref()
            .map_or(Ok(Value::Undefined.into()), |p| {
                p.read().get(name, avm, context, this)
            })
    }

    pub fn set_prototype(&mut self, prototype: GcCell<'gc, Object<'gc>>) {
        self.prototype = Some(prototype);
    }

    pub fn prototype(&self) -> Option<&GcCell<'gc, Object<'gc>>> {
        self.prototype.as_ref()
    }

    /// Delete a given value off the object.
    pub fn delete(&mut self, name: &str) -> bool {
        if let Some(prop) = self.values.get(name) {
            if prop.can_delete() {
                self.values.remove(name);
                return true;
            }
        }

        false
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.has_own_property(name)
            || self
                .prototype
                .as_ref()
                .map_or(false, |p| p.read().has_property(name))
    }

    pub fn has_own_property(&self, name: &str) -> bool {
        if name == "__proto__" {
            return true;
        }
        self.values.contains_key(name)
    }

    pub fn is_property_overwritable(&self, name: &str) -> bool {
        self.values
            .get(name)
            .map(|p| p.is_overwritable())
            .unwrap_or(false)
    }

    pub fn is_property_enumerable(&self, name: &str) -> bool {
        if let Some(prop) = self.values.get(name) {
            prop.is_enumerable()
        } else {
            false
        }
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.values
            .iter()
            .filter_map(|(k, p)| {
                if p.is_enumerable() {
                    Some(k.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Some(function) = &self.function {
            function.exec(avm, context, this, args)
        } else {
            Ok(Value::Undefined.into())
        }
    }

    pub fn as_string(&self) -> String {
        if self.function.is_some() {
            "[type Function]".to_string()
        } else {
            "[object Object]".to_string()
        }
    }

    pub fn set_type_of(&mut self, type_of: &'static str) {
        self.type_of = type_of;
    }

    pub fn type_of(&self) -> &'static str {
        self.type_of
    }

    /// Returns a copy of a given function.
    ///
    /// TODO: We have to clone here because of how executables are stored on
    /// objects directly. This might not be a good idea for performance.
    pub fn as_executable(&self) -> Option<Executable<'gc>> {
        self.function.clone()
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
    use crate::display_object::{DisplayObject, MovieClip};
    use crate::library::Library;
    use crate::prelude::*;
    use gc_arena::rootless_arena;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::sync::Arc;

    fn with_object<F, R>(swf_version: u8, test: F) -> R
    where
        F: for<'a, 'gc> FnOnce(
            &mut Avm1<'gc>,
            &mut UpdateContext<'a, 'gc, '_>,
            GcCell<'gc, Object<'gc>>,
        ) -> R,
    {
        rootless_arena(|gc_context| {
            let mut avm = Avm1::new(gc_context, swf_version);
            let movie_clip: Box<dyn DisplayObject> =
                Box::new(MovieClip::new(swf_version, gc_context));
            let root = GcCell::allocate(gc_context, movie_clip);
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
            };

            let object = GcCell::allocate(
                gc_context,
                Object::object(gc_context, Some(avm.prototypes().object)),
            );

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
                object
                    .read()
                    .get("not_defined", avm, context, object)
                    .unwrap(),
                Value::Undefined.into()
            );
        })
    }

    #[test]
    fn test_set_get() {
        with_object(0, |avm, context, object| {
            object
                .write(context.gc_context)
                .force_set("forced", "forced", EnumSet::empty());
            object
                .write(context.gc_context)
                .set("natural", "natural", avm, context, object)
                .unwrap();

            assert_eq!(
                object.read().get("forced", avm, context, object).unwrap(),
                ReturnValue::Immediate("forced".into())
            );
            assert_eq!(
                object.read().get("natural", avm, context, object).unwrap(),
                ReturnValue::Immediate("natural".into())
            );
        })
    }

    #[test]
    fn test_set_readonly() {
        with_object(0, |avm, context, object| {
            object
                .write(context.gc_context)
                .force_set("normal", "initial", EnumSet::empty());
            object
                .write(context.gc_context)
                .force_set("readonly", "initial", ReadOnly);

            object
                .write(context.gc_context)
                .set("normal", "replaced", avm, context, object)
                .unwrap();
            object
                .write(context.gc_context)
                .set("readonly", "replaced", avm, context, object)
                .unwrap();

            assert_eq!(
                object.read().get("normal", avm, context, object).unwrap(),
                ReturnValue::Immediate("replaced".into())
            );
            assert_eq!(
                object.read().get("readonly", avm, context, object).unwrap(),
                ReturnValue::Immediate("initial".into())
            );
        })
    }

    #[test]
    fn test_deletable_not_readonly() {
        with_object(0, |avm, context, object| {
            object
                .write(context.gc_context)
                .force_set("test", "initial", DontDelete);

            assert_eq!(object.write(context.gc_context).delete("test"), false);
            assert_eq!(
                object.read().get("test", avm, context, object).unwrap(),
                ReturnValue::Immediate("initial".into())
            );

            object
                .write(context.gc_context)
                .set("test", "replaced", avm, context, object)
                .unwrap();

            assert_eq!(object.write(context.gc_context).delete("test"), false);
            assert_eq!(
                object.read().get("test", avm, context, object).unwrap(),
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

            object.write(context.gc_context).force_set_virtual(
                "test",
                getter,
                None,
                EnumSet::empty(),
            );

            assert_eq!(
                object.read().get("test", avm, context, object).unwrap(),
                ReturnValue::Immediate("Virtual!".into())
            );

            // This set should do nothing
            object
                .write(context.gc_context)
                .set("test", "Ignored!", avm, context, object)
                .unwrap();
            assert_eq!(
                object.read().get("test", avm, context, object).unwrap(),
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

            object.write(context.gc_context).force_set_virtual(
                "virtual",
                getter.clone(),
                None,
                EnumSet::empty(),
            );
            object.write(context.gc_context).force_set_virtual(
                "virtual_un",
                getter,
                None,
                DontDelete,
            );
            object
                .write(context.gc_context)
                .force_set("stored", "Stored!", EnumSet::empty());
            object
                .write(context.gc_context)
                .force_set("stored_un", "Stored!", DontDelete);

            assert_eq!(object.write(context.gc_context).delete("virtual"), true);
            assert_eq!(object.write(context.gc_context).delete("virtual_un"), false);
            assert_eq!(object.write(context.gc_context).delete("stored"), true);
            assert_eq!(object.write(context.gc_context).delete("stored_un"), false);
            assert_eq!(
                object.write(context.gc_context).delete("non_existent"),
                false
            );

            assert_eq!(
                object.read().get("virtual", avm, context, object).unwrap(),
                Value::Undefined.into()
            );
            assert_eq!(
                object
                    .read()
                    .get("virtual_un", avm, context, object)
                    .unwrap(),
                ReturnValue::Immediate("Virtual!".into())
            );
            assert_eq!(
                object.read().get("stored", avm, context, object).unwrap(),
                Value::Undefined.into()
            );
            assert_eq!(
                object
                    .read()
                    .get("stored_un", avm, context, object)
                    .unwrap(),
                ReturnValue::Immediate("Stored!".into())
            );
        })
    }

    #[test]
    fn test_iter_values() {
        with_object(0, |_avm, context, object| {
            let getter = Executable::Native(|_avm, _context, _this, _args| Ok(Value::Null.into()));

            object
                .write(context.gc_context)
                .force_set("stored", Value::Null, EnumSet::empty());
            object
                .write(context.gc_context)
                .force_set("stored_hidden", Value::Null, DontEnum);
            object.write(context.gc_context).force_set_virtual(
                "virtual",
                getter.clone(),
                None,
                EnumSet::empty(),
            );
            object.write(context.gc_context).force_set_virtual(
                "virtual_hidden",
                getter,
                None,
                DontEnum,
            );

            let keys = object.read().get_keys();
            assert_eq!(keys.len(), 2);
            assert_eq!(keys.contains(&"stored".to_string()), true);
            assert_eq!(keys.contains(&"stored_hidden".to_string()), false);
            assert_eq!(keys.contains(&"virtual".to_string()), true);
            assert_eq!(keys.contains(&"virtual_hidden".to_string()), false);
        })
    }
}
