//! AVM2 objects.

use crate::avm2::activation::Activation;
use crate::avm2::array::ArrayStorage;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::class::{AllocatorFn, Class};
use crate::avm2::domain::Domain;
use crate::avm2::events::{DispatchList, Event};
use crate::avm2::function::Executable;
use crate::avm2::names::{Multiname, Namespace, QName};
use crate::avm2::regexp::RegExp;
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::value::{Hint, Value};
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::backend::audio::{SoundHandle, SoundInstanceHandle};
use crate::bitmap::bitmap_data::BitmapData;
use crate::display_object::DisplayObject;
use gc_arena::{Collect, GcCell, MutationContext};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

mod array_object;
mod bitmapdata_object;
mod bytearray_object;
mod class_object;
mod custom_object;
mod dispatch_object;
mod domain_object;
mod event_object;
mod function_object;
mod loaderinfo_object;
mod namespace_object;
mod primitive_object;
mod regexp_object;
mod script_object;
mod sound_object;
mod soundchannel_object;
mod stage_object;
mod vector_object;
mod xml_object;

pub use crate::avm2::object::array_object::{array_allocator, ArrayObject};
pub use crate::avm2::object::bitmapdata_object::{bitmapdata_allocator, BitmapDataObject};
pub use crate::avm2::object::bytearray_object::{bytearray_allocator, ByteArrayObject};
pub use crate::avm2::object::class_object::ClassObject;
pub use crate::avm2::object::dispatch_object::DispatchObject;
pub use crate::avm2::object::domain_object::{appdomain_allocator, DomainObject};
pub use crate::avm2::object::event_object::{event_allocator, EventObject};
pub use crate::avm2::object::function_object::FunctionObject;
pub use crate::avm2::object::loaderinfo_object::{
    loaderinfo_allocator, LoaderInfoObject, LoaderStream,
};
pub use crate::avm2::object::namespace_object::{namespace_allocator, NamespaceObject};
pub use crate::avm2::object::primitive_object::{primitive_allocator, PrimitiveObject};
pub use crate::avm2::object::regexp_object::{regexp_allocator, RegExpObject};
pub use crate::avm2::object::script_object::ScriptObject;
pub use crate::avm2::object::sound_object::{sound_allocator, SoundObject};
pub use crate::avm2::object::soundchannel_object::{soundchannel_allocator, SoundChannelObject};
pub use crate::avm2::object::stage_object::{stage_allocator, StageObject};
pub use crate::avm2::object::vector_object::{vector_allocator, VectorObject};
pub use crate::avm2::object::xml_object::{xml_allocator, XmlObject};

