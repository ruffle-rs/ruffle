//! Function object impl

use crate::avm2::activation::Activation;
use crate::avm2::function::BoundMethod;
use crate::avm2::method::{Method, NativeMethod};
use crate::avm2::object::script_object::{ScriptObject, ScriptObjectData};
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::scope::ScopeChain;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname};
use core::fmt;
use gc_arena::{Collect, Gc, GcCell, GcWeakCell, Mutation};
use std::cell::{Ref, RefMut};

/// A class instance allocator that allocates Function objects.
/// This is only used when ActionScript manually calls 'new Function()',
/// which produces a dummy object that just returns `Value::Undefined`
/// when called.
///
/// Normal `FunctionObject` creation goes through `FunctionObject::from_method`
/// or `FunctionObject::from_function`.
pub fn function_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    let dummy = Gc::new(
        activation.context.gc_context,
        NativeMethod {
            method: |_, _, _| Ok(Value::Undefined),
            name: "<Empty Function>",
            signature: vec![],
            resolved_signature: GcCell::new(activation.context.gc_context, None),
            return_type: Multiname::any(activation.context.gc_context),
            is_variadic: true,
        },
    );

    Ok(FunctionObject(GcCell::new(
        activation.context.gc_context,
        FunctionObjectData {
            base,
            exec: BoundMethod::from_method(
                Method::Native(dummy),
                activation.create_scopechain(),
                None,
                None,
            ),
            prototype: None,
        },
    ))
    .into())
}

/// An Object which can be called to execute its function code.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct FunctionObject<'gc>(pub GcCell<'gc, FunctionObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct FunctionObjectWeak<'gc>(pub GcWeakCell<'gc, FunctionObjectData<'gc>>);

impl fmt::Debug for FunctionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionObject")
            .field("ptr", &self.0.as_ptr())
            .field("name", &self.0.try_read().map(|r| r.exec.debug_full_name()))
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct FunctionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Executable code
    exec: BoundMethod<'gc>,

    /// Attached prototype (note: not the same thing as base object's proto)
    prototype: Option<Object<'gc>>,
}

impl<'gc> FunctionObject<'gc> {
    /// Construct a function from an ABC method and the current closure scope.
    ///
    /// This associated constructor will also create and initialize an empty
    /// `Object` prototype for the function.
    pub fn from_function(
        activation: &mut Activation<'_, 'gc>,
        method: Method<'gc>,
        scope: ScopeChain<'gc>,
    ) -> Result<FunctionObject<'gc>, Error<'gc>> {
        let this = Self::from_method(activation, method, scope, None, None);
        let es3_proto = ScriptObject::custom_object(
            activation.context.gc_context,
            // TODO: is this really a class-less object?
            // (also: how much of "ES3 class-less object" is even true?)
            None,
            Some(activation.avm2().classes().object.prototype()),
        );

        this.0.write(activation.context.gc_context).prototype = Some(es3_proto);

        Ok(this)
    }

    /// Construct a method from an ABC method and the current closure scope.
    ///
    /// The given `receiver`, if supplied, will override any user-specified
    /// `this` parameter.
    pub fn from_method(
        activation: &mut Activation<'_, 'gc>,
        method: Method<'gc>,
        scope: ScopeChain<'gc>,
        receiver: Option<Object<'gc>>,
        subclass_object: Option<ClassObject<'gc>>,
    ) -> FunctionObject<'gc> {
        let fn_class = activation.avm2().classes().function;
        let exec = BoundMethod::from_method(method, scope, receiver, subclass_object);

        FunctionObject(GcCell::new(
            activation.context.gc_context,
            FunctionObjectData {
                base: ScriptObjectData::new(fn_class),
                exec,
                prototype: None,
            },
        ))
    }

    pub fn prototype(&self) -> Option<Object<'gc>> {
        self.0.read().prototype
    }

    pub fn set_prototype(&self, proto: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        self.0.write(mc).prototype = proto;
    }

    pub fn num_parameters(&self) -> usize {
        self.0.read().exec.num_parameters()
    }
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: &Mutation<'gc>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn to_locale_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        self.to_string(activation)
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }

    fn as_executable(&self) -> Option<Ref<BoundMethod<'gc>>> {
        Some(Ref::map(self.0.read(), |r| &r.exec))
    }

    fn as_function_object(&self) -> Option<FunctionObject<'gc>> {
        Some(*self)
    }

    fn call(
        self,
        receiver: Value<'gc>,
        arguments: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // NOTE: Cloning an executable does not allocate new memory
        let exec = self.0.read().exec.clone();

        exec.exec(receiver, arguments, activation, self.into())
    }

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error<'gc>> {
        let prototype = if let Some(proto) = self.prototype() {
            proto
        } else {
            let proto = activation.avm2().classes().object.prototype();
            self.set_prototype(Some(proto), activation.gc());
            proto
        };

        let instance =
            ScriptObject::custom_object(activation.context.gc_context, None, Some(prototype));

        self.call(instance.into(), arguments, activation)?;

        Ok(instance)
    }
}
