//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{AllocatorFn, Class, CustomConstructorFn};
use crate::avm2::error::{argument_error, make_error_1127, type_error};
use crate::avm2::function::{exec, FunctionArgs};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{Object, ScriptObject, TObject};
use crate::avm2::scope::{Scope, ScopeChain};
use crate::avm2::value::Value;
use crate::avm2::vtable::VTable;
use crate::avm2::Error;
use crate::avm2::QName;
use crate::avm2::TranslationUnit;
use crate::string::AvmString;
use crate::utils::HasPrefixField;
use fnv::FnvHashMap;
use gc_arena::barrier::unlock;
use gc_arena::{
    lock::{Lock, RefLock},
    Collect, Gc, GcWeak, Mutation,
};
use ruffle_macros::istr;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// An Object which can be called to execute its function code.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct ClassObject<'gc>(pub Gc<'gc, ClassObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct ClassObjectWeak<'gc>(pub GcWeak<'gc, ClassObjectData<'gc>>);

#[derive(Collect, Clone, HasPrefixField)]
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
    ///
    /// Newly-created `ClassObject`s begin with a dummy vtable: the
    /// `init_instance_vtable` method should be called to replace it with the
    /// real vtable.
    instance_vtable: Lock<VTable<'gc>>,
}

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
    ) -> Object<'gc> {
        let proto = ScriptObject::new_object(activation);

        if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object.prototype();
            proto.set_proto(activation.gc(), base_proto);
        }
        proto
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
        let class_object = Self::from_class_partial(activation, class, superclass_object);

        let class_proto = class_object.allocate_prototype(activation, superclass_object);
        class_object.link_prototype(activation, class_proto);

        let class_class_proto = activation.avm2().classes().class.prototype();
        class_object.link_type(activation.gc(), class_class_proto);

        class_object.into_finished_class(activation);

        class_object.validate_class(activation)?;
        class_object.run_class_initializer(activation)?;

        Ok(class_object)
    }

    /// Allocate a class but do not properly construct it.
    ///
    /// This function does the bare minimum to allocate classes, without taking
    /// any action that would require the existence of any other objects in the
    /// object graph. The resulting class will be a bare object and should not
    /// be used or presented to user code until you finish initializing it. You
    /// do that by calling `link_prototype`, `link_type`, `into_finished_class`,
    /// `validate_class`, and `run_class_initializer` in that order.
    ///
    /// This returns the class object directly (*not* an `Object`), to allow
    /// further manipulation of the class once it's dependent types have been
    /// allocated.
    pub fn from_class_partial(
        activation: &mut Activation<'_, 'gc>,
        class: Class<'gc>,
        superclass_object: Option<ClassObject<'gc>>,
    ) -> Self {
        let c_class = class
            .c_class()
            .expect("Can only call ClassObject::from_class on i_classes");

        let scope = activation.create_scopechain();

        let mc = activation.gc();

        let class_object = ClassObject(Gc::new(
            activation.gc(),
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
                instance_vtable: Lock::new(c_class.vtable()),
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
        class_object.init_instance_vtable(activation);

        class.add_class_object(activation.gc(), class_object);

        class_object
    }

    fn init_instance_vtable(self, activation: &mut Activation<'_, 'gc>) {
        let mc = activation.gc();
        let class = self.inner_class_definition();

        let vtable = VTable::new_with_interface_properties(
            class,
            self.superclass_object(),
            Some(self.instance_scope()),
            self.superclass_object().map(|cls| cls.instance_vtable()),
            activation.context,
        );

        unlock!(Gc::write(mc, self.0), ClassObjectData, instance_vtable).set(vtable);
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
    pub fn into_finished_class(self, activation: &mut Activation<'_, 'gc>) {
        let i_class = self.inner_class_definition();
        let c_class = i_class
            .c_class()
            .expect("ClassObject should have an i_class");

        let class_classobject = activation.avm2().classes().class;

        // class vtable == class traits + Class instance traits
        let class_vtable = VTable::new(
            c_class,
            Some(class_classobject),
            Some(self.class_scope()),
            Some(class_classobject.instance_vtable()),
            activation.gc(),
        );

        self.set_vtable(activation.gc(), class_vtable);
    }

    /// Link this class to a prototype.
    pub fn link_prototype(self, activation: &mut Activation<'_, 'gc>, class_proto: Object<'gc>) {
        let mc = activation.gc();

        unlock!(Gc::write(mc, self.0), ClassObjectData, prototype).set(Some(class_proto));
        class_proto.set_dynamic_property(istr!("constructor"), self.into(), mc);
        class_proto.set_local_property_is_enumerable(mc, istr!("constructor"), false);
    }

    /// Manually set the type of this `Class`.
    ///
    /// This is intended to support initialization of early types such as
    /// `Class` and `Object`. All other types should pull `Class`'s prototype
    /// and type object from the `Avm2` instance.
    pub fn link_type(self, gc_context: &Mutation<'gc>, proto: Object<'gc>) {
        self.base().set_proto(gc_context, proto);
    }

    /// Validate signatures of the class.
    pub fn validate_class(self, activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
        let class = self.inner_class_definition();
        let c_class = class.c_class().expect("ClassObject stores an i_class");

        class.validate_signatures(activation)?;
        c_class.validate_signatures(activation)?;

        Ok(())
    }

    /// Run the class's initializer method.
    pub fn run_class_initializer(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        let c_class = self
            .inner_class_definition()
            .c_class()
            .expect("ClassObject stores an i_class");

        let self_value: Value<'gc> = self.into();
        let class_classobject = activation.avm2().classes().class;

        let scope = self.0.class_scope;

        let Some(class_initializer) = c_class.instance_init() else {
            unreachable!("c_classes should always have initializer methods")
        };

        let class_init_fn = FunctionObject::from_method(
            activation,
            class_initializer,
            scope,
            Some(self_value),
            Some(class_classobject),
            Some(c_class),
        );

        class_init_fn.call(activation, self_value, &[])?;

        Ok(())
    }

    /// Call the instance initializer.
    ///
    /// This method may panic if called with a Null or Undefined receiver.
    pub fn call_init(
        self,
        receiver: Value<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.call_init_with_args(receiver, FunctionArgs::AsArgSlice { arguments }, activation)
    }

    pub fn call_init_with_args(
        self,
        receiver: Value<'gc>,
        arguments: FunctionArgs<'_, 'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let scope = self.0.instance_scope.get();
        let method = self.init_method();

        if let Some(method) = method {
            // Provide a callee object if necessary
            let callee = if method.needs_arguments_object() {
                Some(FunctionObject::from_method(
                    activation,
                    method,
                    scope,
                    Some(receiver),
                    self.superclass_object(),
                    Some(self.inner_class_definition()),
                ))
            } else {
                None
            };

            exec(
                method,
                scope,
                receiver,
                self.superclass_object(),
                Some(self.inner_class_definition()),
                arguments,
                activation,
                callee,
            )
        } else {
            unreachable!("Cannot instantiate a class without an init method")
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
            Gc::write(activation.gc(), self.0),
            ClassObjectData,
            applications
        )
        .borrow_mut()
        .insert(class_param, class_object);

        Ok(class_object)
    }

    pub fn call(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        if let Some(call_handler) = self.call_handler() {
            call_handler(activation, self.into(), arguments)
        } else if arguments.len() == 1 {
            arguments[0].coerce_to_type(activation, self.inner_class_definition())
        } else {
            Err(Error::avm_error(argument_error(
                activation,
                &format!(
                    "Error #1112: Argument count mismatch on class coercion.  Expected 1, got {}.",
                    arguments.len()
                ),
                1112,
            )?))
        }
    }

    pub fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.construct_with_args(activation, FunctionArgs::AsArgSlice { arguments })
    }

    pub fn construct_with_args(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: FunctionArgs<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        if let Some(custom_constructor) = self.custom_constructor() {
            let arguments = &arguments.to_slice();
            custom_constructor(activation, arguments)
        } else {
            let instance_allocator = self.instance_allocator();

            let instance = instance_allocator(self, activation)?;

            self.call_init_with_args(instance.into(), arguments, activation)?;

            Ok(instance.into())
        }
    }

    pub fn translation_unit(self) -> Option<TranslationUnit<'gc>> {
        self.init_method().map(|m| m.translation_unit())
    }

    pub fn init_method(self) -> Option<Method<'gc>> {
        self.inner_class_definition().instance_init()
    }

    pub fn call_handler(self) -> Option<NativeMethodImpl> {
        self.inner_class_definition().call_handler()
    }

    pub fn instance_vtable(self) -> VTable<'gc> {
        self.0.instance_vtable.get()
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

    fn custom_constructor(self) -> Option<CustomConstructorFn> {
        self.inner_class_definition()
            .custom_constructor()
            .map(|c| c.0)
    }

    pub fn name(&self) -> QName<'gc> {
        self.inner_class_definition().name()
    }
}

impl<'gc> TObject<'gc> for ClassObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn to_string(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let class_name = self.0.class.name().local_name();

        AvmString::new_utf8(mc, format!("[class {class_name}]"))
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
                .to_qualified_name(activation.gc());

            return Err(Error::avm_error(type_error(
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
            .field("name", &self.name())
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
