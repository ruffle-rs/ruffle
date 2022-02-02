//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Allocator, AllocatorFn, Class};
use crate::avm2::function::Executable;
use crate::avm2::method::Method;
use crate::avm2::names::QName;
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::{scriptobject_allocator, ScriptObjectData};
use crate::avm2::object::{Multiname, Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::value::Value;
use crate::avm2::vtable::VTable;
use crate::avm2::Error;
use crate::string::AvmString;
use fnv::FnvHashMap;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
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

    /// The associated prototype.
    /// Should always be non-None after initialization.
    prototype: Option<Object<'gc>>,

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

    /// The customization point for `Class(args...)` without `new`
    /// If None, a simple coercion is done.
    call_handler: Option<Method<'gc>>,

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
    applications: FnvHashMap<Option<ClassObject<'gc>>, ClassObject<'gc>>,

    /// Interfaces implemented by this class.
    interfaces: Vec<ClassObject<'gc>>,

    /// VTable used for instances of this class.
    instance_vtable: VTable<'gc>,

    /// VTable used for a ScriptObject of this class object.
    class_vtable: VTable<'gc>,
}

impl<'gc> ClassObject<'gc> {
    /// Allocate the prototype for this class.
    ///
    /// This function is not used during the initialization of "early classes",
    /// i.e. `Object`, `Function`, and `Class`. Those classes and their
    /// prototypes are weaved together separately.
    fn allocate_prototype(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Object<'gc>, Error> {
        let proto = activation
            .avm2()
            .classes()
            .object
            .construct(activation, &[])?;

        if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object.prototype();
            proto.set_proto(activation.context.gc_context, base_proto);
        }
        Ok(proto)
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
                prototype: None,
                class_scope: scope,
                instance_scope: scope,
                superclass_object,
                instance_allocator: Allocator(instance_allocator),
                constructor: class.read().instance_init(),
                native_constructor: class.read().native_instance_init(),
                call_handler: class.read().call_handler(),
                params: None,
                applications: Default::default(),
                interfaces: Vec::new(),
                instance_vtable: VTable::empty(activation.context.gc_context),
                class_vtable: VTable::empty(activation.context.gc_context),
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
        self.instance_of().ok_or(
            "Cannot finish initialization of core class without it being linked to a type!",
        )?;

        class.read().validate_class(self.superclass_object())?;

        self.instance_vtable().init_vtable(
            Some(self),
            class.read().instance_traits(),
            self.instance_scope(),
            self.superclass_object().map(|cls| cls.instance_vtable()),
            activation,
        )?;

        // class vtable == class traits + Class instance traits
        self.class_vtable().init_vtable(
            Some(self),
            class.read().class_traits(),
            self.class_scope(),
            Some(self.instance_of().unwrap().instance_vtable()),
            activation,
        )?;

        self.link_interfaces(activation)?;
        self.install_class_vtable_and_slots(activation);
        self.run_class_initializer(activation)?;

