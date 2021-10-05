use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{ExecutionName, ExecutionReason};
use crate::avm1::property::{Attribute, Property};
use crate::avm1::property_map::{Entry, PropertyMap};
use crate::avm1::{Object, ObjectPtr, TObject, Value};
use crate::string::AvmString;
use core::fmt;
use gc_arena::{Collect, GcCell, MutationContext};

pub const TYPE_OF_OBJECT: &str = "object";

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub struct Watcher<'gc> {
    callback: Object<'gc>,
    user_data: Value<'gc>,
}

impl<'gc> Watcher<'gc> {
    pub fn new(callback: Object<'gc>, user_data: Value<'gc>) -> Self {
        Self {
            callback,
            user_data,
        }
    }

    pub fn call(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        old_value: Value<'gc>,
        new_value: Value<'gc>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let args = [Value::String(name), old_value, new_value, self.user_data];
        let exec = self.callback.as_executable().unwrap();
        exec.exec(
            ExecutionName::Dynamic(name),
            activation,
            this,
            0,
            &args,
            ExecutionReason::Special,
            self.callback,
        )
    }
}

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct ScriptObjectData<'gc> {
    properties: PropertyMap<'gc, Property<'gc>>,
    interfaces: Vec<Object<'gc>>,
    type_of: &'static str,
    watchers: PropertyMap<'gc, Watcher<'gc>>,
}

impl fmt::Debug for ScriptObjectData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Object")
            .field("properties", &self.properties)
            .field("watchers", &self.watchers)
            .finish()
    }
}

