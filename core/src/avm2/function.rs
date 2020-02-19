//! AVM2 executables.

use crate::avm2::activation::Activation;
use crate::avm2::names::QName;
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
    Object<'gc>,
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
    pub fn exec(
        &self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        reciever: Object<'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Executable::Native(nf) => nf(avm, context, reciever, arguments),
            Executable::Action(a2f) => {
                let activation = GcCell::allocate(
                    context.gc_context,
                    Activation::from_action(context, &a2f, reciever, None)?,
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

    /// The ABC class (used to define static properties)
    pub abc_class: u32,

    /// The ABC instance (used to define both instance properties as well as
    /// "prototype" methods)
    pub abc_instance: u32,
}

impl Avm2ClassEntry {
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
        self.abc.instances.get(self.abc_instance as usize).unwrap()
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
    exec: Executable<'gc>,

    /// The class that defined this function object, if any.
    class: Option<Avm2ClassEntry>,
}

impl<'gc> FunctionObject<'gc> {
    /// Construct a class from an ABC class/instance pair.
    ///
    /// If the initializer method cannot be found, this function returns None.
    pub fn from_abc_class(
        mc: MutationContext<'gc, '_>,
        class: Avm2ClassEntry,
    ) -> Option<Object<'gc>> {
        let initializer_index = class.class().init_method.clone();
        let initializer = Avm2MethodEntry::from_method_index(class.abc(), initializer_index)?;

        Some(
            FunctionObject(GcCell::allocate(
                mc,
                FunctionObjectData {
                    base: ScriptObjectData::base_new(None),
                    exec: Avm2Function::from_method(initializer, None).into(),
                    class: Some(class),
                },
            ))
            .into(),
        )
    }

    /// Construct a function from an ABC method and the current closure scope.
    pub fn from_abc_method(
        mc: MutationContext<'gc, '_>,
        method: Avm2MethodEntry,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Object<'gc> {
        let exec = Avm2Function::from_method(method, scope).into();

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

    fn has_property(self, name: &QName) -> bool {
        self.0.read().base.has_property(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn call(
        self,
        reciever: Object<'gc>,
        arguments: &[Value<'gc>],
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().exec.exec(avm, context, reciever, arguments)
    }

    fn install_trait(
        &mut self,
        mc: MutationContext<'gc, '_>,
        abc: Rc<AbcFile>,
        trait_entry: &AbcTrait,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
    ) -> Result<(), Error> {
        self.0
            .write(mc)
            .base
            .install_trait(mc, abc, trait_entry, scope, fn_proto)
    }
}
