//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{AllocatorFn, Class};
use crate::avm2::error::{argument_error, make_error_1127, reference_error, type_error};
use crate::avm2::function::exec;
use crate::avm2::method::Method;
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::value::Value;
use crate::avm2::vtable::{ClassBoundMethod, VTable};
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;
use crate::avm2::TranslationUnit;
use crate::string::AvmString;
use fnv::FnvHashMap;
use gc_arena::barrier::unlock;
use gc_arena::{
    lock::{Lock, RefLock},
    Collect, Gc, GcWeak, Mutation,
};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// An Object which can be called to execute its function code.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct ClassObject<'gc>(pub Gc<'gc, ClassObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct ClassObjectWeak<'gc>(pub GcWeak<'gc, ClassObjectData<'gc>>);

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct ClassObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// The class associated with this class object.
    class: Class<'gc>,

    /// The associated prototype.
    /// Should always be non-None after initialization.
    prototype: Lock<Option<Object<'gc>>>,

    /// The captured scope that all class traits will use.
    class_scope: ScopeChain<'gc>,

    /// The captured scope that all instance traits will use.
    instance_scope: Lock<ScopeChain<'gc>>,

    /// The base class of this one.
    ///
    /// If `None`, this class has no parent. In practice, this is only used for
    /// interfaces (at least by the AS3 compiler in Animate CC 2020.)
    superclass_object: Option<ClassObject<'gc>>,

    /// List of all applications of this class.
    ///
    /// Only applicable if this class is generic.
    ///
    /// It is legal to apply a type with the value `null`, which is represented
    /// as `None` here. AVM2 considers both applications to be separate
    /// classes, though we consider the parameter to be the class `Object` when
    /// we get a param of `null`.
    applications: RefLock<FnvHashMap<Option<Class<'gc>>, ClassObject<'gc>>>,

    /// VTable used for instances of this class.
    instance_vtable: VTable<'gc>,
}

