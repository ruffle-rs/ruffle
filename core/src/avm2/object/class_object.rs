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

    /// The scope this class was defined in
    scope: Option<GcCell<'gc, Scope<'gc>>>,

    /// The instance constructor function
    instance_constr: Executable<'gc>,
}

pub fn implicit_deriver<'gc>(
    base_proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
    class: GcCell<'gc, Class<'gc>>,
    scope: Option<GcCell<'gc, Scope<'gc>>>,
) -> Result<Object<'gc>, Error> {
    base_proto.derive(activation, class, scope)
}

impl<'gc> ClassObject<'gc> {
    /// Construct a class.
    ///
    /// This function returns both the class itself, and the static class
    /// initializer method that you should call before interacting with the
    /// class. The latter should be called using the former as a receiver.
    ///
    /// `base_class` is allowed to be `None`, corresponding to a `null` value
    /// in the VM. This corresponds to no base class, and in practice appears
    /// to be limited to interfaces (at least by the AS3 compiler in Animate
    /// CC 2020.)
    pub fn from_class(
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        base_class: Option<Object<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
        ClassObject::from_class_with_deriver(activation, class, base_class, scope, implicit_deriver)
    }

    /// Construct a class with a different `TObject` implementation than its
    /// base class.
    ///
    /// This is identical to `from_class`, save for the fact that you must also
    /// provide a deriver function to create the subclass prototype with. This
    /// accepts the superclass's prototype, the activation, class definition,
    /// and scope. It must return the prototype object that should be used to
    /// create the class.
    pub fn from_class_with_deriver<DERIVE>(
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        base_class: Option<Object<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        derive: DERIVE,
    ) -> Result<(Object<'gc>, Object<'gc>), Error>
    where
        DERIVE: FnOnce(
            Object<'gc>,
            &mut Activation<'_, 'gc, '_>,
            GcCell<'gc, Class<'gc>>,
            Option<GcCell<'gc, Scope<'gc>>>,
        ) -> Result<Object<'gc>, Error>,
    {
        let class_read = class.read();
        let class_proto = if let Some(mut base_class) = base_class {
            let super_proto: Result<_, Error> = base_class
                .get_property(
                    base_class,
                    &QName::new(Namespace::public(), "prototype"),
                    activation,
                )?
                .coerce_to_object(activation)
                .map_err(|_| {
                    format!(
                        "Could not resolve superclass prototype {:?}",
                        class_read
                            .super_class_name()
                            .as_ref()
                            .map(|p| p.local_name())
                            .unwrap_or_else(|| Some("Object".into()))
                    )
                    .into()
                });

            derive(super_proto?, activation, class, scope)?
        } else {
            ScriptObject::bare_object(activation.context.gc_context)
        };

        ClassObject::from_class_and_proto(activation, class, class_proto, scope)
    }

    /// Construct a class with a custom object type as its prototype.
    fn from_class_and_proto(
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        mut class_proto: Object<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
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
        let class_constr_proto = activation.avm2().prototypes().class;

        let class_read = class.read();
        let instance_constr = Executable::from_method(
            class.read().instance_init(),
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
                instance_constr,
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

        let class_initializer = class_read.class_init();
        let class_constr = FunctionObject::from_method(
            activation.context.gc_context,
            class_initializer,
            scope,
            class_constr_proto,
            None,
        );

        Ok((constr, class_constr))
    }

    /// Construct a builtin type from a Rust constructor and prototype.
    pub fn from_builtin_constr(
        mc: MutationContext<'gc, '_>,
        mut prototype: Object<'gc>,
        fn_proto: Object<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let scope = prototype.get_scope();
        let class: Result<_, Error> = prototype
            .as_class()
            .ok_or_else(|| "Cannot construct builtin type without a class".into());
        let class = class?;
        let instance_constr =
            Executable::from_method(class.read().instance_init(), scope, None, mc);
        let mut base: Object<'gc> = ClassObject(GcCell::allocate(
            mc,
            ClassObjectData {
                base: ScriptObjectData::base_new(
                    Some(fn_proto),
                    ScriptObjectClass::ClassConstructor(class, scope),
                ),
                class,
                scope,
                instance_constr,
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

        Ok(base)
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
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        let instance_constr = self.0.read().instance_constr;

        instance_constr.exec(receiver, arguments, activation, base_proto, self.into())
    }

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Ok(ClassObject(GcCell::allocate(
            activation.context.gc_context,
            self.0.read().clone(),
        ))
        .into())
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _class: GcCell<'gc, Class<'gc>>,
        _scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        Ok(ClassObject(GcCell::allocate(
            activation.context.gc_context,
            self.0.read().clone(),
        ))
        .into())
    }
}
