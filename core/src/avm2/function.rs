//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::scope::Scope;
use crate::avm2::script_object::ScriptObjectData;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, CollectionContext, GcCell, MutationContext};
use std::fmt;
use std::rc::Rc;
use swf::avm2::types::{
    AbcFile, Class as AbcClass, Index, Instance as AbcInstance, Method as AbcMethod,
    MethodBody as AbcMethodBody,
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
}

impl<'gc> Avm2Function<'gc> {
    pub fn from_method(method: Avm2MethodEntry, scope: Option<GcCell<'gc, Scope<'gc>>>) -> Self {
        Self { method, scope }
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
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Executable::Native(nf) => nf(avm, context, reciever, arguments),
            Executable::Action(a2f) => {
                let base_proto = reciever.and_then(|o| o.proto());
                let activation = GcCell::allocate(
                    context.gc_context,
                    Activation::from_action(context, &a2f, reciever, arguments, base_proto)?,
                );

                avm.insert_stack_frame(activation);
                Ok(activation.into())
            }
        }
    }

    /// Execute a method that is the `super` of an existing method.
    ///
    /// The primary difference between `exec` and `exec_super` is that the
    /// former always resets the `base_proto` to the current `reciever` while
    /// the latter sets it to the next object up the prototype chain. The
    /// latter behavior is necessary to ensure that chains of `callsuper` and
    /// `constructsuper` operate correctly.
    pub fn exec_super(
        &self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reciever: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Executable::Native(nf) => nf(avm, context, reciever, arguments),
            Executable::Action(a2f) => {
                let base_proto = avm
                    .current_stack_frame()
                    .and_then(|sf| sf.read().base_proto())
                    .and_then(|o| o.proto());
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

    /// The class that defined this function object, if any.
    class: Option<Avm2ClassEntry>,
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
        base_class: Object<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<(Object<'gc>, Object<'gc>), Error> {
        let super_proto: Result<Object<'gc>, Error> = base_class
            .get_property(
                &QName::new(Namespace::public_namespace(), "prototype"),
                avm,
                context,
            )?
            .resolve(avm, context)?
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
        let mut class_proto = super_proto?.construct(avm, context, &[])?;

        for trait_entry in class.instance().traits.iter() {
            class_proto.install_trait(avm, context, class.abc(), trait_entry, scope, fn_proto)?;
        }

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
                base: ScriptObjectData::base_new(Some(fn_proto)),
                exec: Some(Avm2Function::from_method(initializer?, scope).into()),
                class: Some(class.clone()),
            },
        ))
        .into();

        for trait_entry in class.class().traits.iter() {
            constr.install_trait(avm, context, class.abc(), trait_entry, scope, fn_proto)?;
        }

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
        let class_constr = FunctionObject(GcCell::allocate(
            context.gc_context,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto)),
                exec: Some(Avm2Function::from_method(class_initializer?, scope).into()),
                class: Some(class.clone()),
            },
        ))
        .into();

        Ok((constr, class_constr))
    }

    /// Construct a function from an ABC method and the current closure scope.
    pub fn from_abc_method(
        mc: MutationContext<'gc, '_>,
        method: Avm2MethodEntry,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Object<'gc> {
        let exec = Some(Avm2Function::from_method(method, scope).into());

        FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto)),
                exec,
                class: None,
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
                base: ScriptObjectData::base_new(Some(fn_proto)),
                exec: Some(nf.into()),
                class: None,
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
                base: ScriptObjectData::base_new(Some(fn_proto)),
                exec: Some(constr.into()),
                class: None,
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
    fn get_property(
        self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0
            .read()
            .base
            .get_property(name, avm, context, self.into())
    }

    fn set_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .base
            .set_property(name, value, avm, context, self.into())
    }

    fn init_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .base
            .init_property(name, value, avm, context, self.into())
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

    fn has_property(self, name: &QName) -> bool {
        self.0.read().base.has_property(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
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
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Some(exec) = &self.0.read().exec {
            exec.exec(avm, context, reciever, arguments)
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
        let base = ScriptObjectData::base_new(Some(this));

        Ok(FunctionObject(GcCell::allocate(
            context.gc_context,
            FunctionObjectData {
                base,
                exec: None,
                class: None,
            },
        ))
        .into())
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
