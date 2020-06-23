//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, CollectionContext, GcCell, MutationContext};
use std::fmt;
use std::rc::Rc;
use swf::avm2::types::{
    AbcFile, Class as AbcClass, Index, Instance as AbcInstance, Method as AbcMethod,
    MethodBody as AbcMethodBody, Trait as AbcTrait,
};

/// Represents a function defined in Ruffle's code.
///
/// Parameters are as follows:
///
///  * The AVM2 runtime
///  * The action context
///  * The current `this` object
///  * The arguments this function was called with
///
/// Native functions are allowed to return a value or `None`. `None` indicates
/// that the given value will not be returned on the stack and instead will
/// resolve on the AVM stack, as if you had called a non-native function. If
/// your function yields `None`, you must ensure that the top-most activation
/// in the AVM1 runtime will return with the value of this function.
pub type NativeFunction<'gc> = fn(
    &mut Avm2<'gc>,
    &mut UpdateContext<'_, 'gc, '_>,
    Option<Object<'gc>>,
    &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error>;

/// Represents a reference to an AVM2 method and body.
#[derive(Collect, Clone, Debug)]
#[collect(require_static)]
pub struct Avm2MethodEntry {
    /// The ABC file this function was defined in.
    pub abc: Rc<AbcFile>,

    /// The ABC method this function uses.
    pub abc_method: u32,

    /// The ABC method body this function uses.
    pub abc_method_body: u32,
}

impl Avm2MethodEntry {
    /// Construct an `Avm2MethodEntry` from an `AbcFile` and method index.
    ///
    /// The method body index will be determined by searching through the ABC
    /// for a matching method. If none exists, this function returns `None`.
    pub fn from_method_index(abc: Rc<AbcFile>, abc_method: Index<AbcMethod>) -> Option<Self> {
        if abc.methods.get(abc_method.0 as usize).is_some() {
            for (index, method_body) in abc.method_bodies.iter().enumerate() {
                if method_body.method.0 == abc_method.0 {
                    return Some(Self {
                        abc,
                        abc_method: abc_method.0,
                        abc_method_body: index as u32,
                    });
                }
            }
        }

        None
    }

    /// Get the underlying ABC file.
    #[allow(dead_code)]
    pub fn abc(&self) -> Rc<AbcFile> {
        self.abc.clone()
    }

    /// Get a reference to the ABC method entry this refers to.
    pub fn method(&self) -> &AbcMethod {
        self.abc.methods.get(self.abc_method as usize).unwrap()
    }

    /// Get a reference to the ABC method body entry this refers to.
    pub fn body(&self) -> &AbcMethodBody {
        self.abc
            .method_bodies
            .get(self.abc_method_body as usize)
            .unwrap()
    }
}

/// Represents an AVM2 function.
#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
pub struct Avm2Function<'gc> {
    /// The AVM method entry used to create this function.
    pub method: Avm2MethodEntry,

    /// Closure scope stack at time of creation
    pub scope: Option<GcCell<'gc, Scope<'gc>>>,

    /// The reciever this method is attached to.
    ///
    /// Objects without a reciever are free functions that can be invoked with
    /// any desired parameter for `this`.
    pub reciever: Option<Object<'gc>>,
}

impl<'gc> Avm2Function<'gc> {
    pub fn from_method(
        method: Avm2MethodEntry,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        reciever: Option<Object<'gc>>,
    ) -> Self {
        Self {
            method,
            scope,
            reciever,
        }
    }
}

/// Represents code that can be executed by some means.
#[derive(Clone)]
pub enum Executable<'gc> {
    Native(NativeFunction<'gc>),
    Action(Avm2Function<'gc>),
}

unsafe impl<'gc> Collect for Executable<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Self::Action(a2f) => a2f.trace(cc),
            Self::Native(_nf) => {}
        }
    }
}

