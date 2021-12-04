//! Default AVM2 object impl

use crate::avm2::activation::Activation;
use crate::avm2::names::{Multiname};
use crate::avm2::object::{FunctionObject, ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::vtable::VTable;
use crate::string::AvmString;
use fnv::FnvHashMap;
use std::collections::hash_map::Entry;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
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
    /// Values stored on this object.
    values: FnvHashMap<AvmString<'gc>, Value<'gc>>,

    /// Slots stored on this object.
    slots: Vec<Value<'gc>>,

    /// Methods stored on this object.
    bound_methods: Vec<Option<FunctionObject<'gc>>>,

    /// Implicit prototype of this script object.
    proto: Option<Object<'gc>>,

    /// The class object that this is an instance of.
    /// If `none`, this is not an ES4 object at all.
    instance_of: Option<ClassObject<'gc>>,

    /// The table used for non-dynamic property lookups.
    vtable: Option<VTable<'gc>>,

    /// Enumeratable property names.
    enumerants: Vec<AvmString<'gc>>,
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

    /// Construct an instance with a class and proto chain.
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

    /// Construct an instance with a class chain, but no prototype.
    pub fn bare_instance(mc: MutationContext<'gc, '_>, class: ClassObject<'gc>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(None, Some(class)),
        ))
        .into()
    }
}

impl<'gc> ScriptObjectData<'gc> {
    pub fn base_new(proto: Option<Object<'gc>>, instance_of: Option<ClassObject<'gc>>) -> Self {
        ScriptObjectData {
            values: Default::default(),
            slots: Vec::new(),
            bound_methods: Vec::new(),
            proto,
            instance_of,
            vtable: instance_of.map(|cls| cls.instance_vtable()),
            enumerants: Vec::new(),
        }
    }

    pub fn get_property_local(
        &self,
        receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        if !multiname.contains_public_namespace() {
            todo!("throw error")
        }

        let local_name = match multiname.local_name() {
            None => todo!("throw error"),
            Some(name) => name,
        };

        let value = self.values.get(&local_name);


        if let Some(value) = value {
            return Ok(value.clone());
        } else {
            if let Some(proto) = self.proto() {
                return proto.get_property_local(receiver, multiname, activation);
            }    
        }

        // Special case: Unresolvable properties on dynamic classes are treated
        // as dynamic properties that have not yet been set, and yield
        // `undefined`
        if self.instance_of()
            .map(|cls| cls.inner_class_definition().read().is_sealed())
            .unwrap_or(false)
        {
            return Err(format!("Cannot get undefined property {:?}", local_name).into());
        } else {
            return Ok(Value::Undefined);
        }
    }

    pub fn set_property_local(
        &mut self,
        _receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        _activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if self.instance_of()
            .map(|cls| cls.inner_class_definition().read().is_sealed())
            .unwrap_or(false)
        {
            return Err(format!("Cannot set undefined property {:?}", multiname.local_name()).into());
        }

        if !multiname.contains_public_namespace() {
            todo!("throw error")
        }

        let local_name = match multiname.local_name() {
            None => todo!("throw error"),
            Some(name) => name,
        };

        match self.values.entry(local_name) {
            Entry::Occupied(mut o) => { o.insert(value); },
            Entry::Vacant(v) => {
                //TODO: Not all classes are dynamic like this
                self.enumerants.push(local_name);
                v.insert(value);
            }
        };
        Ok(())
    }

    pub fn init_property_local(
        &mut self,
        receiver: Object<'gc>,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.set_property_local(receiver, multiname, value, activation)
    }

    pub fn delete_property_local(&mut self, multiname: &Multiname<'gc>) -> bool {
        if !multiname.contains_public_namespace() {
            return false;
        }
        if let Some(name) = multiname.local_name() {
            self.values.remove(&name);
            true
        } else {
            false
        }
    }

