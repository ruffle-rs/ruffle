use self::Attribute::*;
use crate::avm1::function::{Avm1Function, Executable, NativeFunction};
use crate::avm1::{ActionContext, Avm1, Value};
use crate::display_object::DisplayNode;
use core::fmt;
use enumset::{EnumSet, EnumSetType};
use gc_arena::{GcCell, MutationContext};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem::replace;

pub const TYPE_OF_OBJECT: &str = "object";
pub const TYPE_OF_FUNCTION: &str = "function";
pub const TYPE_OF_MOVIE_CLIP: &str = "movieclip";

fn default_to_string<'gc>(
    _: &mut Avm1<'gc>,
    _: &mut ActionContext<'_, 'gc, '_>,
    _: GcCell<'gc, Object<'gc>>,
    _: &[Value<'gc>],
) -> Value<'gc> {
    "[Object object]".to_string().into()
}

#[derive(EnumSetType, Debug)]
pub enum Attribute {
    DontDelete,
    DontEnum,
    ReadOnly,
}

#[derive(Clone)]
pub enum Property<'gc> {
    Virtual {
        get: NativeFunction<'gc>,
        set: Option<NativeFunction<'gc>>,
        attributes: EnumSet<Attribute>,
    },
    Stored {
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    },
}

impl<'gc> Property<'gc> {
    pub fn get(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
    ) -> Value<'gc> {
        match self {
            Property::Virtual { get, .. } => get(avm, context, this, &[]),
            Property::Stored { value, .. } => value.to_owned(),
        }
    }

    pub fn set(
        &mut self,
        avm: &mut Avm1<'gc>,
        context: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        new_value: Value<'gc>,
    ) {
        match self {
            Property::Virtual { set, .. } => {
                if let Some(function) = set {
                    function(avm, context, this, &[new_value]);
                }
            }
            Property::Stored {
                value, attributes, ..
            } => {
                if !attributes.contains(ReadOnly) {
                    replace::<Value<'gc>>(value, new_value);
                }
            }
        }
    }

    pub fn can_delete(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(DontDelete),
            Property::Stored { attributes, .. } => !attributes.contains(DontDelete),
        }
    }

    pub fn is_enumerable(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(DontEnum),
            Property::Stored { attributes, .. } => !attributes.contains(DontEnum),
        }
    }
}

unsafe impl<'gc> gc_arena::Collect for Property<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            Property::Virtual { get, set, .. } => {
                get.trace(cc);
                set.trace(cc);
            }
            Property::Stored { value, .. } => value.trace(cc),
        }
    }
}

impl fmt::Debug for Property<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Property::Virtual {
                get: _,
                set,
                attributes,
            } => f
                .debug_struct("Property::Virtual")
                .field("get", &true)
                .field("set", &set.is_some())
                .field("attributes", &attributes)
                .finish(),
            Property::Stored { value, attributes } => f
                .debug_struct("Property::Stored")
                .field("value", &value)
                .field("attributes", &attributes)
                .finish(),
        }
    }
}

#[derive(Clone)]
pub struct Object<'gc> {
    display_node: Option<DisplayNode<'gc>>,
    values: HashMap<String, Property<'gc>>,
    function: Option<Executable<'gc>>,
    type_of: &'static str,
}

unsafe impl<'gc> gc_arena::Collect for Object<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.display_node.trace(cc);
        self.values.trace(cc);
    }
}

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Object")
            .field("display_node", &self.display_node)
            .field("values", &self.values)
            .field("function", &self.function.is_some())
            .finish()
    }
}

impl<'gc> Object<'gc> {
    pub fn object(gc_context: MutationContext<'gc, '_>) -> Self {
        let mut result = Self {
            type_of: TYPE_OF_OBJECT,
            display_node: None,
            values: HashMap::new(),
            function: None,
        };

        result.force_set_function(
            "toString",
            default_to_string,
            gc_context,
            DontDelete | DontEnum,
        );

        result
    }

    /// Constructs an object with no values, not even builtins.
    ///
    /// Intended for constructing scope chains, since they exclusively use the
    /// object values, but can't just have a hashmap because of `with` and
    /// friends.
    pub fn bare_object() -> Self {
        Self {
            type_of: TYPE_OF_OBJECT,
            display_node: None,
            values: HashMap::new(),
            function: None,
        }
    }

    pub fn native_function(function: NativeFunction<'gc>) -> Self {
        Self {
            type_of: TYPE_OF_FUNCTION,
            function: Some(Executable::Native(function)),
            display_node: None,
            values: HashMap::new(),
        }
    }

