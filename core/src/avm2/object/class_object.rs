//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Allocator, AllocatorFn, Class};
use crate::avm2::function::Executable;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::{scriptobject_allocator, ScriptObject, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;

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
    scope: Option<GcCell<'gc, Scope<'gc>>>,

    /// The base class of this one.
    ///
    /// If `None`, this class has no parent. In practice, this is only used for
    /// interfaces (at least by the AS3 compiler in Animate CC 2020.)
    superclass_object: Option<Object<'gc>>,

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
    params: Option<Option<Object<'gc>>>,

    /// List of all applications of this class.
    ///
    /// Only applicable if this class is generic.
    ///
    /// It is legal to apply a type with the value `null`, which is represented
    /// as `None` here. AVM2 considers both applications to be separate
    /// classes, though we consider the parameter to be the class `Object` when
    /// we get a param of `null`.
    applications: HashMap<Option<Object<'gc>>, Object<'gc>>,

    /// Interfaces implemented by this class.
    interfaces: Vec<Object<'gc>>,
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
        superclass_object: Option<Object<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let class_object = Self::from_class_partial(activation, class, superclass_object, scope)?;

        //TODO: Class prototypes are *not* instances of their class and should
        //not be allocated by the class allocator, but instead should be
        //regular objects
        let class_proto = if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object
                .get_property(
                    superclass_object,
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
        superclass_object: Option<Object<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Self, Error> {
        if let Some(base_class) = superclass_object.and_then(|b| b.as_class_definition()) {
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
            .or_else(|| {
                superclass_object
                    .and_then(|c| c.as_class_object())
                    .and_then(|c| c.instance_allocator())
            })
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
                scope,
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
    ) -> Result<Object<'gc>, Error> {
        let class = self.inner_class_definition();
        let class_class = self.instance_of().ok_or(
            "Cannot finish initialization of core class without it being linked to a type!",
        )?;

        self.link_interfaces(activation)?;
        self.install_traits(activation, class.read().class_traits())?;
        self.install_instance_traits(activation, class_class)?;
        self.run_class_initializer(activation)?;

        Ok(self.into())
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
        let scope = self.get_scope();

        let interface_names = class.read().interfaces().to_vec();
        let mut interfaces = Vec::with_capacity(interface_names.len());
        for interface_name in interface_names {
            let interface = if let Some(scope) = scope {
                scope
                    .write(activation.context.gc_context)
                    .resolve(&interface_name, activation)?
            } else {
                None
            };

            if interface.is_none() {
                return Err(format!("Could not resolve interface {:?}", interface_name).into());
            }

            let interface = interface.unwrap().coerce_to_object(activation)?;
            if let Some(class) = interface.as_class_definition() {
                if !class.read().is_interface() {
                    return Err(format!(
                        "Class {:?} is not an interface and cannot be implemented by classes",
                        class.read().name().local_name()
                    )
                    .into());
                }
            }

            interfaces.push(interface);
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
        instance_of: Object<'gc>,
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

        let class = self.0.read().class;
        let class_read = class.read();

        if !class_read.is_class_initialized() {
            let class_initializer = class_read.class_init();
            let class_init_fn = FunctionObject::from_method(
                activation,
                class_initializer,
                self.get_scope(),
                Some(object),
            );

            drop(class_read);
            class
                .write(activation.context.gc_context)
                .mark_class_initialized();

            class_init_fn.call(Some(object), &[], activation, None)?;
        }

        Ok(())
    }

    pub fn inner_class_definition(self) -> GcCell<'gc, Class<'gc>> {
        self.0.read().class
    }

    pub fn interfaces(self) -> Vec<Object<'gc>> {
        self.0.read().interfaces.clone()
    }

    pub fn superclass_object(self) -> Option<Object<'gc>> {
        self.0.read().superclass_object
    }

    pub fn as_class_params(self) -> Option<Option<Object<'gc>>> {
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
        _superclass_object: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        arguments
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_type(activation, self.into())
    }

    fn call_init(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        superclass_object: Option<Object<'gc>>,
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

    fn call_native_init(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        superclass_object: Option<Object<'gc>>,
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

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let instance_allocator = self.0.read().instance_allocator.0;
        let class_object: Object<'gc> = self.into();
        let prototype = self
            .get_property(
                class_object,
                &QName::new(Namespace::public(), "prototype").into(),
                activation,
            )?
            .coerce_to_object(activation)?;

        let mut instance = instance_allocator(class_object, prototype, activation)?;

        instance.install_instance_traits(activation, class_object)?;

        self.call_init(Some(instance), arguments, activation, Some(class_object))?;

        Ok(instance)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        Ok(ClassObject(GcCell::allocate(
            activation.context.gc_context,
            self.0.read().clone(),
        ))
        .into())
    }

    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.0.read().scope
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
    ) -> Result<Object<'gc>, Error> {
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
        let mut object_params = Vec::new();
        for param in nullable_params {
            object_params.push(match param {
                Value::Null => None,
                Value::Undefined => return Err("Undefined is not a valid type parameter".into()),
                v => Some(v.coerce_to_object(activation)?),
            });
        }

        if let Some(application) = self.0.read().applications.get(&object_params[0]) {
            return Ok(*application);
        }

        let mut class_params = Vec::new();
        for param in object_params.iter() {
            class_params.push(
                param
                    .unwrap_or(activation.avm2().classes().object)
                    .as_class_definition()
                    .ok_or(format!(
                        "Cannot apply class {:?} with non-class parameter",
                        self_class.read().name()
                    ))?,
            );
        }

        let parameterized_class = self_class
            .read()
            .with_type_params(&class_params, activation.context.gc_context);

        let scope = self.get_scope();
        let instance_allocator = self.0.read().instance_allocator.clone();
        let superclass_object = self.0.read().superclass_object;

        //TODO: Class prototypes are *not* instances of their class and should
        //not be allocated by the class allocator, but instead should be
        //regular objects
        let class_proto = if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object
                .get_property(
                    superclass_object,
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
                params: Some(object_params[0]),
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
            .insert(object_params[0], class_object.into());

        Ok(class_object.into())
    }
}
