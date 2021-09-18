//! Function object impl

use crate::avm2::activation::Activation;
use crate::avm2::function::Executable;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObject, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::{
    impl_avm2_custom_object, impl_avm2_custom_object_instance, impl_avm2_custom_object_properties,
};
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
    /// This associated constructor will also create and initialize an empty
    /// `Object` prototype for the function.
    pub fn from_function(
        activation: &mut Activation<'_, 'gc, '_>,
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
    ) -> Result<Object<'gc>, Error> {
        let mut this = Self::from_method(activation, method, scope, None);
        let es3_proto = ScriptObject::object(
            activation.context.gc_context,
            activation.avm2().prototypes().object,
        );

        this.install_slot(
            activation.context.gc_context,
            QName::new(Namespace::public(), "prototype"),
            0,
            es3_proto.into(),
            false,
        );

        Ok(this)
    }

    /// Construct a method from an ABC method and the current closure scope.
    ///
    /// The given `reciever`, if supplied, will override any user-specified
    /// `this` parameter.
    pub fn from_method(
        activation: &mut Activation<'_, 'gc, '_>,
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        receiver: Option<Object<'gc>>,
    ) -> Object<'gc> {
        let fn_proto = activation.avm2().prototypes().function;
        let exec = Some(Executable::from_method(
            method,
            scope,
            receiver,
            activation.context.gc_context,
        ));

        FunctionObject(GcCell::allocate(
            activation.context.gc_context,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), None),
                exec,
            },
        ))
        .into()
    }

    /// Construct a function from an ABC method, the current closure scope, and
    /// a function prototype.
    ///
    /// The given `reciever`, if supplied, will override any user-specified
    /// `this` parameter.
    ///
    /// This function exists primarily for early globals. Unless you are in a
    /// position where you cannot access `Function.prototype` yet, you should
    /// use `from_method` instead.
    pub fn from_method_and_proto(
        mc: MutationContext<'gc, '_>,
        method: Method<'gc>,
        scope: Option<GcCell<'gc, Scope<'gc>>>,
        fn_proto: Object<'gc>,
        receiver: Option<Object<'gc>>,
    ) -> Object<'gc> {
        FunctionObject(GcCell::allocate(
            mc,
            FunctionObjectData {
                base: ScriptObjectData::base_new(Some(fn_proto), None),
                exec: Some(Executable::from_method(method, scope, receiver, mc)),
            },
        ))
        .into()
    }
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);
    impl_avm2_custom_object_instance!(base);

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok("function Function() {}".into())
    }

    fn to_locale_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        self.to_string(mc)
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.0.read().exec.clone()
    }

    fn call(
        self,
        receiver: Option<Object<'gc>>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc, '_>,
        subclass_object: Option<Object<'gc>>,
    ) -> Result<Value<'gc>, Error> {
        if let Some(exec) = &self.0.read().exec {
            exec.exec(
                receiver,
                arguments,
                activation,
                subclass_object,
                self.into(),
            )
        } else {
            Err("Not a callable function!".into())
        }
    }

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        let class: Object<'gc> = self.into();
        let prototype = self
            .get_property(
                class,
                &QName::new(Namespace::public(), "prototype"),
                activation,
            )?
            .coerce_to_object(activation)?;

        let instance = prototype.derive(activation)?;

        self.call(Some(instance), arguments, activation, None)?;

        Ok(instance)
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::FunctionObject(*self);
        let base = ScriptObjectData::base_new(Some(this), None);

        Ok(FunctionObject(GcCell::allocate(
            activation.context.gc_context,
            FunctionObjectData { base, exec: None },
        ))
        .into())
    }
}