impl<'gc> Executable<'gc> {
    /// Execute a method.
    ///
    /// The function will either be called directly if it is a Rust builtin, or
    /// placed on the stack of the passed-in AVM2 otherwise. As a result, we
    /// return a `ReturnValue` which can be used to force execution of the
    /// given stack frame and obtain it's return value or to push said value
    /// onto the AVM operand stack.
    pub fn exec(
        &self,
        unbound_reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Executable::Native(nf) => nf(avm, context, unbound_reciever, arguments),
            Executable::Action(a2f) => {
                let reciever = a2f.reciever.or(unbound_reciever);
                let activation = GcCell::allocate(
                    context.gc_context,
                    Activation::from_action(context, &a2f, reciever, arguments, base_proto)?,
                );

                avm.insert_stack_frame(activation);
                Ok(activation.into())
            }
        }
    }
}

impl<'gc> fmt::Debug for Executable<'gc> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Action(a2f) => fmt.debug_tuple("Executable::Action").field(a2f).finish(),
            Self::Native(nf) => fmt
                .debug_tuple("Executable::Native")
                .field(&format!("{:p}", nf))
                .finish(),
        }
    }
}

impl<'gc> From<NativeFunction<'gc>> for Executable<'gc> {
    fn from(nf: NativeFunction<'gc>) -> Self {
        Self::Native(nf)
    }
}

impl<'gc> From<Avm2Function<'gc>> for Executable<'gc> {
    fn from(a2f: Avm2Function<'gc>) -> Self {
        Self::Action(a2f)
    }
}

/// Represents a reference to an AVM2 class.
///
/// For some reason, this comes in two parts, one for static properties (called
/// the "class") and one for dynamic properties (called the "instance", even
/// though it really defines what ES3/AS2 would call a prototype)
#[derive(Collect, Clone, Debug)]
#[collect(require_static)]
pub struct Avm2ClassEntry {
    /// The ABC file this function was defined in.
    pub abc: Rc<AbcFile>,

    /// The ABC class (used to define static properties).
    ///
    /// This is also the index of the ABC instance, which holds instance
    /// properties.
    pub abc_class: u32,
}

impl Avm2ClassEntry {
    /// Construct an `Avm2MethodEntry` from an `AbcFile` and method index.
    ///
    /// This function returns `None` if the given class index does not resolve
    /// to a valid ABC class, or a valid ABC instance. As mentioned in the type
    /// documentation, ABC classes and instances are intended to be paired.
    pub fn from_class_index(abc: Rc<AbcFile>, abc_class: Index<AbcClass>) -> Option<Self> {
        if abc.classes.get(abc_class.0 as usize).is_some()
            && abc.instances.get(abc_class.0 as usize).is_some()
        {
            return Some(Self {
                abc,
                abc_class: abc_class.0,
            });
        }

        None
    }

    /// Get the underlying ABC file.
    pub fn abc(&self) -> Rc<AbcFile> {
        self.abc.clone()
    }

    /// Get a reference to the ABC class entry this refers to.
    pub fn class(&self) -> &AbcClass {
        self.abc.classes.get(self.abc_class as usize).unwrap()
    }

