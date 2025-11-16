use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{ExecutionName, ExecutionReason, FunctionObject};
use crate::avm1::object::{stage_object, NativeObject};
use crate::avm1::property::{Attribute, Property};
use crate::avm1::property_map::{Entry, PropertyMap};
use crate::avm1::{ObjectPtr, Value};
use crate::context::UpdateContext;
use crate::display_object::{DisplayObject, TDisplayObject as _};
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::{AvmString, StringContext};
use core::{fmt, mem};
use gc_arena::lock::RefLock;
use gc_arena::{Collect, DynamicRoot, Gc, GcWeak, Mutation, Rootable};
use ruffle_macros::istr;

use super::super_object::SuperObject;

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Watcher<'gc> {
    callback: Object<'gc>,
    user_data: Value<'gc>,
}

impl<'gc> Watcher<'gc> {
    fn new(callback: Object<'gc>, user_data: Value<'gc>) -> Self {
        Self {
            callback,
            user_data,
        }
    }

    fn call(
        &self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        old_value: Value<'gc>,
        new_value: Value<'gc>,
        this: Object<'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let args = [Value::String(name), old_value, new_value, self.user_data];
        let exec = self.callback.as_function().unwrap();
        exec.exec(
            ExecutionName::Dynamic(name),
            activation,
            this.into(),
            0,
            &args,
            ExecutionReason::Special,
            self.callback,
        )
    }
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct Object<'gc>(Gc<'gc, RefLock<ObjectData<'gc>>>);

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct ObjectWeak<'gc>(GcWeak<'gc, RefLock<ObjectData<'gc>>>);

impl<'gc> ObjectWeak<'gc> {
    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<Object<'gc>> {
        self.0.upgrade(mc).map(Object)
    }
}

impl fmt::Debug for ObjectWeak<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectWeak")
            .field("ptr", &GcWeak::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone)]
#[expect(clippy::type_complexity)]
pub struct ObjectHandle(DynamicRoot<Rootable![RefLock<ObjectData<'_>>]>);

impl ObjectHandle {
    pub fn stash<'gc>(context: &UpdateContext<'gc>, this: Object<'gc>) -> Self {
        Self(context.dynamic_root.stash(context.gc(), this.0))
    }

    pub fn fetch<'gc>(&self, context: &UpdateContext<'gc>) -> Object<'gc> {
        Object(context.dynamic_root.fetch(&self.0))
    }
}

#[derive(Collect)]
#[collect(no_drop)]
struct ObjectData<'gc> {
    native: NativeObject<'gc>,
    properties: PropertyMap<'gc, Property<'gc>>,
    interfaces: Option<Vec<Object<'gc>>>,
    watchers: PropertyMap<'gc, Watcher<'gc>>,
}

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

