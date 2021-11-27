//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Allocator, AllocatorFn, Class};
use crate::avm2::function::Executable;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::{scriptobject_allocator, ScriptObject, ScriptObjectData};
use crate::avm2::object::{Multiname, Object, ObjectPtr, TObject};
use crate::avm2::property_map::PropertyMap;
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// An Object which can be called to execute its function code.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct ClassObject<'gc>(GcCell<'gc, ClassObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct ClassObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The class associated with this class object.
    class: GcCell<'gc, Class<'gc>>,

    /// The captured scope that all class traits will use.
    class_scope: ScopeChain<'gc>,

    /// The captured scope that all instance traits will use.
    instance_scope: ScopeChain<'gc>,

    /// The base class of this one.
    ///
    /// If `None`, this class has no parent. In practice, this is only used for
    /// interfaces (at least by the AS3 compiler in Animate CC 2020.)
    superclass_object: Option<ClassObject<'gc>>,

    /// The instance allocator for this class.
    instance_allocator: Allocator,

    /// The instance constructor function
    constructor: Method<'gc>,

    /// The native instance constructor function
    native_constructor: Method<'gc>,

    /// The parameters of this specialized class.
    ///
    /// None flags that this class has not been specialized.
    ///
    /// An individual parameter of `None` signifies the parameter `*`, which is
    /// represented in AVM2 as `null` with regards to type application.
    params: Option<Option<ClassObject<'gc>>>,

    /// List of all applications of this class.
    ///
    /// Only applicable if this class is generic.
    ///
    /// It is legal to apply a type with the value `null`, which is represented
    /// as `None` here. AVM2 considers both applications to be separate
    /// classes, though we consider the parameter to be the class `Object` when
    /// we get a param of `null`.
    applications: HashMap<Option<ClassObject<'gc>>, ClassObject<'gc>>,

    /// Interfaces implemented by this class.
    interfaces: Vec<ClassObject<'gc>>,

    /// All traits present on instances of this class.
    ///
    /// This is a comprehensive list of all traits, including all superclass
    /// and interface traits, as well as the superclass that defined the trait.
    resolved_instance_traits: PropertyMap<'gc, Vec<(ClassObject<'gc>, Trait<'gc>)>>,

    /// All traits present on the class itself.
    resolved_class_traits: PropertyMap<'gc, Vec<Trait<'gc>>>,
}

