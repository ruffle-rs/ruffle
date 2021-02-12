use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, ExecutionReason, FunctionObject, NativeFunction};
use crate::avm1::property::{Attribute, Property};
use crate::avm1::{AvmString, Object, ObjectPtr, TObject, Value};
use crate::property_map::{Entry, PropertyMap};
use core::fmt;
use gc_arena::{Collect, GcCell, MutationContext};
use std::borrow::Cow;

pub const TYPE_OF_OBJECT: &str = "object";

#[derive(Debug, Clone, Collect)]
#[collect(no_drop)]
pub enum ArrayStorage<'gc> {
    Vector(Vec<Value<'gc>>),
    Properties { length: usize },
}

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

    #[allow(clippy::too_many_arguments)]
    pub fn call(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,

        name: &str,
        old_value: Value<'gc>,
        new_value: Value<'gc>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, crate::avm1::error::Error<'gc>> {
        let args = [
            Value::String(AvmString::new(
                activation.context.gc_context,
                name.to_string(),
            )),
            old_value,
            new_value,
            self.user_data,
        ];
        if let Some(executable) = self.callback.as_executable() {
            executable.exec(
                name,
                activation,
                this,
                base_proto,
                &args,
                ExecutionReason::Special,
                self.callback,
            )
        } else {
            Ok(Value::Undefined)
        }
    }
}

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct ScriptObjectData<'gc> {
    prototype: Option<Object<'gc>>,
    values: PropertyMap<Property<'gc>>,
    interfaces: Vec<Object<'gc>>,
    type_of: &'static str,
    array: ArrayStorage<'gc>,
    watchers: PropertyMap<Watcher<'gc>>,
}

