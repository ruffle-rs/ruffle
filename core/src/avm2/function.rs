//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{BytecodeMethod, Method, NativeMethod};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::r#trait::Trait;
use crate::avm2::scope::Scope;
use crate::avm2::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::context::UpdateContext;
use gc_arena::{Collect, CollectionContext, Gc, GcCell, MutationContext};
use std::fmt;

/// Represents code written in AVM2 bytecode that can be executed by some
/// means.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BytecodeExecutable<'gc> {
    /// The method code to execute from a given ABC file.
    method: Gc<'gc, BytecodeMethod<'gc>>,

    /// The scope stack to pull variables from.
    scope: Option<GcCell<'gc, Scope<'gc>>>,

    /// The reciever that this function is always called with.
    ///
    /// If `None`, then the reciever provided by the caller is used. A
    /// `Some` value indicates a bound executable.
    reciever: Option<Object<'gc>>,
}

/// Represents code that can be executed by some means.
#[derive(Copy, Clone)]
pub enum Executable<'gc> {
    /// Code defined in Ruffle's binary.
    ///
    /// The second parameter stores the bound reciever for this function.
    Native(NativeMethod<'gc>, Option<Object<'gc>>),

    /// Code defined in a loaded ABC file.
    Action(Gc<'gc, BytecodeExecutable<'gc>>),
}

unsafe impl<'gc> Collect for Executable<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Action(be) => be.trace(cc),
            Self::Native(_nf, reciever) => reciever.trace(cc),
        }
    }
}

impl<'gc> Executable<'gc> {
    /// Convert a method into an executable.
    pub fn from_method(
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        reciever: Option<Object<'gc>>,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        match method {
            Method::Native(nf) => Self::Native(nf, reciever),
            Method::Entry(a2me) => Self::Action(Gc::allocate(
                mc,
                BytecodeExecutable {
                    method: a2me,
                    scope,
                    reciever,
                },
            )),
        }
    }

    /// Execute a method.
    ///
    /// The function will either be called directly if it is a Rust builtin, or
    /// executed on the same AVM2 instance as the activation passed in here.
    /// The value returned in either case will be provided here.
    ///
    /// It is a panicing logic error to attempt to execute user code while any
    /// reachable object is currently under a GcCell write lock.
    pub fn exec(
        &self,
        unbound_reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        match self {
            Executable::Native(nf, reciever) => nf(
                activation,
                context,
                reciever.or(unbound_reciever),
                arguments,
            ),
            Executable::Action(bm) => {
                let reciever = bm.reciever.or(unbound_reciever);
                let mut activation = Activation::from_method(
                    activation.avm2(),
                    context,
                    bm.method,
                    bm.scope,
                    reciever,
                    arguments,
                    base_proto,
                )?;

                activation.run_actions(bm.method, context)
            }
        }
    }
}

impl<'gc> fmt::Debug for Executable<'gc> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Action(be) => fmt
                .debug_struct("Executable::Action")
                .field("method", &be.method)
                .field("scope", &be.scope)
                .field("reciever", &be.reciever)
                .finish(),
            Self::Native(nf, reciever) => fmt
                .debug_tuple("Executable::Native")
                .field(&format!("{:p}", nf))
                .field(reciever)
                .finish(),
        }
    }
}

/// An Object which can be called to execute it's function code.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct FunctionObject<'gc>(GcCell<'gc, FunctionObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
#[collect(no_drop)]
pub struct FunctionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Executable code
    exec: Option<Executable<'gc>>,
}

