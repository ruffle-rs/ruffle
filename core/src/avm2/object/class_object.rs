//! Class object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::function::Executable;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::function_object::FunctionObject;
use crate::avm2::object::script_object::{ScriptObject, ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
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
    base_class_constr: Option<Object<'gc>>,

    /// The instance constructor function
    instance_constr: Executable<'gc>,

    /// The native instance constructor function
    native_instance_constr: Executable<'gc>,
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
        base_class_constr: Option<Object<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        //TODO: Class prototypes are *not* instances of their class and should
        //not be allocated by a deriver, but instead should be regular objects
        let mut class_proto = if let Some(mut base_class_constr) = base_class_constr {
            let base_proto = base_class_constr
                .get_property(
                    base_class_constr,
                    &QName::new(Namespace::public(), "prototype"),
                    activation,
                )?
                .coerce_to_object(activation)?;
            let derive = class.read().instance_deriver();
            derive(base_class_constr, base_proto, activation)?
        } else {
            ScriptObject::bare_object(activation.context.gc_context)
        };

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

            let mut interface = interface.unwrap().coerce_to_object(activation)?;
            let iface_proto = interface
                .get_property(
                    interface,
                    &QName::new(Namespace::public(), "prototype"),
                    activation,
                )?
                .coerce_to_object(activation)?;

            interfaces.push(iface_proto);
        }

        if !interfaces.is_empty() {
            class_proto.set_interfaces(activation.context.gc_context, interfaces);
        }

        let fn_proto = activation.avm2().prototypes().function;

        let class_read = class.read();
        let instance_constr = Executable::from_method(
            class.read().instance_init(),
            scope,
            None,
            activation.context.gc_context,
        );
        let native_instance_constr = Executable::from_method(
            class.read().native_instance_init(),
            scope,
            None,
            activation.context.gc_context,
        );

        let mut constr: Object<'gc> = ClassObject(GcCell::allocate(
            activation.context.gc_context,
            ClassObjectData {
                base: ScriptObjectData::base_new(
                    Some(fn_proto),
                    ScriptObjectClass::ClassConstructor(class, scope),
                ),
                class,
                scope,
                base_class_constr,
                instance_constr,
                native_instance_constr,
            },
        ))
        .into();

        constr.install_dynamic_property(
            activation.context.gc_context,
            QName::new(Namespace::public(), "prototype"),
            class_proto.into(),
        )?;
        class_proto.install_dynamic_property(
            activation.context.gc_context,
            QName::new(Namespace::public(), "constructor"),
            constr.into(),
        )?;

        if !class_read.is_class_initialized() {
            let class_initializer = class_read.class_init();
            let class_init_fn =
                FunctionObject::from_method(activation, class_initializer, scope, Some(constr));

            drop(class_read);
            class
                .write(activation.context.gc_context)
                .mark_class_initialized();

            class_init_fn.call(Some(constr), &[], activation, None)?;
        }

        Ok(constr)
    }

    /// Construct a builtin type from a Rust constructor and prototype.
    ///
    /// This function returns both the class constructor object and the
    /// class initializer to call before the class is used. The constructor
    /// should be used in all cases where the type needs to be referred to. You
    /// must call the class initializer yourself.
    ///
    /// `base_class` is allowed to be `None`, corresponding to a `null` value
    /// in the VM. This corresponds to no base class, and in practice appears
    /// to be limited to interfaces.
    pub fn from_builtin_constr(
        mc: MutationContext<'gc, '_>,
        base_class_constr: Option<Object<'gc>>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        mut prototype: Object<'gc>,
        fn_proto: Object<'gc>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
        let instance_constr =
            Executable::from_method(class.read().instance_init(), scope, None, mc);
        let native_instance_constr =
            Executable::from_method(class.read().native_instance_init(), scope, None, mc);
        let mut base: Object<'gc> = ClassObject(GcCell::allocate(
            mc,
            ClassObjectData {
                base: ScriptObjectData::base_new(
                    Some(fn_proto),
                    ScriptObjectClass::ClassConstructor(class, scope),
                ),
                class,
                scope,
                base_class_constr,
                instance_constr,
                native_instance_constr,
            },
        ))
        .into();

        base.install_dynamic_property(
            mc,
            QName::new(Namespace::public(), "prototype"),
            prototype.into(),
        )?;
        prototype.install_dynamic_property(
            mc,
            QName::new(Namespace::public(), "constructor"),
            base.into(),
        )?;

        let class_initializer = class.read().class_init();
        let class_constr = FunctionObject::from_method_and_proto(
            mc,
            class_initializer,
            scope,
            fn_proto,
            Some(base),
        );

        Ok((base, class_constr))
    }
}

impl<'gc> TObject<'gc> for ClassObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn to_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        if let ScriptObjectClass::ClassConstructor(class, ..) = self.0.read().base.class() {
            Ok(AvmString::new(mc, format!("[class {}]", class.read().name().local_name())).into())
        } else {
            Ok("function Function() {}".into())
        }
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
        base_constr: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let instance_constr = self.0.read().instance_constr;

        instance_constr.exec(receiver, arguments, activation, base_constr, self.into())
    }

    fn call_initializer(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        base_constr: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let instance_constr = self.0.read().instance_constr;

        instance_constr.exec(receiver, arguments, activation, base_constr, self.into())
    }

    fn call_native_initializer(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        base_constr: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let native_instance_constr = self.0.read().native_instance_constr;

        native_instance_constr.exec(receiver, arguments, activation, base_constr, self.into())
    }

    fn construct(
        mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let class = self.as_class().ok_or("Cannot construct classless class!")?;
        let deriver = class.read().instance_deriver();
        let constr: Object<'gc> = self.into();
        let prototype = self
            .get_property(
                constr,
                &QName::new(Namespace::public(), "prototype"),
                activation,
            )?
            .coerce_to_object(activation)?;

        let instance = deriver(constr, prototype, activation)?;

        self.call_initializer(Some(instance), arguments, activation, Some(constr))?;

        Ok(instance)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        Ok(ClassObject(GcCell::allocate(
            activation.context.gc_context,
            self.0.read().clone(),
        ))
        .into())
    }

    /// Get the base class constructor of this object.
    fn base_class_constr(self) -> Option<Object<'gc>> {
        self.0.read().base_class_constr
    }
}
