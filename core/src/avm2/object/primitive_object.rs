//! Boxed primitives

use std::cell::{Ref, RefMut};

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::script_object::{ScriptObjectClass, ScriptObjectData};
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::scope::Scope;
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::{impl_avm2_custom_object, impl_avm2_custom_object_properties};
use gc_arena::{Collect, GcCell, MutationContext};

/// A class instance allocator that allocates primitive objects.
pub fn primitive_allocator<'gc>(
    class: Object<'gc>,
    proto: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error> {
    let base = ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(class));

    Ok(PrimitiveObject(GcCell::allocate(
        activation.context.gc_context,
        PrimitiveObjectData {
            base,
            primitive: Value::Undefined,
        },
    ))
    .into())
}

/// An Object which represents a primitive value of some other kind.
#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct PrimitiveObject<'gc>(GcCell<'gc, PrimitiveObjectData<'gc>>);

#[derive(Collect, Debug, Clone)]
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
    pub fn from_primitive(
        primitive: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Object<'gc>, Error> {
        if !primitive.is_primitive() {
            return Err("Attempted to box an object as a primitive".into());
        }

        if matches!(primitive, Value::Undefined) {
            return Err("Cannot box an undefined value".into());
        } else if matches!(primitive, Value::Null) {
            return Err("Cannot box a null value".into());
        }

        let proto = match primitive {
            Value::Bool(_) => activation.avm2().prototypes().boolean,
            Value::Number(_) => activation.avm2().prototypes().number,
            Value::Unsigned(_) => activation.avm2().prototypes().uint,
            Value::Integer(_) => activation.avm2().prototypes().int,
            Value::String(_) => activation.avm2().prototypes().string,
            _ => unreachable!(),
        };
        let class = match primitive {
            Value::Bool(_) => activation.avm2().classes().boolean,
            Value::Number(_) => activation.avm2().classes().number,
            Value::Unsigned(_) => activation.avm2().classes().uint,
            Value::Integer(_) => activation.avm2().classes().int,
            Value::String(_) => activation.avm2().classes().string,
            _ => unreachable!(),
        };

        let base = ScriptObjectData::base_new(Some(proto), ScriptObjectClass::ClassInstance(class));
        let mut this: Object<'gc> = PrimitiveObject(GcCell::allocate(
            activation.context.gc_context,
            PrimitiveObjectData { base, primitive },
        ))
        .into();
        this.install_instance_traits(activation, class)?;

        class.call_native_init(Some(this), &[], activation, Some(class))?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for PrimitiveObject<'gc> {
    impl_avm2_custom_object!(base);
    impl_avm2_custom_object_properties!(base);

    fn to_string(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().primitive.clone())
    }

    fn to_locale_string(&self, mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        match self.0.read().primitive.clone() {
            val @ Value::Integer(_) | val @ Value::Unsigned(_) => Ok(val),
            _ => {
                let class_name = self
                    .as_class()
                    .map(|c| c.read().name().local_name())
                    .unwrap_or_else(|| "Object".into());

                Ok(AvmString::new(mc, format!("[object {}]", class_name)).into())
            }
        }
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error> {
        Ok(self.0.read().primitive.clone())
    }

    fn derive(&self, activation: &mut Activation<'_, 'gc, '_>) -> Result<Object<'gc>, Error> {
        let this: Object<'gc> = Object::PrimitiveObject(*self);
        let base = ScriptObjectData::base_new(Some(this), ScriptObjectClass::NoClass);

        Ok(PrimitiveObject(GcCell::allocate(
            activation.context.gc_context,
            PrimitiveObjectData {
                base,
                primitive: Value::Undefined,
            },
        ))
        .into())
    }

    fn as_primitive_mut(&self, mc: MutationContext<'gc, '_>) -> Option<RefMut<Value<'gc>>> {
        Some(RefMut::map(self.0.write(mc), |pod| &mut pod.primitive))
    }

    fn as_primitive(&self) -> Option<Ref<Value<'gc>>> {
        Some(Ref::map(self.0.read(), |pod| &pod.primitive))
    }
}
