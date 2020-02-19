//! Default AVM2 object impl

use crate::avm2::function::{
    Avm2ClassEntry, Avm2Function, Avm2MethodEntry, Executable, FunctionObject,
};
use crate::avm2::names::QName;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Trait as AbcTrait, TraitKind as AbcTraitKind};

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
    slots: Vec<Value<'gc>>,

    /// Implicit prototype (or declared base class) of this script object.
    proto: Option<Object<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    fn get_property(
        self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().get_property(name, avm, context, self.into())
    }

    fn set_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .set_property(name, value, avm, context, self.into())
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

    fn has_property(self, name: &QName) -> bool {
        self.0.read().has_property(name)
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

    fn install_trait(
        &mut self,
        mc: MutationContext<'gc, '_>,
        abc: Rc<AbcFile>,
        trait_entry: &AbcTrait,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .install_trait(mc, abc, trait_entry, scope, fn_proto)
    }

    fn install_method(&mut self, mc: MutationContext<'gc, '_>, name: QName, function: Object<'gc>) {
        self.0.write(mc).install_method(name, function)
    }

    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).install_dynamic_property(name, value)
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

    /// Construct the instance prototype half of a class.
    pub fn instance_prototype(
        mc: MutationContext<'gc, '_>,
        type_entry: Avm2ClassEntry,
        base_class: Object<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let mut data = ScriptObjectData::base_new(Some(base_class));

        for trait_entry in type_entry.instance().traits.iter() {
            data.install_trait(mc, type_entry.abc(), trait_entry, scope, fn_proto)?;
        }

        Ok(ScriptObject(GcCell::allocate(mc, data)).into())
    }
}

impl<'gc> ScriptObjectData<'gc> {
    pub fn base_new(proto: Option<Object<'gc>>) -> Self {
        ScriptObjectData {
            values: HashMap::new(),
            slots: Vec::new(),
            proto,
        }
    }

    pub fn get_property(
        &self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let prop = self.values.get(name);

        if let Some(prop) = prop {
            prop.get(avm, context, this)
        } else {
            Ok(Value::Undefined.into())
        }
    }

    pub fn set_property(
        &mut self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error> {
        if let Some(prop) = self.values.get_mut(name) {
            prop.set(avm, context, this, value)?;
        } else {
            //TODO: Not all classes are dynamic like this
            self.values
                .insert(name.clone(), Property::new_dynamic_property(value));
        }

        Ok(())
    }

    pub fn get_slot(&self, id: u32) -> Result<Value<'gc>, Error> {
        self.slots
            .get(id as usize)
            .cloned()
            .ok_or_else(|| format!("Slot index {} out of bounds!", id).into())
    }

    /// Set a slot by it's index.
    pub fn set_slot(
        &mut self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        if let Some(slot) = self.slots.get_mut(id as usize) {
            *slot = value;

            Ok(())
        } else {
            Err(format!("Slot index {} out of bounds!", id).into())
        }
    }

    pub fn has_property(&self, name: &QName) -> bool {
        self.values.get(name).is_some()
    }

    pub fn proto(&self) -> Option<Object<'gc>> {
        self.proto
    }

    pub fn install_trait(
        &mut self,
        mc: MutationContext<'gc, '_>,
        abc: Rc<AbcFile>,
        trait_entry: &AbcTrait,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<(), Error> {
        let trait_name = QName::from_abc_multiname(&abc, trait_entry.name.clone())?;
        match &trait_entry.kind {
            AbcTraitKind::Method { method, .. } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function = FunctionObject::from_abc_method(mc, method, scope, fn_proto);
                self.install_method(trait_name, function);
            }
            AbcTraitKind::Getter { method, .. } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let exec = Avm2Function::from_method(method, scope).into();
                self.install_getter(trait_name, exec)?;
            }
            AbcTraitKind::Setter { method, .. } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let exec = Avm2Function::from_method(method, scope).into();
                self.install_setter(trait_name, exec)?;
            }
            _ => return Err("".into()),
        }

        Ok(())
    }

    /// Install a method into the object.
    pub fn install_method(&mut self, name: QName, function: Object<'gc>) {
        self.values.insert(name, Property::new_method(function));
    }

    /// Install a getter into the object.
    ///
    /// This is a little more complicated than methods, since virtual property
    /// slots can be installed in two parts. Thus, we need to support
    /// installing them in either order.
    fn install_getter(&mut self, name: QName, function: Executable<'gc>) -> Result<(), Error> {
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
    fn install_setter(&mut self, name: QName, function: Executable<'gc>) -> Result<(), Error> {
        if !self.values.contains_key(&name) {
            self.values.insert(name.clone(), Property::new_virtual());
        }

        self.values
            .get_mut(&name)
            .unwrap()
            .install_virtual_setter(function)
    }

    /// Install a class into the object.
    ///
    /// Classes are fairly complicated. We desugar them into an ES3-style
    /// prototype chain, which means we need to build a function, prototype,
    /// and so on. In concert with the `new` Rust trait function we also ensure
    /// that subclasses of an object impl use the same impl (e.g. subclasses of
    /// `MovieClip` remain movie clips).
    fn install_class(
        &mut self,
        name: QName,
        type_entry: Avm2ClassEntry,
        slot: u32,
    ) -> Result<(), Error> {
        Err("unimplemented".into())
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
}