impl<'gc> ClassObject<'gc> {
    /// Allocate the prototype for this class.
    ///
    /// This function is not used during the initialization of "early classes",
    /// i.e. `Object`, `Function`, and `Class`. Those classes and their
    /// prototypes are weaved together separately.
    ///
    /// The returned prototype will be an instance of `Object` (and thus not
    /// have this class's instance traits), but will be allocated by this
    /// class's instance allocator.
    fn allocate_prototype(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Object<'gc>, Error> {
        let instance_allocator = self.0.read().instance_allocator.clone();

        if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object
                .get_property(
                    superclass_object.into(),
                    &QName::new(Namespace::public(), "prototype").into(),
                    activation,
                )?
                .coerce_to_object(activation)?;

            //NOTE: If we do not use `instance_allocator` here, then Vector
            //enumeration will break.
            (instance_allocator.0)(activation.avm2().classes().object, base_proto, activation)
        } else {
            Ok(ScriptObject::bare_object(activation.context.gc_context))
        }
    }

    /// Construct a class.
    ///
    /// This function returns the class constructor object, which should be
    /// used in all cases where the type needs to be referred to. It's class
    /// initializer will be executed during this function call.
    ///
    /// `base_class` is allowed to be `None`, corresponding to a `null` value
    /// in the VM. This corresponds to no base class, and in practice appears
    /// to be limited to interfaces.
    pub fn from_class(
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Self, Error> {
        let class_object = Self::from_class_partial(activation, class, superclass_object)?;
        let class_proto = class_object.allocate_prototype(activation, superclass_object)?;

        class_object.link_prototype(activation, class_proto)?;

        let class_class = activation.avm2().classes().class;
        let class_class_proto = activation.avm2().prototypes().class;

        class_object.link_type(activation, class_class_proto, class_class);
        class_object.into_finished_class(activation)
    }

    /// Allocate a class but do not properly construct it.
    ///
    /// This function does the bare minimum to allocate classes, without taking
    /// any action that would require the existence of any other objects in the
    /// object graph. The resulting class will be a bare object and should not
    /// be used or presented to user code until you finish initializing it. You
    /// do that by calling `link_prototype`, `link_type`, and then
    /// `into_finished_class` in that order.
    ///
    /// This returns the class object directly (*not* an `Object`), to allow
    /// further manipulation of the class once it's dependent types have been
    /// allocated.
    pub fn from_class_partial(
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Self, Error> {
        let scope = activation.create_scopechain();
        if let Some(base_class) = superclass_object.map(|b| b.inner_class_definition()) {
            if base_class.read().is_final() {
                return Err(format!(
                    "Base class {:?} is final and cannot be extended",
                    base_class.read().name().local_name()
                )
                .into());
            }

            if base_class.read().is_interface() {
                return Err(format!(
                    "Base class {:?} is an interface and cannot be extended",
                    base_class.read().name().local_name()
                )
                .into());
            }
        }

        let instance_allocator = class
            .read()
            .instance_allocator()
            .or_else(|| superclass_object.and_then(|c| c.instance_allocator()))
            .unwrap_or(scriptobject_allocator);

        let class_object = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(None, None),
                class,
                class_scope: scope,
                instance_scope: scope,
                superclass_object,
                instance_allocator: Allocator(instance_allocator),
                constructor: class.read().instance_init(),
                native_constructor: class.read().native_instance_init(),
                params: None,
                applications: HashMap::new(),
                interfaces: Vec::new(),
                resolved_instance_traits: PropertyMap::new(),
                resolved_class_traits: PropertyMap::new(),
            },
        ));

        // instance scope = [..., class object]
        let instance_scope = scope.chain(
            activation.context.gc_context,
            &[Scope::new(class_object.into())],
        );

        class_object
            .0
            .write(activation.context.gc_context)
            .instance_scope = instance_scope;

        Ok(class_object)
    }

    /// Finish initialization of the class.
    ///
    /// This is intended for classes that were pre-allocated with
    /// `from_class_partial`. It skips several critical initialization steps
    /// that are necessary to obtain a functioning class object:
    ///
    ///  - The `link_type` step, which makes the class an instance of another
    ///    type
    ///  - The `link_prototype` step, which installs a prototype for instances
    ///    of this type to inherit
    ///
    /// Make sure to call them before calling this function, or it may yield an
    /// error.
    ///
    /// This function is also when class trait validation happens. Verify
    /// errors will be raised at this time.
    pub fn into_finished_class(
        mut self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let class = self.inner_class_definition();
        let class_class = self.instance_of().ok_or(
            "Cannot finish initialization of core class without it being linked to a type!",
        )?;

        self.inner_class_definition()
            .read()
            .validate_class(self.superclass_object())?;
        self.resolve_class_traits(activation)?;
        self.resolve_instance_traits(activation)?;
        self.link_interfaces(activation)?;
        self.install_traits(
            activation,
            class.read().class_traits(),
            self.class_scope(),
            Some(self),
        )?;
        self.install_instance_traits(activation, class_class)?;
        self.run_class_initializer(activation)?;

        Ok(self)
    }

    /// Link this class to a prototype.
    pub fn link_prototype(
        mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        mut class_proto: Object<'gc>,
    ) -> Result<(), Error> {
        self.install_slot(
            activation.context.gc_context,
            QName::new(Namespace::public(), "prototype"),
            0,
            class_proto.into(),
        );
        class_proto.install_slot(
            activation.context.gc_context,
            QName::new(Namespace::public(), "constructor"),
            0,
            self.into(),
        );

        Ok(())
    }

    /// Copy the list of class traits that this class has.
    ///
    /// This should be run during the class finalization step, before instances
    /// are linked (as instances will further add traits to the list).
    pub fn resolve_class_traits(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);

        let static_class = write.class;
        let class_read = static_class.read();
        for trait_data in class_read.class_traits() {
            if let Some(trait_slot) = write.resolved_class_traits.get_mut(trait_data.name()) {
                if matches!(trait_data.kind(), TraitKind::Getter { .. }) {
                    let mut replaced = false;
                    for overriding_trait in trait_slot.iter_mut() {
                        if matches!(overriding_trait.kind(), TraitKind::Getter { .. }) {
                            *overriding_trait = trait_data.clone();
                            replaced = true;
                        }
                    }

                    if !replaced {
                        trait_slot.push(trait_data.clone());
                    }
                } else if matches!(trait_data.kind(), TraitKind::Setter { .. }) {
                    let mut replaced = false;
                    for overriding_trait in trait_slot.iter_mut() {
                        if matches!(overriding_trait.kind(), TraitKind::Setter { .. }) {
                            *overriding_trait = trait_data.clone();
                            replaced = true;
                        }
                    }

                    if !replaced {
                        trait_slot.push(trait_data.clone());
                    }
                } else if let Some(overriding_trait) = trait_slot.first_mut() {
                    *overriding_trait = trait_data.clone();
                } else {
                    trait_slot.push(trait_data.clone());
                }
            } else {
                write
                    .resolved_class_traits
                    .insert(trait_data.name(), vec![trait_data.clone()]);
            }
        }

        Ok(())
    }

    /// Calculate the flattened list of instance traits that this class
    /// maintains.
    ///
    /// This should be run during the class finalization step, before instances
    /// are linked (as instances will further add traits to the list).
    pub fn resolve_instance_traits(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);

        if let Some(superclass) = write.superclass_object {
            write.resolved_instance_traits = superclass.0.read().resolved_instance_traits.clone();
        }

        let static_class = write.class;
        let class_read = static_class.read();
        for trait_data in class_read.instance_traits() {
            let new_trait_slot = (self, trait_data.clone());

            if let Some(trait_slot) = write.resolved_instance_traits.get_mut(trait_data.name()) {
                if matches!(trait_data.kind(), TraitKind::Getter { .. }) {
                    let mut replaced = false;
                    for overriding_trait in trait_slot.iter_mut() {
                        if matches!(overriding_trait.1.kind(), TraitKind::Getter { .. }) {
                            *overriding_trait = new_trait_slot.clone();
                            replaced = true;
                        }
                    }

                    if !replaced {
                        trait_slot.push(new_trait_slot);
                    }
                } else if matches!(trait_data.kind(), TraitKind::Setter { .. }) {
                    let mut replaced = false;
                    for overriding_trait in trait_slot.iter_mut() {
                        if matches!(overriding_trait.1.kind(), TraitKind::Setter { .. }) {
                            *overriding_trait = new_trait_slot.clone();
                            replaced = true;
                        }
                    }

                    if !replaced {
                        trait_slot.push(new_trait_slot);
                    }
                } else if let Some(overriding_trait) = trait_slot.first_mut() {
                    *overriding_trait = new_trait_slot;
                } else {
                    trait_slot.push(new_trait_slot);
                }
            } else {
                write
                    .resolved_instance_traits
                    .insert(trait_data.name(), vec![new_trait_slot]);
            }
        }

        Ok(())
    }

    /// Link this class to it's interfaces.
    ///
    /// This should be done after all instance traits has been resolved, as
    /// instance traits will be resolved to their corresponding methods at this
    /// time.
    pub fn link_interfaces(self, activation: &mut Activation<'_, 'gc, '_>) -> Result<(), Error> {
        let mut write = self.0.write(activation.context.gc_context);
        let class = write.class;
        let scope = write.class_scope;

        let interface_names = class.read().interfaces().to_vec();
        let mut interfaces = Vec::with_capacity(interface_names.len());
        for interface_name in interface_names {
            let interface = scope.resolve(&interface_name, activation)?;

            if interface.is_none() {
                return Err(format!("Could not resolve interface {:?}", interface_name).into());
            }

            let interface = interface.unwrap().coerce_to_object(activation)?;
            let iface_class = interface
                .as_class_object()
                .ok_or_else(|| Error::from("Object is not an interface"))?;
            if !iface_class.inner_class_definition().read().is_interface() {
                return Err(format!(
                    "Class {:?} is not an interface and cannot be implemented by classes",
                    iface_class
                        .inner_class_definition()
                        .read()
                        .name()
                        .local_name()
                )
                .into());
            }
            interfaces.push(iface_class);
        }

        if !interfaces.is_empty() {
            write.interfaces = interfaces;
        }

        //At this point, we need to reresolve *all* interface traits.
        //Otherwise we won't get overrides.
        drop(write);

        let mut class = Some(self);

        while let Some(cls) = class {
            for interface in cls.interfaces() {
                let iface_static_class = interface.inner_class_definition();
                let iface_read = iface_static_class.read();

                for interface_trait in iface_read.instance_traits() {
                    if !interface_trait.name().namespace().is_public() {
                        let public_name = QName::dynamic_name(interface_trait.name().local_name());
                        let trait_slot = self
                            .0
                            .read()
                            .resolved_instance_traits
                            .get(public_name)
                            .cloned();

                        if let Some(trait_slot) = trait_slot {
                            self.0
                                .write(activation.context.gc_context)
                                .resolved_instance_traits
                                .insert(interface_trait.name(), trait_slot);
                        }
                    }
                }
            }

            class = cls.superclass_object();
        }

        Ok(())
    }

    /// Manually set the type of this `Class`.
    ///
    /// This is intended to support initialization of early types such as
    /// `Class` and `Object`. All other types should pull `Class`'s prototype
    /// and type object from the `Avm2` instance.
    pub fn link_type(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        proto: Object<'gc>,
        instance_of: ClassObject<'gc>,
    ) {
        let mut write = self.0.write(activation.context.gc_context);

        write.base.set_instance_of(instance_of);
        write.base.set_proto(proto);
    }

    /// Run the class's initializer method.
    pub fn run_class_initializer(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let object: Object<'gc> = self.into();

        let scope = self.0.read().class_scope;
        let class = self.0.read().class;
        let class_read = class.read();

        if !class_read.is_class_initialized() {
            let class_initializer = class_read.class_init();
            let class_init_fn = FunctionObject::from_method(
                activation,
                class_initializer,
                scope,
                Some(object),
                Some(self),
            );

            drop(class_read);
            class
                .write(activation.context.gc_context)
                .mark_class_initialized();

            class_init_fn.call(Some(object), &[], activation)?;
        }

        Ok(())
    }

    /// Look up an instance trait by name and yield the trait and it's defining
    /// class.
    ///
    /// The trait must match the provided `filter`. If `None`, then no such
    /// trait exists with the requested parameters.
    pub fn lookup_instance_traits(
        self,
        name: QName<'gc>,
        filter: fn(&Trait<'gc>) -> bool,
    ) -> Option<(ClassObject<'gc>, Trait<'gc>)> {
        self.0
            .read()
            .resolved_instance_traits
            .get(name)
            .and_then(|v| v.iter().find(|(_, v)| filter(v)))
            .cloned()
    }

    /// Determine if we have an instance trait with a given name.
    pub fn has_instance_trait(self, name: QName<'gc>) -> bool {
        self.0.read().resolved_instance_traits.get(name).is_some()
    }

    /// List all namespaces that contain one or more instance traits of a given
    /// local name.
    pub fn resolve_instance_trait_ns(self, local_name: AvmString<'gc>) -> Vec<Namespace<'gc>> {
        self.0
            .read()
            .resolved_instance_traits
            .namespaces_of(local_name)
    }

    /// Retrieve the class object that a particular QName trait is defined in.
    ///
    /// Must be called on a class object; will error out if called on
    /// anything else.
    ///
    /// This function returns `None` for non-trait properties, such as actually
    /// defined prototype methods for ES3-style classes.
    pub fn find_class_for_trait_by_disp_id(
        self,
        id: u32,
    ) -> Result<Option<ClassObject<'gc>>, Error> {
        let class_definition = self.inner_class_definition();

        if class_definition.read().has_instance_trait_by_disp_id(id) {
            return Ok(Some(self));
        }

        if let Some(base) = self.superclass_object() {
            return base.find_class_for_trait_by_disp_id(id);
        }

        Ok(None)
    }

    /// Determine if this class has a given type in its superclass chain.
    ///
    /// The given object `test_class` should be either a superclass or
    /// interface we are checking against this class.
    ///
    /// To test if a class *instance* is of a given type, see is_of_type.
    pub fn has_class_in_chain(
        self,
        test_class: ClassObject<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<bool, Error> {
        let mut my_class = Some(self);

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

    /// Call the instance initializer.
    pub fn call_init(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let scope = self.0.read().instance_scope;
        let constructor =
            Executable::from_method(self.0.read().constructor.clone(), scope, None, Some(self));

        constructor.exec(receiver, arguments, activation, self.into())
    }

    /// Call the instance's native initializer.
    ///
    /// The native initializer is called when native code needs to construct an
    /// object, or when supercalling into a parent constructor (as there are
    /// classes that cannot be constructed but can be supercalled).
    pub fn call_native_init(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let scope = self.0.read().instance_scope;
        let constructor = Executable::from_method(
            self.0.read().native_constructor.clone(),
            scope,
            None,
            Some(self),
        );

        constructor.exec(receiver, arguments, activation, self.into())
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
    ///
    /// This method corresponds directly to the AVM2 operation `callsuper`,
    /// with the caveat listed above about what object to call it on.
    pub fn call_super(
        self,
        multiname: &Multiname<'gc>,
        reciever: Object<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let name = reciever.resolve_multiname(multiname)?;
        if name.is_none() {
            return Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }

        let name = name.unwrap();

        let lookup_result =
            self.lookup_instance_traits(name, |t| matches!(t.kind(), TraitKind::Method { .. }));

        if let Some((superclass_object, method_trait)) = lookup_result {
            let scope = superclass_object.class_scope();
            let method = method_trait.as_method().unwrap();
            let callee = FunctionObject::from_method(
                activation,
                method.clone(),
                scope,
                Some(reciever),
                Some(superclass_object),
            );

            callee.call(Some(reciever), arguments, activation)
        } else {
            reciever.call_property(multiname, arguments, activation)
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
    ///
    /// This method corresponds directly to the AVM2 operation `getsuper`,
    /// with the caveat listed above about what object to call it on.
    pub fn get_super(
        self,
        multiname: &Multiname<'gc>,
        reciever: Object<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let name = reciever.resolve_multiname(multiname)?;
        if name.is_none() {
            return Err(format!(
                "Attempted to supercall getter {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }

        let name = name.unwrap();

        let lookup_result =
            self.lookup_instance_traits(name, |t| matches!(t.kind(), TraitKind::Getter { .. }));

        if let Some((superclass_object, method_trait)) = lookup_result {
            let scope = superclass_object.class_scope();
            let method = method_trait.as_method().unwrap();
            let callee = FunctionObject::from_method(
                activation,
                method.clone(),
                scope,
                Some(reciever),
                Some(superclass_object),
            );

            callee.call(Some(reciever), &[], activation)
        } else {
            reciever.get_property(reciever, multiname, activation)
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
    ///
    /// This method corresponds directly to the AVM2 operation `setsuper`,
    /// with the caveat listed above about what object to call it on.
    #[allow(unused_mut)]
    pub fn set_super(
        self,
        multiname: &Multiname<'gc>,
        value: Value<'gc>,
        mut reciever: Object<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let name = reciever.resolve_multiname(multiname)?;
        if name.is_none() {
            return Err(format!(
                "Attempted to supercall setter {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }

        let name = name.unwrap();

        let lookup_result =
            self.lookup_instance_traits(name, |t| matches!(t.kind(), TraitKind::Setter { .. }));

        if let Some((superclass_object, method_trait)) = lookup_result {
            let scope = superclass_object.class_scope();
            let method = method_trait.as_method().unwrap();
            let callee = FunctionObject::from_method(
                activation,
                method.clone(),
                scope,
                Some(reciever),
                Some(superclass_object),
            );

            callee.call(Some(reciever), &[value], activation)?;

            Ok(())
        } else {
            reciever.set_property(reciever, multiname, value, activation)
        }
    }

    /// Retrieve an instance method and it's defining class by name.
    ///
    /// If a trait is returned, it is guaranteed to have a method.
    pub fn instance_method(
        self,
        name: QName<'gc>,
    ) -> Result<Option<(ClassObject<'gc>, Trait<'gc>)>, Error> {
        Ok(self.lookup_instance_traits(name, |t| matches!(t.kind(), TraitKind::Method { .. })))
    }

    /// Retrieve a bound instance method suitable for use as a value.
    ///
    /// This returns the bound method object itself, as well as it's dispatch
    /// ID. You will need the additional properties in order to install the
    /// method into your object.
    ///
    /// You should only call this method once per reciever/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    pub fn bound_instance_method(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        receiver: Object<'gc>,
        name: QName<'gc>,
    ) -> Result<Option<(Object<'gc>, u32)>, Error> {
        if let Some((superclass, method_trait)) = self.instance_method(name)? {
            let method = method_trait.as_method().unwrap();
            let disp_id = method_trait.disp_id().unwrap();
            let scope = self.instance_scope();

            Ok(Some((
                FunctionObject::from_method(
                    activation,
                    method,
                    scope,
                    Some(receiver),
                    Some(superclass),
                ),
                disp_id,
            )))
        } else {
            Ok(None)
        }
    }

    /// Retrieve a bound instance method by slot ID.
    ///
    /// This returns the bound method object itself, as well as it's name. You
    /// will need the additional properties in order to install the method into
    /// your object.
    ///
    /// You should only call this method once per reciever/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    pub fn bound_instance_method_by_id(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        receiver: Object<'gc>,
        id: u32,
    ) -> Result<Option<(Object<'gc>, QName<'gc>)>, Error> {
        if let Some(superclass) = self.find_class_for_trait_by_disp_id(id)? {
            let superclassdef = superclass.inner_class_definition();
            let traits = superclassdef.read().lookup_instance_traits_by_slot(id);

            if let Some(method_trait) = traits.and_then(|t| match t.kind() {
                TraitKind::Method { .. } => Some(t),
                _ => None,
            }) {
                let name = method_trait.name();
                let method = method_trait.as_method().unwrap();
                let scope = self.instance_scope();

                Ok(Some((
                    FunctionObject::from_method(
                        activation,
                        method,
                        scope,
                        Some(receiver),
                        Some(superclass),
                    ),
                    name,
                )))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Look up a class trait by name and yield the trait.
    ///
    /// The trait must match the provided `filter`. If `None`, then no such
    /// trait exists with the requested parameters.
    pub fn lookup_class_traits(
        self,
        name: QName<'gc>,
        filter: fn(&Trait<'gc>) -> bool,
    ) -> Option<Trait<'gc>> {
        self.0
            .read()
            .resolved_class_traits
            .get(name)
            .and_then(|v| v.iter().find(|v| filter(v)))
            .cloned()
    }

    /// Determine if we have a class trait with a given name.
    pub fn has_class_trait(self, name: QName<'gc>) -> bool {
        self.0.read().resolved_class_traits.get(name).is_some()
    }

    /// List all namespaces that contain one or more class traits of a given
    /// local name.
    pub fn resolve_class_trait_ns(self, local_name: AvmString<'gc>) -> Vec<Namespace<'gc>> {
        self.0
            .read()
            .resolved_class_traits
            .namespaces_of(local_name)
    }

    /// Retrieve a class method by name.
    ///
    /// This does not return a defining class as class methods are not
    /// inherited by subclasses.
    ///
    /// If a trait is returned, it is guaranteed to have a method.
    pub fn class_method(self, name: QName<'gc>) -> Result<Option<Trait<'gc>>, Error> {
        Ok(self.lookup_class_traits(name, |t| matches!(t.kind(), TraitKind::Method { .. })))
    }

    /// Retrieve a bound class method suitable for use as a value.
    ///
    /// This returns the bound method object itself, as well as it's dispatch
    /// ID. You will need the additional properties in order to install the
    /// method into your object.
    ///
    /// You should only call this method once per reciever/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    pub fn bound_class_method(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        name: QName<'gc>,
    ) -> Result<Option<(Object<'gc>, u32)>, Error> {
        if let Some(method_trait) = self.class_method(name)? {
            let method = method_trait.as_method().unwrap();
            let disp_id = method_trait.disp_id().unwrap();
            let scope = self.class_scope();

            Ok(Some((
                FunctionObject::from_method(
                    activation,
                    method,
                    scope,
                    Some(self.into()),
                    Some(self),
                ),
                disp_id,
            )))
        } else {
            Ok(None)
        }
    }

    /// Retrieve a bound class method by id.
    ///
    /// This returns the bound method object itself, as well as it's name. You
    /// will need the additional properties in order to install the method into
    /// your object.
    ///
    /// You should only call this method once per reciever/name pair, and cache
    /// the result. Otherwise, code that relies on bound methods having stable
    /// object identitities (e.g. `EventDispatcher.removeEventListener`) will
    /// fail.
    pub fn bound_class_method_by_id(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        id: u32,
    ) -> Result<Option<(Object<'gc>, QName<'gc>)>, Error> {
        let classdef = self.inner_class_definition();
        let traits = classdef.read().lookup_class_traits_by_slot(id);

        if let Some(method_trait) = traits.and_then(|t| match t.kind() {
            TraitKind::Method { .. } => Some(t),
            _ => None,
        }) {
            let method = method_trait.as_method().unwrap();
            let name = method_trait.name();
            let scope = self.class_scope();

            Ok(Some((
                FunctionObject::from_method(
                    activation,
                    method,
                    scope,
                    Some(self.into()),
                    Some(self),
                ),
                name,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn inner_class_definition(self) -> GcCell<'gc, Class<'gc>> {
        self.0.read().class
    }

    pub fn interfaces(self) -> Vec<ClassObject<'gc>> {
        self.0.read().interfaces.clone()
    }

    pub fn class_scope(self) -> ScopeChain<'gc> {
        self.0.read().class_scope
    }

    pub fn instance_scope(self) -> ScopeChain<'gc> {
        self.0.read().instance_scope
    }

    pub fn superclass_object(self) -> Option<ClassObject<'gc>> {
        self.0.read().superclass_object
    }

    pub fn as_class_params(self) -> Option<Option<ClassObject<'gc>>> {
        self.0.read().params
    }

    fn instance_allocator(self) -> Option<AllocatorFn> {
        Some(self.0.read().instance_allocator.0)
    }
}

impl<'gc> TObject<'gc> for ClassObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn to_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(AvmString::new_utf8(
            mc,
            format!("[class {}]", self.0.read().class.read().name().local_name()),
        )
        .into())
    }

    fn to_locale_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        self.to_string(mc)
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn call(
        self,
        _receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        arguments
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_type(activation, self)
    }

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let instance_allocator = self.0.read().instance_allocator.0;
        let prototype = self
            .get_property(
                self.into(),
                &QName::new(Namespace::public(), "prototype").into(),
                activation,
            )?
            .coerce_to_object(activation)?;

        let mut instance = instance_allocator(self, prototype, activation)?;

        instance.install_instance_traits(activation, self)?;

        self.call_init(Some(instance), arguments, activation)?;

        Ok(instance)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        Ok(ClassObject(GcCell::allocate(
            activation.context.gc_context,
            self.0.read().clone(),
        ))
        .into())
    }

    fn has_trait(self, name: QName<'gc>) -> Result<bool, Error> {
        let read = self.0.read();

        Ok(self.has_class_trait(name) || read.base.has_trait(name)?)
    }

    fn has_own_property(self, name: QName<'gc>) -> Result<bool, Error> {
        let read = self.0.read();

        Ok(read.base.has_own_instantiated_property(name)
            || self.has_class_trait(name)
            || read.base.has_trait(name)?)
    }

    fn resolve_ns(self, local_name: AvmString<'gc>) -> Result<Vec<Namespace<'gc>>, Error> {
        let read = self.0.read();

        let mut ns_set = read.base.resolve_ns(local_name)?;

        for trait_ns in self.resolve_class_trait_ns(local_name) {
            if !ns_set.contains(&trait_ns) {
                ns_set.push(trait_ns);
            }
        }

        Ok(ns_set)
    }

    fn as_class_object(&self) -> Option<ClassObject<'gc>> {
        Some(*self)
    }

    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: QName<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        // Traits are never enumerable.
        //
        // We have to do this here because the `ScriptObjectBase` version of
        // this function calls the version of `has_trait` that checks instance
        // traits, and because we're not an instance, we don't have any and
        // thus the check fails.
        if self.has_trait(name)? {
            return Ok(());
        }

        self.0
            .write(mc)
            .base
            .set_local_property_is_enumerable(name, is_enumerable)
    }

    fn apply(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        nullable_params: &[Value<'gc>],
    ) -> Result<ClassObject<'gc>, Error> {
        let self_class = self.inner_class_definition();

        if !self_class.read().is_generic() {
            return Err(format!("Class {:?} is not generic", self_class.read().name()).into());
        }

        if !self_class.read().params().is_empty() {
            return Err(format!("Class {:?} was already applied", self_class.read().name()).into());
        }

        if nullable_params.len() != 1 {
            return Err(format!(
                "Class {:?} only accepts one type parameter, {} given",
                self_class.read().name(),
                nullable_params.len()
            )
            .into());
        }

        //Because `null` is a valid parameter, we have to accept values as
        //parameters instead of objects. We coerce them to objects now.
        let object_param = match &nullable_params[0] {
            Value::Null => None,
            Value::Undefined => return Err("Undefined is not a valid type parameter".into()),
            v => Some(v.coerce_to_object(activation)?),
        };
        let object_param = match object_param {
            None => None,
            Some(cls) => Some(cls.as_class_object().ok_or(format!(
                "Cannot apply class {:?} with non-class parameter",
                self_class.read().name()
            ))?),
        };

        if let Some(application) = self.0.read().applications.get(&object_param) {
            return Ok(*application);
        }

        let class_param = object_param
            .unwrap_or(activation.avm2().classes().object)
            .inner_class_definition();

        let parameterized_class = self_class
            .read()
            .with_type_params(&[class_param], activation.context.gc_context);

        let class_scope = self.0.read().class_scope;
        let instance_scope = self.0.read().instance_scope;
        let instance_allocator = self.0.read().instance_allocator.clone();
        let superclass_object = self.0.read().superclass_object;

        let class_proto = self.allocate_prototype(activation, superclass_object)?;

        let class_class = activation.avm2().classes().class;
        let class_class_proto = activation.avm2().prototypes().class;

        let constructor = self.0.read().constructor.clone();
        let native_constructor = self.0.read().native_constructor.clone();

        let mut class_object = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(Some(class_class_proto), Some(class_class)),
                class: parameterized_class,
                class_scope,
                instance_scope,
                superclass_object,
                instance_allocator,
                constructor,
                native_constructor,
                params: Some(object_param),
                applications: HashMap::new(),
                interfaces: Vec::new(),
                resolved_instance_traits: PropertyMap::new(),
                resolved_class_traits: PropertyMap::new(),
            },
        ));

        class_object
            .inner_class_definition()
            .read()
            .validate_class(class_object.superclass_object())?;
        class_object.resolve_class_traits(activation)?;
        class_object.resolve_instance_traits(activation)?;
        class_object.link_prototype(activation, class_proto)?;
        class_object.link_interfaces(activation)?;
        class_object.install_traits(
            activation,
            parameterized_class.read().class_traits(),
            class_scope,
            Some(class_object),
        )?;
        class_object.install_instance_traits(activation, class_class)?;
        class_object.run_class_initializer(activation)?;

        self.0
            .write(activation.context.gc_context)
            .applications
            .insert(object_param, class_object);

        Ok(class_object)
    }
}

impl<'gc> PartialEq for ClassObject<'gc> {
    fn eq(&self, other: &Self) -> bool {
        Object::ptr_eq(*self, *other)
    }
}

impl<'gc> Eq for ClassObject<'gc> {}

impl<'gc> Hash for ClassObject<'gc> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}
