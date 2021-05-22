//! Default AVM2 object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::property_map::PropertyMap;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::slot::Slot;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;
use std::fmt::Debug;

/// Default implementation of `avm2::Object`.
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

/// Information necessary for a script object to have a class attached to it.
///
/// Classes can be attached to a `ScriptObject` such that the class's traits
/// are instantiated on-demand. Either class or instance traits can be
/// instantiated.
///
/// Trait instantiation obeys prototyping rules: prototypes provide their
/// instances with classes to pull traits from.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum ScriptObjectClass<'gc> {
    /// Instantiate class traits, for class constructors.
    ClassConstructor(GcCell<'gc, Class<'gc>>, Option<GcCell<'gc, Scope<'gc>>>),

    /// Instantiate instance traits, for class instances.
    ClassInstance(Object<'gc>),

    /// Do not instantiate any class or instance traits.
    NoClass,
}

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

    /// The class that this script object represents.
    class: ScriptObjectClass<'gc>,

    /// Enumeratable property names.
    enumerants: Vec<QName<'gc>>,

    /// Interfaces implemented by this object. (prototypes only)
    interfaces: Vec<Object<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    fn get_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let rv = self
            .0
            .read()
            .get_property_local(receiver, name, activation)?;

        rv.resolve(activation)
    }

    fn set_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let rv = self
            .0
            .write(activation.context.gc_context)
            .set_property_local(receiver, name, value, activation)?;

        rv.resolve(activation)?;

        Ok(())
    }

    fn init_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let rv = self
            .0
            .write(activation.context.gc_context)
            .init_property_local(receiver, name, value, activation)?;

        rv.resolve(activation)?;

        Ok(())
    }

    fn is_property_overwritable(
        self,
        gc_context: MutationContext<'gc, '_>,
        name: &QName<'gc>,
    ) -> bool {
        self.0.write(gc_context).is_property_overwritable(name)
    }

    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, name: &QName<'gc>) -> bool {
        self.0.write(gc_context).delete_property(name)
    }

    fn has_slot_local(self, id: u32) -> bool {
        self.0.read().has_slot_local(id)
    }

    fn get_slot_local(self, id: u32) -> Result<Value<'gc>, Error> {
        self.0.read().get_slot_local(id)
    }

    fn set_slot_local(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).set_slot_local(id, value, mc)
    }

    fn init_slot_local(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).init_slot_local(id, value, mc)
    }

    fn get_method(self, id: u32) -> Option<Object<'gc>> {
        self.0.read().get_method(id)
    }

    fn get_trait(self, name: &QName<'gc>) -> Result<Vec<Trait<'gc>>, Error> {
        self.0.read().get_trait(name)
    }

    fn get_trait_slot(self, id: u32) -> Result<Option<Trait<'gc>>, Error> {
        self.0.read().get_trait_slot(id)
    }

    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.0.read().get_scope()
    }

    fn resolve_any(self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error> {
        self.0.read().resolve_any(local_name)
    }

    fn resolve_any_trait(
        self,
        local_name: AvmString<'gc>,
    ) -> Result<Option<Namespace<'gc>>, Error> {
        self.0.read().resolve_any_trait(local_name)
    }

    fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error> {
        self.0.read().has_own_property(name)
    }

    fn has_trait(self, name: &QName<'gc>) -> Result<bool, Error> {
        self.0.read().has_trait(name)
    }

    fn has_instantiated_property(self, name: &QName<'gc>) -> bool {
        self.0.read().has_instantiated_property(name)
    }

    fn has_own_virtual_getter(self, name: &QName<'gc>) -> bool {
        self.0.read().has_own_virtual_getter(name)
    }

    fn has_own_virtual_setter(self, name: &QName<'gc>) -> bool {
        self.0.read().has_own_virtual_setter(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().proto
    }

    fn set_proto(self, mc: MutationContext<'gc, '_>, proto: Object<'gc>) {
        self.0.write(mc).set_proto(proto)
    }

    fn get_enumerant_name(&self, index: u32) -> Option<QName<'gc>> {
        self.0.read().get_enumerant_name(index)
    }

    fn property_is_enumerable(&self, name: &QName<'gc>) -> bool {
        self.0.read().property_is_enumerable(name)
    }

    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: &QName<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .set_local_property_is_enumerable(name, is_enumerable)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ScriptObject(*self);
        Ok(ScriptObject::object(activation.context.gc_context, this))
    }

    fn to_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        if let Some(class) = self.as_class() {
            Ok(AvmString::new(mc, format!("[object {}]", class.read().name().local_name())).into())
        } else {
            Ok("[object Object]".into())
        }
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn install_method(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
    ) {
        self.0.write(mc).install_method(name, disp_id, function)
    }

    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_getter(name, disp_id, function)
    }

    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_setter(name, disp_id, function)
    }

    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_dynamic_property(name, value)
    }

    fn install_slot(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        id: u32,
        value: Value<'gc>,
    ) {
        self.0.write(mc).install_slot(name, id, value)
    }

    fn install_const(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        id: u32,
        value: Value<'gc>,
    ) {
        self.0.write(mc).install_const(name, id, value)
    }

    fn interfaces(&self) -> Vec<Object<'gc>> {
        self.0.read().interfaces()
    }

    fn set_interfaces(&self, gc_context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>) {
        self.0.write(gc_context).set_interfaces(iface_list)
    }

    fn as_class(&self) -> Option<GcCell<'gc, Class<'gc>>> {
        self.0.read().as_class()
    }

    fn as_constr(&self) -> Option<Object<'gc>> {
        self.0.read().as_constr()
    }

    fn set_constr(self, mc: MutationContext<'gc, '_>, constr: Object<'gc>) {
        self.0.write(mc).set_constr(constr);
    }
}

