//! Function object impl

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::function::BoundMethod;
use crate::avm2::method::{Method, NativeMethod};
use crate::avm2::object::script_object::{ScriptObject, ScriptObjectData};
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::scope::ScopeChain;
use crate::avm2::value::Value;
use crate::avm2::Error;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::{
    lock::{Lock, RefLock},
    Collect, Gc, GcCell, GcWeak, Mutation,
};
use std::cell::Ref;

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

    let mc = activation.gc();

    let dummy = Gc::new(
        mc,
        NativeMethod {
            method: |_, _, _| Ok(Value::Undefined),
            name: "<Empty Function>",
            signature: vec![],
            resolved_signature: GcCell::new(mc, None),
            return_type: None,
            is_variadic: true,
        },
    );

    Ok(FunctionObject(Gc::new(
        mc,
        FunctionObjectData {
            base,
            exec: RefLock::new(BoundMethod::from_method(
                Method::Native(dummy),
                activation.create_scopechain(),
                None,
                None,
                None,
            )),
            prototype: Lock::new(None),
        },
    ))
    .into())
}

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
            .field(
                "name",
                &self.0.exec.try_borrow().map(|e| e.debug_full_name()),
            )
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FunctionObjectData<'gc> {
    /// Base script object
    base: ScriptObjectData<'gc>,

    /// Executable code
    exec: RefLock<BoundMethod<'gc>>,

    /// Attached prototype (note: not the same thing as base object's proto)
    prototype: Lock<Option<Object<'gc>>>,
}

const _: () = assert!(std::mem::offset_of!(FunctionObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<FunctionObjectData>() == std::mem::align_of::<ScriptObjectData>());

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
        let this = Self::from_method(activation, method, scope, None, None, None);
        let es3_proto = activation
            .avm2()
            .classes()
            .object
            .construct(activation, &[])?;

        this.set_prototype(Some(es3_proto), activation.gc());

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

        FunctionObject(Gc::new(
            activation.context.gc_context,
            FunctionObjectData {
                base: ScriptObjectData::new(fn_class),
                exec: RefLock::new(exec),
                prototype: Lock::new(None),
            },
        ))
    }

    pub fn prototype(&self) -> Option<Object<'gc>> {
        self.0.prototype.get()
    }

    pub fn set_prototype(&self, proto: Option<Object<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), FunctionObjectData, prototype).set(proto);
    }

    pub fn num_parameters(&self) -> usize {
        self.0.exec.borrow().num_parameters()
    }
}

impl<'gc> TObject<'gc> for FunctionObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
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
        Some(self.0.exec.borrow())
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
        let exec = self.0.exec.borrow().clone();

        exec.exec(receiver, arguments, activation, self.into())
    }

    fn construct(
        self,
        activation: &mut Activation<'_, 'gc>,
        arguments: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error<'gc>> {
        let object_class = activation.avm2().classes().object;

        let prototype = if let Some(proto) = self.prototype() {
            proto
        } else {
            let proto = object_class.prototype();
            self.set_prototype(Some(proto), activation.gc());
            proto
        };

        let instance = ScriptObject::custom_object(
            activation.context.gc_context,
            object_class.inner_class_definition(),
            Some(prototype),
            object_class.instance_vtable(),
        );

        self.call(instance.into(), arguments, activation)?;

        Ok(instance)
    }
}