    /// Get a reference to the ABC class instance entry this refers to.
    pub fn instance(&self) -> &AbcInstance {
        self.abc.instances.get(self.abc_class as usize).unwrap()
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
    /// Construct a class from an ABC class/instance pair.
    ///
    /// This function returns both the class itself, and the static class
    /// initializer method that you should call before interacting with the
    /// class. The latter should be called using the former as a reciever.
    pub fn from_abc_class(
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        class: Avm2ClassEntry,
        mut base_class: Object<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
        let super_proto: Result<Object<'gc>, Error> = base_class
            .get_property(
                base_class,
                &QName::new(Namespace::public_namespace(), "prototype"),
                avm,
                context,
            )?
            .as_object()
            .map_err(|_| {
                let super_name = QName::from_abc_multiname(
                    &class.abc(),
                    class.instance().super_name.clone(),
                );

                if let Ok(super_name) = super_name {
                    format!(
                        "Could not resolve superclass prototype {:?}",
                        super_name.local_name()
                    )
                    .into()
                } else {
                    format!(
                        "Could not resolve superclass prototype, and got this error when getting it's name: {:?}",
                        super_name.unwrap_err()
                    )
                    .into()
                }
            });
        let mut class_proto = super_proto?.derive(avm, context, class.clone(), scope)?;
        let fn_proto = avm.prototypes().function;
        let class_constr_proto = avm.prototypes().class;

        let initializer_index = class.instance().init_method.clone();
        let initializer: Result<Avm2MethodEntry, Error> =
            Avm2MethodEntry::from_method_index(class.abc(), initializer_index.clone()).ok_or_else(
                || {
                    format!(
                        "Instance initializer method index {} does not exist",
                        initializer_index.0
                    )
                    .into()
                },
            );

        let mut constr: Object<'gc> = FunctionObject(GcCell::allocate(
            context.gc_context,
            FunctionObjectData {
                base: ScriptObjectData::base_new(
                    Some(fn_proto),
                    ScriptObjectClass::ClassConstructor(class.clone(), scope),
                ),
                exec: Some(Avm2Function::from_method(initializer?, scope, None).into()),
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

        let class_initializer_index = class.class().init_method.clone();
        let class_initializer: Result<Avm2MethodEntry, Error> =
            Avm2MethodEntry::from_method_index(class.abc(), class_initializer_index.clone())
                .ok_or_else(|| {
                    format!(
                        "Class initializer method index {} does not exist",
                        class_initializer_index.0
                    )
                    .into()
                });
        let class_constr = FunctionObject::from_abc_method(
            context.gc_context,
            class_initializer?,
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
    pub fn from_abc_method(
        mc: MutationContext<'gc, '_>,
        method: Avm2MethodEntry,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
        reciever: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let exec = Some(Avm2Function::from_method(method, scope, reciever).into());

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
        nf: NativeFunction<'gc>,
        fn_proto: Object<'gc>,
    ) -> Object<'gc> {
        FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), ScriptObjectClass::NoClass),
                exec: Some(nf.into()),
            },
        ))
        .into()
    }

    /// Construct a builtin type from a Rust constructor and prototype.
    pub fn from_builtin_constr(
        mc: MutationContext<'gc, '_>,
        constr: NativeFunction<'gc>,
        mut prototype: Object<'gc>,
        fn_proto: Object<'gc>,
    ) -> Result<Object<'gc>, Error> {
        let mut base: Object<'gc> = FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), ScriptObjectClass::NoClass),
                exec: Some(constr.into()),
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
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        self.0
            .read()
            .base
            .get_property_local(reciever, name, avm, context)
    }

    fn set_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let rv = self
            .0
            .write(context.gc_context)
            .base
            .set_property_local(reciever, name, value, avm, context)?;

        rv.resolve(avm, context)?;

        Ok(())
    }

    fn init_property_local(
        self,
        reciever: Object<'gc>,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let rv = self
            .0
            .write(context.gc_context)
            .base
            .init_property_local(reciever, name, value, avm, context)?;

        rv.resolve(avm, context)?;

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

    fn get_trait(self, name: &QName) -> Result<Vec<AbcTrait>, Error> {
        self.0.read().base.get_trait(name)
    }

    fn get_provided_trait(
        &self,
        name: &QName,
        known_traits: &mut Vec<AbcTrait>,
    ) -> Result<(), Error> {
        self.0.read().base.get_provided_trait(name, known_traits)
    }

    fn get_scope(self) -> Option<GcCell<'gc, Scope<'gc>>> {
        self.0.read().base.get_scope()
    }

    fn get_abc(self) -> Option<Rc<AbcFile>> {
        self.0.read().base.get_abc()
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
        self.0.read().exec.clone()
    }

    fn call(
        self,
        reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        if let Some(exec) = &self.0.read().exec {
            exec.exec(reciever, arguments, avm, context, base_proto)?
                .resolve(avm, context)
        } else {
            Err("Not a callable function!".into())
        }
    }

    fn construct(
        &self,
        _avm: &mut Avm2<'gc>,
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
        _avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        class: Avm2ClassEntry,
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
            let name = QName::from_abc_multiname(&class.abc(), class.instance().name.clone())?;
            Ok(format!("[class {}]", name.local_name()).into())
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