impl<'gc> ScriptObject<'gc> {
    /// Construct a bare object with no base class.
    ///
    /// This is *not* the same thing as an object literal, which actually does
    /// have a base class: `Object`.
    pub fn bare_object(mc: MutationContext<'gc, '_>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(None, ScriptObjectClass::NoClass),
        ))
        .into()
    }

    /// Construct an object with a prototype.
    pub fn object(mc: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(Some(proto), ScriptObjectClass::NoClass),
        ))
        .into()
    }

    /// Construct an instance with a class and scope stack.
    pub fn instance(
        mc: MutationContext<'gc, '_>,
        constr: Object<'gc>,
        proto: Object<'gc>,
    ) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(constr)),
        ))
        .into()
    }
}

impl<'gc> ScriptObjectData<'gc> {
    pub fn base_new(proto: Option<Object<'gc>>, trait_source: ScriptObjectClass<'gc>) -> Self {
        ScriptObjectData {
            values: HashMap::new(),
            slots: Vec::new(),
            methods: Vec::new(),
            proto,
            class: trait_source,
            enumerants: Vec::new(),
            interfaces: Vec::new(),
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
                Some(
                    activation
                        .base_constr()
                        .or_else(|| self.as_constr())
                        .unwrap_or(receiver),
                ),
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
        let constr = self.as_constr();
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
            // This doesn't need the non-local version of this property because
            // by the time this has called the slot was already installed
            self.set_slot_local(slot_id, value, activation.context.gc_context)?;
            Ok(Value::Undefined.into())
        } else if self.values.contains_key(name) {
            let prop = self.values.get_mut(name).unwrap();
            prop.set(
                receiver,
                Some(activation.base_constr().or(constr).unwrap_or(receiver)),
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
        let constr = self.as_constr();
        if let Some(prop) = self.values.get_mut(name) {
            if let Some(slot_id) = prop.slot_id() {
                // This doesn't need the non-local version of this property
                // because by the time this has called the slot was already
                // installed
                self.init_slot_local(slot_id, value, activation.context.gc_context)?;
                Ok(Value::Undefined.into())
            } else {
                prop.init(
                    receiver,
                    Some(activation.base_constr().or(constr).unwrap_or(receiver)),
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

    pub fn has_slot_local(&self, id: u32) -> bool {
        self.slots
            .get(id as usize)
            .map(|s| s.is_occupied())
            .unwrap_or(false)
    }

    pub fn get_slot_local(&self, id: u32) -> Result<Value<'gc>, Error> {
        self.slots
            .get(id as usize)
            .cloned()
            .ok_or_else(|| format!("Slot index {} out of bounds!", id).into())
            .map(|slot| slot.get().unwrap_or(Value::Undefined))
    }

    /// Set a slot by its index.
    pub fn set_slot_local(
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
    pub fn init_slot_local(
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

    pub fn get_trait(&self, name: &QName<'gc>) -> Result<Vec<Trait<'gc>>, Error> {
        match &self.class {
            //Class constructors have local traits only.
            ScriptObjectClass::ClassConstructor(class, ..) => {
                let mut known_traits = Vec::new();
                class.read().lookup_class_traits(name, &mut known_traits)?;

                Ok(known_traits)
            }

            //Class instances have all instance traits from all superclasses.
            ScriptObjectClass::ClassInstance(constr) => {
                let mut constr_list = Vec::new();
                let mut cur_constr = Some(*constr);
                while let Some(constr) = cur_constr {
                    constr_list.push(constr);

                    cur_constr = constr.base_class_constr();
                }

                let mut known_traits = Vec::new();
                for constr in constr_list.iter().rev() {
                    let cur_class = constr
                        .as_class()
                        .ok_or("Object is not a class constructor")?;
                    cur_class
                        .read()
                        .lookup_instance_traits(name, &mut known_traits)?;
                }

                Ok(known_traits)
            }

            // Bare objects, ES3 objects, and prototypes do not have traits.
            ScriptObjectClass::NoClass => Ok(Vec::new()),
        }
    }

    pub fn get_trait_slot(&self, id: u32) -> Result<Option<Trait<'gc>>, Error> {
        match &self.class {
            //Class constructors have local slot traits only.
            ScriptObjectClass::ClassConstructor(class, ..) => {
                class.read().lookup_class_traits_by_slot(id)
            }

            //Class instances have all instance slot traits from all superclasses.
            ScriptObjectClass::ClassInstance(constr) => {
                let mut cur_constr = Some(*constr);

                while let Some(constr) = cur_constr {
                    let cur_class = constr
                        .as_class()
                        .ok_or("Object is not a class constructor")?;
                    if let Some(inst_trait) = cur_class.read().lookup_instance_traits_by_slot(id)? {
                        return Ok(Some(inst_trait));
                    }

                    cur_constr = constr.base_class_constr();
                }

                Ok(None)
            }

            // Bare objects, ES3 objects, and prototypes do not have traits.
            ScriptObjectClass::NoClass => Ok(None),
        }
    }

    pub fn has_trait(&self, name: &QName<'gc>) -> Result<bool, Error> {
        match &self.class {
            //Class constructors have local traits only.
            ScriptObjectClass::ClassConstructor(class, ..) => {
                Ok(class.read().has_class_trait(name))
            }

            //Class instances have instance traits from any class in the base
            //class chain.
            ScriptObjectClass::ClassInstance(constr) => {
                let mut cur_constr = Some(*constr);

                while let Some(constr) = cur_constr {
                    let cur_class = constr
                        .as_class()
                        .ok_or("Object is not a class constructor")?;
                    if cur_class.read().has_instance_trait(name) {
                        return Ok(true);
                    }

                    cur_constr = constr.base_class_constr();
                }

                Ok(false)
            }

            // Bare objects, ES3 objects, and prototypes do not have traits.
            ScriptObjectClass::NoClass => Ok(false),
        }
    }

    pub fn get_scope(&self) -> Option<GcCell<'gc, Scope<'gc>>> {
        match &self.class {
            ScriptObjectClass::ClassConstructor(_class, scope) => *scope,
            ScriptObjectClass::ClassInstance(constr) => constr.get_scope(),
            ScriptObjectClass::NoClass => None,
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

        match &self.class {
            ScriptObjectClass::ClassConstructor(class, ..) => {
                Ok(class.read().resolve_any_class_trait(local_name))
            }
            ScriptObjectClass::ClassInstance(constr) => {
                let mut cur_constr = Some(*constr);

                while let Some(constr) = cur_constr {
                    let cur_class = constr
                        .as_class()
                        .ok_or("Object is not a class constructor")?;
                    if let Some(ns) = cur_class.read().resolve_any_instance_trait(local_name) {
                        return Ok(Some(ns));
                    }

                    cur_constr = constr.base_class_constr();
                }

                Ok(None)
            }
            ScriptObjectClass::NoClass => Ok(None),
        }
    }

    pub fn has_own_property(&self, name: &QName<'gc>) -> Result<bool, Error> {
        Ok(self.values.get(name).is_some() || self.has_trait(name)?)
    }

    pub fn has_instantiated_property(&self, name: &QName<'gc>) -> bool {
        self.values.get(name).is_some()
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

    pub fn get_enumerant_name(&self, index: u32) -> Option<QName<'gc>> {
        // NOTE: AVM2 object enumeration is one of the weakest parts of an
        // otherwise well-designed VM. Notably, because of the way they
        // implemented `hasnext` and `hasnext2`, all enumerants start from ONE.
        // Hence why we have to `checked_sub` here in case some miscompiled
        // code doesn't check for the zero index, which is actually a failure
        // sentinel.
        let true_index = (index as usize).checked_sub(1)?;

        self.enumerants.get(true_index).cloned()
    }

    pub fn property_is_enumerable(&self, name: &QName<'gc>) -> bool {
        self.enumerants.contains(name)
    }

    pub fn set_local_property_is_enumerable(
        &mut self,
        name: &QName<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        if is_enumerable && self.values.contains_key(name) && !self.enumerants.contains(name) {
            // Traits are never enumerable
            if self.has_trait(name)? {
                return Ok(());
            }

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

    pub fn class(&self) -> &ScriptObjectClass<'gc> {
        &self.class
    }

    /// Install a method into the object.
    pub fn install_method(&mut self, name: QName<'gc>, disp_id: u32, function: Object<'gc>) {
        if disp_id > 0 {
            if self.methods.len() <= disp_id as usize {
                self.methods
                    .resize_with(disp_id as usize + 1, Default::default);
            }

            *self.methods.get_mut(disp_id as usize).unwrap() = Some(function);
        }

        self.values.insert(name, Property::new_method(function));
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
            self.values.insert(name.clone(), Property::new_virtual());
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
            self.values.insert(name.clone(), Property::new_virtual());
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
        self.values
            .insert(name, Property::new_dynamic_property(value));

        Ok(())
    }

    /// Install a slot onto the object.
    ///
    /// Slot number zero indicates a slot ID that is unknown and should be
    /// allocated by the VM - as far as I know, there is no way to discover
    /// slot IDs, so we don't allocate a slot for them at all.
    pub fn install_slot(&mut self, name: QName<'gc>, id: u32, value: Value<'gc>) {
        if id == 0 {
            self.values.insert(name, Property::new_stored(value));
        } else {
            self.values.insert(name, Property::new_slot(id));
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
    pub fn install_const(&mut self, name: QName<'gc>, id: u32, value: Value<'gc>) {
        if id == 0 {
            self.values.insert(name, Property::new_const(value));
        } else {
            self.values.insert(name, Property::new_slot(id));
            if self.slots.len() < id as usize + 1 {
                self.slots.resize_with(id as usize + 1, Default::default);
            }

            if let Some(slot) = self.slots.get_mut(id as usize) {
                *slot = Slot::new_const(value);
            }
        }
    }

    /// Enumerate all interfaces implemented by this object.
    pub fn interfaces(&self) -> Vec<Object<'gc>> {
        self.interfaces.clone()
    }

    /// Set the interface list for this object.
    pub fn set_interfaces(&mut self, iface_list: Vec<Object<'gc>>) {
        self.interfaces = iface_list;
    }

    /// Get the class for this object, if it has one.
    pub fn as_class(&self) -> Option<GcCell<'gc, Class<'gc>>> {
        match self.class {
            ScriptObjectClass::ClassConstructor(class, _) => Some(class),
            ScriptObjectClass::ClassInstance(constr) => constr.as_class(),
            ScriptObjectClass::NoClass => None,
        }
    }

    /// Get the class constructor for this object, if it has one.
    pub fn as_constr(&self) -> Option<Object<'gc>> {
        match self.class {
            ScriptObjectClass::ClassConstructor(..) => None,
            ScriptObjectClass::ClassInstance(constr) => Some(constr),
            ScriptObjectClass::NoClass => None,
        }
    }

    /// Associate the object with a particular constructor.
    ///
    /// This turns the object into an instance of that class. It should only be
    /// used in situations where the object cannot be made an instance of the
    /// class at allocation time, such as during early runtime setup.
    pub fn set_constr(&mut self, constr: Object<'gc>) {
        self.class = ScriptObjectClass::ClassInstance(constr);
    }
}
