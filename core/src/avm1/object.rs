use crate::avm1::{ActionContext, Avm1, Value};
use crate::avm1::function::{Executable, NativeFunction, Avm1Function};
use crate::display_object::DisplayNode;
use crate::tag_utils::SwfSlice;
use core::fmt;
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
    Value::String("[Object object]".to_string())
}

#[derive(Clone)]
pub enum Property<'gc> {
    Virtual {
        get: NativeFunction<'gc>,
        set: Option<NativeFunction<'gc>>,
    },
    Stored {
        value: Value<'gc>,
        // TODO: attributes
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
            Property::Stored { value, .. } => {
                replace::<Value<'gc>>(value, new_value);
            }
        }
    }
}

unsafe impl<'gc> gc_arena::Collect for Property<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            Property::Virtual { get, set } => {
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
            Property::Virtual { get: _, set } => f
                .debug_struct("Property::Virtual")
                .field("get", &true)
                .field("set", &set.is_some())
                .finish(),
            Property::Stored { value } => f
                .debug_struct("Property::Stored")
                .field("value", &value)
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

        result.force_set_function("toString", default_to_string, gc_context);

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

    pub fn action_function(swf_version: u8, actions: SwfSlice, name: &str, params: &[&str]) -> Self {
        Self {
            type_of: TYPE_OF_FUNCTION,
            function: Some(Executable::Action(Avm1Function::new(swf_version, actions, name, params))),
            display_node: None,
            values: HashMap::new()
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
                entry.insert(Property::Stored { value });
            }
        }
    }

    pub fn force_set_virtual(
        &mut self,
        name: &str,
        get: NativeFunction<'gc>,
        set: Option<NativeFunction<'gc>>,
    ) {
        self.values
            .insert(name.to_owned(), Property::Virtual { get, set });
    }

    pub fn force_set(&mut self, name: &str, value: Value<'gc>) {
        self.values
            .insert(name.to_string(), Property::Stored { value });
    }

    pub fn set_native_function(
        &mut self,
        name: &str,
        function: NativeFunction<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut ActionContext<'_, 'gc, '_>,
        this: GcCell<'gc, Object<'gc>>,
    ) {
        self.set(
            name,
            Value::Object(GcCell::allocate(
                context.gc_context,
                Object::native_function(function),
            )),
            avm,
            context,
            this,
        )
    }

    pub fn force_set_function(
        &mut self,
        name: &str,
        function: NativeFunction<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) {
        self.force_set(
            name,
            Value::Object(GcCell::allocate(gc_context, Object::native_function(function))),
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

    pub fn has_property(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn has_own_property(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn iter_values(&self) -> impl Iterator<Item=(&String, &Value<'gc>)> {
        self.values.iter()
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
            let movie_clip: Box<dyn DisplayObject> = Box::new(MovieClip::new(gc_context));
            let root = GcCell::allocate(gc_context, movie_clip);
            let mut context = ActionContext {
                gc_context,
                global_time: 0,
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
            object
                .write(context.gc_context)
                .force_set("forced", Value::String("forced".to_string()));
            object.write(context.gc_context).set(
                "natural",
                Value::String("natural".to_string()),
                avm,
                context,
                object,
            );

            assert_eq!(
                object.read().get("forced", avm, context, object),
                Value::String("forced".to_string())
            );
            assert_eq!(
                object.read().get("natural", avm, context, object),
                Value::String("natural".to_string())
            );
        })
    }

    #[test]
    fn test_virtual_get() {
        with_object(0, |avm, context, object| {
            let getter: NativeFunction =
                |_avm, _context, _this, _args| Value::String("Virtual!".to_string());
            object
                .write(context.gc_context)
                .force_set_virtual("test", getter, None);

            assert_eq!(
                object.read().get("test", avm, context, object),
                Value::String("Virtual!".to_string())
            );

            // This set should do nothing
            object.write(context.gc_context).set(
                "test",
                Value::String("Ignored!".to_string()),
                avm,
                context,
                object,
            );
            assert_eq!(
                object.read().get("test", avm, context, object),
                Value::String("Virtual!".to_string())
            );
        })
    }
}