impl<'gc> ScriptObject<'gc> {
    pub fn object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        let object = Self(GcCell::allocate(
            gc_context,
            ScriptObjectData {
                type_of: TYPE_OF_OBJECT,
                properties: PropertyMap::new(),
                interfaces: vec![],
                watchers: PropertyMap::new(),
            },
        ));
        if let Some(proto) = proto {
            object.define_value(
                gc_context,
                "__proto__",
                proto.into(),
                Attribute::DONT_ENUM | Attribute::DONT_DELETE,
            );
        }
        object
    }

    /// Constructs and allocates an empty but normal object in one go.
    pub fn object_cell(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> Object<'gc> {
        Self::object(gc_context, proto).into()
    }

    /// Constructs an object with no properties, not even builtins.
    ///
    /// Intended for constructing scope chains, since they exclusively use the
    /// object properties, but can't just have a hashmap because of `with` and
    /// friends.
    pub fn bare_object(gc_context: MutationContext<'gc, '_>) -> Self {
        Self::object(gc_context, None)
    }

    pub fn set_type_of(&mut self, gc_context: MutationContext<'gc, '_>, type_of: &'static str) {
        self.0.write(gc_context).type_of = type_of;
    }

    /// Gets the value of a data property on this object.
    ///
    /// Doesn't look up the prototype chain and ignores virtual properties, thus cannot cause
    /// any side-effects.
    pub fn get_data(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Value<'gc> {
        self.0
            .read()
            .properties
            .get(name, activation.is_case_sensitive())
            .map_or(Value::Undefined, |property| property.data())
    }

    /// Sets a data property on this object.
    ///
    /// Doesn't look up the prototype chain and ignores virtual properties, but still might
    /// call to watchers.
    pub fn set_data(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        // TODO: Call watchers.
        match self
            .0
            .write(activation.context.gc_context)
            .properties
            .entry(name, activation.is_case_sensitive())
        {
            Entry::Occupied(mut entry) => entry.get_mut().set_data(value),
            Entry::Vacant(entry) => entry.insert(Property::new_stored(value, Attribute::empty())),
        }
        Ok(())
    }
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    /// Get the value of a particular non-virtual property on this object.
    fn get_local_stored(
        &self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Value<'gc>> {
        self.0
            .read()
            .properties
            .get(name.into(), activation.is_case_sensitive())
            .map(|property| property.data())
    }

    /// Set a named property on the object.
    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let setter = match self
            .0
            .write(activation.context.gc_context)
            .properties
            .entry(name, activation.is_case_sensitive())
        {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                entry.set_data(value);
                entry.setter()
            }
            Entry::Vacant(entry) => {
                entry.insert(Property::new_stored(value, Attribute::empty()));
                None
            }
        };

        if let Some(setter) = setter {
            if let Some(exec) = setter.as_executable() {
                if let Err(Error::ThrownValue(e)) = exec.exec(
                    ExecutionName::Static("[Setter]"),
                    activation,
                    this,
                    1,
                    &[value],
                    ExecutionReason::Special,
                    setter,
                ) {
                    return Err(Error::ThrownValue(e));
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
        _name: AvmString<'gc>,
        _activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Undefined)
    }

    fn getter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0
            .read()
            .properties
            .get(name, activation.is_case_sensitive())
            .and_then(|property| property.getter())
    }

    fn setter(
        &self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        self.0
            .read()
            .properties
            .get(name, activation.is_case_sensitive())
            .and_then(|property| property.setter())
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(ScriptObject::object(activation.context.gc_context, Some(this)).into())
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        if let Entry::Occupied(mut entry) = self
            .0
            .write(activation.context.gc_context)
            .properties
            .entry(name, activation.is_case_sensitive())
        {
            if entry.get().can_delete() {
                entry.remove_entry();
                return true;
            }
        }
        false
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: AvmString<'gc>,
        getter: Object<'gc>,
        setter: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        match self.0.write(gc_context).properties.entry(name, false) {
            Entry::Occupied(mut entry) => entry.get_mut().set_virtual(getter, setter),
            Entry::Vacant(entry) => entry.insert(Property::new_virtual(getter, setter, attributes)),
        }
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        getter: Object<'gc>,
        setter: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        match self
            .0
            .write(activation.context.gc_context)
            .properties
            .entry(name, activation.is_case_sensitive())
        {
            Entry::Occupied(mut entry) => entry.get_mut().set_virtual(getter, setter),
            Entry::Vacant(entry) => entry.insert(Property::new_virtual(getter, setter, attributes)),
        }
    }

    fn call_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let mut result = Ok(());
        let watcher = self
            .0
            .read()
            .watchers
            .get(name, activation.is_case_sensitive())
            .cloned();
        if let Some(watcher) = watcher {
            let old_value = self.get_stored(name, activation)?;
            match watcher.call(activation, name, old_value, *value, this) {
                Ok(v) => *value = v,
                Err(Error::ThrownValue(e)) => {
                    *value = Value::Undefined;
                    result = Err(Error::ThrownValue(e));
                }
                Err(_) => *value = Value::Undefined,
            };
        }

        result
    }

    fn watch(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.0.write(activation.context.gc_context).watchers.insert(
            name,
            Watcher::new(callback, user_data),
            activation.is_case_sensitive(),
        );
    }

    fn unwatch(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.0
            .write(activation.context.gc_context)
            .watchers
            .remove(name, activation.is_case_sensitive())
            .is_some()
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.0.write(gc_context).properties.insert(
            name.into(),
            Property::new_stored(value, attributes),
            true,
        );
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<AvmString<'gc>>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        match name {
            None => {
                // Change *all* attributes.
                for (_name, prop) in self.0.write(gc_context).properties.iter_mut() {
                    let new_atts = (prop.attributes() - clear_attributes) | set_attributes;
                    prop.set_attributes(new_atts);
                }
            }
            Some(name) => {
                if let Some(prop) = self.0.write(gc_context).properties.get_mut(name, false) {
                    let new_atts = (prop.attributes() - clear_attributes) | set_attributes;
                    prop.set_attributes(new_atts);
                }
            }
        }
    }

    fn proto(&self, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        self.get_data("__proto__".into(), activation)
    }

    /// Checks if the object has a given named property.
    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: AvmString<'gc>) -> bool {
        self.has_own_property(activation, name)
            || if let Value::Object(proto) = self.proto(activation) {
                proto.has_property(activation, name)
            } else {
                false
            }
    }

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    fn has_own_property(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0
            .read()
            .properties
            .contains_key(name, activation.is_case_sensitive())
    }

    fn has_own_virtual(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0
            .read()
            .properties
            .get(name, activation.is_case_sensitive())
            .map_or(false, |property| property.is_virtual())
    }

    /// Checks if a named property appears when enumerating the object.
    fn is_property_enumerable(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: AvmString<'gc>,
    ) -> bool {
        self.0
            .read()
            .properties
            .get(name, activation.is_case_sensitive())
            .map_or(false, |property| property.is_enumerable())
    }

    /// Enumerate the object.
    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<AvmString<'gc>> {
        let proto_keys = if let Value::Object(proto) = self.proto(activation) {
            proto.get_keys(activation)
        } else {
            Vec::new()
        };
        let mut out_keys = vec![];

        // Prototype keys come first.
        out_keys.extend(
            proto_keys
                .into_iter()
                .filter(|k| !self.has_own_property(activation, *k)),
        );

        // Then our own keys.
        out_keys.extend(self.0.read().properties.iter().filter_map(move |(k, p)| {
            if p.is_enumerable() {
                Some(k)
            } else {
                None
            }
        }));

        out_keys
    }

    fn type_of(&self) -> &'static str {
        self.0.read().type_of
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().interfaces.clone()
    }

    fn set_interfaces(&self, gc_context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0.write(gc_context).interfaces = iface_list;
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn length(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<i32, Error<'gc>> {
        self.get_data("length".into(), activation)
            .coerce_to_i32(activation)
    }

    fn set_length(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        new_length: i32,
    ) -> Result<(), Error<'gc>> {
        self.set_data("length".into(), new_length.into(), activation)
    }

    fn has_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> bool {
        let index_str = AvmString::new_utf8(activation.context.gc_context, index.to_string());
        self.has_own_property(activation, index_str)
    }

    fn get_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> Value<'gc> {
        let index_str = AvmString::new_utf8(activation.context.gc_context, index.to_string());
        self.get_data(index_str, activation)
    }

    fn set_element(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        let index_str = AvmString::new_utf8(activation.context.gc_context, index.to_string());
        self.set_data(index_str, value, activation)
    }

    fn delete_element(&self, activation: &mut Activation<'_, 'gc, '_>, index: i32) -> bool {
        let index_str = AvmString::new_utf8(activation.context.gc_context, index.to_string());
        self.delete(activation, index_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::avm1::function::Executable;
    use crate::avm1::globals::system::SystemProperties;
    use crate::avm1::property::Attribute;
    use crate::avm1::{activation::ActivationIdentifier, function::FunctionObject};
    use crate::avm1::{Avm1, Timers};
    use crate::avm2::Avm2;
    use crate::backend::audio::{AudioManager, NullAudioBackend};
    use crate::backend::locale::NullLocaleBackend;
    use crate::backend::log::NullLogBackend;
    use crate::backend::navigator::NullNavigatorBackend;
    use crate::backend::render::NullRenderer;
    use crate::backend::storage::MemoryStorageBackend;
    use crate::backend::ui::NullUiBackend;
    use crate::backend::video::NullVideoBackend;
    use crate::context::UpdateContext;
    use crate::display_object::{MovieClip, Stage};
    use crate::focus_tracker::FocusTracker;
    use crate::library::Library;
    use crate::loader::LoadManager;
    use crate::prelude::*;
    use crate::tag_utils::SwfMovie;
    use crate::vminterface::Instantiator;
    use gc_arena::rootless_arena;
    use instant::Instant;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    fn with_object<F, R>(swf_version: u8, test: F) -> R
    where
        F: for<'a, 'gc> FnOnce(&mut Activation<'_, 'gc, '_>, Object<'gc>) -> R,
    {
        rootless_arena(|gc_context| {
            let mut avm1 = Avm1::new(gc_context, swf_version);
            let mut avm2 = Avm2::new(gc_context);
            let swf = Arc::new(SwfMovie::empty(swf_version));
            let root: DisplayObject<'_> = MovieClip::new(swf.clone(), gc_context).into();
            root.set_depth(gc_context, 0);

            let stage = Stage::empty(gc_context, 550, 400);
            let mut frame_rate = 12.0;

            let object = ScriptObject::object(gc_context, Some(avm1.prototypes().object)).into();
            let globals = avm1.global_object_cell();

            let mut context = UpdateContext {
                gc_context,
                player_version: 32,
                swf: &swf,
                stage,
                rng: &mut SmallRng::from_seed([0u8; 32]),
                action_queue: &mut crate::context::ActionQueue::new(),
                audio: &mut NullAudioBackend::new(),
                audio_manager: &mut AudioManager::new(),
                ui: &mut NullUiBackend::new(),
                library: &mut Library::empty(gc_context),
                navigator: &mut NullNavigatorBackend::new(),
                renderer: &mut NullRenderer::new(),
                locale: &mut NullLocaleBackend::new(),
                log: &mut NullLogBackend::new(),
                video: &mut NullVideoBackend::new(),
                mouse_over_object: None,
                mouse_down_object: None,
                mouse_position: &(Twips::ZERO, Twips::ZERO),
                drag_object: &mut None,
                player: None,
                load_manager: &mut LoadManager::new(),
                system: &mut SystemProperties::default(),
                instance_counter: &mut 0,
                storage: &mut MemoryStorageBackend::default(),
                shared_objects: &mut HashMap::new(),
                unbound_text_fields: &mut Vec::new(),
                timers: &mut Timers::new(),
                current_context_menu: &mut None,
                needs_render: &mut false,
                avm1: &mut avm1,
                avm2: &mut avm2,
                external_interface: &mut Default::default(),
                update_start: Instant::now(),
                max_execution_duration: Duration::from_secs(15),
                focus_tracker: FocusTracker::new(gc_context),
                times_get_time_called: 0,
                time_offset: &mut 0,
                frame_rate: &mut frame_rate,
            };
            context.stage.replace_at_depth(&mut context, root, 0);

            root.post_instantiation(&mut context, root, None, Instantiator::Movie, false);
            root.set_name(context.gc_context, "".into());

            let swf_version = context.swf.version();
            let mut activation = Activation::from_nothing(
                context,
                ActivationIdentifier::root("[Test]"),
                swf_version,
                globals,
                root,
            );

            test(&mut activation, object)
        })
    }

    #[test]
    fn test_get_undefined() {
        with_object(0, |activation, object| {
            assert_eq!(
                object.get("not_defined", activation).unwrap(),
                Value::Undefined
            );
        })
    }

    #[test]
    fn test_set_get() {
        with_object(0, |activation, object| {
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "forced",
                "forced".into(),
                Attribute::empty(),
            );
            object.set("natural", "natural".into(), activation).unwrap();

            assert_eq!(object.get("forced", activation).unwrap(), "forced".into());
            assert_eq!(object.get("natural", activation).unwrap(), "natural".into());
        })
    }

    #[test]
    fn test_set_readonly() {
        with_object(0, |activation, object| {
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "normal",
                "initial".into(),
                Attribute::empty(),
            );
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "readonly",
                "initial".into(),
                Attribute::READ_ONLY,
            );

            object.set("normal", "replaced".into(), activation).unwrap();
            object
                .set("readonly", "replaced".into(), activation)
                .unwrap();

            assert_eq!(object.get("normal", activation).unwrap(), "replaced".into());
            assert_eq!(
                object.get("readonly", activation).unwrap(),
                "initial".into()
            );
        })
    }

    #[test]
    fn test_deletable_not_readonly() {
        with_object(0, |activation, object| {
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "test",
                "initial".into(),
                Attribute::DONT_DELETE,
            );

            assert!(!object.delete(activation, "test".into()));
            assert_eq!(object.get("test", activation).unwrap(), "initial".into());

            object
                .as_script_object()
                .unwrap()
                .set("test", "replaced".into(), activation)
                .unwrap();

            assert!(!object.delete(activation, "test".into()));
            assert_eq!(object.get("test", activation).unwrap(), "replaced".into());
        })
    }

    #[test]
    fn test_virtual_get() {
        with_object(0, |activation, object| {
            let getter = FunctionObject::function(
                activation.context.gc_context,
                Executable::Native(|_avm, _this, _args| Ok("Virtual!".into())),
                None,
                activation.context.avm1.prototypes.function,
            );

            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "test".into(),
                getter,
                None,
                Attribute::empty(),
            );

            assert_eq!(object.get("test", activation).unwrap(), "Virtual!".into());

            // This set should do nothing
            object.set("test", "Ignored!".into(), activation).unwrap();
            assert_eq!(object.get("test", activation).unwrap(), "Virtual!".into());
        })
    }

    #[test]
    fn test_delete() {
        with_object(0, |activation, object| {
            let getter = FunctionObject::function(
                activation.context.gc_context,
                Executable::Native(|_avm, _this, _args| Ok("Virtual!".into())),
                None,
                activation.context.avm1.prototypes.function,
            );

            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "virtual".into(),
                getter,
                None,
                Attribute::empty(),
            );
            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "virtual_un".into(),
                getter,
                None,
                Attribute::DONT_DELETE,
            );
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "stored",
                "Stored!".into(),
                Attribute::empty(),
            );
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "stored_un",
                "Stored!".into(),
                Attribute::DONT_DELETE,
            );

            assert!(object.delete(activation, "virtual".into()));
            assert!(!object.delete(activation, "virtual_un".into()));
            assert!(object.delete(activation, "stored".into()));
            assert!(!object.delete(activation, "stored_un".into()));
            assert!(!object.delete(activation, "non_existent".into()));

            assert_eq!(object.get("virtual", activation).unwrap(), Value::Undefined);
            assert_eq!(
                object.get("virtual_un", activation).unwrap(),
                "Virtual!".into()
            );
            assert_eq!(object.get("stored", activation).unwrap(), Value::Undefined);
            assert_eq!(
                object.get("stored_un", activation).unwrap(),
                "Stored!".into()
            );
        })
    }

    #[test]
    fn test_get_keys() {
        with_object(0, |activation, object| {
            let getter = FunctionObject::function(
                activation.context.gc_context,
                Executable::Native(|_avm, _this, _args| Ok(Value::Null)),
                None,
                activation.context.avm1.prototypes.function,
            );

            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "stored",
                Value::Null,
                Attribute::empty(),
            );
            object.as_script_object().unwrap().define_value(
                activation.context.gc_context,
                "stored_hidden",
                Value::Null,
                Attribute::DONT_ENUM,
            );
            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "virtual".into(),
                getter,
                None,
                Attribute::empty(),
            );
            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "virtual_hidden".into(),
                getter,
                None,
                Attribute::DONT_ENUM,
            );

            let keys: Vec<_> = object.get_keys(activation);
            assert_eq!(keys.len(), 2);
            assert!(keys.contains(&"stored".into()));
            assert!(!keys.contains(&"stored_hidden".into()));
            assert!(keys.contains(&"virtual".into()));
            assert!(!keys.contains(&"virtual_hidden".into()));
        })
    }
}