impl fmt::Debug for ScriptObjectData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Object")
            .field("prototype", &self.prototype)
            .field("values", &self.values)
            .field("array", &self.array)
            .field("watchers", &self.watchers)
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
                values: PropertyMap::new(),
                array: ArrayStorage::Properties { length: 0 },
                interfaces: vec![],
                watchers: PropertyMap::new(),
            },
        ))
    }

    pub fn array(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> ScriptObject<'gc> {
        let object = ScriptObject(GcCell::allocate(
            gc_context,
            ScriptObjectData {
                prototype: proto,
                type_of: TYPE_OF_OBJECT,
                values: PropertyMap::new(),
                array: ArrayStorage::Vector(Vec::new()),
                interfaces: vec![],
                watchers: PropertyMap::new(),
            },
        ));
        object.sync_native_property("length", gc_context, Some(0.into()), false);
        object
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
                values: PropertyMap::new(),
                array: ArrayStorage::Properties { length: 0 },
                interfaces: vec![],
                watchers: PropertyMap::new(),
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
                values: PropertyMap::new(),
                array: ArrayStorage::Properties { length: 0 },
                interfaces: vec![],
                watchers: PropertyMap::new(),
            },
        ))
    }

    /// Declare a native function on the current object.
    ///
    /// This is intended for use with defining host object prototypes. Notably,
    /// this creates a function object without an explicit `prototype`, which
    /// is only possible when defining host functions. User-defined functions
    /// always get a fresh explicit prototype, so you should never force set a
    /// user-defined function.
    pub fn force_set_function(
        &mut self,
        name: &str,
        function: NativeFunction<'gc>,
        gc_context: MutationContext<'gc, '_>,
        attributes: Attribute,
        fn_proto: Option<Object<'gc>>,
    ) {
        self.define_value(
            gc_context,
            name,
            FunctionObject::bare_function(
                gc_context,
                Some(function),
                Option::<Executable>::None,
                fn_proto,
            )
            .into(),
            attributes,
        )
    }

    pub fn set_type_of(&mut self, gc_context: MutationContext<'gc, '_>, type_of: &'static str) {
        self.0.write(gc_context).type_of = type_of;
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn sync_native_property(
        &self,
        name: &str,
        gc_context: MutationContext<'gc, '_>,
        native_value: Option<Value<'gc>>,
        is_enumerable: bool,
    ) {
        match self.0.write(gc_context).values.entry(name, false) {
            Entry::Occupied(mut entry) => {
                if let Property::Stored { value, .. } = entry.get_mut() {
                    match native_value {
                        None => {
                            entry.remove_entry();
                        }
                        Some(native_value) => {
                            *value = native_value;
                        }
                    }
                }
            }
            Entry::Vacant(entry) => {
                if let Some(native_value) = native_value {
                    entry.insert(Property::Stored {
                        value: native_value,
                        attributes: if is_enumerable {
                            Attribute::empty()
                        } else {
                            Attribute::DONT_ENUM
                        },
                    });
                }
            }
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(crate) fn internal_set(
        &self,
        name: &str,
        mut value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if name == "__proto__" {
            self.0.write(activation.context.gc_context).prototype =
                Some(value.coerce_to_object(activation));
        } else if let Ok(index) = name.parse::<usize>() {
            self.set_array_element(index, value.to_owned(), activation.context.gc_context);
        } else if !name.is_empty() {
            if name == "length" {
                let length = value
                    .coerce_to_f64(activation)
                    .map(|v| v.abs() as i32)
                    .unwrap_or(0);
                if length > 0 {
                    self.set_length(activation.context.gc_context, length as usize);
                } else {
                    self.set_length(activation.context.gc_context, 0);
                }
            }

            //Before actually inserting a new property, we need to crawl the
            //prototype chain for virtual setters, which kind of break how
            //ECMAScript `[[Set]]` is supposed to work...
            let is_vacant = !self
                .0
                .read()
                .values
                .contains_key(name, activation.is_case_sensitive());
            let mut worked = false;

            if is_vacant {
                let mut proto: Option<Object<'gc>> = Some((*self).into());
                while let Some(this_proto) = proto {
                    if this_proto.has_own_virtual(activation, name) {
                        break;
                    }

                    proto = this_proto.proto();
                }

                if let Some(this_proto) = proto {
                    worked = true;
                    if let Some(rval) = this_proto.call_setter(name, value, activation) {
                        if let Some(exec) = rval.as_executable() {
                            let _ = exec.exec(
                                "[Setter]",
                                activation,
                                this,
                                Some(this_proto),
                                &[value],
                                ExecutionReason::Special,
                                rval,
                            );
                        }
                    }
                }
            }

            //This signals we didn't call a virtual setter above. Normally,
            //we'd resolve and return up there, but we have borrows that need
            //to end before we can do so.
            if !worked {
                let watcher = self
                    .0
                    .read()
                    .watchers
                    .get(name, activation.is_case_sensitive())
                    .cloned();
                let mut return_value = Ok(());
                if let Some(watcher) = watcher {
                    let old_value = self.get(name, activation)?;
                    value = match watcher.call(activation, name, old_value, value, this, base_proto)
                    {
                        Ok(value) => value,
                        Err(Error::ThrownValue(error)) => {
                            return_value = Err(Error::ThrownValue(error));
                            Value::Undefined
                        }
                        Err(_) => Value::Undefined,
                    };
                }

                let rval = match self
                    .0
                    .write(activation.context.gc_context)
                    .values
                    .entry(name, activation.is_case_sensitive())
                {
                    Entry::Occupied(mut entry) => entry.get_mut().set(value),
                    Entry::Vacant(entry) => {
                        entry.insert(Property::Stored {
                            value,
                            attributes: Attribute::empty(),
                        });

                        None
                    }
                };

                if let Some(rval) = rval {
                    if let Some(exec) = rval.as_executable() {
                        let _ = exec.exec(
                            "[Setter]",
                            activation,
                            this,
                            base_proto,
                            &[value],
                            ExecutionReason::Special,
                            rval,
                        );
                    }
                }

                return return_value;
            }
        }

        Ok(())
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
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if name == "__proto__" {
            return Ok(self.proto().map_or(Value::Undefined, Value::Object));
        }

        let mut getter = None;

        if let Some(value) = self
            .0
            .read()
            .values
            .get(name, activation.is_case_sensitive())
        {
            match value {
                Property::Virtual { get, .. } => getter = Some(get.to_owned()),
                Property::Stored { value, .. } => return Ok(value.to_owned()),
            }
        }

        if let Some(getter) = getter {
            if let Some(exec) = getter.as_executable() {
                // Errors, even fatal ones, are completely and silently ignored here.
                match exec.exec(
                    "[Getter]",
                    activation,
                    this,
                    Some((*self).into()),
                    &[],
                    ExecutionReason::Special,
                    getter,
                ) {
                    Ok(value) => Ok(value),
                    Err(_) => Ok(Value::Undefined),
                }
            } else {
                Ok(Value::Undefined)
            }
        } else {
            Ok(Value::Undefined)
        }
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
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        self.internal_set(
            name,
            value,
            activation,
            (*self).into(),
            Some((*self).into()),
        )
    }

    /// Call the underlying object.
    ///
    /// This function takes a redundant `this` parameter which should be
    /// the object's own `GcCell`, so that it can pass it to user-defined
    /// overrides that may need to interact with the underlying object.
    fn call(
        &self,
        _name: &str,
        _activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
        _base_proto: Option<Object<'gc>>,
        _args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Undefined)
    }

    fn call_setter(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Option<Object<'gc>> {
        match self
            .0
            .write(activation.context.gc_context)
            .values
            .get_mut(name, activation.is_case_sensitive())
        {
            Some(propref) if propref.is_virtual() => propref.set(value),
            _ => None,
        }
    }

    #[allow(clippy::new_ret_no_self)]
    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        match self.0.read().array {
            ArrayStorage::Vector(_) => {
                Ok(ScriptObject::array(activation.context.gc_context, Some(this)).into())
            }
            ArrayStorage::Properties { .. } => {
                Ok(ScriptObject::object(activation.context.gc_context, Some(this)).into())
            }
        }
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        let mut object = self.0.write(activation.context.gc_context);
        if let Some(prop) = object.values.get(name, activation.is_case_sensitive()) {
            if prop.can_delete() {
                object.values.remove(name, activation.is_case_sensitive());
                return true;
            }
        }

        false
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.0.write(gc_context).values.insert(
            name,
            Property::Virtual {
                get,
                set,
                attributes,
            },
            false,
        );
    }

    fn add_property_with_case(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Object<'gc>,
        set: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        self.0.write(gc_context).values.insert(
            name,
            Property::Virtual {
                get,
                set,
                attributes,
            },
            activation.is_case_sensitive(),
        );
    }

    fn set_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        self.0.write(gc_context).watchers.insert(
            &name,
            Watcher::new(callback, user_data),
            activation.is_case_sensitive(),
        );
    }

    fn remove_watcher(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        gc_context: MutationContext<'gc, '_>,
        name: Cow<str>,
    ) -> bool {
        let old = self
            .0
            .write(gc_context)
            .watchers
            .remove(name.as_ref(), activation.is_case_sensitive());
        old.is_some()
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        self.0
            .write(gc_context)
            .values
            .insert(name, Property::Stored { value, attributes }, false);
    }

    fn set_attributes(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: Option<&str>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        match name {
            None => {
                // Change *all* attributes.
                for (_name, prop) in self.0.write(gc_context).values.iter_mut() {
                    let new_atts = (prop.attributes() - clear_attributes) | set_attributes;
                    prop.set_attributes(new_atts);
                }
            }
            Some(name) => {
                if let Some(prop) = self.0.write(gc_context).values.get_mut(name, false) {
                    let new_atts = (prop.attributes() - clear_attributes) | set_attributes;
                    prop.set_attributes(new_atts);
                }
            }
        }
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().prototype
    }

    fn set_proto(&self, gc_context: MutationContext<'gc, '_>, prototype: Option<Object<'gc>>) {
        self.0.write(gc_context).prototype = prototype;
    }

    /// Checks if the object has a given named property.
    fn has_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        self.has_own_property(activation, name)
            || self
                .proto()
                .as_ref()
                .map_or(false, |p| p.has_property(activation, name))
    }

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    fn has_own_property(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        if name == "__proto__" {
            return true;
        }
        self.0
            .read()
            .values
            .contains_key(name, activation.is_case_sensitive())
    }

    fn has_own_virtual(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        if let Some(slot) = self
            .0
            .read()
            .values
            .get(name, activation.is_case_sensitive())
        {
            slot.is_virtual()
        } else {
            false
        }
    }

    /// Checks if a named property appears when enumerating the object.
    fn is_property_enumerable(&self, activation: &mut Activation<'_, 'gc, '_>, name: &str) -> bool {
        if let Some(prop) = self
            .0
            .read()
            .values
            .get(name, activation.is_case_sensitive())
        {
            prop.is_enumerable()
        } else {
            false
        }
    }

    /// Enumerate the object.
    fn get_keys(&self, activation: &mut Activation<'_, 'gc, '_>) -> Vec<String> {
        let proto_keys = self
            .proto()
            .map_or_else(Vec::new, |p| p.get_keys(activation));
        let mut out_keys = vec![];
        let object = self.0.read();

        // Prototype keys come first.
        out_keys.extend(proto_keys.into_iter().filter(|k| {
            !object
                .values
                .contains_key(k, activation.is_case_sensitive())
        }));

        // Then our own keys.
        out_keys.extend(self.0.read().values.iter().filter_map(move |(k, p)| {
            if p.is_enumerable() {
                Some(k.to_string())
            } else {
                None
            }
        }));

        out_keys
    }

    fn as_string(&self) -> Cow<str> {
        Cow::Borrowed("[object Object]")
    }

    fn type_of(&self) -> &'static str {
        self.0.read().type_of
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().interfaces.clone()
    }

    fn set_interfaces(&self, context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0.write(context).interfaces = iface_list;
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn length(&self) -> usize {
        match &self.0.read().array {
            ArrayStorage::Vector(vector) => vector.len(),
            ArrayStorage::Properties { length } => *length,
        }
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, new_length: usize) {
        let mut to_remove = None;

        match &mut self.0.write(gc_context).array {
            ArrayStorage::Vector(vector) => {
                let old_length = vector.len();
                vector.resize(new_length, Value::Undefined);
                if new_length < old_length {
                    to_remove = Some(new_length..old_length);
                }
            }
            ArrayStorage::Properties { length } => {
                *length = new_length;
            }
        }
        if let Some(to_remove) = to_remove {
            for i in to_remove {
                self.sync_native_property(&i.to_string(), gc_context, None, true);
            }
        }
        self.sync_native_property("length", gc_context, Some(new_length.into()), false);
    }

    fn array(&self) -> Vec<Value<'gc>> {
        match &self.0.read().array {
            ArrayStorage::Vector(vector) => vector.to_owned(),
            ArrayStorage::Properties { length } => {
                let mut values = Vec::new();
                for i in 0..*length {
                    values.push(self.array_element(i));
                }
                values
            }
        }
    }

    fn array_element(&self, index: usize) -> Value<'gc> {
        match &self.0.read().array {
            ArrayStorage::Vector(vector) => {
                if let Some(value) = vector.get(index) {
                    value.to_owned()
                } else {
                    Value::Undefined
                }
            }
            ArrayStorage::Properties { length } => {
                if index < *length {
                    if let Some(Property::Stored { value, .. }) =
                        self.0.read().values.get(&index.to_string(), false)
                    {
                        return value.to_owned();
                    }
                }
                Value::Undefined
            }
        }
    }

    fn set_array_element(
        &self,
        index: usize,
        value: Value<'gc>,
        gc_context: MutationContext<'gc, '_>,
    ) -> usize {
        self.sync_native_property(&index.to_string(), gc_context, Some(value), true);
        let mut adjust_length = false;
        let length = match &mut self.0.write(gc_context).array {
            ArrayStorage::Vector(vector) => {
                if index >= vector.len() {
                    vector.resize(index + 1, Value::Undefined);
                }
                vector[index] = value;
                adjust_length = true;
                vector.len()
            }
            ArrayStorage::Properties { length } => *length,
        };
        if adjust_length {
            self.sync_native_property("length", gc_context, Some(length.into()), false);
        }
        length
    }

    fn delete_array_element(&self, index: usize, gc_context: MutationContext<'gc, '_>) {
        if let ArrayStorage::Vector(vector) = &mut self.0.write(gc_context).array {
            if index < vector.len() {
                vector[index] = Value::Undefined;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::avm1::activation::ActivationIdentifier;
    use crate::avm1::function::Executable;
    use crate::avm1::globals::system::SystemProperties;
    use crate::avm1::property::Attribute;
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
    use crate::display_object::MovieClip;
    use crate::focus_tracker::FocusTracker;
    use crate::library::Library;
    use crate::loader::LoadManager;
    use crate::prelude::*;
    use crate::tag_utils::{SwfMovie, SwfSlice};
    use crate::vminterface::Instantiator;
    use gc_arena::rootless_arena;
    use instant::Instant;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::collections::{BTreeMap, HashMap};
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
            let root: DisplayObject<'_> =
                MovieClip::new(SwfSlice::empty(swf.clone()), gc_context).into();
            root.set_depth(gc_context, 0);
            let mut levels = BTreeMap::new();
            levels.insert(0, root);

            let object = ScriptObject::object(gc_context, Some(avm1.prototypes().object)).into();
            let globals = avm1.global_object_cell();

            let mut context = UpdateContext {
                gc_context,
                player_version: 32,
                swf: &swf,
                levels: &mut levels,
                rng: &mut SmallRng::from_seed([0u8; 32]),
                action_queue: &mut crate::context::ActionQueue::new(),
                audio: &mut NullAudioBackend::new(),
                audio_manager: &mut AudioManager::new(),
                ui: &mut NullUiBackend::new(),
                background_color: &mut None,
                library: &mut Library::empty(gc_context),
                navigator: &mut NullNavigatorBackend::new(),
                renderer: &mut NullRenderer::new(),
                locale: &mut NullLocaleBackend::new(),
                log: &mut NullLogBackend::new(),
                video: &mut NullVideoBackend::new(),
                mouse_hovered_object: None,
                mouse_position: &(Twips::new(0), Twips::new(0)),
                drag_object: &mut None,
                stage_size: (Twips::from_pixels(550.0), Twips::from_pixels(400.0)),
                player: None,
                load_manager: &mut LoadManager::new(),
                system: &mut SystemProperties::default(),
                instance_counter: &mut 0,
                storage: &mut MemoryStorageBackend::default(),
                shared_objects: &mut HashMap::new(),
                unbound_text_fields: &mut Vec::new(),
                timers: &mut Timers::new(),
                needs_render: &mut false,
                avm1: &mut avm1,
                avm2: &mut avm2,
                external_interface: &mut Default::default(),
                update_start: Instant::now(),
                max_execution_duration: Duration::from_secs(15),
                focus_tracker: FocusTracker::new(gc_context),
                times_get_time_called: 0,
                time_offset: &mut 0,
            };

            root.post_instantiation(&mut context, root, None, Instantiator::Movie, false);
            root.set_name(context.gc_context, "");

            let base_clip = *context.levels.get(&0).unwrap();
            let swf_version = context.swf.version();
            let mut activation = Activation::from_nothing(
                context,
                ActivationIdentifier::root("[Test]"),
                swf_version,
                globals,
                base_clip,
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

            assert_eq!(object.delete(activation, "test"), false);
            assert_eq!(object.get("test", activation).unwrap(), "initial".into());

            object
                .as_script_object()
                .unwrap()
                .set("test", "replaced".into(), activation)
                .unwrap();

            assert_eq!(object.delete(activation, "test"), false);
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
                "test",
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
                "virtual",
                getter,
                None,
                Attribute::empty(),
            );
            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "virtual_un",
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

            assert_eq!(object.delete(activation, "virtual"), true);
            assert_eq!(object.delete(activation, "virtual_un"), false);
            assert_eq!(object.delete(activation, "stored"), true);
            assert_eq!(object.delete(activation, "stored_un"), false);
            assert_eq!(object.delete(activation, "non_existent"), false);

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
    fn test_iter_values() {
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
                "virtual",
                getter,
                None,
                Attribute::empty(),
            );
            object.as_script_object().unwrap().add_property(
                activation.context.gc_context,
                "virtual_hidden",
                getter,
                None,
                Attribute::DONT_ENUM,
            );

            let keys: Vec<_> = object.get_keys(activation);
            assert_eq!(keys.len(), 2);
            assert_eq!(keys.contains(&"stored".to_string()), true);
            assert_eq!(keys.contains(&"stored_hidden".to_string()), false);
            assert_eq!(keys.contains(&"virtual".to_string()), true);
            assert_eq!(keys.contains(&"virtual_hidden".to_string()), false);
        })
    }
}