impl<'gc> Object<'gc> {
    pub fn as_weak(self) -> ObjectWeak<'gc> {
        ObjectWeak(Gc::downgrade(self.0))
    }

    pub fn new(context: &StringContext<'gc>, proto: Option<Object<'gc>>) -> Self {
        let object = Self(Gc::new(
            context.gc(),
            RefLock::new(ObjectData {
                native: NativeObject::None,
                properties: PropertyMap::new(),
                interfaces: None,
                watchers: PropertyMap::new(),
            }),
        ));
        if let Some(proto) = proto {
            object.define_value(
                context.gc(),
                istr!(context, "__proto__"),
                proto.into(),
                Attribute::DONT_ENUM | Attribute::DONT_DELETE,
            );
        }
        object
    }

    pub fn new_with_native(
        context: &StringContext<'gc>,
        proto: Option<Object<'gc>>,
        native: NativeObject<'gc>,
    ) -> Self {
        let obj = Self::new(context, proto);
        obj.set_native(context.gc(), native);
        obj
    }

    // Creates a Object, without assigning any __proto__ property.
    pub fn new_without_proto(gc_context: &Mutation<'gc>) -> Self {
        Self(Gc::new(
            gc_context,
            RefLock::new(ObjectData {
                native: NativeObject::None,
                properties: PropertyMap::new(),
                interfaces: None,
                watchers: PropertyMap::new(),
            }),
        ))
    }

    /// Gets the value of a data property on this object.
    ///
    /// Doesn't look up the prototype chain and ignores virtual properties, thus cannot cause
    /// any side-effects.
    pub(super) fn get_data(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Value<'gc> {
        self.0
            .borrow()
            .properties
            .get(name, activation.is_case_sensitive())
            .map_or(Value::Undefined, |property| property.data())
    }

    /// Sets a data property on this object.
    ///
    /// Doesn't look up the prototype chain and ignores virtual properties, but still might
    /// call to watchers.
    pub fn set_data(
        self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        // TODO: Call watchers.
        match self
            .0
            .borrow_mut(activation.gc())
            .properties
            .entry(name, activation.is_case_sensitive())
        {
            Entry::Occupied(mut entry) => entry.get_mut().set_data(value),
            Entry::Vacant(entry) => entry.insert(Property::new_stored(value, Attribute::empty())),
        }
        Ok(())
    }

    // TODO: Make an iterator?
    pub fn own_properties(self) -> Vec<(AvmString<'gc>, Value<'gc>)> {
        self.0
            .borrow()
            .properties
            .iter()
            .filter_map(|(k, p)| {
                if p.is_enumerable() {
                    Some((k, p.data()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Retrieve a named, non-virtual property from this object exclusively.
    ///
    /// This function should not inspect prototype chains. Instead, use
    /// `get_stored` to do ordinary property look-up and resolution.
    pub fn get_local_stored(
        self,
        name: impl Into<AvmString<'gc>>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Value<'gc>> {
        // TODO(moulins): can this special case be removed? (as `super` never has properties)
        if self.as_super_object().is_some() {
            return None;
        }

        let name = name.into();
        let read = self.0.borrow();

        read.properties
            .get(name, activation.is_case_sensitive())
            .filter(|property| property.allow_swf_version(activation.swf_version()))
            .map(|property| property.data())
            .or_else(|| {
                read.native
                    .as_display_object()
                    .and_then(|dobj| stage_object::get_property(dobj, name, activation))
            })
    }

    /// Set a named property on the object.
    pub(super) fn set_local(
        self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        match self.native_no_super() {
            // TODO: `super` cannot have properties set on it
            // TODO: What happens if you set `super.__proto__`?
            NativeObject::Super(_) => return Ok(()),
            NativeObject::Array(_) => {
                if name == istr!("length") {
                    let new_length = value.coerce_to_i32(activation)?;
                    let old_length = self.get_data(istr!("length"), activation);
                    if let Value::Number(old_length) = old_length {
                        for i in new_length.max(0)..f64_to_wrapping_i32(old_length) {
                            self.delete_element(activation, i);
                        }
                    }
                } else if let Some(index) = parse_array_index(name) {
                    let length = self.length(activation)?;
                    if index >= length {
                        self.set_length(activation, index.wrapping_add(1))?;
                    }
                }
            }
            native => {
                if let Some(dobj) = native.as_display_object() {
                    stage_object::notify_property_change(dobj, name, value, activation)?;
                    // 'magic' display object properties (such as _x, _y, etc) take
                    // priority over properties in prototypes.
                    if !self.has_own_property(activation, name) {
                        if let Some(property) = activation
                            .context
                            .avm1
                            .display_properties()
                            .get_by_name(name)
                        {
                            return property.set(activation, dobj, value);
                        }
                    }
                }
            }
        }

        let setter = match self
            .0
            .borrow_mut(activation.gc())
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
            if let Some(exec) = setter.as_function() {
                if let Err(Error::ThrownValue(e)) = exec.exec(
                    ExecutionName::Static("[Setter]"),
                    activation,
                    this.into(),
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
    /// This function takes a  `this` parameter which generally
    /// refers to the object which has this property, although
    /// it can be changed by `Function.apply`/`Function.call`.
    pub fn call(
        self,
        name: impl Into<ExecutionName<'gc>>,
        activation: &mut Activation<'_, 'gc>,
        this: Value<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self.native_no_super() {
            NativeObject::Super(zuper) => zuper.call(name, activation, args),
            NativeObject::Function(func) => func.call(name, activation, self, this, args),
            _ => Ok(Value::Undefined),
        }
    }

    /// Takes an already existing object and performs this constructor (if valid) on it.
    pub fn construct_on_existing(
        self,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<(), Error<'gc>> {
        // `super` cannot be called as a constructor with `new`.
        if let NativeObject::Function(func) = self.native_no_super() {
            func.construct_on_existing(activation, self, this, args)
        } else {
            Ok(())
        }
    }

    /// Construct the underlying object, if this is a valid constructor, and returns the result.
    /// Calling this on something other than a constructor will return a new Undefined object.
    pub fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        // `super` cannot be called as a constructor with `new`.
        if let NativeObject::Function(func) = self.native_no_super() {
            func.construct(activation, self, args)
        } else {
            Ok(Value::Undefined)
        }
    }

    /// Retrieve a getter defined on this object.
    pub(super) fn getter(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Object<'gc>> {
        // TODO(moulins): is this special case necessary?
        if let Some(zuper) = self.as_super_object() {
            return zuper.this().getter(name, activation);
        }

        self.0
            .borrow()
            .properties
            .get(name, activation.is_case_sensitive())
            .filter(|property| property.allow_swf_version(activation.swf_version()))
            .and_then(|property| property.getter())
    }

    /// Retrieve a setter defined on this object.
    pub(super) fn setter(
        self,
        name: AvmString<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Option<Object<'gc>> {
        // TODO(moulins): is this special case necessary?
        if let Some(zuper) = self.as_super_object() {
            return zuper.this().setter(name, activation);
        }

        self.0
            .borrow()
            .properties
            .get(name, activation.is_case_sensitive())
            .filter(|property| property.allow_swf_version(activation.swf_version()))
            .and_then(|property| property.setter())
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    pub fn delete(self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        // TODO(moulins): can this special case be removed (`super` never has properties to delete)
        if self.as_super_object().is_some() {
            // `super` cannot have properties deleted from it
            return false;
        }

        if let Entry::Occupied(mut entry) = self
            .0
            .borrow_mut(activation.gc())
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

    /// Define a virtual property onto a given object.
    ///
    /// A virtual property is a set of get/set functions that are called when a
    /// given named property is retrieved or stored on an object. These
    /// functions are then responsible for providing or accepting the value
    /// that is given to or taken from the AVM.
    ///
    /// It is not guaranteed that all objects accept virtual properties,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    pub fn add_property(
        self,
        gc_context: &Mutation<'gc>,
        name: AvmString<'gc>,
        getter: Object<'gc>,
        setter: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
        if self.as_super_object().is_some() {
            return;
        }

        match self.0.borrow_mut(gc_context).properties.entry(name, false) {
            Entry::Occupied(mut entry) => entry.get_mut().set_virtual(getter, setter),
            Entry::Vacant(entry) => entry.insert(Property::new_virtual(getter, setter, attributes)),
        }
    }

    /// Define a virtual property onto a given object.
    ///
    /// A virtual property is a set of get/set functions that are called when a
    /// given named property is retrieved or stored on an object. These
    /// functions are then responsible for providing or accepting the value
    /// that is given to or taken from the AVM.
    ///
    /// It is not guaranteed that all objects accept virtual properties,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    pub fn add_property_with_case(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        getter: Object<'gc>,
        setter: Option<Object<'gc>>,
        attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
        if self.as_super_object().is_some() {
            return;
        }

        match self
            .0
            .borrow_mut(activation.gc())
            .properties
            .entry(name, activation.is_case_sensitive())
        {
            Entry::Occupied(mut entry) => entry.get_mut().set_virtual(getter, setter),
            Entry::Vacant(entry) => entry.insert(Property::new_virtual(getter, setter, attributes)),
        }
    }

    /// Calls the 'watcher' of a given property, if it exists.
    pub(super) fn call_watcher(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        value: &mut Value<'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        // TODO(moulins): is this special case necessary?
        if let Some(zuper) = self.as_super_object() {
            return zuper.this().call_watcher(activation, name, value, this);
        }

        let mut result = Ok(());
        let watcher = self
            .0
            .borrow()
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

    /// Set the 'watcher' of a given property.
    ///
    /// The property does not need to exist at the time of this being called.
    pub fn watch(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
        callback: Object<'gc>,
        user_data: Value<'gc>,
    ) {
        //`super` cannot have properties defined on it
        if self.as_super_object().is_some() {
            return;
        }

        self.0.borrow_mut(activation.gc()).watchers.insert(
            name,
            Watcher::new(callback, user_data),
            activation.is_case_sensitive(),
        );
    }

    /// Removed any assigned 'watcher' from the given property.
    ///
    /// The return value will indicate if there was a watcher present before this method was
    /// called.
    pub fn unwatch(self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        //`super` cannot have properties defined on it
        // TODO(moulins): can this special case be removed? `super` can never have watched properties.
        if self.as_super_object().is_some() {
            return false;
        }

        self.0
            .borrow_mut(activation.gc())
            .watchers
            .remove(name, activation.is_case_sensitive())
            .is_some()
    }

    /// Define a value on an object.
    ///
    /// Unlike setting a value, this function is intended to replace any
    /// existing virtual or built-in properties already installed on a given
    /// object. As such, this should not run any setters; the resulting name
    /// slot should either be completely replaced with the value or completely
    /// untouched.
    ///
    /// It is not guaranteed that all objects accept value definitions,
    /// especially if a property name conflicts with a built-in property, such
    /// as `__proto__`.
    pub fn define_value(
        self,
        gc_context: &Mutation<'gc>,
        name: impl Into<AvmString<'gc>>,
        value: Value<'gc>,
        attributes: Attribute,
    ) {
        //`super` cannot have properties defined on it
        if self.as_super_object().is_some() {
            return;
        }

        self.0.borrow_mut(gc_context).properties.insert(
            name.into(),
            Property::new_stored(value, attributes),
            true,
        );
    }

    /// Set the attributes of a given property.
    ///
    /// Leaving `name` unspecified allows setting all properties on a given
    /// object to the same set of properties.
    ///
    /// Attributes can be set, cleared, or left as-is using the pairs of `set_`
    /// and `clear_attributes` parameters.
    pub fn set_attributes(
        self,
        gc_context: &Mutation<'gc>,
        name: Option<AvmString<'gc>>,
        set_attributes: Attribute,
        clear_attributes: Attribute,
    ) {
        // TODO(moulins): try to remove this special case? (`super` never has any properties)
        if self.as_super_object().is_some() {
            //TODO: Does ASSetPropFlags work on `super`? What would it even work on?
            return;
        }

        match name {
            None => {
                // Change *all* attributes.
                for (_name, prop) in self.0.borrow_mut(gc_context).properties.iter_mut() {
                    let new_atts = (prop.attributes() - clear_attributes) | set_attributes;
                    prop.set_attributes(new_atts);
                }
            }
            Some(name) => {
                if let Some(prop) = self
                    .0
                    .borrow_mut(gc_context)
                    .properties
                    .get_mut(name, false)
                {
                    let new_atts = (prop.attributes() - clear_attributes) | set_attributes;
                    prop.set_attributes(new_atts);
                }
            }
        }
    }

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    pub(super) fn proto(self, activation: &mut Activation<'_, 'gc>) -> Value<'gc> {
        if let Some(zuper) = self.as_super_object() {
            return zuper.proto(activation);
        }

        self.get_data(istr!("__proto__"), activation)
    }

    /// Checks if the object has a given named property.
    pub fn has_property(self, activation: &mut Activation<'_, 'gc>, name: AvmString<'gc>) -> bool {
        let dobj = match self.native_no_super() {
            // `super` forwards property membership tests to its underlying object.
            NativeObject::Super(zuper) => {
                return zuper.this().has_property(activation, name);
            }
            native => native.as_display_object(),
        };

        // Normal property checks
        if dobj.is_none_or(|o| !o.avm1_removed()) {
            if self.has_own_property(activation, name) {
                return true;
            } else if let Value::Object(proto) = self.proto(activation) {
                if proto.has_property(activation, name) {
                    return true;
                }
            }
        }

        // Fallback: display object properties
        dobj.is_some_and(|o| stage_object::has_display_object_property(o, activation, name))
    }

    /// Checks if the object has a given named property on itself (and not,
    /// say, the object's prototype or superclass)
    pub fn has_own_property(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
    ) -> bool {
        if let Some(zuper) = self.as_super_object() {
            // `super` forwards property membership tests to its underlying object,
            // even though it can't be enumerated.
            return zuper.this().has_own_property(activation, name);
        }

        // Note that `hasOwnProperty` does NOT return true for display object properties.
        self.0
            .borrow()
            .properties
            .contains_key(name, activation.is_case_sensitive())
    }

    /// Checks if the object has a given named property on itself that is
    /// virtual.
    pub(super) fn has_own_virtual(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
    ) -> bool {
        if let Some(zuper) = self.as_super_object() {
            // `super` forwards property membership tests to its underlying object,
            // even though it can't be enumerated.
            return zuper.this().has_own_virtual(activation, name);
        }

        self.0
            .borrow()
            .properties
            .get(name, activation.is_case_sensitive())
            .is_some_and(|property| {
                property.is_virtual() && property.allow_swf_version(activation.swf_version())
            })
    }

    /// Checks if a named property appears when enumerating the object.
    pub fn is_property_enumerable(
        self,
        activation: &mut Activation<'_, 'gc>,
        name: AvmString<'gc>,
    ) -> bool {
        if let Some(zuper) = self.as_super_object() {
            // `super` forwards property membership tests to its underlying object,
            // even though it can't be enumerated.
            return zuper.this().is_property_enumerable(activation, name);
        }

        self.0
            .borrow()
            .properties
            .get(name, activation.is_case_sensitive())
            .is_some_and(|property| property.is_enumerable())
    }

    /// Enumerate the object.
    pub fn get_keys(
        self,
        activation: &mut Activation<'_, 'gc>,
        include_hidden: bool,
    ) -> Vec<AvmString<'gc>> {
        // TODO(moulins): can this special case be removed?
        if self.as_super_object().is_some() {
            return vec![];
        }

        let proto_keys = if let Value::Object(proto) = self.proto(activation) {
            proto.get_keys(activation, include_hidden)
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
        let read = self.0.borrow();
        out_keys.extend(read.properties.iter().filter_map(move |(k, p)| {
            if include_hidden || p.is_enumerable() {
                Some(k)
            } else {
                None
            }
        }));

        // Then display object keys.
        if let Some(dobj) = read.native.as_display_object() {
            stage_object::enumerate_keys(dobj, &mut out_keys);
        }

        out_keys
    }

    /// Enumerate all interfaces implemented by this object.
    pub(super) fn interfaces(self) -> Vec<Object<'gc>> {
        if self.as_super_object().is_some() {
            // `super` does not implement interfaces
            return vec![];
        }

        self.0.borrow().interfaces.clone().unwrap_or_default()
    }

    /// Set the interface list for this object. (Only useful for prototypes.)
    /// Calling this a second time will have no effect.
    pub fn set_interfaces(self, gc_context: &Mutation<'gc>, iface_list: Vec<Object<'gc>>) {
        if self.as_super_object().is_some() {
            // `super` probably cannot have interfaces set on it
            return;
        }

        let mut write = self.0.borrow_mut(gc_context);
        write.interfaces.get_or_insert(iface_list);
    }

    pub(super) fn native_no_super(self) -> NativeObject<'gc> {
        self.0.borrow().native
    }

    pub fn native(self) -> NativeObject<'gc> {
        match self.0.borrow().native {
            // TODO(moulins): can `super` point to another `super`?
            NativeObject::Super(zuper) => zuper.this().native(),
            native => native,
        }
    }

    pub fn set_native(self, mc: &Mutation<'gc>, native: NativeObject<'gc>) {
        assert!(!matches!(native, NativeObject::None));

        let old_native = self.0.borrow().native;
        match old_native {
            // TODO(moulins): can `super` point to another `super`?
            NativeObject::Super(zuper) => zuper.this().set_native(mc, native),
            NativeObject::None => self.0.borrow_mut(mc).native = native,
            // Trying to construct the same object twice (e.g. with `super()`) does nothing.
            _ if mem::discriminant(&old_native) == mem::discriminant(&native) => (),
            // FIXME: in FP, some native classes (e.g. Sound) appear to be constructible over
            // display objects; we don't support this properly.
            // See the `avm1/register_class_with_sound` test for an example.
            _ => tracing::warn!(
                "AVM1: tried to call set_native on an-already constructed native Object"
            ),
        }
    }

    /// Get the underlying super object, if it exists.
    pub(super) fn as_super_object(self) -> Option<SuperObject<'gc>> {
        if let NativeObject::Super(zuper) = self.native_no_super() {
            Some(zuper)
        } else {
            None
        }
    }

    /// Get the underlying function for this object, if it exists.
    pub fn as_function(self) -> Option<FunctionObject<'gc>> {
        // Even though `super` calls the class constructor, it doesn't count as a function.
        if let NativeObject::Function(func) = self.native_no_super() {
            Some(*func)
        } else {
            None
        }
    }

    /// Get the underlying stage object, if it exists, but doesn't follow `super` objects.
    pub fn as_display_object_no_super(self) -> Option<DisplayObject<'gc>> {
        self.0.borrow().native.as_display_object()
    }

    /// Get the underlying display node for this object, if it exists.
    pub fn as_display_object(self) -> Option<DisplayObject<'gc>> {
        //`super` actually can be used to invoke MovieClip methods
        self.native().as_display_object()
    }

    pub fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    /// Gets the length of this object, as if it were an array.
    pub fn length(self, activation: &mut Activation<'_, 'gc>) -> Result<i32, Error<'gc>> {
        // TODO(moulins): can this special case be removed? `super` should never have a length property
        if self.as_super_object().is_some() {
            return Ok(0);
        }

        self.get_data(istr!("length"), activation)
            .coerce_to_i32(activation)
    }

    /// Sets the length of this object, as if it were an array.
    pub fn set_length(
        self,
        activation: &mut Activation<'_, 'gc>,
        new_length: i32,
    ) -> Result<(), Error<'gc>> {
        // `super` cannot have properties set on it
        if self.as_super_object().is_some() {
            return Ok(());
        }

        if let NativeObject::Array(_) = self.native() {
            let old_length = self.get_data(istr!("length"), activation);
            if let Value::Number(old_length) = old_length {
                for i in new_length.max(0)..f64_to_wrapping_i32(old_length) {
                    self.delete_element(activation, i);
                }
            }
        }

        self.set_data(istr!("length"), new_length.into(), activation)
    }

    /// Checks if this object has an element, as if it were an array.
    pub fn has_element(self, activation: &mut Activation<'_, 'gc>, index: i32) -> bool {
        // TODO(moulins): can this special case be removed? (as `super` never has elements)
        if self.as_super_object().is_some() {
            return false;
        }

        let index_str = AvmString::new_utf8(activation.gc(), index.to_string());
        self.has_own_property(activation, index_str)
    }

    /// Gets a property of this object, as if it were an array.
    pub fn get_element(self, activation: &mut Activation<'_, 'gc>, index: i32) -> Value<'gc> {
        // TODO(moulins): can this special case be removed? (as `super` never has elements to delete)
        if self.as_super_object().is_some() {
            return Value::Undefined;
        }

        let index_str = AvmString::new_utf8(activation.gc(), index.to_string());
        self.get_data(index_str, activation)
    }

    /// Sets a property of this object, as if it were an array.
    pub fn set_element(
        self,
        activation: &mut Activation<'_, 'gc>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        match self.native_no_super() {
            // `super` cannot have properties set on it
            NativeObject::Super(_) => return Ok(()),
            NativeObject::Array(_) => {
                let length = self.length(activation)?;
                if index >= length {
                    self.set_length(activation, index.wrapping_add(1))?;
                }
            }
            _ => (),
        }

        let index_str = AvmString::new_utf8(activation.gc(), index.to_string());
        self.set_data(index_str, value, activation)
    }

    /// Deletes a property of this object as if it were an array.
    pub fn delete_element(self, activation: &mut Activation<'_, 'gc>, index: i32) -> bool {
        // TODO(moulins): can this special case be removed? (as `super` never has elements to delete)
        if self.as_super_object().is_some() {
            return false;
        }

        let index_str = AvmString::new_utf8(activation.gc(), index.to_string());
        self.delete(activation, index_str)
    }
}

fn parse_array_index(name: AvmString<'_>) -> Option<i32> {
    let name = name.trim_start_matches(|c| match u8::try_from(c) {
        Ok(c) => c.is_ascii_whitespace(),
        Err(_) => false,
    });

    name.parse::<std::num::Wrapping<i32>>().ok().map(|i| i.0)
}