    pub fn action_function(func: Avm1Function<'gc>) -> Self {
        Self {
            type_of: TYPE_OF_FUNCTION,
            function: Some(Executable::Action(func)),
            display_node: None,
            values: HashMap::new(),
        }
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
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
    ) {
        match self.values.entry(name.to_owned()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().set(avm, context, this, value);
            }
            Entry::Vacant(entry) => {
                entry.insert(Property::Stored {
                    value,
                    attributes: Default::default(),
                });
            }
        }
    }

    pub fn force_set_virtual<A>(
        &mut self,
        name: &str,
        get: NativeFunction<'gc>,
        set: Option<NativeFunction<'gc>>,
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

    pub fn force_set<A>(&mut self, name: &str, value: Value<'gc>, attributes: A)
    where
        A: Into<EnumSet<Attribute>>,
    {
        self.values.insert(
            name.to_string(),
            Property::Stored {
                value,
                attributes: attributes.into(),
            },
        );
    }

    pub fn force_set_function<A>(
        &mut self,
        name: &str,
        function: NativeFunction<'gc>,
        gc_context: MutationContext<'gc, '_>,
        attributes: A,
    ) where
        A: Into<EnumSet<Attribute>>,
    {
        self.force_set(
            name,
            Value::Object(GcCell::allocate(
                gc_context,
                Object::native_function(function),
            )),
            attributes,
        )
    }

    pub fn get(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
    ) -> Value<'gc> {
        if let Some(value) = self.values.get(name) {
            return value.get(avm, context, this);
        }
        Value::Undefined
    }

    /// Retrieve a value from an object if and only if the value in the object
    /// property is non-virtual.
    pub fn force_get(&self, name: &str) -> Value<'gc> {
        if let Some(Property::Stored { value, .. }) = self.values.get(name) {
            return value.to_owned();
        }
        Value::Undefined
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
        self.values.contains_key(name)
    }

    pub fn has_own_property(&self, name: &str) -> bool {
        self.values.contains_key(name)
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
        context: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
        args: &[Value<'gc>],
    ) -> Option<Value<'gc>> {
        if let Some(function) = &self.function {
            function.exec(avm, context, this, args)
        } else {
            Some(Value::Undefined)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::avm1::activation::Activation;
    use crate::backend::audio::NullAudioBackend;
    use crate::backend::navigator::NullNavigatorBackend;
    use crate::display_object::DisplayObject;
    use crate::movie_clip::MovieClip;
    use gc_arena::rootless_arena;
    use rand::{rngs::SmallRng, SeedableRng};

    fn with_object<F, R>(swf_version: u8, test: F) -> R
    where
        F: for<'a, 'gc> FnOnce(
            &mut Avm1<'gc>,
            &mut ActionContext<'a, 'gc, '_>,
            GcCell<'gc, Object<'gc>>,
        ) -> R,
    {
        rootless_arena(|gc_context| {
            let mut avm = Avm1::new(gc_context, swf_version);
            let movie_clip: Box<dyn DisplayObject> =
                Box::new(MovieClip::new(swf_version, gc_context));
            let root = GcCell::allocate(gc_context, movie_clip);
            let mut context = ActionContext {
                gc_context,
                global_time: 0,
                player_version: 32,
                root,
                start_clip: root,
                active_clip: root,
                target_clip: Some(root),
                target_path: Value::Undefined,
                rng: &mut SmallRng::from_seed([0u8; 16]),
                audio: &mut NullAudioBackend::new(),
                navigator: &mut NullNavigatorBackend::new(),
            };
            let object = GcCell::allocate(gc_context, Object::object(gc_context));

            let globals = avm.global_object_cell();
            avm.insert_stack_frame(
                Activation::from_nothing(swf_version, globals, gc_context),
                &mut context,
            );

            test(&mut avm, &mut context, object)
        })
    }

    #[test]
    fn test_get_undefined() {
        with_object(0, |avm, context, object| {
            assert_eq!(
                object.read().get("not_defined", avm, context, object),
                Value::Undefined
            );
        })
    }

    #[test]
    fn test_set_get() {
        with_object(0, |avm, context, object| {
            object.write(context.gc_context).force_set(
                "forced",
                "forced".to_string().into(),
                EnumSet::empty(),
            );
            object.write(context.gc_context).set(
                "natural",
                "natural".to_string().into(),
                avm,
                context,
                object,
            );

            assert_eq!(
                object.read().get("forced", avm, context, object),
                "forced".to_string().into()
            );
            assert_eq!(
                object.read().get("natural", avm, context, object),
                "natural".to_string().into()
            );
        })
    }

    #[test]
    fn test_set_readonly() {
        with_object(0, |avm, context, object| {
            object.write(context.gc_context).force_set(
                "normal",
                "initial".to_string().into(),
                EnumSet::empty(),
            );
            object.write(context.gc_context).force_set(
                "readonly",
                "initial".to_string().into(),
                ReadOnly,
            );

            object.write(context.gc_context).set(
                "normal",
                "replaced".to_string().into(),
                avm,
                context,
                object,
            );
            object.write(context.gc_context).set(
                "readonly",
                "replaced".to_string().into(),
                avm,
                context,
                object,
            );

            assert_eq!(
                object.read().get("normal", avm, context, object),
                "replaced".to_string().into()
            );
            assert_eq!(
                object.read().get("readonly", avm, context, object),
                "initial".to_string().into()
            );
        })
    }

    #[test]
    fn test_deletable_not_readonly() {
        with_object(0, |avm, context, object| {
            object.write(context.gc_context).force_set(
                "test",
                "initial".to_string().into(),
                DontDelete,
            );

            assert_eq!(object.write(context.gc_context).delete("test"), false);
            assert_eq!(
                object.read().get("test", avm, context, object),
                "initial".to_string().into()
            );

            object.write(context.gc_context).set(
                "test",
                "replaced".to_string().into(),
                avm,
                context,
                object,
            );

            assert_eq!(object.write(context.gc_context).delete("test"), false);
            assert_eq!(
                object.read().get("test", avm, context, object),
                "replaced".to_string().into()
            );
        })
    }

    #[test]
    fn test_virtual_get() {
        with_object(0, |avm, context, object| {
            let getter: NativeFunction =
                |_avm, _context, _this, _args| "Virtual!".to_string().into();
            object.write(context.gc_context).force_set_virtual(
                "test",
                getter,
                None,
                EnumSet::empty(),
            );

            assert_eq!(
                object.read().get("test", avm, context, object),
                "Virtual!".to_string().into()
            );

            // This set should do nothing
            object.write(context.gc_context).set(
                "test",
                "Ignored!".to_string().into(),
                avm,
                context,
                object,
            );
            assert_eq!(
                object.read().get("test", avm, context, object),
                "Virtual!".to_string().into()
            );
        })
    }

    #[test]
    fn test_delete() {
        with_object(0, |avm, context, object| {
            let getter: NativeFunction =
                |_avm, _context, _this, _args| "Virtual!".to_string().into();

            object.write(context.gc_context).force_set_virtual(
                "virtual",
                getter,
                None,
                EnumSet::empty(),
            );
            object.write(context.gc_context).force_set_virtual(
                "virtual_un",
                getter,
                None,
                DontDelete,
            );
            object.write(context.gc_context).force_set(
                "stored",
                "Stored!".to_string().into(),
                EnumSet::empty(),
            );
            object.write(context.gc_context).force_set(
                "stored_un",
                "Stored!".to_string().into(),
                DontDelete,
            );

            assert_eq!(object.write(context.gc_context).delete("virtual"), true);
            assert_eq!(object.write(context.gc_context).delete("virtual_un"), false);
            assert_eq!(object.write(context.gc_context).delete("stored"), true);
            assert_eq!(object.write(context.gc_context).delete("stored_un"), false);
            assert_eq!(
                object.write(context.gc_context).delete("non_existent"),
                false
            );

            assert_eq!(
                object.read().get("virtual", avm, context, object),
                Value::Undefined
            );
            assert_eq!(
                object.read().get("virtual_un", avm, context, object),
                "Virtual!".to_string().into()
            );
            assert_eq!(
                object.read().get("stored", avm, context, object),
                Value::Undefined
            );
            assert_eq!(
                object.read().get("stored_un", avm, context, object),
                "Stored!".to_string().into()
            );
        })
    }

    #[test]
    fn test_iter_values() {
        with_object(0, |_avm, context, object| {
            let getter: NativeFunction = |_avm, _context, _this, _args| Value::Null;

            object
                .write(context.gc_context)
                .force_set("stored", Value::Null, EnumSet::empty());
            object
                .write(context.gc_context)
                .force_set("stored_hidden", Value::Null, DontEnum);
            object.write(context.gc_context).force_set_virtual(
                "virtual",
                getter,
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