        Ok(self)
    }

    fn install_class_vtable_and_slots(&mut self, activation: &mut Activation<'_, 'gc, '_>) {
        self.set_vtable(activation.context.gc_context, self.class_vtable());
        self.base_mut(activation.context.gc_context)
            .install_instance_slots();
    }

    /// Link this class to a prototype.
    pub fn link_prototype(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        class_proto: Object<'gc>,
    ) -> Result<(), Error> {
        self.0.write(activation.context.gc_context).prototype = Some(class_proto);
        class_proto.set_property_local(
            &Multiname::public("constructor"),
            self.into(),
            activation,
        )?;
        class_proto.set_local_property_is_enumerable(
            activation.context.gc_context,
            "constructor".into(),
            false,
        )?;

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
                        self.instance_vtable().copy_property_for_interface(
                            activation.context.gc_context,
                            public_name,
                            interface_trait.name(),
                        );
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
        let instance_vtable = instance_of.instance_vtable();

        let mut write = self.0.write(activation.context.gc_context);

        write.base.set_instance_of(instance_of, instance_vtable);
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
        let property = self.instance_vtable().get_trait(multiname);
        if property.is_none() {
            return Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }
        if let Some(Property::Method { disp_id, .. }) = property {
            // todo: handle errors
            let (superclass_object, method) =
                self.instance_vtable().get_full_method(disp_id).unwrap();
            let scope = superclass_object.unwrap().class_scope();
            let callee = FunctionObject::from_method(
                activation,
                method.clone(),
                scope,
                Some(reciever),
                superclass_object,
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
        let property = self.instance_vtable().get_trait(multiname);
        if property.is_none() {
            return Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }
        if let Some(Property::Virtual {
            get: Some(disp_id), ..
        }) = property
        {
            // todo: handle errors
            let (superclass_object, method) =
                self.instance_vtable().get_full_method(disp_id).unwrap();
            let scope = superclass_object.unwrap().class_scope();
            let callee = FunctionObject::from_method(
                activation,
                method.clone(),
                scope,
                Some(reciever),
                superclass_object,
            );

            callee.call(Some(reciever), &[], activation)
        } else {
            reciever.get_property(multiname, activation)
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
        let property = self.instance_vtable().get_trait(multiname);
        if property.is_none() {
            return Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }
        if let Some(Property::Virtual {
            set: Some(disp_id), ..
        }) = property
        {
            // todo: handle errors
            let (superclass_object, method) =
                self.instance_vtable().get_full_method(disp_id).unwrap();
            let scope = superclass_object.unwrap().class_scope();
            let callee = FunctionObject::from_method(
                activation,
                method.clone(),
                scope,
                Some(reciever),
                superclass_object,
            );

            callee.call(Some(reciever), &[value], activation)?;

            Ok(())
        } else {
            reciever.set_property(multiname, value, activation)
        }
    }

    pub fn instance_vtable(self) -> VTable<'gc> {
        self.0.read().instance_vtable
    }

    pub fn class_vtable(self) -> VTable<'gc> {
        self.0.read().class_vtable
    }

    pub fn inner_class_definition(self) -> GcCell<'gc, Class<'gc>> {
        self.0.read().class
    }

    pub fn prototype(self) -> Object<'gc> {
        self.0.read().prototype.unwrap()
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
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        if let Some(call_handler) = self.0.read().call_handler.clone() {
            let scope = self.0.read().class_scope;
            let func = Executable::from_method(call_handler, scope, None, Some(self));

            func.exec(receiver, arguments, activation, self.into())
        } else {
            arguments
                .get(0)
                .cloned()
                .unwrap_or(Value::Undefined)
                .coerce_to_type(activation, self)
        }
    }

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let instance_allocator = self.0.read().instance_allocator.0;
        let prototype = self.0.read().prototype.unwrap();

        let mut instance = instance_allocator(self, prototype, activation)?;

        instance.install_instance_slots(activation);

        self.call_init(Some(instance), arguments, activation)?;

        Ok(instance)
    }

    fn has_own_property(self, name: &Multiname<'gc>) -> bool {
        let read = self.0.read();

        read.base.has_own_dynamic_property(name) || self.class_vtable().has_trait(name)
    }

    fn as_class_object(&self) -> Option<ClassObject<'gc>> {
        Some(*self)
    }

    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: AvmString<'gc>,
        is_enumerable: bool,
    ) -> Result<(), Error> {
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
        let call_handler = self.0.read().call_handler.clone();

        let mut class_object = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(Some(class_class_proto), Some(class_class)),
                class: parameterized_class,
                prototype: None,
                class_scope,
                instance_scope,
                superclass_object,
                instance_allocator,
                constructor,
                native_constructor,
                call_handler,
                params: Some(object_param),
                applications: Default::default(),
                interfaces: Vec::new(),
                instance_vtable: VTable::empty(activation.context.gc_context),
                class_vtable: VTable::empty(activation.context.gc_context),
            },
        ));

        class_object
            .inner_class_definition()
            .read()
            .validate_class(class_object.superclass_object())?;

        class_object.instance_vtable().init_vtable(
            Some(class_object),
            parameterized_class.read().instance_traits(),
            class_object.instance_scope(),
            class_object
                .superclass_object()
                .map(|cls| cls.instance_vtable()),
            activation,
        )?;

        // class vtable == class traits + Class instance traits
        class_object.class_vtable().init_vtable(
            Some(class_object),
            parameterized_class.read().class_traits(),
            class_object.class_scope(),
            Some(class_object.instance_of().unwrap().instance_vtable()),
            activation,
        )?;

        class_object.link_prototype(activation, class_proto)?;
        class_object.link_interfaces(activation)?;
        class_object.install_class_vtable_and_slots(activation);
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