const _: () = assert!(std::mem::offset_of!(ClassObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<ClassObjectData>() == std::mem::align_of::<ScriptObjectData>());

impl<'gc> ClassObject<'gc> {
    /// Allocate the prototype for this class.
    ///
    /// This function is not used during the initialization of "early classes",
    /// i.e. `Object`, `Function`, and `Class`. Those classes and their
    /// prototypes are weaved together separately.
    fn allocate_prototype(
        self,
        activation: &mut Activation<'_, 'gc>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Object<'gc>, Error<'gc>> {
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
        activation: &mut Activation<'_, 'gc>,
        class: Class<'gc>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Self, Error<'gc>> {
        let class_object = Self::from_class_partial(activation, class, superclass_object)?;
        let class_proto = class_object.allocate_prototype(activation, superclass_object)?;

        class_object.link_prototype(activation, class_proto)?;

        let class_class_proto = activation.avm2().classes().class.prototype();
        class_object.link_type(activation.context.gc_context, class_class_proto);

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
        activation: &mut Activation<'_, 'gc>,
        class: Class<'gc>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Result<Self, Error<'gc>> {
        let c_class = class
            .c_class()
            .expect("Can only call ClassObject::from_class on i_classes");

        let scope = activation.create_scopechain();
        if let Some(base_class) = superclass_object.map(|b| b.inner_class_definition()) {
            if base_class.is_final() {
                return Err(format!(
                    "Base class {:?} is final and cannot be extended",
                    base_class.name().local_name()
                )
                .into());
            }

            if base_class.is_interface() {
                return Err(format!(
                    "Base class {:?} is an interface and cannot be extended",
                    base_class.name().local_name()
                )
                .into());
            }
        }

        let mc = activation.context.gc_context;

        let class_object = ClassObject(Gc::new(
            activation.context.gc_context,
            ClassObjectData {
                // We pass `custom_new` the temporary vtable of the class object
                // because we don't have the full vtable created yet. We'll
                // set it to the true vtable in `into_finished_class`.
                base: ScriptObjectData::custom_new(c_class, None, c_class.vtable()),
                class,
                prototype: Lock::new(None),
                class_scope: scope,
                instance_scope: Lock::new(scope),
                superclass_object,
                applications: RefLock::new(Default::default()),
                instance_vtable: VTable::empty(mc),
            },
        ));

        // instance scope = [..., class object]
        let instance_scope = scope.chain(mc, &[Scope::new(class_object.into())]);

        unlock!(
            Gc::write(mc, class_object.0),
            ClassObjectData,
            instance_scope
        )
        .set(instance_scope);
        class_object.init_instance_vtable(activation)?;

        class.add_class_object(activation.context.gc_context, class_object);

        Ok(class_object)
    }

    fn init_instance_vtable(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        let class = self.inner_class_definition();

        class.validate_class(self.superclass_object())?;

        self.instance_vtable().init_vtable(
            class,
            self.superclass_object(),
            Some(self.instance_scope()),
            self.superclass_object().map(|cls| cls.instance_vtable()),
            activation.context.gc_context,
        );

        self.link_interfaces(activation)?;

        Ok(())
    }

    /// Finish initialization of the class.
    ///
    /// This is intended for classes that were pre-allocated with
    /// `from_class_partial`. It skips two critical initialization steps
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
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let i_class = self.inner_class_definition();
        let c_class = i_class
            .c_class()
            .expect("ClassObject should have an i_class");

        let class_classobject = activation.avm2().classes().class;

        // class vtable == class traits + Class instance traits
        let class_vtable = VTable::empty(activation.context.gc_context);
        class_vtable.init_vtable(
            c_class,
            Some(class_classobject),
            Some(self.class_scope()),
            Some(class_classobject.instance_vtable()),
            activation.context.gc_context,
        );

        self.set_vtable(activation.context.gc_context, class_vtable);

        self.run_class_initializer(activation)?;

        Ok(self)
    }

    /// Link this class to a prototype.
    pub fn link_prototype(
        self,
        activation: &mut Activation<'_, 'gc>,
        class_proto: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        let mc = activation.context.gc_context;

        unlock!(Gc::write(mc, self.0), ClassObjectData, prototype).set(Some(class_proto));
        class_proto.set_string_property_local("constructor", self.into(), activation)?;
        class_proto.set_local_property_is_enumerable(mc, "constructor".into(), false);

        Ok(())
    }

    /// Link this class to it's interfaces.
    ///
    /// This should be done after all instance traits has been resolved, as
    /// instance traits will be resolved to their corresponding methods at this
    /// time.
    pub fn link_interfaces(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        let class = self.inner_class_definition();

        // FIXME - we should only be copying properties for newly-implemented
        // interfaces (i.e. those that were not already implemented by the superclass)
        // Otherwise, our behavior diverges from Flash Player in certain cases.
        // See the ignored test 'tests/tests/swfs/avm2/weird_superinterface_properties/'
        let internal_ns = activation.avm2().namespaces.public_vm_internal();
        for interface in &*class.all_interfaces() {
            for interface_trait in &*interface.traits() {
                if !interface_trait.name().namespace().is_public() {
                    let public_name = QName::new(internal_ns, interface_trait.name().local_name());
                    self.instance_vtable().copy_property_for_interface(
                        activation.context.gc_context,
                        public_name,
                        interface_trait.name(),
                    );
                }
            }
        }

        Ok(())
    }

    /// Manually set the type of this `Class`.
    ///
    /// This is intended to support initialization of early types such as
    /// `Class` and `Object`. All other types should pull `Class`'s prototype
    /// and type object from the `Avm2` instance.
    pub fn link_type(self, gc_context: &Mutation<'gc>, proto: Object<'gc>) {
        self.base().set_proto(gc_context, proto);
    }

    /// Run the class's initializer method.
    pub fn run_class_initializer(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let object: Object<'gc> = self.into();
        let class_classobject = activation.avm2().classes().class;

        let scope = self.0.class_scope;
        let c_class = self
            .inner_class_definition()
            .c_class()
            .expect("ClassObject stores an i_class");

        let class_initializer = c_class.instance_init();
        let class_init_fn = FunctionObject::from_method(
            activation,
            class_initializer,
            scope,
            Some(object),
            Some(class_classobject),
            Some(c_class),
        );

        class_init_fn.call(object.into(), &[], activation)?;

        Ok(())
    }

    /// Call the instance initializer.
    pub fn call_init(
        self,
        receiver: Value<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let scope = self.0.instance_scope.get();
        let method = self.constructor();
        exec(
            method,
            scope,
            receiver.coerce_to_object(activation)?,
            self.superclass_object(),
            Some(self.inner_class_definition()),
            arguments,
            activation,
            self.into(),
        )
    }

    /// Call the instance's native initializer.
    ///
    /// The native initializer is called when native code needs to construct an
    /// object, or when supercalling into a parent constructor (as there are
    /// classes that cannot be constructed but can be supercalled).
    pub fn call_super_init(
        self,
        receiver: Value<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let scope = self.0.instance_scope.get();
        let method = self.super_constructor();
        exec(
            method,
            scope,
            receiver.coerce_to_object(activation)?,
            self.superclass_object(),
            Some(self.inner_class_definition()),
            arguments,
            activation,
            self.into(),
        )
    }

    /// Supercall a method defined in this class.
    ///
    /// This is intended to be called on the class object that is the
    /// superclass of the one that defined the currently called property. If no
    /// such superclass exists, you should use the class object for the
    /// receiver's actual type (i.e. the lowest in the chain). This ensures
    /// that repeated supercalls to the same method will call parent and
    /// grandparent methods, and so on.
    ///
    /// If no method exists with the given name, this falls back to calling a
    /// property of the `receiver`. This fallback only triggers if the property
    /// is associated with a trait. Dynamic properties will still error out.
    ///
    /// This function will search through the class object tree starting from
    /// this class up to `Object` for a method trait with the given name. If it
    /// is found, it will be called with the receiver and arguments you
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
        receiver: Object<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let property = self.instance_vtable().get_trait(multiname);
        if property.is_none() {
            let qualified_multiname_name = multiname.as_uri(activation.strings());
            let qualified_class_name = self
                .inner_class_definition()
                .name()
                .to_qualified_name_err_message(activation.context.gc_context);

            return Err(Error::AvmError(reference_error(
                activation,
                &format!(
                    "Error #1070: Method {} not found on {}",
                    qualified_multiname_name, qualified_class_name
                ),
                1070,
            )?));
        }

        if let Some(Property::Method { disp_id, .. }) = property {
            // todo: handle errors
            let ClassBoundMethod {
                class,
                super_class_obj,
                scope,
                method,
            } = self.instance_vtable().get_full_method(disp_id).unwrap();
            let callee = FunctionObject::from_method(
                activation,
                method,
                scope.expect("Scope should exist here"),
                Some(receiver),
                super_class_obj,
                Some(class),
            );

            callee.call(receiver.into(), arguments, activation)
        } else {
            receiver.call_property(multiname, arguments, activation)
        }
    }

    /// Supercall a getter defined in this class.
    ///
    /// This is intended to be called on the class object that is the
    /// superclass of the one that defined the currently called property. If no
    /// such superclass exists, you should use the class object for the
    /// receiver's actual type (i.e. the lowest in the chain). This ensures
    /// that repeated supercalls to the same getter will call parent and
    /// grandparent getters, and so on.
    ///
    /// If no getter exists with the given name, this falls back to getting a
    /// property of the `receiver`. This fallback only triggers if the property
    /// is associated with a trait. Dynamic properties will still error out.
    ///
    /// This function will search through the class object tree starting from
    /// this class up to `Object` for a getter trait with the given name. If it
    /// is found, it will be called with the receiver you provided, as if it
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
        receiver: Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let property = self.instance_vtable().get_trait(multiname);

        match property {
            Some(
                Property::Virtual {
                    get: Some(disp_id), ..
                }
                | Property::Method { disp_id },
            ) => {
                // todo: handle errors
                let ClassBoundMethod {
                    class,
                    super_class_obj,
                    scope,
                    method,
                } = self.instance_vtable().get_full_method(disp_id).unwrap();
                let callee = FunctionObject::from_method(
                    activation,
                    method,
                    scope.expect("Scope should exist here"),
                    Some(receiver),
                    super_class_obj,
                    Some(class),
                );

                // We call getters, but return the actual function object for normal methods
                if matches!(property, Some(Property::Virtual { .. })) {
                    callee.call(receiver.into(), &[], activation)
                } else {
                    Ok(callee.into())
                }
            }
            Some(Property::Virtual { .. }) => Err(format!(
                "Attempting to use get_super on non-getter property {:?}",
                multiname
            )
            .into()),
            Some(Property::Slot { .. } | Property::ConstSlot { .. }) => {
                receiver.get_property(multiname, activation)
            }
            None => Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                multiname.local_name()
            )
            .into()),
        }
    }

    /// Supercall a setter defined in this class.
    ///
    /// This is intended to be called on the class object that is the
    /// superclass of the one that defined the currently called property. If no
    /// such superclass exists, you should use the class object for the
    /// receiver's actual type (i.e. the lowest in the chain). This ensures
    /// that repeated supercalls to the same setter will call parent and
    /// grandparent setter, and so on.
    ///
    /// If no setter exists with the given name, this falls back to setting a
    /// property of the `receiver`. This fallback only triggers if the property
    /// is associated with a trait. Dynamic properties will still error out.
    ///
    /// This function will search through the class object tree starting from
    /// this class up to `Object` for a setter trait with the given name. If it
    /// is found, it will be called with the receiver and value you provided,
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
        mut receiver: Object<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let property = self.instance_vtable().get_trait(multiname);
        if property.is_none() {
            return Err(format!(
                "Attempted to supercall method {:?}, which does not exist",
                multiname.local_name()
            )
            .into());
        }

        match property {
            Some(Property::Virtual {
                set: Some(disp_id), ..
            }) => {
                // todo: handle errors
                let ClassBoundMethod {
                    class,
                    super_class_obj,
                    scope,
                    method,
                } = self.instance_vtable().get_full_method(disp_id).unwrap();
                let callee =
                    FunctionObject::from_method(activation, method, scope.expect("Scope should exist here"), Some(receiver), super_class_obj, Some(class));

                callee.call(receiver.into(), &[value], activation)?;
                Ok(())
            }
            Some(Property::Slot { .. }) => {
                receiver.set_property(multiname, value, activation)?;
                Ok(())
            }
            _ => {
                Err(format!("set_super on {receiver:?} {multiname:?} with {value:?} resolved to unexpected property {property:?}").into())
            }
        }
    }

    pub fn add_application(
        &self,
        mc: &Mutation<'gc>,
        param: Option<Class<'gc>>,
        cls: ClassObject<'gc>,
    ) {
        unlock!(Gc::write(mc, self.0), ClassObjectData, applications)
            .borrow_mut()
            .insert(param, cls);
    }

    /// Parametrize this class. This does not check to ensure that this class is generic.
    pub fn parametrize(
        &self,
        activation: &mut Activation<'_, 'gc>,
        class_param: Option<Class<'gc>>,
    ) -> Result<ClassObject<'gc>, Error<'gc>> {
        let self_class = self.inner_class_definition();

        if let Some(application) = self.0.applications.borrow().get(&class_param) {
            return Ok(*application);
        }

        // if it's not a known application, then it's not int/uint/Number/*,
        // so it must be a simple Vector.<*>-derived class.

        let parameterized_class =
            Class::with_type_param(activation.context, self_class, class_param);

        // NOTE: this isn't fully accurate, but much simpler.
        // FP's Vector is more of special case that literally copies some parent class's properties
        // main example: Vector.<Object>.prototype === Vector.<*>.prototype

        let vector_star_cls = activation.avm2().classes().object_vector;
        let class_object =
            Self::from_class(activation, parameterized_class, Some(vector_star_cls))?;

        unlock!(
            Gc::write(activation.context.gc_context, self.0),
            ClassObjectData,
            applications
        )
        .borrow_mut()
        .insert(class_param, class_object);

        Ok(class_object)
    }

    pub fn translation_unit(self) -> Option<TranslationUnit<'gc>> {
        if let Method::Bytecode(bc) = self.constructor() {
            Some(bc.txunit)
        } else {
            None
        }
    }

    pub fn constructor(self) -> Method<'gc> {
        self.inner_class_definition().instance_init()
    }

    pub fn super_constructor(self) -> Method<'gc> {
        self.inner_class_definition().super_init()
    }

    pub fn call_handler(self) -> Option<Method<'gc>> {
        self.inner_class_definition().call_handler()
    }

    pub fn instance_vtable(self) -> VTable<'gc> {
        self.0.instance_vtable
    }

    pub fn inner_class_definition(self) -> Class<'gc> {
        self.0.class
    }

    pub fn prototype(self) -> Object<'gc> {
        self.0.prototype.get().unwrap()
    }

    pub fn class_scope(self) -> ScopeChain<'gc> {
        self.0.class_scope
    }

    pub fn instance_scope(self) -> ScopeChain<'gc> {
        self.0.instance_scope.get()
    }

    pub fn superclass_object(self) -> Option<ClassObject<'gc>> {
        self.0.superclass_object
    }

    fn instance_allocator(self) -> AllocatorFn {
        self.inner_class_definition().instance_allocator().0
    }

    /// Attempts to obtain the name of this class.
    /// If we are unable to read from a necessary `GcCell`,
    /// the returned value will be some kind of error message.
    ///
    /// This should only be used in a debug context, where
    /// we need infallible access to *something* to print
    /// out.
    pub fn debug_class_name(&self) -> Box<dyn Debug + 'gc> {
        let class_name = self.inner_class_definition().try_name();

        match class_name {
            Ok(class_name) => Box::new(class_name),
            Err(err) => Box::new(err),
        }
    }
}

