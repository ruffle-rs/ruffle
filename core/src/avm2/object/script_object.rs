//! Default AVM2 object impl

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::property_map::PropertyMap;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::slot::Slot;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;

/// A class instance allocator that allocates `ScriptObject`s.
pub fn scriptobject_allocator<'gc>(
    class: ClassObject<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), Some(class));

    Ok(ScriptObject(GcCell::allocate(activation.context.gc_context, base)).into())
}

/// Default implementation of `avm2::Object`.
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

/// Base data common to all `TObject` implementations.
///
/// Host implementations of `TObject` should embed `ScriptObjectData` and
/// forward any trait method implementations it does not overwrite to this
/// struct.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ScriptObjectData<'gc> {
    /// Properties stored on this object.
    values: PropertyMap<'gc, Property<'gc>>,

    /// Slots stored on this object.
    slots: Vec<Slot<'gc>>,

    /// Methods stored on this object.
    methods: Vec<Option<Object<'gc>>>,

    /// Implicit prototype of this script object.
    proto: Option<Object<'gc>>,

    /// The class object that this is an instance of.
    /// If `None`, this is either a class itself, or not an ES4 object at all.
    instance_of: Option<ClassObject<'gc>>,

    /// Enumeratable property names.
    enumerants: Vec<QName<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        self.0.read()
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        self.0.write(mc)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ScriptObject(*self);
        Ok(ScriptObject::object(activation.context.gc_context, this))
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }
}

impl<'gc> ScriptObject<'gc> {
    /// Construct a bare object with no base class.
    ///
    /// This is *not* the same thing as an object literal, which actually does
    /// have a base class: `Object`.
    pub fn bare_object(mc: MutationContext<'gc, '_>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(mc, ScriptObjectData::base_new(None, None))).into()
    }

    /// Construct an object with a prototype.
    pub fn object(mc: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(Some(proto), None),
        ))
        .into()
    }

    /// Construct an instance with a class and scope stack.
    pub fn instance(
        mc: MutationContext<'gc, '_>,
        class: ClassObject<'gc>,
        proto: Object<'gc>,
    ) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(Some(proto), Some(class)),
        ))
        .into()
    }
}

impl<'gc> ScriptObjectData<'gc> {
    pub fn base_new(proto: Option<Object<'gc>>, instance_of: Option<ClassObject<'gc>>) -> Self {
        ScriptObjectData {
            values: HashMap::new(),
            slots: Vec::new(),
            methods: Vec::new(),
            proto,
            instance_of,
            enumerants: Vec::new(),
        }
    }

    pub fn get_property_local(
        &self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let prop = self.values.get(name);

        if let Some(prop) = prop {
            prop.get(
                receiver,
                // TODO: This used to also .unwrap_or(receiver),
                // but this no longer can be done as it's not a ClassObject.
                // Despite this, somehow, no tests fail.
                activation.subclass_object().or_else(|| self.instance_of()),
            )
        } else {
            Ok(Value::Undefined.into())
        }
    }

    pub fn set_property_local(
        &mut self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let class = self.instance_of();
        let slot_id = if let Some(prop) = self.values.get(name) {
            if let Some(slot_id) = prop.slot_id() {
                Some(slot_id)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(slot_id) = slot_id {
            self.set_slot(slot_id, value, activation.context.gc_context)?;
            Ok(Value::Undefined.into())
        } else if self.values.contains_key(name) {
            let prop = self.values.get_mut(name).unwrap();
            prop.set(
                receiver,
                // TODO: This used to also .unwrap_or(receiver),
                // but this no longer can be done as it's not a ClassObject.
                // Despite this, somehow, no tests fail.
                activation.subclass_object().or(class),
                value,
            )
        } else {
            //TODO: Not all classes are dynamic like this
            self.enumerants.push(name.clone());
            self.values
                .insert(name.clone(), Property::new_dynamic_property(value));

            Ok(Value::Undefined.into())
        }
    }

    pub fn init_property_local(
        &mut self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let class = self.instance_of();
        if let Some(prop) = self.values.get_mut(name) {
            if let Some(slot_id) = prop.slot_id() {
                self.init_slot(slot_id, value, activation.context.gc_context)?;
                Ok(Value::Undefined.into())
            } else {
                prop.init(
                    receiver,
                    // TODO: This used to also .unwrap_or(receiver),
                    // but this no longer can be done as it's not a ClassObject.
                    // Despite this, somehow, no tests fail.
                    activation.subclass_object().or(class),
                    value,
                )
            }
        } else {
            //TODO: Not all classes are dynamic like this
            self.values
                .insert(name.clone(), Property::new_dynamic_property(value));

            Ok(Value::Undefined.into())
        }
    }

    pub fn is_property_overwritable(&self, name: &QName<'gc>) -> bool {
        self.values
            .get(name)
            .map(|p| p.is_overwritable())
            .unwrap_or(true)
    }

    pub fn is_property_final(&self, name: &QName<'gc>) -> bool {
        self.values.get(name).map(|p| p.is_final()).unwrap_or(false)
    }

    pub fn delete_property(&mut self, name: &QName<'gc>) -> bool {
        let can_delete = if let Some(prop) = self.values.get(name) {
            prop.can_delete()
        } else {
            false
        };

        if can_delete {
            self.values.remove(name);
        }

        can_delete
    }

    pub fn get_slot(&self, id: u32) -> Result<Value<'gc>, Error> {
        self.slots
            .get(id as usize)
            .cloned()
            .ok_or_else(|| format!("Slot index {} out of bounds!", id).into())
            .map(|slot| slot.get().unwrap_or(Value::Undefined))
    }

    /// Set a slot by its index.
    pub fn set_slot(
        &mut self,
        id: u32,
        value: Value<'gc>,
        _mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(slot) = self.slots.get_mut(id as usize) {
            slot.set(value)
        } else {
            Err(format!("Slot index {} out of bounds!", id).into())
        }
    }

    /// Initialize a slot by its index.
    pub fn init_slot(
        &mut self,
        id: u32,
        value: Value<'gc>,
        _mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(slot) = self.slots.get_mut(id as usize) {
            slot.init(value)
        } else {
            Err(format!("Slot index {} out of bounds!", id).into())
        }
    }

    /// Retrieve a method from the method table.
    pub fn get_method(&self, id: u32) -> Option<Object<'gc>> {
        self.methods.get(id as usize).and_then(|v| *v)
    }

