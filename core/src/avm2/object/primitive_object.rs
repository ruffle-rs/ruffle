//! Boxed primitives

use core::fmt;
use std::cell::{Ref, RefMut};

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, GcWeakCell, Mutation};

/// A class instance allocator that allocates primitive objects.
pub fn primitive_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(PrimitiveObject(GcCell::new(
        activation.context.gc_context,
        PrimitiveObjectData {
            base,
            primitive: Value::Undefined,
        },
    ))
    .into())
}

/// An Object which represents a primitive value of some other kind.
#[derive(Collect, Clone, Copy)]
#[collect(no_drop)]
pub struct PrimitiveObject<'gc>(pub GcCell<'gc, PrimitiveObjectData<'gc>>);

#[derive(Collect, Clone, Copy, Debug)]
#[collect(no_drop)]
pub struct PrimitiveObjectWeak<'gc>(pub GcWeakCell<'gc, PrimitiveObjectData<'gc>>);

impl fmt::Debug for PrimitiveObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrimitiveObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

#[derive(Collect, Clone)]
#[collect(no_drop)]
pub struct PrimitiveObjectData<'gc> {
    /// All normal script data.
    base: ScriptObjectData<'gc>,

    /// The primitive value this object represents.
    primitive: Value<'gc>,
}

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
        let this: Object<'gc> = PrimitiveObject(GcCell::new(
            activation.context.gc_context,
            PrimitiveObjectData { base, primitive },
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
        match self.0.read().primitive {
            val @ Value::Integer(_) => Ok(val),
            _ => {
                let class_name = self
                    .instance_class()
                    .map(|c| c.name().local_name())
                    .unwrap_or_else(|| "Object".into());

                Ok(AvmString::new_utf8(
                    activation.context.gc_context,
                    format!("[object {class_name}]"),
                )
                .into())
            }
        }
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(self.0.read().primitive)
    }

    fn as_primitive_mut(&self, mc: &Mutation<'gc>) -> Option<RefMut<Value<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |pod| &mut pod.primitive))
    }

    fn as_primitive(&self) -> Option<Ref<Value<'gc>>> {
        Some(Ref::map(self.0.read(), |pod| &pod.primitive))
    }
}