impl<'gc> TObject<'gc> for ClassObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn to_string(&self, activation: &mut Activation<'_, 'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(AvmString::new_utf8(
            activation.context.gc_context,
            format!("[class {}]", self.0.class.name().local_name()),
        )
        .into())
    }

    fn to_locale_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.to_string(activation)
    }

    fn call(
        self,
        _receiver: Value<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if let Some(call_handler) = self.call_handler() {
            let scope = self.0.class_scope;
            exec(
                call_handler,
                scope,
                self.into(),
                self.superclass_object(),
                Some(self.inner_class_definition()),
                arguments,
                activation,
                self.into(),
            )
        } else if arguments.len() == 1 {
            arguments[0].coerce_to_type(activation, self.inner_class_definition())
        } else {
            Err(Error::AvmError(argument_error(
                activation,
                &format!(
                    "Error #1112: Argument count mismatch on class coercion.  Expected 1, got {}.",
                    arguments.len()
                ),
                1112,
            )?))
        }
    }

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error<'gc>> {
        let instance_allocator = self.instance_allocator();

        let instance = instance_allocator(self, activation)?;

        self.call_init(instance.into(), arguments, activation)?;

        Ok(instance)
    }

    fn as_class_object(&self) -> Option<ClassObject<'gc>> {
        Some(*self)
    }

    fn apply(
        &self,
        activation: &mut Activation<'_, 'gc>,
        nullable_params: &[Value<'gc>],
    ) -> Result<ClassObject<'gc>, Error<'gc>> {
        let self_class = self.inner_class_definition();

        if !self_class.is_generic() {
            return Err(make_error_1127(activation));
        }

        if nullable_params.len() != 1 {
            let class_name = self
                .inner_class_definition()
                .name()
                .to_qualified_name(activation.context.gc_context);

            return Err(Error::AvmError(type_error(
                activation,
                &format!(
                    "Error #1128: Incorrect number of type parameters for {}. Expected 1, got {}.",
                    class_name,
                    nullable_params.len()
                ),
                1128,
            )?));
        }

        //Because `null` is a valid parameter, we have to accept values as
        //parameters instead of objects. We coerce them to objects now.
        let object_param = match nullable_params[0] {
            Value::Null => None,
            v => Some(v),
        };
        let class_param = match object_param {
            None => None,
            Some(cls) => Some(
                cls.as_object()
                    .and_then(|c| c.as_class_object())
                    .ok_or_else(|| {
                        // Note: FP throws VerifyError #1107 here
                        format!(
                            "Cannot apply class {:?} with non-class parameter",
                            self_class.name()
                        )
                    })?
                    .inner_class_definition(),
            ),
        };

        self.parametrize(activation, class_param)
    }
}

impl PartialEq for ClassObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        Object::ptr_eq(*self, *other)
    }
}

impl Eq for ClassObject<'_> {}

impl Hash for ClassObject<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

impl Debug for ClassObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("ClassObject")
            .field("name", &self.debug_class_name())
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
