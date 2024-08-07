//! Boxed primitives

use core::fmt;
use std::cell::{Ref, RefMut};

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::barrier::unlock;
use gc_arena::{lock::RefLock, Collect, Gc, GcWeak, Mutation};

/// A class instance allocator that allocates primitive objects.
pub fn primitive_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(PrimitiveObject(Gc::new(
        activation.context.gc_context,
        PrimitiveObjectData {
            base,
            primitive: RefLock::new(Value::Undefined),
        },
    ))
    .into())
}

/// An Object which represents a primitive value of some other kind.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct PrimitiveObject<'gc>(pub Gc<'gc, PrimitiveObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct PrimitiveObjectWeak<'gc>(pub GcWeak<'gc, PrimitiveObjectData<'gc>>);

impl fmt::Debug for PrimitiveObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrimitiveObject")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct PrimitiveObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The primitive value this object represents.
    primitive: RefLock<Value<'gc>>,
}

const _: () = assert!(std::mem::offset_of!(PrimitiveObjectData, base) == 0);
const _: () = assert!(
    std::mem::align_of::<PrimitiveObjectData>() == std::mem::align_of::<ScriptObjectData>()
);

impl<'gc> PrimitiveObject<'gc> {
    /// Box a primitive into an object.
    ///
    /// This function will yield an error if `primitive` is `Undefined`, `Null`,
    /// or an object already.
    ///
    /// In order to prevent stack overflow, this function does *not* call the
    /// initializer of the primitive class being constructed.
    pub fn from_primitive(
        primitive: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        if !primitive.is_primitive() {
            return Err("Attempted to box an object as a primitive".into());
        }

        if matches!(primitive, Value::Undefined) {
            return Err("Cannot box an undefined value".into());
        } else if matches!(primitive, Value::Null) {
            return Err("Cannot box a null value".into());
        }

        let class = match primitive {
            Value::Bool(_) => activation.avm2().classes().boolean,
            Value::Number(_) => activation.avm2().classes().number,
            Value::Integer(_) => activation.avm2().classes().int,
            Value::String(_) => activation.avm2().classes().string,
            _ => unreachable!(),
        };

        let base = ScriptObjectData::new(class);
        let this: Object<'gc> = PrimitiveObject(Gc::new(
            activation.context.gc_context,
            PrimitiveObjectData {
                base,
                primitive: RefLock::new(primitive),
            },
        ))
        .into();
        this.install_instance_slots(activation.context.gc_context);

        //We explicitly DO NOT CALL the native initializers of primitives here.
        //If we did so, then those primitive initializers' method types would
        //trigger the construction of primitive objects... which would need to
        //be initialized, which forms a cycle.

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for PrimitiveObject<'gc> {
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
        match *self.0.primitive.borrow() {
            val @ Value::Integer(_) => Ok(val),
            _ => {
                let class_name = self.instance_class().name().local_name();

                Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!("[object {class_name}]"),
                )
                .into())
            }
        }
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(*self.0.primitive.borrow())
    }

    fn as_primitive(&self) -> Option<Ref<Value<'gc>>> {
        Some(self.0.primitive.borrow())
    }

    fn as_primitive_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<Value<'gc>>> {
        Some(unlock!(Gc::write(mc, self.0), PrimitiveObjectData, primitive).borrow_mut())
    }
}
