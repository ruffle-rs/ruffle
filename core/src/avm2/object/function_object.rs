//! Function object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::function::Executable;
use crate::avm2::method::{Method, NativeMethod};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
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
    /// Construct a function from an ABC method and the current closure scope.
    ///
    /// The given `reciever`, if supplied, will override any user-specified
    /// `this` parameter.
    pub fn from_method(
        mc: MutationContext<'gc, '_>,
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
        receiver: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let exec = Some(Executable::from_method(method, scope, receiver, mc));

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
        nf: NativeMethod,
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
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
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

    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.0.read().exec
    }

    fn call(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        if let Some(exec) = &self.0.read().exec {
            exec.exec(receiver, arguments, activation, base_proto, self.into())
        } else {
            Err("Not a callable function!".into())
        }
    }

    fn construct(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::FunctionObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(FunctionObject(GcCell::allocate(
            activation.context.gc_context,
            FunctionObjectData { base, exec: None },
        ))
        .into())
    }

    fn derive(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        class: GcCell<'gc, Class<'gc>>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::FunctionObject(*self);
        let base = ScriptObjectData::base_new(
            Some(this),
            ScriptObjectClass::InstancePrototype(class, scope),
        );

        Ok(FunctionObject(GcCell::allocate(
            activation.context.gc_context,
            FunctionObjectData { base, exec: None },
        ))
        .into())
    }
}
