//! Default AVM2 object impl

use crate::avm2::function::Executable;
use crate::avm2::names::QName;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::slot::Slot;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;
use std::fmt::Debug;

/// Default implementation of `avm2::Object`.
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ScriptObjectData<'gc> {
    /// Properties stored on this object.
    values: HashMap<QName, Property<'gc>>,

    /// Slots stored on this object.
    slots: Vec<Slot<'gc>>,

    /// Methods stored on this object.
    methods: Vec<Option<Object<'gc>>>,

    /// Implicit prototype (or declared base class) of this script object.
    proto: Option<Object<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    fn get_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0
            .read()
            .get_property_local(reciever, name, avm, context)
    }

    fn set_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .set_property_local(reciever, name, value, avm, context)
    }

    fn init_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .init_property_local(reciever, name, value, avm, context)
    }

    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, multiname: &QName) -> bool {
        self.0.write(gc_context).delete_property(multiname)
    }

    fn get_slot(self, id: u32) -> Result<Value<'gc>, Error> {
        self.0.read().get_slot(id)
    }

    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).set_slot(id, value, mc)
    }

    fn init_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).init_slot(id, value, mc)
    }

    fn get_method(self, id: u32) -> Option<Object<'gc>> {
        self.0.read().get_method(id)
    }

    fn has_own_property(self, name: &QName) -> bool {
        self.0.read().has_own_property(name)
    }

    fn has_own_virtual_setter(self, name: &QName) -> bool {
        self.0.read().has_own_virtual_setter(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().proto
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn construct(
        &self,
        _avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::ScriptObject(*self);
        Ok(ScriptObject::object(context.gc_context, this))
    }

    fn install_method(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) {
        self.0.write(mc).install_method(name, disp_id, function)
    }

    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_getter(name, disp_id, function)
    }

    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_setter(name, disp_id, function)
    }

    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_dynamic_property(name, value)
    }

    fn install_slot(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        id: u32,
        value: Value<'gc>,
    ) {
        self.0.write(mc).install_slot(name, id, value)
    }

    fn install_const(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        id: u32,
        value: Value<'gc>,
    ) {
        self.0.write(mc).install_const(name, id, value)
    }
}

impl<'gc> ScriptObject<'gc> {
    /// Construct a bare object with no base class.
    ///
    /// This is *not* the same thing as an object literal, which actually does
    /// have a base class: `Object`.
    pub fn bare_object(mc: MutationContext<'gc, '_>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(mc, ScriptObjectData::base_new(None))).into()
    }

    /// Construct an object with a base class.
    pub fn object(mc: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(
            mc,
            ScriptObjectData::base_new(Some(proto)),
        ))
        .into()
    }
}

impl<'gc> ScriptObjectData<'gc> {
    pub fn base_new(proto: Option<Object<'gc>>) -> Self {
        ScriptObjectData {
            values: HashMap::new(),
            slots: Vec::new(),
            methods: Vec::new(),
            proto,
        }
    }

    pub fn get_property_local(
        &self,
        reciever: Object<'gc>,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let prop = self.values.get(name);

        if let Some(prop) = prop {
            prop.get(avm, context, reciever)
        } else {
            Ok(Value::Undefined.into())
        }
    }

    pub fn set_property_local(
        &mut self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(prop) = self.values.get_mut(name) {
            if let Some(slot_id) = prop.slot_id() {
                self.set_slot(slot_id, value, context.gc_context)?;
            } else {
                prop.set(avm, context, reciever, value)?;
            }
        } else {
            //TODO: Not all classes are dynamic like this
            self.values
                .insert(name.clone(), Property::new_dynamic_property(value));
        }

        Ok(())
    }

    pub fn init_property_local(
        &mut self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(prop) = self.values.get_mut(name) {
            if let Some(slot_id) = prop.slot_id() {
                self.init_slot(slot_id, value, context.gc_context)?;
            } else {
                prop.init(avm, context, reciever, value)?;
            }
        } else {
            //TODO: Not all classes are dynamic like this
            self.values
                .insert(name.clone(), Property::new_dynamic_property(value));
        }

        Ok(())
    }

    pub fn delete_property(&mut self, name: &QName) -> bool {
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
        //TODO: slot inheritance, I think?
        self.slots
            .get(id as usize)
            .cloned()
            .ok_or_else(|| format!("Slot index {} out of bounds!", id).into())
            .map(|slot| slot.get().unwrap_or(Value::Undefined))
    }

    /// Set a slot by it's index.
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

    /// Set a slot by it's index.
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

    pub fn has_own_property(&self, name: &QName) -> bool {
        self.values.get(name).is_some()
    }

    pub fn has_own_virtual_setter(&self, name: &QName) -> bool {
        match self.values.get(name) {
            Some(Property::Virtual { .. }) => true,
            _ => false,
        }
    }

    pub fn proto(&self) -> Option<Object<'gc>> {
        self.proto
    }

    /// Install a method into the object.
    pub fn install_method(&mut self, name: QName, disp_id: u32, function: Object<'gc>) {
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
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        let executable: Result<Executable<'gc>, Error> = function
            .as_executable()
            .ok_or_else(|| "Attempted to install getter without a valid method".into());
        let executable = executable?;

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
            .install_virtual_getter(executable)
    }

    /// Install a setter into the object.
    ///
    /// This is a little more complicated than methods, since virtual property
    /// slots can be installed in two parts. Thus, we need to support
    /// installing them in either order.
    pub fn install_setter(
        &mut self,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        let executable: Result<Executable<'gc>, Error> = function
            .as_executable()
            .ok_or_else(|| "Attempted to install setter without a valid method".into());
        let executable = executable?;

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
            .install_virtual_setter(executable)
    }

    pub fn install_dynamic_property(
        &mut self,
        name: QName,
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
    pub fn install_slot(&mut self, name: QName, id: u32, value: Value<'gc>) {
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
    pub fn install_const(&mut self, name: QName, id: u32, value: Value<'gc>) {
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
}