    pub fn get_slot(&self, id: u32) -> Result<Value<'gc>, Error> {
        self.slots
            .get(id as usize)
            .cloned()
            .ok_or_else(|| format!("Slot index {} out of bounds!", id).into())
    }

    /// Set a slot by its index.
    pub fn set_slot(
        &mut self,
        id: u32,
        value: Value<'gc>,
        _mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(slot) = self.slots.get_mut(id as usize) {
            *slot = value;
            Ok(())
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
            *slot = value;
            Ok(())
        } else {
            Err(format!("Slot index {} out of bounds!", id).into())
        }
    }

    pub fn install_instance_slots(
        &mut self,
        _activation: &mut Activation<'_, 'gc, '_>,
        slots: Vec<Option<Value<'gc>>>,
    ) {
        for value in slots {
            if let Some(value) = value {
                self.slots.push(value);
            } else {
                self.slots.push(Value::Undefined)
            }
        }
    }

    /// Set a slot by its index. This does extend the array if needed.
    /// This should only be used during AVM initialization, not at runtime.
    pub fn install_const_slot_late(
        &mut self,
        id: u32,
        value: Value<'gc>,
    ) {
        if self.slots.len() < id as usize + 1 {
            self.slots.resize(id as usize + 1, Value::Undefined);
        }
        if let Some(slot) = self.slots.get_mut(id as usize) {
            *slot = value;
        }
    }

    /// Retrieve a bound method from the method table.
    pub fn get_bound_method(&self, id: u32) -> Option<FunctionObject<'gc>> {
        self.bound_methods.get(id as usize).and_then(|v| *v)
    }

    pub fn has_trait(&self, name: &Multiname<'gc>) -> bool {
        match self.vtable {
            //Class instances have instance traits from any class in the base
            //class chain.
            Some(vtable) => vtable.has_trait(name),

            // Bare objects, ES3 objects, and prototypes do not have traits.
            None => false,
        }
    }

    pub fn has_own_dynamic_property(&self, name: &Multiname<'gc>) -> bool {
        if name.contains_public_namespace() {
            if let Some(name) = name.local_name() {
                return self.values.get(&name).is_some();
            }
        }
        false
    }

    pub fn has_own_property(&self, name: &Multiname<'gc>) -> bool {
        self.has_trait(name) || self.has_own_dynamic_property(name)
    }

    pub fn proto(&self) -> Option<Object<'gc>> {
        self.proto
    }

    pub fn set_proto(&mut self, proto: Object<'gc>) {
        self.proto = Some(proto)
    }

    pub fn get_next_enumerant(&self, last_index: u32) -> Option<u32> {
        if last_index < self.enumerants.len() as u32 {
            Some(last_index.saturating_add(1))
        } else {
            None
        }
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
            .map(|q| q.into())
    }

    pub fn property_is_enumerable(&self, name: AvmString<'gc>) -> bool {
        self.enumerants.contains(&name)
    }

    pub fn set_local_property_is_enumerable(
        &mut self,
        name: AvmString<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        if is_enumerable && self.values.contains_key(&name) && !self.enumerants.contains(&name) {
            self.enumerants.push(name);
        } else if !is_enumerable && self.enumerants.contains(&name) {
            let mut index = None;
            for (i, other_name) in self.enumerants.iter().enumerate() {
                if *other_name == name {
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
        self.enumerants.len() as u32
    }

    /// Install a method into the object.
    pub fn install_bound_method(&mut self, disp_id: u32, function: FunctionObject<'gc>) {
        if self.bound_methods.len() <= disp_id as usize {
            self.bound_methods
                .resize_with(disp_id as usize + 1, Default::default);
        }

        *self.bound_methods.get_mut(disp_id as usize).unwrap() = Some(function);
    }



    /// Get the class object for this object, if it has one.
    pub fn instance_of(&self) -> Option<ClassObject<'gc>> {
        self.instance_of
    }

    /// Get the vtable for this object, if it has one.
    pub fn vtable(&self) -> Option<VTable<'gc>> {
        self.vtable
    }

    /// Set the class object for this object.
    pub fn set_instance_of(&mut self, instance_of: ClassObject<'gc>, vtable: VTable<'gc>) {
        self.instance_of = Some(instance_of);
        self.vtable = Some(vtable);
    }

    pub fn set_vtable(&mut self, vtable: VTable<'gc>) {
        self.vtable = Some(vtable);
    }
}