    pub fn has_trait(&self, name: &QName<'gc>) -> Result<bool, Error> {
        match self.instance_of {
            //Class instances have instance traits from any class in the base
            //class chain.
            Some(class) => {
                let mut cur_class = Some(class);

                while let Some(class) = cur_class {
                    let cur_static_class = class.inner_class_definition();
                    if cur_static_class.read().has_instance_trait(name) {
                        return Ok(true);
                    }

                    cur_class = class.superclass_object();
                }

                Ok(false)
            }

            // Bare objects, ES3 objects, and prototypes do not have traits.
            None => Ok(false),
        }
    }

    pub fn resolve_any(&self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error> {
        for (key, _value) in self.values.iter() {
            if key.local_name() == local_name {
                return Ok(Some(key.namespace().clone()));
            }
        }

        let trait_ns = self.resolve_any_trait(local_name)?;

        if trait_ns.is_none() {
            if let Some(proto) = self.proto() {
                proto.resolve_any(local_name)
            } else {
                Ok(None)
            }
        } else {
            Ok(trait_ns)
        }
    }

    pub fn resolve_any_trait(
        &self,
        local_name: AvmString<'gc>,
    ) -> Result<Option<Namespace<'gc>>, Error> {
        if let Some(proto) = self.proto {
            let proto_trait_name = proto.resolve_any_trait(local_name)?;
            if let Some(ns) = proto_trait_name {
                return Ok(Some(ns));
            }
        }

        match &self.instance_of {
            Some(class) => {
                let mut cur_class = Some(*class);

                while let Some(class) = cur_class {
                    let cur_static_class = class.inner_class_definition();
                    if let Some(ns) = cur_static_class
                        .read()
                        .resolve_any_instance_trait(local_name)
                    {
                        return Ok(Some(ns));
                    }

                    cur_class = class.superclass_object();
                }

                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn has_own_property(&self, name: &QName<'gc>) -> Result<bool, Error> {
        Ok(self.values.get(name).is_some() || self.has_trait(name)?)
    }

    pub fn has_own_virtual_getter(&self, name: &QName<'gc>) -> bool {
        matches!(
            self.values.get(name),
            Some(Property::Virtual { get: Some(_), .. })
        )
    }

    pub fn has_own_virtual_setter(&self, name: &QName<'gc>) -> bool {
        matches!(
            self.values.get(name),
            Some(Property::Virtual { set: Some(_), .. })
        )
    }

    pub fn proto(&self) -> Option<Object<'gc>> {
        self.proto
    }

    pub fn set_proto(&mut self, proto: Object<'gc>) {
        self.proto = Some(proto)
    }

    pub fn get_enumerant_name(&self, index: u32) -> Option<Value<'gc>> {
        // NOTE: AVM2 object enumeration is one of the weakest parts of an
        // otherwise well-designed VM. Notably, because of the way they
        // implemented `hasnext` and `hasnext2`, all enumerants start from ONE.
        // Hence why we have to `checked_sub` here in case some miscompiled
        // code doesn't check for the zero index, which is actually a failure
        // sentinel.
        let true_index = (index as usize).checked_sub(1)?;

        self.enumerants
            .get(true_index)
            .cloned()
            .map(|q| q.local_name().into())
    }

    pub fn property_is_enumerable(&self, name: &QName<'gc>) -> bool {
        self.enumerants.contains(name)
    }

    pub fn set_local_property_is_enumerable(
        &mut self,
        name: &QName<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        // Traits are never enumerable
        if self.has_trait(name)? {
            return Ok(());
        }

        if is_enumerable && self.values.contains_key(name) && !self.enumerants.contains(name) {
            self.enumerants.push(name.clone());
        } else if !is_enumerable && self.enumerants.contains(name) {
            let mut index = None;
            for (i, other_name) in self.enumerants.iter().enumerate() {
                if other_name == name {
                    index = Some(i);
                }
            }

            if let Some(index) = index {
                self.enumerants.remove(index);
            }
        }

        Ok(())
    }

    /// Get the end of (standard) enumerant space.
    ///
    /// Intended for objects that need to extend enumerant space. The index
    /// returned is guaranteed to be unused by the base enumerant list.
    pub fn get_last_enumerant(&self) -> u32 {
        (self.enumerants.len() as u32).saturating_add(1)
    }

    /// Install a method into the object.
    pub fn install_method(
        &mut self,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) {
        if disp_id > 0 {
            if self.methods.len() <= disp_id as usize {
                self.methods
                    .resize_with(disp_id as usize + 1, Default::default);
            }

            *self.methods.get_mut(disp_id as usize).unwrap() = Some(function);
        }

        self.values
            .insert(name, Property::new_method(function, is_final));
    }

    /// Install a getter into the object.
    ///
    /// This is a little more complicated than methods, since virtual property
    /// slots can be installed in two parts. Thus, we need to support
    /// installing them in either order.
    pub fn install_getter(
        &mut self,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) -> Result<(), Error> {
        function
            .as_executable()
            .ok_or_else(|| Error::from("Attempted to install getter without a valid method"))?;

        if disp_id > 0 {
            if self.methods.len() <= disp_id as usize {
                self.methods
                    .resize_with(disp_id as usize + 1, Default::default);
            }

            *self.methods.get_mut(disp_id as usize).unwrap() = Some(function);
        }

        if !self.values.contains_key(&name) {
            self.values
                .insert(name.clone(), Property::new_virtual(is_final));
        }

        self.values
            .get_mut(&name)
            .unwrap()
            .install_virtual_getter(function)
    }

    /// Install a setter into the object.
    ///
    /// This is a little more complicated than methods, since virtual property
    /// slots can be installed in two parts. Thus, we need to support
    /// installing them in either order.
    pub fn install_setter(
        &mut self,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) -> Result<(), Error> {
        function
            .as_executable()
            .ok_or_else(|| Error::from("Attempted to install setter without a valid method"))?;

        if disp_id > 0 {
            if self.methods.len() <= disp_id as usize {
                self.methods
                    .resize_with(disp_id as usize + 1, Default::default);
            }

            *self.methods.get_mut(disp_id as usize).unwrap() = Some(function);
        }

        if !self.values.contains_key(&name) {
            self.values
                .insert(name.clone(), Property::new_virtual(is_final));
        }

        self.values
            .get_mut(&name)
            .unwrap()
            .install_virtual_setter(function)
    }

    pub fn install_dynamic_property(
        &mut self,
        name: QName<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        if let Some(class) = self.instance_of() {
            let class = class.inner_class_definition();
            if class.read().is_sealed() {
                return Err(format!(
                    "Objects of type {:?} are not dynamic",
                    class.read().name().local_name()
                )
                .into());
            }
        }

        self.values
            .insert(name, Property::new_dynamic_property(value));

        Ok(())
    }

    /// Install a slot onto the object.
    ///
    /// Slot number zero indicates a slot ID that is unknown and should be
    /// allocated by the VM - as far as I know, there is no way to discover
    /// slot IDs, so we don't allocate a slot for them at all.
    pub fn install_slot(&mut self, name: QName<'gc>, id: u32, value: Value<'gc>, is_final: bool) {
        if id == 0 {
            self.values
                .insert(name, Property::new_stored(value, is_final));
        } else {
            self.values.insert(name, Property::new_slot(id, is_final));
            if self.slots.len() < id as usize + 1 {
                self.slots.resize_with(id as usize + 1, Default::default);
            }

            if let Some(slot) = self.slots.get_mut(id as usize) {
                *slot = Slot::new(value);
            }
        }
    }

    /// Install a const onto the object.
    ///
    /// Slot number zero indicates a slot ID that is unknown and should be
    /// allocated by the VM - as far as I know, there is no way to discover
    /// slot IDs, so we don't allocate a slot for them at all.
    pub fn install_const(&mut self, name: QName<'gc>, id: u32, value: Value<'gc>, is_final: bool) {
        if id == 0 {
            self.values
                .insert(name, Property::new_const(value, is_final));
        } else {
            self.values.insert(name, Property::new_slot(id, is_final));
            if self.slots.len() < id as usize + 1 {
                self.slots.resize_with(id as usize + 1, Default::default);
            }

            if let Some(slot) = self.slots.get_mut(id as usize) {
                *slot = Slot::new_const(value);
            }
        }
    }

    /// Get the class object for this object, if it has one.
    pub fn instance_of(&self) -> Option<ClassObject<'gc>> {
        self.instance_of
    }

    /// Set the class object for this object.
    pub fn set_instance_of(&mut self, instance_of: ClassObject<'gc>) {
        self.instance_of = Some(instance_of);
    }
}
