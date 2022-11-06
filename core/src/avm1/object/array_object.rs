use crate::avm1::property::Attribute;
use crate::avm1::{Activation, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::AvmString;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ArrayObject<'gc>(GcCell<'gc, ScriptObject<'gc>>);

impl fmt::Debug for ArrayObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ArrayObject").finish()
    }
}

impl<'gc> ArrayObject<'gc> {
    pub fn empty(activation: &Activation<'_, 'gc, '_>) -> Self {
        Self::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            [],
        )
    }

    pub fn empty_with_proto(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Self {
        Self::new_internal(gc_context, proto, [])
    }

    pub fn new(
        gc_context: MutationContext<'gc, '_>,
        array_proto: Object<'gc>,
        elements: impl IntoIterator<Item = Value<'gc>>,
    ) -> Self {
        Self::new_internal(gc_context, array_proto, elements)
    }

    fn new_internal(
        gc_context: MutationContext<'gc, '_>,
        proto: Object<'gc>,
        elements: impl IntoIterator<Item = Value<'gc>>,
    ) -> Self {
        let base = ScriptObject::new(gc_context, Some(proto));
        let mut length: i32 = 0;
        for value in elements.into_iter() {
            let length_str = AvmString::new_utf8(gc_context, length.to_string());
            base.define_value(gc_context, length_str, value, Attribute::empty());
            length += 1;
        }
        base.define_value(
            gc_context,
            "length",
            length.into(),
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );
        Self(GcCell::allocate(gc_context, base))
    }

    fn parse_index(name: AvmString<'gc>) -> Option<i32> {
        let name = name.trim_start_matches(|c| match u8::try_from(c) {
            Ok(c) => c.is_ascii_whitespace(),
            Err(_) => false,
        });

        name.parse::<std::num::Wrapping<i32>>().ok().map(|i| i.0)
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    fn raw_script_object(&self) -> ScriptObject<'gc> {
        *self.0.read()
    }

    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        if &name == b"length" {
            let new_length = value.coerce_to_i32(activation)?;
            self.set_length(activation, new_length)?;
        } else if let Some(index) = Self::parse_index(name) {
            let length = self.length(activation)?;
            if index >= length {
                self.set_length(activation, index.wrapping_add(1))?;
            }
        }

        self.0.read().set_local(name, value, activation, this)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(Self::empty_with_proto(activation.context.gc_context, this).into())
    }

    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        Some(*self)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.read().as_ptr() as *const ObjectPtr
    }

    fn set_length(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        new_length: i32,
    ) -> Result<(), Error<'gc>> {
        if let Value::Number(old_length) = self.0.read().get_data("length".into(), activation) {
            for i in new_length.max(0)..f64_to_wrapping_i32(old_length) {
                self.delete_element(activation, i);
            }
        }
        self.0.read().set_length(activation, new_length)
    }

    fn set_element(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        let length = self.length(activation)?;
        if index >= length {
            self.set_length(activation, index.wrapping_add(1))?;
        }
        self.0.read().set_element(activation, index, value)
    }
}