/// Represents an object that can be directly interacted with by the AVM2
/// runtime.
#[enum_trait_object(
    #[allow(clippy::enum_variant_names)]
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum Object<'gc> {
        ScriptObject(ScriptObject<'gc>),
        FunctionObject(FunctionObject<'gc>),
        PrimitiveObject(PrimitiveObject<'gc>),
        NamespaceObject(NamespaceObject<'gc>),
        ArrayObject(ArrayObject<'gc>),
        StageObject(StageObject<'gc>),
        DomainObject(DomainObject<'gc>),
        EventObject(EventObject<'gc>),
        DispatchObject(DispatchObject<'gc>),
        XmlObject(XmlObject<'gc>),
        RegExpObject(RegExpObject<'gc>),
        ByteArrayObject(ByteArrayObject<'gc>),
        LoaderInfoObject(LoaderInfoObject<'gc>),
        ClassObject(ClassObject<'gc>),
        VectorObject(VectorObject<'gc>),
        SoundObject(SoundObject<'gc>),
        SoundChannelObject(SoundChannelObject<'gc>),
        BitmapDataObject(BitmapDataObject<'gc>),
    }
)]
pub trait TObject<'gc>: 'gc + Collect + Debug + Into<Object<'gc>> + Clone + Copy {
    /// Retrieve a property by its QName, without taking prototype lookups
    /// into account.
    fn get_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error>;

    /// Retrieve a property by its QName.
    fn get_property(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let has_no_getter = self.has_own_virtual_setter(name) && !self.has_own_virtual_getter(name);

        if self.has_own_property(name)? && !has_no_getter {
            return self.get_property_local(receiver, name, activation);
        }

        if let Some(proto) = self.proto() {
            return proto.get_property(receiver, name, activation);
        }

        Ok(Value::Undefined)
    }

    /// Retrieve the class object that a particular QName trait is defined in.
    ///
    /// Must be called on a class object; will error out if called on
    /// anything else.
    ///
    /// This function returns `None` for non-trait properties, such as actually
    /// defined prototype methods for ES3-style classes.
    fn find_class_for_trait(self, name: &QName<'gc>) -> Result<Option<Object<'gc>>, Error> {
        let class = self
            .as_class()
            .ok_or("Cannot get base traits on non-class object")?;

        if class.read().has_instance_trait(name) {
            return Ok(Some(self.into()));
        }

        if let Some(base) = self.superclass_object() {
            return base.find_class_for_trait(name);
        }

        Ok(None)
    }

    /// Set a property on this specific object.
    fn set_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Set a property by its QName.
    fn set_property(
        &mut self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.set_property_local(receiver, name, value, activation)
    }

    /// Init a property on this specific object.
    fn init_property_local(
        self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Init a property by its QName.
    fn init_property(
        &mut self,
        receiver: Object<'gc>,
        name: &QName<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if self.has_own_virtual_setter(name) {
            return self.init_property_local(receiver, name, value, activation);
        }

        let mut proto = self.proto();
        while let Some(mut my_proto) = proto {
            //NOTE: This only works because we validate ahead-of-time that
            //we're calling a virtual setter. If you call `set_property` on
            //a non-virtual you will actually alter the prototype.
            if my_proto.has_own_virtual_setter(name) {
                return my_proto.init_property(receiver, name, value, activation);
            }

            proto = my_proto.proto();
        }

        receiver.init_property_local(receiver, name, value, activation)
    }

    /// Retrieve a slot by its index.
    fn get_slot(self, id: u32) -> Result<Value<'gc>, Error>;

    /// Set a slot by its index.
    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error>;

    /// Initialize a slot by its index.
    fn init_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error>;

    /// Retrieve a method by its index.
    fn get_method(self, id: u32) -> Option<Object<'gc>>;

    /// Retrieves the scope chain of the object at time of its creation.
    ///
    /// The scope chain is used to determine the starting scope stack when an
    /// object is called, as well as any class methods on the object.
    /// Non-method functions and prototype functions (ES3 methods) do not use
    /// this scope chain.
    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>>;

    /// Resolve a multiname into a single QName, if any of the namespaces
    /// match.
    fn resolve_multiname(self, multiname: &Multiname<'gc>) -> Result<Option<QName<'gc>>, Error> {
        for ns in multiname.namespace_set() {
            if ns.is_any() {
                if let Some(name) = multiname.local_name() {
                    let ns = self.resolve_any(name)?;
                    return Ok(ns.map(|ns| QName::new(ns, name)));
                } else {
                    return Ok(None);
                }
            } else if let Some(name) = multiname.local_name() {
                let qname = QName::new(ns.clone(), name);
                if self.has_property(&qname)? {
                    return Ok(Some(qname));
                }
            } else {
                return Ok(None);
            }
        }

        if let Some(proto) = self.proto() {
            return proto.resolve_multiname(multiname);
        }

        Ok(None)
    }

    /// Given a local name, find the namespace it resides in, if any.
    ///
    /// The `Namespace` must not be `Namespace::Any`, as this function exists
    /// specifically resolve names in that namespace.
    ///
    /// Trait names will be resolve on class objects and object instances, but
    /// not prototypes. If you want to search a prototype's provided traits you
    /// must walk the prototype chain using `resolve_any_trait`.
    fn resolve_any(self, local_name: AvmString<'gc>) -> Result<Option<Namespace<'gc>>, Error>;

    /// Given a local name of a trait, find the namespace it resides in, if any.
    ///
    /// This function only works for names which are trait properties, not
    /// dynamic or prototype properties. Furthermore, instance prototypes *will*
    /// resolve trait names here, contrary to their behavior in `resolve_any.`
    fn resolve_any_trait(self, local_name: AvmString<'gc>)
        -> Result<Option<Namespace<'gc>>, Error>;

    /// Indicates whether or not a property exists on an object.
    fn has_property(self, name: &QName<'gc>) -> Result<bool, Error> {
        if self.has_own_property(name)? {
            Ok(true)
        } else if let Some(proto) = self.proto() {
            Ok(proto.has_own_property(name)?)
        } else {
            Ok(false)
        }
    }

    /// Indicates whether or not a property or trait exists on an object and is
    /// not part of the prototype chain.
    fn has_own_property(self, name: &QName<'gc>) -> Result<bool, Error>;

    /// Returns true if an object has one or more traits of a given name.
    fn has_trait(self, name: &QName<'gc>) -> Result<bool, Error>;

    /// Check if a particular object contains a virtual getter by the given
    /// name.
    fn has_own_virtual_getter(self, name: &QName<'gc>) -> bool;

    /// Check if a particular object contains a virtual setter by the given
    /// name.
    fn has_own_virtual_setter(self, name: &QName<'gc>) -> bool;

    /// Indicates whether or not a property is overwritable.
    fn is_property_overwritable(
        self,
        gc_context: MutationContext<'gc, '_>,
        _name: &QName<'gc>,
    ) -> bool;

    /// Indicates whether or not a property is final.
    fn is_property_final(self, _name: &QName<'gc>) -> bool;

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, name: &QName<'gc>) -> bool;

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>>;

    /// Change the `__proto__` on this object.
    ///
    /// This method primarily exists so that the global scope that player
    /// globals loads into can be created before its superclasses are. It
    /// should be used sparingly, if at all.
    fn set_proto(self, mc: MutationContext<'gc, '_>, proto: Object<'gc>);

    /// Retrieve a given enumerable name by index.
    ///
    /// Enumerants are listed by index, starting from zero. A value of `None`
    /// indicates that no enumerant with that index, or any greater index,
    /// exists. (In other words, it means stop.)
    ///
    /// Objects are responsible for maintaining a consistently ordered and
    /// indexed list of enumerable names which can be queried by this
    /// mechanism.
    fn get_enumerant_name(&self, index: u32) -> Option<QName<'gc>>;

    /// Determine if a property is currently enumerable.
    ///
    /// Properties that do not exist are also not enumerable.
    fn property_is_enumerable(&self, name: &QName<'gc>) -> bool;

    /// Mark a dynamic property on this object as enumerable.
    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: &QName<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error>;

    /// Install a method (or any other non-slot value) on an object.
    fn install_method(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    );

    /// Install a getter method on an object property.
    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) -> Result<(), Error>;

    /// Install a setter method on an object property.
    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        disp_id: u32,
        function: Object<'gc>,
        is_final: bool,
    ) -> Result<(), Error>;

    /// Install a dynamic or built-in value property on an object.
    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        value: Value<'gc>,
    ) -> Result<(), Error>;

    /// Install a slot on an object property.
    fn install_slot(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        id: u32,
        value: Value<'gc>,
        is_final: bool,
    );

    /// Install a const on an object property.
    fn install_const(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        id: u32,
        value: Value<'gc>,
        is_final: bool,
    );

    /// Install all instance traits provided by a class.
    ///
    /// This method will also install superclass instance traits first. By
    /// calling this method with the lowest class in the chain, you will ensure
    /// all instance traits are installed.
    ///
    /// Read the documentation for `install_trait` to learn more about exactly
    /// how traits are instantiated.
    fn install_instance_traits(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        from_class_object: Object<'gc>,
    ) -> Result<(), Error> {
        if let Some(superclass_object) = from_class_object.superclass_object() {
            self.install_instance_traits(activation, superclass_object)?;
        }

        if let Some(class) = from_class_object.as_class() {
            self.install_traits(activation, class.read().instance_traits())?;
        }

        Ok(())
    }

    /// Install a list of traits into this object.
    ///
    /// This function should be called immediately after object allocation and
    /// before any constructors have a chance to run.
    ///
    /// Read the documentation for `install_trait` to learn more about exactly
    /// how traits are instantiated.
    fn install_traits(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        traits: &[Trait<'gc>],
    ) -> Result<(), Error> {
        for trait_entry in traits {
            self.install_trait(activation, trait_entry)?;
        }

        Ok(())
    }

    /// Install a single trait into this object.
    ///
    /// This function should be called immediately after object allocation and
    /// before any constructors have a chance to run. It should also only be
    /// called once per name and/or slot ID, as reinstalling a trait may unset
    /// already set properties.
    ///
    /// Class and function traits are *not* instantiated at installation time.
    /// Instead, installing such traits is treated as installing a const with
    /// `undefined` as its value.
    ///
    /// All traits that are instantiated at install time will be instantiated
    /// with this object's current scope stack and this object as a bound
    /// receiver.
    ///
    /// The value of the trait at the time of installation will be returned
    /// here, or `undefined` for classes and functions.
    fn install_trait(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        trait_entry: &Trait<'gc>,
    ) -> Result<Value<'gc>, Error> {
        let receiver = (*self).into();
        let scope = self.get_scope();
        let trait_name = trait_entry.name().clone();

        if trait_entry.is_override() && !self.has_own_property(&trait_name)? {
            return Err(format!(
                "Attempted to override property {:?}, which is not already defined",
                trait_name
            )
            .into());
        }

        //AS3 considers the setter and getter half of a final property to be
        //separate from one another and *not* overriding each other, which can
        //cause false verify errors if we don't exempt this particular case.
        //
        //TODO: We should actually check to see *what* this trait is overriding.
        //TODO: We should also tighten the override check above using the same
        //rationale.
        let is_final = trait_entry.is_final();
        let is_second_half_of_property = (self.has_own_virtual_getter(&trait_name)
            && !self.has_own_virtual_setter(&trait_name))
            || (!self.has_own_virtual_getter(&trait_name)
                && self.has_own_virtual_setter(&trait_name));
        let is_overriding_final =
            self.has_own_property(&trait_name)? && self.is_property_final(&trait_name);
        if is_overriding_final && !is_second_half_of_property {
            return Err(format!(
                "Attempted to override property {:?}, which is final",
                trait_name
            )
            .into());
        }

        avm_debug!(
            activation.avm2(),
            "Installing trait {:?} of kind {:?}",
            trait_name,
            trait_entry.kind()
        );

        match trait_entry.kind() {
            TraitKind::Slot {
                slot_id,
                default_value,
                ..
            } => {
                let value = default_value.clone().unwrap_or(Value::Undefined);
                self.install_slot(
                    activation.context.gc_context,
                    trait_name,
                    *slot_id,
                    value.clone(),
                    is_final,
                );

                Ok(value)
            }
            TraitKind::Method {
                disp_id, method, ..
            } => {
                let function =
                    FunctionObject::from_method(activation, method.clone(), scope, Some(receiver));
                self.install_method(
                    activation.context.gc_context,
                    trait_name,
                    *disp_id,
                    function,
                    is_final,
                );

                Ok(function.into())
            }
            TraitKind::Getter {
                disp_id, method, ..
            } => {
                let function =
                    FunctionObject::from_method(activation, method.clone(), scope, Some(receiver));
                self.install_getter(
                    activation.context.gc_context,
                    trait_name,
                    *disp_id,
                    function,
                    is_final,
                )?;

                Ok(function.into())
            }
            TraitKind::Setter {
                disp_id, method, ..
            } => {
                let function =
                    FunctionObject::from_method(activation, method.clone(), scope, Some(receiver));
                self.install_setter(
                    activation.context.gc_context,
                    trait_name,
                    *disp_id,
                    function,
                    is_final,
                )?;

                Ok(function.into())
            }
            TraitKind::Class { slot_id, .. } => {
                self.install_const(
                    activation.context.gc_context,
                    trait_name,
                    *slot_id,
                    Value::Undefined,
                    is_final,
                );

                Ok(Value::Undefined)
            }
            TraitKind::Function { slot_id, .. } => {
                self.install_const(
                    activation.context.gc_context,
                    trait_name,
                    *slot_id,
                    Value::Undefined,
                    is_final,
                );

                Ok(Value::Undefined)
            }
            TraitKind::Const {
                slot_id,
                default_value,
                ..
            } => {
                let value = default_value.clone().unwrap_or(Value::Undefined);
                self.install_const(
                    activation.context.gc_context,
                    trait_name,
                    *slot_id,
                    value.clone(),
                    is_final,
                );

                Ok(value)
            }
        }
    }

    /// Call the object.
    fn call(
        self,
        _reciever: Option<Object<'gc>>,
        _arguments: &[Value<'gc>],
        _activation: &mut Activation<'_, 'gc, '_>,
        _subclass_object: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        Err("Object is not callable".into())
    }

    /// Call the instance initializer.
    fn call_init(
        self,
        _reciever: Option<Object<'gc>>,
        _arguments: &[Value<'gc>],
        _activation: &mut Activation<'_, 'gc, '_>,
        _subclass_object: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        Err("Object is not a Class".into())
    }

    /// Call the instance's native initializer.
    ///
    /// The native initializer is called when native code needs to construct an
    /// object, or when supercalling into a parent constructor (as there are
    /// classes that cannot be constructed but can be supercalled).
    fn call_native_init(
        self,
        _reciever: Option<Object<'gc>>,
        _arguments: &[Value<'gc>],
        _activation: &mut Activation<'_, 'gc, '_>,
        _subclass_object: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        Err("Object is not a Class".into())
    }

    /// Supercall a method defined in this class.
    ///
    /// This is intended to be called on the class object that is the
    /// superclass of the one that defined the currently called property. If no
    /// such superclass exists, you should use the class object for the
    /// reciever's actual type (i.e. the lowest in the chain). This ensures
    /// that repeated supercalls to the same method will call parent and
    /// grandparent methods, and so on.
    ///
    /// If no method exists with the given name, this falls back to calling a
    /// property of the `reciever`. This fallback only triggers if the property
    /// is associated with a trait. Dynamic properties will still error out.
    ///
    /// This function will search through the class object tree starting from
    /// this class up to `Object` for a method trait with the given name. If it
    /// is found, it will be called with the reciever and arguments you
    /// provided, as if it were defined on the target instance object.
    ///
    /// The class that defined the method being called will also be provided to
    /// the `Activation` that the method runs on so that further supercalls
    /// will work as expected.
    fn supercall_method(
        self,
        name: &QName<'gc>,
        reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let superclass_object = self.find_class_for_trait(name)?.ok_or_else(|| {
            format!(
                "Attempted to supercall method {:?}, which does not exist",
                name
            )
        })?;
        let mut class_traits = Vec::new();
        superclass_object
            .as_class()
            .unwrap()
            .read()
            .lookup_instance_traits(name, &mut class_traits)?;
        let base_trait = class_traits
            .iter()
            .find(|t| matches!(t.kind(), TraitKind::Method { .. }));
        let scope = superclass_object.get_scope();

        if let Some(TraitKind::Method { method, .. }) = base_trait.map(|b| b.kind()) {
            let callee = FunctionObject::from_method(activation, method.clone(), scope, reciever);

            callee.call(reciever, arguments, activation, Some(superclass_object))
        } else {
            if let Some(reciever) = reciever {
                if let Ok(callee) = reciever
                    .get_property(reciever, name, activation)
                    .and_then(|v| v.coerce_to_object(activation))
                {
                    return callee.call(Some(reciever), arguments, activation, None);
                }
            }

            Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                name
            )
            .into())
        }
    }

    /// Supercall a getter defined in this class.
    ///
    /// This is intended to be called on the class object that is the
    /// superclass of the one that defined the currently called property. If no
    /// such superclass exists, you should use the class object for the
    /// reciever's actual type (i.e. the lowest in the chain). This ensures
    /// that repeated supercalls to the same getter will call parent and
    /// grandparent getters, and so on.
    ///
    /// If no getter exists with the given name, this falls back to getting a
    /// property of the `reciever`. This fallback only triggers if the property
    /// is associated with a trait. Dynamic properties will still error out.
    ///
    /// This function will search through the class object tree starting from
    /// this class up to `Object` for a getter trait with the given name. If it
    /// is found, it will be called with the reciever you provided, as if it
    /// were defined on the target instance object.
    ///
    /// The class that defined the getter being called will also be provided to
    /// the `Activation` that the getter runs on so that further supercalls
    /// will work as expected.
    fn supercall_getter(
        self,
        name: &QName<'gc>,
        reciever: Option<Object<'gc>>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let superclass_object = self.find_class_for_trait(name)?.ok_or_else(|| {
            format!(
                "Attempted to supercall getter {:?}, which does not exist",
                name
            )
        })?;
        let mut class_traits = Vec::new();
        superclass_object
            .as_class()
            .unwrap()
            .read()
            .lookup_instance_traits(name, &mut class_traits)?;
        let base_trait = class_traits
            .iter()
            .find(|t| matches!(t.kind(), TraitKind::Getter { .. }));
        let scope = superclass_object.get_scope();

        if let Some(TraitKind::Getter { method, .. }) = base_trait.map(|b| b.kind()) {
            let callee = FunctionObject::from_method(activation, method.clone(), scope, reciever);

            callee.call(reciever, &[], activation, Some(superclass_object))
        } else if let Some(reciever) = reciever {
            reciever.get_property(reciever, name, activation)
        } else {
            Err(format!(
                "Attempted to supercall getter {:?}, which does not exist",
                name
            )
            .into())
        }
    }

    /// Supercall a setter defined in this class.
    ///
    /// This is intended to be called on the class object that is the
    /// superclass of the one that defined the currently called property. If no
    /// such superclass exists, you should use the class object for the
    /// reciever's actual type (i.e. the lowest in the chain). This ensures
    /// that repeated supercalls to the same setter will call parent and
    /// grandparent setter, and so on.
    ///
    /// If no setter exists with the given name, this falls back to setting a
    /// property of the `reciever`. This fallback only triggers if the property
    /// is associated with a trait. Dynamic properties will still error out.
    ///
    /// This function will search through the class object tree starting from
    /// this class up to `Object` for a setter trait with the given name. If it
    /// is found, it will be called with the reciever and value you provided,
    /// as if it were defined on the target instance object.
    ///
    /// The class that defined the setter being called will also be provided to
    /// the `Activation` that the setter runs on so that further supercalls
    /// will work as expected.
    fn supercall_setter(
        self,
        name: &QName<'gc>,
        value: Value<'gc>,
        reciever: Option<Object<'gc>>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let superclass_object = self.find_class_for_trait(name)?.ok_or_else(|| {
            format!(
                "Attempted to supercall setter {:?}, which does not exist",
                name
            )
        })?;
        let mut class_traits = Vec::new();
        superclass_object
            .as_class()
            .unwrap()
            .read()
            .lookup_instance_traits(name, &mut class_traits)?;
        let base_trait = class_traits
            .iter()
            .find(|t| matches!(t.kind(), TraitKind::Setter { .. }));
        let scope = superclass_object.get_scope();

        if let Some(TraitKind::Setter { method, .. }) = base_trait.map(|b| b.kind()) {
            let callee = FunctionObject::from_method(activation, method.clone(), scope, reciever);

            callee.call(reciever, &[value], activation, Some(superclass_object))?;

            Ok(())
        } else if let Some(mut reciever) = reciever {
            reciever.set_property(reciever, name, value, activation)
        } else {
            Err(format!(
                "Attempted to supercall setter {:?}, which does not exist",
                name
            )
            .into())
        }
    }

    /// Construct a Class or Function and return an instance of it.
    ///
    /// As the first step in object construction, the `construct` method is
    /// called on the class object to produce an instance of that class. The
    /// constructor is then expected to perform the following steps, in order:
    ///
    /// 1. Allocate the instance object. For ES4 classes, the class's instance
    /// allocator is used to allocate the object. ES3-style classes use the
    /// prototype to derive instances.
    /// 2. Associate the instance object with the class's explicit `prototype`.
    /// 3. If the class has instance traits, install them at this time.
    /// 4. Call the constructor method with the newly-allocated object as
    /// reciever. For ES3 classes, this is just the function's associated
    /// method.
    /// 5. Yield the allocated object. (The return values of constructors are
    /// ignored.)
    fn construct(
        self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Err("Object is not constructable".into())
    }

    /// Construct a host object prototype of some kind and return it.
    ///
    /// This is called specifically to allocate old-style ES3 instances. The
    /// returned object should have no properties upon it.
    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error>;

    /// Construct a parameterization of this particular type and return it.
    ///
    /// This is called specifically to parameterize generic types, of which
    /// only one exists: `Vector`. When `Vector` is applied with a given
    /// parameter, a new type is returned which can be used to construct
    /// `Vector`s of that type.
    ///
    /// If the object is not a parameterized type, this yields an error. In
    /// practice, this means only `Vector` can use this method. Parameters must
    /// be class objects or `null`, which indicates any type.
    ///
    /// When a given type is parameterized with the same parameters multiple
    /// times, each application must return the same object. This is because
    /// each application has a separate prototype that accepts dynamic
    /// parameters.
    fn apply(
        &self,
        _activation: &mut Activation<'_, 'gc, '_>,
        _params: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Err("Not a parameterized type".into())
    }

    /// Determine the type of primitive coercion this object would prefer, in
    /// the case that there is no obvious reason to prefer one type over the
    /// other.
    ///
    /// All native ECMAScript objects prefer numerical coercions, except `Date`,
    /// which wants string coercions.
    fn default_hint(&self) -> Hint {
        Hint::Number
    }

    /// Implement the result of calling `Object.prototype.toString` on this
    /// object class.
    ///
    /// `toString` is a method used to request an object be coerced to a string
    /// value. The default implementation is stored here. User-specified string
    /// coercions happen by defining `toString` in a downstream class or
    /// prototype; this is then picked up by the VM runtime when doing
    /// coercions.
    fn to_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        let class_name = self
            .as_class()
            .map(|c| c.read().name().local_name())
            .unwrap_or_else(|| "Object".into());

        Ok(AvmString::new(mc, format!("[object {}]", class_name)).into())
    }

    /// Implement the result of calling `Object.prototype.toLocaleString` on this
    /// object class.
    ///
    /// `toLocaleString` is a method used to request an object be coerced to a
    /// locale-dependent string value. The default implementation appears to
    /// generate a debug-style string based on the name of the class this
    /// object is, in the format of `[object Class]` (where `Class` is the name
    /// of the class that created this object).
    fn to_locale_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        let class_name = self
            .as_class()
            .map(|c| c.read().name().local_name())
            .unwrap_or_else(|| "Object".into());

        Ok(AvmString::new(mc, format!("[object {}]", class_name)).into())
    }

    /// Implement the result of calling `Object.prototype.valueOf` on this
    /// object class.
    ///
    /// `valueOf` is a method used to request an object be coerced to a
    /// primitive value. Typically, this would be a number of some kind.
    fn value_of(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error>;

    /// Enumerate all interfaces implemented by this class object.
    ///
    /// Non-classes do not implement interfaces.
    fn interfaces(&self) -> Vec<Object<'gc>>;

    /// Set the interface list for this class object.
    fn set_interfaces(&self, gc_context: MutationContext<'gc, '_>, iface_list: Vec<Object<'gc>>);

    /// Determine if this object is an instance of a given type.
    ///
    /// This uses the ES3 definition of instance, which walks the prototype
    /// chain. For the ES4 definition of instance, use `is_of_type`, which uses
    /// the class object chain and accounts for interfaces.
    ///
    /// The given object should be the class object for the given type we are
    /// checking against this object. Its prototype will be extracted and
    /// searched in the prototype chain of this object.
    fn is_instance_of(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        class: Object<'gc>,
    ) -> Result<bool, Error> {
        let type_proto = class
            .get_property(class, &QName::dynamic_name("prototype"), activation)?
            .coerce_to_object(activation)?;

        self.has_prototype_in_chain(type_proto)
    }

    /// Determine if this object has a given prototype in its prototype chain.
    ///
    /// The given object `type_proto` should be the prototype we are checking
    /// against this object.
    fn has_prototype_in_chain(&self, type_proto: Object<'gc>) -> Result<bool, Error> {
        let mut my_proto = self.proto();

        //TODO: Is it a verification error to do `obj instanceof bare_object`?
        while let Some(proto) = my_proto {
            if Object::ptr_eq(proto, type_proto) {
                return Ok(true);
            }

            my_proto = proto.proto()
        }

        Ok(false)
    }

    /// Determine if this object is an instance of a given type.
    ///
    /// This uses the ES4 definition of instance, which walks the class object
    /// chain and accounts for interfaces. For the ES3 definition of instance,
    /// use `is_instance_of`, which uses the prototype chain.
    ///
    /// The given object should be the class object for the given type we are
    /// checking against this object.
    fn is_of_type(
        &self,
        test_class: Object<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<bool, Error> {
        let my_class = self.as_class_object();

        // ES3 objects are not class instances but are still treated as
        // instances of Object, which is an ES4 class.
        if my_class.is_none() && Object::ptr_eq(test_class, activation.avm2().classes().object) {
            Ok(true)
        } else if let Some(my_class) = my_class {
            my_class.has_class_in_chain(test_class, activation)
        } else {
            Ok(false)
        }
    }

    /// Determine if this class has a given type in its superclass chain.
    ///
    /// The given object `test_class` should be either a superclass or
    /// interface we are checking against this class.
    ///
    /// To test if a class *instance* is of a given type, see is_of_type.
    fn has_class_in_chain(
        &self,
        test_class: Object<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<bool, Error> {
        let my_class: Object<'gc> = (*self).into();
        let mut my_class = Some(my_class);

        while let Some(class) = my_class {
            if Object::ptr_eq(class, test_class) {
                return Ok(true);
            }

            for interface in class.interfaces() {
                if Object::ptr_eq(interface, test_class) {
                    return Ok(true);
                }
            }

            if let (Some(my_param), Some(test_param)) =
                (class.as_class_params(), test_class.as_class_params())
            {
                let mut are_all_params_coercible = true;

                are_all_params_coercible &= match (my_param, test_param) {
                    (Some(my_param), Some(test_param)) => {
                        my_param.has_class_in_chain(test_param, activation)?
                    }
                    (None, Some(_)) => false,
                    _ => true,
                };

                if are_all_params_coercible {
                    return Ok(true);
                }
            }

            my_class = class.superclass_object()
        }

        Ok(false)
    }

    /// Get a raw pointer value for this object.
    fn as_ptr(&self) -> *const ObjectPtr;

    /// Get this object's `Class`, if it has one.
    fn as_class(&self) -> Option<GcCell<'gc, Class<'gc>>>;

    /// Get this object's class object, if it has one.
    fn as_class_object(&self) -> Option<Object<'gc>>;

    /// Get the parameters of this class object, if present.
    ///
    /// Only specialized generic classes will yield their parameters.
    fn as_class_params(&self) -> Option<Option<Object<'gc>>> {
        None
    }

    /// Associate the object with a particular class object.
    ///
    /// This turns the object into an instance of that class. It should only be
    /// used in situations where the object cannot be made an instance of the
    /// class at allocation time, such as during early runtime setup.
    fn set_class_object(self, mc: MutationContext<'gc, '_>, class_object: Object<'gc>);

    /// Get the superclass object of this object.
    fn superclass_object(self) -> Option<Object<'gc>> {
        None
    }

    /// Get this class's instance allocator.
    fn instance_allocator(self) -> Option<AllocatorFn> {
        None
    }

    /// Get this object's `Executable`, if it has one.
    fn as_executable(&self) -> Option<Executable<'gc>> {
        None
    }

    /// Unwrap this object's `Namespace`, if the object is a boxed namespace.
    fn as_namespace(&self) -> Option<Ref<Namespace<'gc>>> {
        None
    }

    /// Unwrap this object as array storage.
    fn as_array_storage(&self) -> Option<Ref<ArrayStorage<'gc>>> {
        None
    }

    /// Unwrap this object as bytearray.
    fn as_bytearray(&self) -> Option<Ref<ByteArrayStorage>> {
        None
    }

    fn as_bytearray_mut(&self, _mc: MutationContext<'gc, '_>) -> Option<RefMut<ByteArrayStorage>> {
        None
    }

    fn as_bytearray_object(&self) -> Option<ByteArrayObject<'gc>> {
        None
    }

    /// Unwrap this object as mutable array storage.
    fn as_array_storage_mut(
        &self,
        _mc: MutationContext<'gc, '_>,
    ) -> Option<RefMut<ArrayStorage<'gc>>> {
        None
    }

    /// Unwrap this object as vector storage.
    fn as_vector_storage(&self) -> Option<Ref<VectorStorage<'gc>>> {
        None
    }

    /// Unwrap this object as mutable vector storage.
    fn as_vector_storage_mut(
        &self,
        _mc: MutationContext<'gc, '_>,
    ) -> Option<RefMut<VectorStorage<'gc>>> {
        None
    }

    /// Get this object's `DisplayObject`, if it has one.
    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        None
    }

    /// Associate this object with a display object, if it can support such an
    /// association.
    ///
    /// If not, then this function does nothing.
    fn init_display_object(&self, _mc: MutationContext<'gc, '_>, _obj: DisplayObject<'gc>) {}

    /// Unwrap this object as an ApplicationDomain.
    fn as_application_domain(&self) -> Option<Domain<'gc>> {
        None
    }

    /// Unwrap this object as an event.
    fn as_event(&self) -> Option<Ref<Event<'gc>>> {
        None
    }

    /// Unwrap this object as a mutable event.
    fn as_event_mut(&self, _mc: MutationContext<'gc, '_>) -> Option<RefMut<Event<'gc>>> {
        None
    }

    /// Unwrap this object as a list of event handlers.
    fn as_dispatch(&self) -> Option<Ref<DispatchList<'gc>>> {
        None
    }

    /// Unwrap this object as a mutable list of event handlers.
    fn as_dispatch_mut(&self, _mc: MutationContext<'gc, '_>) -> Option<RefMut<DispatchList<'gc>>> {
        None
    }

    /// Unwrap this object as an immutable primitive value.
    ///
    /// This function should not be called in cases where a normal `Value`
    /// coercion would do. It *only* accounts for boxed primitives, and not
    /// `valueOf`.
    fn as_primitive(&self) -> Option<Ref<Value<'gc>>> {
        None
    }

    /// Unwrap this object as a mutable primitive value.
    fn as_primitive_mut(&self, _mc: MutationContext<'gc, '_>) -> Option<RefMut<Value<'gc>>> {
        None
    }

    /// Unwrap this object as a regexp.
    fn as_regexp(&self) -> Option<Ref<RegExp<'gc>>> {
        None
    }

    /// Unwrap this object as a mutable regexp.
    fn as_regexp_mut(&self, _mc: MutationContext<'gc, '_>) -> Option<RefMut<RegExp<'gc>>> {
        None
    }

    /// Unwrap this object's loader stream
    fn as_loader_stream(&self) -> Option<Ref<LoaderStream<'gc>>> {
        None
    }

    /// Unwrap this object's sound handle.
    fn as_sound(self) -> Option<SoundHandle> {
        None
    }

    /// Associate the object with a particular sound handle.
    ///
    /// This does nothing if the object is not a sound.
    fn set_sound(self, _mc: MutationContext<'gc, '_>, _sound: SoundHandle) {}

    /// Unwrap this object's sound instance handle.
    fn as_sound_instance(self) -> Option<SoundInstanceHandle> {
        None
    }

    /// Associate the object with a particular sound instance handle.
    ///
    /// This does nothing if the object is not a sound channel.
    fn set_sound_instance(self, _mc: MutationContext<'gc, '_>, _sound: SoundInstanceHandle) {}

    /// Unwrap this object's bitmap data
    fn as_bitmap_data(&self) -> Option<GcCell<'gc, BitmapData>> {
        None
    }

    /// Initialize the bitmap data in this object, if it's capable of
    /// supporting said data
    fn init_bitmap_data(
        &self,
        _mc: MutationContext<'gc, '_>,
        _new_bitmap: GcCell<'gc, BitmapData>,
    ) {
    }
}

pub enum ObjectPtr {}

impl<'gc> Object<'gc> {
    pub fn ptr_eq(a: Object<'gc>, b: Object<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}

impl<'gc> PartialEq for Object<'gc> {
    fn eq(&self, other: &Self) -> bool {
        Object::ptr_eq(*self, *other)
    }
}

impl<'gc> Eq for Object<'gc> {}

impl<'gc> Hash for Object<'gc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}
