//! AVM2 objects.

use crate::avm2::function::{Avm2ClassEntry, Avm2MethodEntry, Executable, FunctionObject};
use crate::avm2::names::{Multiname, Namespace, QName};
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
    ) -> Result<Value<'gc>, Error>;

    /// Retrieve a property by it's QName.
    fn get_property(
        &mut self,
        reciever: Object<'gc>,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        if !self.has_instantiated_property(name) {
            for abc_trait in self.get_trait(name)? {
                self.install_trait(avm, context, &abc_trait, reciever)?;
            }
        }

        let has_no_getter = self.has_own_virtual_setter(name) && !self.has_own_virtual_getter(name);

        if self.has_own_property(name)? && !has_no_getter {
            return self.get_property_local(reciever, name, avm, context);
        }

        if let Some(mut proto) = self.proto() {
            return proto.get_property(reciever, name, avm, context);
        }

        Ok(Value::Undefined)
    }

    /// Retrieve the base prototype that a particular QName trait is defined in.
    ///
    /// This function returns `None` for non-trait properties, such as actually
    /// defined prototype methods for ES3-style classes.
    fn get_base_proto(self, name: &QName) -> Result<Option<Object<'gc>>, Error> {
        if self.provides_trait(name)? {
            return Ok(Some(self.into()));
        }

        if let Some(proto) = self.proto() {
            return proto.get_base_proto(name);
        }

        Ok(None)
    }

    /// Set a property on this specific object.
    fn set_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Set a property by it's QName.
    fn set_property(
        &mut self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if !self.has_instantiated_property(name) {
            for abc_trait in self.get_trait(name)? {
                self.install_trait(avm, context, &abc_trait, reciever)?;
            }
        }

        if self.has_own_virtual_setter(name) {
            return self.set_property_local(reciever, name, value, avm, context);
        }

        let mut proto = self.proto();
        while let Some(mut my_proto) = proto {
            //NOTE: This only works because we validate ahead-of-time that
            //we're calling a virtual setter. If you call `set_property` on
            //a non-virtual you will actually alter the prototype.
            if my_proto.has_own_virtual_setter(name) {
                return my_proto.set_property(reciever, name, value, avm, context);
            }

            proto = my_proto.proto();
        }

        reciever.set_property_local(reciever, name, value, avm, context)
    }

    /// Init a property on this specific object.
    fn init_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error>;

    /// Init a property by it's QName.
    fn init_property(
        &mut self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if !self.has_instantiated_property(name) {
            for abc_trait in self.get_trait(name)? {
                self.install_trait(avm, context, &abc_trait, reciever)?;
            }
        }

        if self.has_own_virtual_setter(name) {
            return self.init_property_local(reciever, name, value, avm, context);
        }

        let mut proto = self.proto();
        while let Some(mut my_proto) = proto {
            //NOTE: This only works because we validate ahead-of-time that
            //we're calling a virtual setter. If you call `set_property` on
            //a non-virtual you will actually alter the prototype.
            if my_proto.has_own_virtual_setter(name) {
                return my_proto.init_property(reciever, name, value, avm, context);
            }

            proto = my_proto.proto();
        }

        reciever.init_property_local(reciever, name, value, avm, context)
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

    /// Retrieves a trait entry by name.
    ///
    /// This function returns `None` if no such trait exists, or the object
    /// does not have traits. It returns `Err` if *any* trait in the object is
    /// malformed in some way.
    fn get_trait(self, name: &QName) -> Result<Vec<AbcTrait>, Error>;

    /// Populate a list of traits that this object provides.
    ///
    /// This function yields traits for class constructors and prototypes, but
    /// not instances. For resolving traits for normal `TObject` methods, use
    /// `get_trait` and `has_trait` as it will tell you if the current object
    /// has a given trait.
    fn get_provided_trait(
        &self,
        name: &QName,
        known_traits: &mut Vec<AbcTrait>,
    ) -> Result<(), Error>;

    /// Retrieves the scope chain of the object at time of it's creation.
    ///
    /// The scope chain is used to determine the starting scope stack when an
    /// object is called, as well as any class methods on the object.
    /// Non-method functions and prototype functions (ES3 methods) do not use
    /// this scope chain.
    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>>;

    /// Retrieves the ABC file that this object, or it's class, was defined in.
    ///
    /// Objects that were not defined in an ABC file or created from a class
    /// defined in an ABC file will return `None`. This can happen for things
    /// such as object or array literals. If this object does not have an ABC
    /// file, then it must also not have traits.
    fn get_abc(self) -> Option<Rc<AbcFile>>;

    /// Resolve a multiname into a single QName, if any of the namespaces
    /// match.
    fn resolve_multiname(self, multiname: &Multiname) -> Result<Option<QName>, Error> {
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
            return Ok(proto.resolve_multiname(multiname)?);
        }

        Ok(None)
    }

    /// Given a local name, find the namespace it resides in, if any.
    ///
    /// The `Namespace` must not be `Namespace::Any`, as this function exists
    /// specifically resolve names in that namespace.
    ///
    /// Trait names will be resolve on class constructors and object instances,
    /// but not prototypes. If you want to search a prototype's provided traits
    /// you must walk the prototype chain using `resolve_any_trait`.
    fn resolve_any(self, local_name: &str) -> Result<Option<Namespace>, Error>;

    /// Given a local name of a trait, find the namespace it resides in, if any.
    ///
    /// This function only works for names which are trait properties, not
    /// dynamic or prototype properties. Furthermore, instance prototypes *will*
    /// resolve trait names here, contrary to their behavior in `resolve_any.`
    fn resolve_any_trait(self, local_name: &str) -> Result<Option<Namespace>, Error>;

    /// Indicates whether or not a property exists on an object.
    fn has_property(self, name: &QName) -> Result<bool, Error> {
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
    fn has_own_property(self, name: &QName) -> Result<bool, Error>;

    /// Returns true if an object has one or more traits of a given name.
    fn has_trait(self, name: &QName) -> Result<bool, Error>;

    /// Returns true if an object is part of a class that defines a trait of a
    /// given name on itself (as opposed to merely inheriting a superclass
    /// trait.)
    fn provides_trait(self, name: &QName) -> Result<bool, Error>;

    /// Indicates whether or not a property or *instantiated* trait exists on
    /// an object and is not part of the prototype chain.
    ///
    /// Unlike `has_own_property`, this will not yield `true` for traits this
    /// object can have but has not yet instantiated.
    fn has_instantiated_property(self, name: &QName) -> bool;

    /// Check if a particular object contains a virtual getter by the given
    /// name.
    fn has_own_virtual_getter(self, name: &QName) -> bool;

    /// Check if a particular object contains a virtual setter by the given
    /// name.
    fn has_own_virtual_setter(self, name: &QName) -> bool;

    /// Indicates whether or not a property is overwritable.
    fn is_property_overwritable(self, gc_context: MutationContext<'gc, '_>, _name: &QName) -> bool;

    /// Delete a named property from the object.
    ///
    /// Returns false if the property cannot be deleted.
    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, name: &QName) -> bool;

    /// Retrieve the `__proto__` of a given object.
    ///
    /// The proto is another object used to resolve methods across a class of
    /// multiple objects. It should also be accessible as `__proto__` from
    /// `get`.
    fn proto(&self) -> Option<Object<'gc>>;

    /// Retrieve a given enumerable name by index.
    ///
    /// Enumerants are listed by index, starting from zero. A value of `None`
    /// indicates that no enumerant with that index, or any greater index,
    /// exists. (In other words, it means stop.)
    ///
    /// Objects are responsible for maintaining a consistently ordered and
    /// indexed list of enumerable names which can be queried by this
    /// mechanism.
    fn get_enumerant_name(&self, index: u32) -> Option<QName>;

    /// Determine if a property is currently enumerable.
    ///
    /// Properties that do not exist are also not enumerable.
    fn property_is_enumerable(&self, name: &QName) -> bool;

    /// Mark a dynamic property on this object as enumerable.
    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: &QName,
        is_enumerable: bool,
    ) -> Result<(), Error>;

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

    /// Install a trait from the current object.
    ///
    /// This function should only be called once, as reinstalling a trait may
    /// also unset already set properties. It may either be called immediately
    /// when the object is instantiated or lazily; this behavior is ostensibly
    /// controlled by the `lazy_init` flag provided to `load_abc`, but in
    /// practice every version of Flash and Animate uses lazy trait
    /// installation.
    ///
    /// The `reciever` property allows specifying the object that methods are
    /// bound to. It should always be `self` except when doing things with
    /// `super`, which needs to create bound methods pointing to a different
    /// object.
    fn install_trait(
        &mut self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        trait_entry: &AbcTrait,
        reciever: Object<'gc>,
    ) -> Result<(), Error> {
        let scope = self.get_scope();
        let abc: Result<Rc<AbcFile>, Error> = self.get_abc().ok_or_else(|| {
            "Object with traits must have an ABC file!"
                .to_string()
                .into()
        });
        self.install_foreign_trait(avm, context, abc?, trait_entry, scope, reciever)
    }

    /// Install a trait from anywyere.
    fn install_foreign_trait(
        &mut self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        abc: Rc<AbcFile>,
        trait_entry: &AbcTrait,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        reciever: Object<'gc>,
    ) -> Result<(), Error> {
        let fn_proto = avm.prototypes().function;
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
                let function = FunctionObject::from_abc_method(
                    context.gc_context,
                    method,
                    scope,
                    fn_proto,
                    Some(reciever),
                );
                self.install_method(context.gc_context, trait_name, *disp_id, function);
            }
            AbcTraitKind::Getter {
                disp_id, method, ..
            } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function = FunctionObject::from_abc_method(
                    context.gc_context,
                    method,
                    scope,
                    fn_proto,
                    Some(reciever),
                );
                self.install_getter(context.gc_context, trait_name, *disp_id, function)?;
            }
            AbcTraitKind::Setter {
                disp_id, method, ..
            } => {
                let method = Avm2MethodEntry::from_method_index(abc, method.clone()).unwrap();
                let function = FunctionObject::from_abc_method(
                    context.gc_context,
                    method,
                    scope,
                    fn_proto,
                    Some(reciever),
                );
                self.install_setter(context.gc_context, trait_name, *disp_id, function)?;
            }
            AbcTraitKind::Class { slot_id, class } => {
                let type_entry = Avm2ClassEntry::from_class_index(abc, class.clone()).unwrap();
                let super_name = QName::from_abc_multiname(
                    &type_entry.abc(),
                    type_entry.instance().super_name.clone(),
                )?;
                let super_class: Result<Object<'gc>, Error> = self
                    .get_property(reciever, &super_name, avm, context)?
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
                let mut function = FunctionObject::from_abc_method(
                    context.gc_context,
                    method,
                    scope,
                    fn_proto,
                    None,
                );
                let es3_proto = ScriptObject::object(context.gc_context, avm.prototypes().object);

                function.install_slot(
                    context.gc_context,
                    QName::new(Namespace::public_namespace(), "prototype"),
                    0,
                    es3_proto.into(),
                );
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
    ) -> Result<Value<'gc>, Error> {
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
    /// `construct`ed objects should instantiate instance traits of the class
    /// that this prototype represents.
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

    /// Construct a host object prototype of some kind and return it.
    ///
    /// This is called specifically to construct prototypes. The primary
    /// difference is that a new class and scope closure are defined here.
    /// Objects constructed from the new prototype should use that new class
    /// and scope closure when instantiating non-prototype traits.
    ///
    /// Unlike `construct`, `derive`d objects should *not* instantiate instance
    /// traits.
    fn derive(
        &self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        class: Avm2ClassEntry,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error>;

    /// Implement the result of calling `Object.prototype.toString` on this
    /// object class.
    ///
    /// `toString` is a method used to request an object be coerced to a string
    /// value. The default implementation is stored here. User-specified string
    /// coercions happen by defining `toString` in a downstream class or
    /// prototype; this is then picked up by the VM runtime when doing
    /// coercions.
    fn to_string(&self) -> Result<Value<'gc>, Error>;

    /// Implement the result of calling `Object.prototype.valueOf` on this
    /// object class.
    ///
    /// `valueOf` is a method used to request an object be coerced to a
    /// primitive value. Typically, this would be a number of some kind.
    fn value_of(&self) -> Result<Value<'gc>, Error>;

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
