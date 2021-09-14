//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Allocator, AllocatorFn, Class};
use crate::avm2::function::Executable;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::{scriptobject_allocator, ScriptObject, ScriptObjectData};
use crate::avm2::object::{Multiname, Object, ObjectPtr, TObject};
use crate::avm2::scope::ScopeChain;
use crate::avm2::traits::TraitKind;
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

    /// The scope this class was defined in.
    scope: ScopeChain<'gc>,

    /// The base class of this one.
    ///
    /// If `None`, this class has no parent. In practice, this is only used for
    /// interfaces (at least by the AS3 compiler in Animate CC 2020.)
    superclass_object: Option<ClassObject<'gc>>,

    /// The instance allocator for this class.
    instance_allocator: Allocator,

    /// The instance constructor function
    constructor: Executable<'gc>,

    /// The native instance constructor function
    native_constructor: Executable<'gc>,

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
}

impl<'gc> ClassObject<'gc> {
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

        //TODO: Class prototypes are *not* instances of their class and should
        //not be allocated by the class allocator, but instead should be
        //regular objects
        let class_proto = if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object
                .get_property(
                    superclass_object.into(),
                    &QName::new(Namespace::public(), "prototype").into(),
                    activation,
                )?
                .coerce_to_object(activation)?;
            ScriptObject::object(activation.context.gc_context, base_proto)
        } else {
            ScriptObject::bare_object(activation.context.gc_context)
        };

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

        let constructor = Executable::from_method(
            class.read().instance_init(),
            scope,
            None,
            activation.context.gc_context,
        );
        let native_constructor = Executable::from_method(
            class.read().native_instance_init(),
            scope,
            None,
            activation.context.gc_context,
        );

        let class_object = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(None, None),
                class,
                scope: scope,
                superclass_object,
                instance_allocator: Allocator(instance_allocator),
                constructor,
                native_constructor,
                params: None,
                applications: HashMap::new(),
                interfaces: Vec::new(),
            },
        ));

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
    pub fn into_finished_class(
        mut self,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let class = self.inner_class_definition();
        let class_class = self.instance_of().ok_or(
            "Cannot finish initialization of core class without it being linked to a type!",
        )?;

        self.link_interfaces(activation)?;
        self.install_traits(activation, class.read().class_traits())?;
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
            false,
        );
        class_proto.install_slot(
            activation.context.gc_context,
            QName::new(Namespace::public(), "constructor"),
            0,
            self.into(),
            false,
        );

        Ok(())
    }

    /// Link this class to it's interfaces.
    pub fn link_interfaces(self, activation: &mut Activation<'_, 'gc, '_>) -> Result<(), Error> {
        let class = self.0.read().class;
        let scope = self.0.read().scope;

        let interface_names = class.read().interfaces().to_vec();
        let mut interfaces = Vec::with_capacity(interface_names.len());
        for interface_name in interface_names {
            let interface = scope.resolve(&interface_name, activation)?;

            if interface.is_none() {
                return Err(format!("Could not resolve interface {:?}", interface_name).into());
            }

            let interface = interface.unwrap().coerce_to_object(activation)?;
            let class = interface
                .as_class_object()
                .ok_or_else(|| Error::from("Object is not an interface"))?;
            if !class.inner_class_definition().read().is_interface() {
                return Err(format!(
                    "Class {:?} is not an interface and cannot be implemented by classes",
                    class.inner_class_definition().read().name().local_name()
                )
                .into());
            }
            interfaces.push(class);
        }

        if !interfaces.is_empty() {
            self.0.write(activation.context.gc_context).interfaces = interfaces;
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

        let scope = self.0.read().scope;
        let class = self.0.read().class;
        let class_read = class.read();

        if !class_read.is_class_initialized() {
            let class_initializer = class_read.class_init();
            let class_init_fn =
                FunctionObject::from_method(activation, class_initializer, scope, Some(object));

            drop(class_read);
            class
                .write(activation.context.gc_context)
                .mark_class_initialized();

            class_init_fn.call(Some(object), &[], activation, None)?;
        }

        Ok(())
    }

    /// Retrieve the class object that a particular QName trait is defined in.
    ///
    /// Must be called on a class object; will error out if called on
    /// anything else.
    ///
    /// This function returns `None` for non-trait properties, such as actually
    /// defined prototype methods for ES3-style classes.
    pub fn find_class_for_trait(
        self,
        name: &QName<'gc>,
    ) -> Result<Option<ClassObject<'gc>>, Error> {
        let class_definition = self.inner_class_definition();

        if class_definition.read().has_instance_trait(name) {
            return Ok(Some(self));
        }

        if let Some(base) = self.superclass_object() {
            return base.find_class_for_trait(name);
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
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let constructor = self.0.read().constructor.clone();

        constructor.exec(
            receiver,
            arguments,
            activation,
            superclass_object,
            self.into(),
        )
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
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let native_constructor = self.0.read().native_constructor.clone();

        native_constructor.exec(
            receiver,
            arguments,
            activation,
            superclass_object,
            self.into(),
        )
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

        let superclass_object = self.find_class_for_trait(&name)?.ok_or_else(|| {
            format!(
                "Attempted to supercall method {:?}, which does not exist",
                name
            )
        })?;
        let mut class_traits = Vec::new();
        superclass_object
            .inner_class_definition()
            .read()
            .lookup_instance_traits(&name, &mut class_traits)?;
        let base_trait = class_traits
            .iter()
            .find(|t| matches!(t.kind(), TraitKind::Method { .. }));
        let scope = superclass_object.scope();

        if let Some(TraitKind::Method { method, .. }) = base_trait.map(|b| b.kind()) {
            let callee =
                FunctionObject::from_method(activation, method.clone(), scope, Some(reciever));

            callee.call(
                Some(reciever),
                arguments,
                activation,
                Some(superclass_object),
            )
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

        let superclass_object = self.find_class_for_trait(&name)?.ok_or_else(|| {
            format!(
                "Attempted to supercall getter {:?}, which does not exist",
                name
            )
        })?;
        let mut class_traits = Vec::new();
        superclass_object
            .inner_class_definition()
            .read()
            .lookup_instance_traits(&name, &mut class_traits)?;
        let base_trait = class_traits
            .iter()
            .find(|t| matches!(t.kind(), TraitKind::Getter { .. }));
        let scope = superclass_object.scope();

        if let Some(TraitKind::Getter { method, .. }) = base_trait.map(|b| b.kind()) {
            let callee =
                FunctionObject::from_method(activation, method.clone(), scope, Some(reciever));

            callee.call(Some(reciever), &[], activation, Some(superclass_object))
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

        let superclass_object = self.find_class_for_trait(&name)?.ok_or_else(|| {
            format!(
                "Attempted to supercall setter {:?}, which does not exist",
                name
            )
        })?;
        let mut class_traits = Vec::new();
        superclass_object
            .inner_class_definition()
            .read()
            .lookup_instance_traits(&name, &mut class_traits)?;
        let base_trait = class_traits
            .iter()
            .find(|t| matches!(t.kind(), TraitKind::Setter { .. }));
        let scope = superclass_object.scope();

        if let Some(TraitKind::Setter { method, .. }) = base_trait.map(|b| b.kind()) {
            let callee =
                FunctionObject::from_method(activation, method.clone(), scope, Some(reciever));

            callee.call(
                Some(reciever),
                &[value],
                activation,
                Some(superclass_object),
            )?;

            Ok(())
        } else {
            reciever.set_property(reciever, multiname, value, activation)
        }
    }

    pub fn inner_class_definition(self) -> GcCell<'gc, Class<'gc>> {
        self.0.read().class
    }

    pub fn interfaces(self) -> Vec<ClassObject<'gc>> {
        self.0.read().interfaces.clone()
    }

    pub fn scope(self) -> ScopeChain<'gc> {
        self.0.read().scope
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
        Ok(AvmString::new(
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
        _superclass_object: Option<ClassObject<'gc>>,
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

        self.call_init(Some(instance), arguments, activation, Some(self))?;

        Ok(instance)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        Ok(ClassObject(GcCell::allocate(
            activation.context.gc_context,
            self.0.read().clone(),
        ))
        .into())
    }

    fn has_trait(self, name: &QName<'gc>) -> Result<bool, Error> {
        Ok(self.0.read().class.read().has_class_trait(name))
    }

    fn resolve_any_trait(
        self,
        local_name: AvmString<'gc>,
    ) -> Result<Option<Namespace<'gc>>, Error> {
        if let Some(proto) = self.proto() {
            let proto_trait_name = proto.resolve_any_trait(local_name)?;
            if let Some(ns) = proto_trait_name {
                return Ok(Some(ns));
            }
        }

        Ok(self
            .0
            .read()
            .class
            .read()
            .resolve_any_class_trait(local_name))
    }

    fn as_class_object(&self) -> Option<ClassObject<'gc>> {
        Some(*self)
    }

    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: &QName<'gc>,
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

        let scope = self.0.read().scope;
        let instance_allocator = self.0.read().instance_allocator.clone();
        let superclass_object = self.0.read().superclass_object;

        //TODO: Class prototypes are *not* instances of their class and should
        //not be allocated by the class allocator, but instead should be
        //regular objects
        let class_proto = if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object
                .get_property(
                    superclass_object.into(),
                    &QName::new(Namespace::public(), "prototype").into(),
                    activation,
                )?
                .coerce_to_object(activation)?;
            (instance_allocator.0)(superclass_object, base_proto, activation)?
        } else {
            ScriptObject::bare_object(activation.context.gc_context)
        };

        let class_class = activation.avm2().classes().class;
        let class_class_proto = activation.avm2().prototypes().class;

        let constructor = self.0.read().constructor.clone();
        let native_constructor = self.0.read().native_constructor.clone();

        let mut class_object = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(Some(class_class_proto), Some(class_class)),
                class: parameterized_class,
                scope,
                superclass_object,
                instance_allocator,
                constructor,
                native_constructor,
                params: Some(object_param),
                applications: HashMap::new(),
                interfaces: Vec::new(),
            },
        ));

        class_object.link_prototype(activation, class_proto)?;
        class_object.link_interfaces(activation)?;
        class_object.install_traits(activation, parameterized_class.read().class_traits())?;
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
