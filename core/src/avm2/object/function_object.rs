//! Function object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::function::{BoundMethod, FunctionArgs};
use crate::avm2::method::Method;
use crate::avm2::object::script_object::{ScriptObject, ScriptObjectData};
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::scope::ScopeChain;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{lock::Lock, Collect, Gc, GcWeak, Mutation};

/// An Object which can be called to execute its function code.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct FunctionObject<'gc>(pub Gc<'gc, FunctionObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct FunctionObjectWeak<'gc>(pub GcWeak<'gc, FunctionObjectData<'gc>>);

impl fmt::Debug for FunctionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .field("name", &self.0.exec.debug_full_name())
            .finish()
    }
}

#[derive(Collect, Clone, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FunctionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Executable code
    exec: BoundMethod<'gc>,

    /// Attached prototype (note: not the same thing as base object's proto)
    prototype: Lock<Option<Object<'gc>>>,
}

impl<'gc> FunctionObject<'gc> {
    /// Construct a function from an ABC method and the current closure scope.
    ///
    /// This associated constructor will also create and initialize an empty
    /// `Object` prototype for the function. The given `receiver`, if supplied,
    /// will override any user-specified `this` parameter.
    ///
    /// It is the caller's responsibility to ensure that the `receiver` passed
    /// to this method is not Value::Null or Value::Undefined.
    pub fn from_method(
        activation: &mut Activation<'_, 'gc>,
        method: Method<'gc>,
        scope: ScopeChain<'gc>,
        receiver: Option<Value<'gc>>,
        bound_superclass_object: Option<ClassObject<'gc>>,
        bound_class: Option<Class<'gc>>,
    ) -> FunctionObject<'gc> {
        let fn_class = activation.avm2().classes().function;
        let exec = BoundMethod::from_method(
            method,
            scope,
            receiver,
            bound_superclass_object,
            bound_class,
        );

        let es3_proto = ScriptObject::new_object(activation);

        FunctionObject(Gc::new(
            activation.gc(),
            FunctionObjectData {
                base: ScriptObjectData::new(fn_class),
                exec,
                prototype: Lock::new(Some(es3_proto)),
            },
        ))
    }

    pub fn call(
        self,
        activation: &mut Activation<'_, 'gc>,
        receiver: Value<'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let exec = &self.0.exec;

        exec.exec(
            receiver,
            FunctionArgs::AsArgSlice { arguments },
            activation,
            self.into(),
        )
    }

    pub fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: FunctionArgs<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        let arguments = &arguments.to_slice();

        let object_class = activation.avm2().classes().object;

        let prototype = if let Some(proto) = self.prototype() {
            proto
        } else {
            let proto = object_class.prototype();
            self.set_prototype(Some(proto), activation.gc());
            proto
        };

        let instance = ScriptObject::custom_object(
            activation.gc(),
            object_class.inner_class_definition(),
            Some(prototype),
            object_class.instance_vtable(),
        );

        self.call(activation, instance.into(), arguments)?;

        Ok(instance)
    }

    pub fn prototype(&self) -> Option<Object<'gc>> {
        self.0.prototype.get()
    }

    pub fn set_prototype(&self, proto: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), FunctionObjectData, prototype).set(proto);
    }

    pub fn executable(&self) -> &BoundMethod<'gc> {
        &self.0.exec
    }
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn to_string(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let method = self.0.exec.as_method();
        let method_index = method.abc_method_index();

        AvmString::new_utf8(mc, format!("[object Function-{method_index}]"))
    }

    fn as_function_object(&self) -> Option<FunctionObject<'gc>> {
        Some(*self)
    }
}