impl<'gc> FunctionObject<'gc> {
    /// Construct a class.
    ///
    /// This function returns both the class itself, and the static class
    /// initializer method that you should call before interacting with the
    /// class. The latter should be called using the former as a reciever.
    pub fn from_class(
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        mut base_class: Object<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
        let class_read = class.read();
        let super_proto: Result<Object<'gc>, Error> = base_class
            .get_property(
                base_class,
                &QName::new(Namespace::public_namespace(), "prototype"),
                activation,
                context,
            )?
            .as_object()
            .map_err(|_| {
                format!(
                    "Could not resolve superclass prototype {:?}",
                    class_read
                        .super_class_name()
                        .as_ref()
                        .map(|p| p.local_name())
                        .unwrap_or(Some("Object"))
                )
                .into()
            });
        let mut class_proto = super_proto?.derive(activation, context, class, scope)?;
        let fn_proto = activation.avm2().prototypes().function;
        let class_constr_proto = activation.avm2().prototypes().class;

        let initializer = class_read.instance_init();

        let mut constr: Object<'gc> = FunctionObject(GcCell::allocate(
            context.gc_context,
            FunctionObjectData {
                base: ScriptObjectData::base_new(
                    Some(fn_proto),
                    ScriptObjectClass::ClassConstructor(class, scope),
                ),
                exec: Some(Executable::from_method(
                    initializer,
                    scope,
                    None,
                    context.gc_context,
                )),
            },
        ))
        .into();

        constr.install_dynamic_property(
            context.gc_context,
            QName::new(Namespace::public_namespace(), "prototype"),
            class_proto.into(),
        )?;
        class_proto.install_dynamic_property(
            context.gc_context,
            QName::new(Namespace::public_namespace(), "constructor"),
            constr.into(),
        )?;

        let class_initializer = class_read.class_init();
        let class_constr = FunctionObject::from_method(
            context.gc_context,
            class_initializer,
            scope,
            class_constr_proto,
            None,
        );

        Ok((constr, class_constr))
    }

    /// Construct a function from an ABC method and the current closure scope.
    ///
    /// The given `reciever`, if supplied, will override any user-specified
    /// `this` parameter.
    pub fn from_method(
        mc: MutationContext<'gc, '_>,
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
        reciever: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let exec = Some(Executable::from_method(method, scope, reciever, mc));

        FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), ScriptObjectClass::NoClass),
                exec,
            },
        ))
        .into()
    }

    /// Construct a builtin function object from a Rust function.
    pub fn from_builtin(
        mc: MutationContext<'gc, '_>,
        nf: NativeMethod<'gc>,
        fn_proto: Object<'gc>,
    ) -> Object<'gc> {
        FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), ScriptObjectClass::NoClass),
                exec: Some(Executable::from_method(nf.into(), None, None, mc)),
            },
        ))
        .into()
    }

    /// Construct a builtin type from a Rust constructor and prototype.
    pub fn from_builtin_constr(
        mc: MutationContext<'gc, '_>,
        constr: NativeMethod<'gc>,
        mut prototype: Object<'gc>,
        fn_proto: Object<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let mut base: Object<'gc> = FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), ScriptObjectClass::NoClass),
                exec: Some(Executable::from_method(constr.into(), None, None, mc)),
            },
        ))
        .into();

        base.install_dynamic_property(
            mc,
            QName::new(Namespace::public_namespace(), "prototype"),
            prototype.into(),
        )?;
        prototype.install_dynamic_property(
            mc,
            QName::new(Namespace::public_namespace(), "constructor"),
            base.into(),
        )?;

        Ok(base)
    }
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    fn get_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let read = self.0.read();
        let rv = read.base.get_property_local(reciever, name, activation)?;

        drop(read);

        rv.resolve(activation, context)
    }

    fn set_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(context.gc_context);
        let rv = write
            .base
            .set_property_local(reciever, name, value, activation, context)?;

        drop(write);

        rv.resolve(activation, context)?;

        Ok(())
    }

    fn init_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let mut write = self.0.write(context.gc_context);
        let rv = write
            .base
            .init_property_local(reciever, name, value, activation, context)?;

        drop(write);

        rv.resolve(activation, context)?;

        Ok(())
    }

    fn is_property_overwritable(self, gc_context: MutationContext<'gc, '_>, name: &QName) -> bool {
        self.0.write(gc_context).base.is_property_overwritable(name)
    }

    fn delete_property(&self, gc_context: MutationContext<'gc, '_>, multiname: &QName) -> bool {
        self.0.write(gc_context).base.delete_property(multiname)
    }

    fn get_slot(self, id: u32) -> Result<Value<'gc>, Error> {
        self.0.read().base.get_slot(id)
    }

    fn set_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).base.set_slot(id, value, mc)
    }

    fn init_slot(
        self,
        id: u32,
        value: Value<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<(), Error> {
        self.0.write(mc).base.init_slot(id, value, mc)
    }

    fn get_method(self, id: u32) -> Option<Object<'gc>> {
        self.0.read().base.get_method(id)
    }

    fn get_trait(self, name: &QName) -> Result<Vec<Trait<'gc>>, Error> {
        self.0.read().base.get_trait(name)
    }

    fn get_provided_trait(
        &self,
        name: &QName,
        known_traits: &mut Vec<Trait<'gc>>,
    ) -> Result<(), Error> {
        self.0.read().base.get_provided_trait(name, known_traits)
    }

    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.0.read().base.get_scope()
    }

    fn resolve_any(self, local_name: &str) -> Result<Option<Namespace>, Error> {
        self.0.read().base.resolve_any(local_name)
    }

    fn resolve_any_trait(self, local_name: &str) -> Result<Option<Namespace>, Error> {
        self.0.read().base.resolve_any_trait(local_name)
    }

    fn has_own_property(self, name: &QName) -> Result<bool, Error> {
        self.0.read().base.has_own_property(name)
    }

    fn has_trait(self, name: &QName) -> Result<bool, Error> {
        self.0.read().base.has_trait(name)
    }

    fn provides_trait(self, name: &QName) -> Result<bool, Error> {
        self.0.read().base.provides_trait(name)
    }

    fn has_instantiated_property(self, name: &QName) -> bool {
        self.0.read().base.has_instantiated_property(name)
    }

    fn has_own_virtual_getter(self, name: &QName) -> bool {
        self.0.read().base.has_own_virtual_getter(name)
    }

    fn has_own_virtual_setter(self, name: &QName) -> bool {
        self.0.read().base.has_own_virtual_setter(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
    }

    fn get_enumerant_name(&self, index: u32) -> Option<QName> {
        self.0.read().base.get_enumerant_name(index)
    }

    fn property_is_enumerable(&self, name: &QName) -> bool {
        self.0.read().base.property_is_enumerable(name)
    }

    fn set_local_property_is_enumerable(
        &self,
        mc: MutationContext<'gc, '_>,
        name: &QName,
        is_enumerable: bool,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .set_local_property_is_enumerable(name, is_enumerable)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.0.read().exec
    }

    fn call(
        self,
        reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        if let Some(exec) = &self.0.read().exec {
            exec.exec(reciever, arguments, activation, context, base_proto)
        } else {
            Err("Not a callable function!".into())
        }
    }

    fn construct(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::FunctionObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(FunctionObject(GcCell::allocate(
            context.gc_context,
            FunctionObjectData { base, exec: None },
        ))
        .into())
    }

    fn derive(
        &self,
        _activation: &mut Activation<'_, 'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::FunctionObject(*self);
        let base = ScriptObjectData::base_new(
            Some(this),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(FunctionObject(GcCell::allocate(
            context.gc_context,
            FunctionObjectData { base, exec: None },
        ))
        .into())
    }

    fn to_string(&self) -> Result<Value<'gc>, Error> {
        if let ScriptObjectClass::ClassConstructor(class, ..) = self.0.read().base.class() {
            Ok(format!("[class {}]", class.read().name().local_name()).into())
        } else {
            Ok("function Function() {}".into())
        }
    }

    fn value_of(&self) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn install_method(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) {
        self.0
            .write(mc)
            .base
            .install_method(name, disp_id, function)
    }

    fn install_getter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .install_getter(name, disp_id, function)
    }

    fn install_setter(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        disp_id: u32,
        function: Object<'gc>,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .install_setter(name, disp_id, function)
    }

    fn install_dynamic_property(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        value: Value<'gc>,
    ) -> Result<(), Error> {
        self.0.write(mc).base.install_dynamic_property(name, value)
    }

    fn install_slot(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        id: u32,
        value: Value<'gc>,
    ) {
        self.0.write(mc).base.install_slot(name, id, value)
    }

    fn install_const(
        &mut self,
        mc: MutationContext<'gc, '_>,
        name: QName,
        id: u32,
        value: Value<'gc>,
    ) {
        self.0.write(mc).base.install_const(name, id, value)
    }
}
