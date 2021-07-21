//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::{Allocator, AllocatorFn, Class};
use crate::avm2::function::Executable;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::{scriptobject_allocator, ScriptObject, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};

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
        if let Some(base_class) = superclass_object.and_then(|b| b.as_class()) {
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

        //TODO: Class prototypes are *not* instances of their class and should
        //not be allocated by the class allocator, but instead should be
        //regular objects
        let mut class_proto = if let Some(superclass_object) = superclass_object {
            let base_proto = superclass_object
                .get_property(
                    superclass_object,
                    &QName::new(Namespace::public(), "prototype"),
                    activation,
                )?
                .coerce_to_object(activation)?;
            instance_allocator(superclass_object, base_proto, activation)?
        } else {
            ScriptObject::bare_object(activation.context.gc_context)
        };

        let fn_proto = activation.avm2().prototypes().function;

        let class_read = class.read();
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

        let mut class_object: Object<'gc> = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), None),
                class,
                scope,
                superclass_object,
                instance_allocator: Allocator(instance_allocator),
                constructor,
                native_constructor,
            },
        ))
        .into();

        class_object.install_slot(
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
            class_object.into(),
            false,
        );

        let interface_names = class.read().interfaces().to_vec();
        let mut interfaces = Vec::with_capacity(interface_names.len());
        for interface_name in interface_names {
            let interface = if let Some(scope) = scope {
                scope.read().resolve(&interface_name, activation)?
            } else {
                None
            };

            if interface.is_none() {
                return Err(format!("Could not resolve interface {:?}", interface_name).into());
            }

            let interface = interface.unwrap().coerce_to_object(activation)?;
            if let Some(class) = interface.as_class() {
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
            class_object.set_interfaces(activation.context.gc_context, interfaces);
        }

        class_object.install_traits(activation, class_read.class_traits())?;

        if !class_read.is_class_initialized() {
            let class_initializer = class_read.class_init();
            let class_init_fn = FunctionObject::from_method(
                activation,
                class_initializer,
                scope,
                Some(class_object),
            );

            drop(class_read);
            class
                .write(activation.context.gc_context)
                .mark_class_initialized();

            class_init_fn.call(Some(class_object), &[], activation, None)?;
        }

        Ok(class_object)
    }

    /// Construct a builtin type from a Rust constructor and prototype.
    ///
    /// This function returns both the class constructor object and the
    /// class initializer to call before the class is used. The constructor
    /// should be used in all cases where the type needs to be referred to. You
    /// must call the class initializer yourself.
    ///
    /// You are also required to install class constructor traits yourself onto
    /// the returned object. This is due to the fact that normal trait
    /// installation requires a working `context.avm2` with a link to the
    /// function prototype, and this is intended to be called before that link
    /// has been established.
    ///
    /// `base_class` is allowed to be `None`, corresponding to a `null` value
    /// in the VM. This corresponds to no base class, and in practice appears
    /// to be limited to interfaces.
    pub fn from_builtin_class(
        mc: MutationContext<'gc, '_>,
        superclass_object: Option<Object<'gc>>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        mut prototype: Object<'gc>,
        fn_proto: Object<'gc>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
        let instance_allocator = class
            .read()
            .instance_allocator()
            .or_else(|| superclass_object.and_then(|c| c.instance_allocator()))
            .unwrap_or(scriptobject_allocator);

        let constructor = Executable::from_method(class.read().instance_init(), scope, None, mc);
        let native_constructor =
            Executable::from_method(class.read().native_instance_init(), scope, None, mc);
        let mut base: Object<'gc> = ClassObject(GcCell::allocate(
            mc,
            ClassObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), None),
                class,
                scope,
                superclass_object,
                instance_allocator: Allocator(instance_allocator),
                constructor,
                native_constructor,
            },
        ))
        .into();

        base.install_slot(
            mc,
            QName::new(Namespace::public(), "prototype"),
            0,
            prototype.into(),
            false,
        );
        prototype.install_slot(
            mc,
            QName::new(Namespace::public(), "constructor"),
            0,
            base.into(),
            false,
        );

        let class_initializer = class.read().class_init();
        let class_object = FunctionObject::from_method_and_proto(
            mc,
            class_initializer,
            scope,
            fn_proto,
            Some(base),
        );

        Ok((base, class_object))
    }
}

impl<'gc> TObject<'gc> for ClassObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

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
                &QName::new(Namespace::public(), "prototype"),
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

    fn superclass_object(self) -> Option<Object<'gc>> {
        self.0.read().superclass_object
    }

    fn instance_allocator(self) -> Option<AllocatorFn> {
        Some(self.0.read().instance_allocator.0)
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

    fn as_class(&self) -> Option<GcCell<'gc, Class<'gc>>> {
        Some(self.0.read().class)
    }

    fn as_class_object(&self) -> Option<Object<'gc>> {
        None //AS3 does not have metaclasses
    }

    fn set_class_object(self, _mc: MutationContext<'gc, '_>, _class_object: Object<'gc>) {
        //Do nothing, as classes cannot be turned into instances.
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
}
