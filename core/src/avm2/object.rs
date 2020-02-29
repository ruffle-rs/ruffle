//! AVM2 objects.

use crate::avm2::function::{Avm2ClassEntry, Avm2MethodEntry, Executable, FunctionObject};
use crate::avm2::names::{Multiname, QName};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::script_object::ScriptObject;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_macros::enum_trait_object;
use std::fmt::Debug;
use std::rc::Rc;
use swf::avm2::types::{AbcFile, Trait as AbcTrait, TraitKind as AbcTraitKind};

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        FunctionObject(FunctionObject<'gc>)
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Retrieve a property by it's QName, without taking prototype lookups
    /// into account.
    fn get_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Retrieve a property by it's QName.
    fn get_property(
        self,
        reciever: Object<'gc>,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let has_no_getter = self.has_own_virtual_setter(name) && !self.has_own_virtual_getter(name);

        if self.has_own_property(name) && !has_no_getter {
            return self.get_property_local(reciever, name, avm, context);
        }

        if let Some(proto) = self.proto() {
            return proto.get_property(reciever, name, avm, context);
        }

        Ok(Value::Undefined.into())
    }

    /// Retrieve the base prototype that a particular QName is defined in.
    fn get_base_proto(self, name: &QName) -> Option<Object<'gc>> {
        if self.has_own_property(name) {
            return Some(self.into());
        }

        if let Some(proto) = self.proto() {
            return proto.get_base_proto(name);
        }

        None
    }

    /// Set a property on this specific object.
    ///
    /// This function returns a `ReturnValue` which should be resolved. The
    /// resulting `Value` is unimportant and should be discarded.
    fn set_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Set a property by it's QName.
    fn set_property(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if self.has_own_virtual_setter(name) {
            self.set_property_local(reciever, name, value, avm, context)?
                .resolve(avm, context)?;

            return Ok(());
        }

        let mut proto = self.proto();
        while let Some(my_proto) = proto {
            //NOTE: This only works because we validate ahead-of-time that
            //we're calling a virtual setter. If you call `set_property` on
            //a non-virtual you will actually alter the prototype.
            if my_proto.has_own_virtual_setter(name) {
                return my_proto.set_property(reciever, name, value, avm, context);
            }

            proto = my_proto.proto();
        }

        reciever
            .set_property_local(reciever, name, value, avm, context)?
            .resolve(avm, context)?;

        Ok(())
    }

    /// Init a property on this specific object.
    ///
    /// This function returns a `ReturnValue` which should be resolved. The
    /// resulting `Value` is unimportant and should be discarded.
    fn init_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error>;

    /// Init a property by it's QName.
    fn init_property(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if self.has_own_virtual_setter(name) {
            self.init_property_local(reciever, name, value, avm, context)?
                .resolve(avm, context)?;

            return Ok(());
        }

        let mut proto = self.proto();
        while let Some(my_proto) = proto {
            //NOTE: This only works because we validate ahead-of-time that
            //we're calling a virtual setter. If you call `set_property` on
            //a non-virtual you will actually alter the prototype.
            if my_proto.has_own_virtual_setter(name) {
                return my_proto.init_property(reciever, name, value, avm, context);
            }

            proto = my_proto.proto();
        }

        reciever
            .init_property_local(reciever, name, value, avm, context)?
            .resolve(avm, context)?;

        Ok(())
    }

    /// Retrieve a slot by it's index.
    fn get_slot(self, id: u32) -> Result<Value<'gc>, Error>;

    /// Set a slot by it's index.
    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error>;

    /// Initialize a slot by it's index.
    fn init_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error>;

    /// Retrieve a method by it's index.
    fn get_method(self, id: u32) -> Option<Object<'gc>>;

    /// Resolve a multiname into a single QName, if any of the namespaces
    /// match.
    fn resolve_multiname(self, multiname: &Multiname) -> Option<QName> {
        for ns in multiname.namespace_set() {
            let qname = QName::new(ns.clone(), multiname.local_name()?);
            if self.has_property(&qname) {
                return Some(qname);
            }
        }

        if let Some(proto) = self.proto() {
            return proto.resolve_multiname(multiname);
        }

        None
    }

    /// Indicates whether or not a property exists on an object.
    fn has_property(self, name: &QName) -> bool {
        if self.has_own_property(name) {
            true
        } else if let Some(proto) = self.proto() {
            proto.has_own_property(name)
        } else {
            false
        }
    }

    /// Indicates whether or not a property exists on an object and is not part
    /// of the prototype chain.
    fn has_own_property(self, name: &QName) -> bool;

    /// Check if a particular object contains a virtual getter by the given
    /// name.
    fn has_own_virtual_getter(self, name: &QName) -> bool;

    /// Check if a particular object contains a virtual setter by the given
    /// name.
    fn has_own_virtual_setter(self, name: &QName) -> bool;

    /// Indicates whether or not a property is overwritable.
    fn is_property_overwritable(self, _name: &QName) -> bool {
        false
    }

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, multiname: &QName) -> bool;

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>>;

    /// Install a method (or any other non-slot value) on an object.
    fn install_method(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    );

    /// Install a getter method on an object property.
    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error>;

    /// Install a setter method on an object property.
    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error>;

    /// Install a dynamic or built-in value property on an object.
    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        value: Value<'gc>,
    ) -> Result<(), Error>;

    /// Install a slot on an object property.
    fn install_slot(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        id: u32,
        value: Value<'gc>,
    );

    /// Install a const on an object property.
    fn install_const(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        id: u32,
        value: Value<'gc>,
    );

    /// Install a trait from an ABC file on an object.
    fn install_trait(
        &mut self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        abc: Rc<AbcFile>,
        trait_entry: &AbcTrait,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<(), Error> {
        let trait_name = QName::from_abc_multiname(&abc, trait_entry.name.clone())?;
        avm_debug!(
            "Installing trait {:?} of kind {:?}",
            trait_name,
            trait_entry.kind
        );

        match &trait_entry.kind {
            AbcTraitKind::Slot { slot_id, value, .. } => {
                let value = if let Some(value) = value {
                    abc_default_value(&abc, value)?
                } else {
                    Value::Undefined
                };
                self.install_slot(context.gc_context, trait_name, *slot_id, value);
            }
            AbcTraitKind::Method {
                disp_id, method, ..
            } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function =
                    FunctionObject::from_abc_method(context.gc_context, method, scope, fn_proto);
                self.install_method(context.gc_context, trait_name, *disp_id, function);
            }
            AbcTraitKind::Getter {
                disp_id, method, ..
            } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function =
                    FunctionObject::from_abc_method(context.gc_context, method, scope, fn_proto);
                self.install_getter(context.gc_context, trait_name, *disp_id, function)?;
            }
            AbcTraitKind::Setter {
                disp_id, method, ..
            } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function =
                    FunctionObject::from_abc_method(context.gc_context, method, scope, fn_proto);
                self.install_setter(context.gc_context, trait_name, *disp_id, function)?;
            }
            AbcTraitKind::Class { slot_id, class } => {
                let type_entry = Avm2ClassEntry::from_class_index(abc, class.clone()).unwrap();
                let super_name = QName::from_abc_multiname(
                    &type_entry.abc(),
                    type_entry.instance().super_name.clone(),
                )?;
                let reciever: Object<'gc> = (*self).into();
                let super_class: Result<Object<'gc>, Error> = self
                    .get_property(reciever, &super_name, avm, context)?
                    .resolve(avm, context)?
                    .as_object()
                    .map_err(|_e| {
                        format!("Could not resolve superclass {:?}", super_name.local_name()).into()
                    });

                let (class, _cinit) = FunctionObject::from_abc_class(
                    avm,
                    context,
                    type_entry.clone(),
                    super_class?,
                    scope,
                    fn_proto,
                )?;
                let class_name = QName::from_abc_multiname(
                    &type_entry.abc(),
                    type_entry.instance().name.clone(),
                )?;
                self.install_const(context.gc_context, class_name, *slot_id, class.into());
            }
            AbcTraitKind::Function {
                slot_id, function, ..
            } => {
                let method = Avm2MethodEntry::from_method_index(abc, function.clone()).unwrap();
                let function =
                    FunctionObject::from_abc_method(context.gc_context, method, scope, fn_proto);
                self.install_const(context.gc_context, trait_name, *slot_id, function.into());
            }
            AbcTraitKind::Const { slot_id, value, .. } => {
                let value = if let Some(value) = value {
                    abc_default_value(&abc, value)?
                } else {
                    Value::Undefined
                };
                self.install_const(context.gc_context, trait_name, *slot_id, value);
            }
        }

        Ok(())
    }

    /// Call the object.
    fn call(
        self,
        _reciever: Option<Object<'gc>>,
        _arguments: &[Value<'gc>],
        _avm: &mut Avm2<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _base_proto: Option<Object<'gc>>,
    ) -> Result<ReturnValue<'gc>, Error> {
        Err("Object is not callable".into())
    }

    /// Construct a host object of some kind and return it's cell.
    ///
    /// As the first step in object construction, the `construct` method is
    /// called on the prototype to create a new object. The prototype may
    /// construct any object implementation it wants, however, it's expected
    /// to produce a like `TObject` implementor with itself as the new object's
    /// proto.
    ///
    /// After construction, the constructor function is `call`ed with the new
    /// object as `this` to initialize the object.
    ///
    /// The arguments passed to the constructor are provided here; however, all
    /// object construction should happen in `call`, not `new`. `new` exists
    /// purely so that host objects can be constructed by the VM.
    fn construct(
        &self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error>;

    /// Get a raw pointer value for this object.
    fn as_ptr(&self) -> *const ObjectPtr;

    /// Get this object's `Executable`, if it has one.
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}
