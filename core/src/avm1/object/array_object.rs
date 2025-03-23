use crate::avm1::{Activation, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::ecma_conversions::f64_to_wrapping_i32;
use crate::string::AvmString;
use gc_arena::Collect;
use ruffle_macros::istr;
use std::fmt;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ArrayObject<'gc>(pub(crate) ScriptObject<'gc>);

impl fmt::Debug for ArrayObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ArrayObject")
            .field("ptr", &self.0.as_ptr())
            .finish()
    }
}

impl<'gc> ArrayObject<'gc> {
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
        self.0
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr()
    }

    fn set_local(
        &self,
        name: AvmString<'gc>,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<(), Error<'gc>> {
        if name == istr!("length") {
            let new_length = value.coerce_to_i32(activation)?;
            self.set_length(activation, new_length)?;
        } else if let Some(index) = Self::parse_index(name) {
            let length = self.length(activation)?;
            if index >= length {
                self.set_length(activation, index.wrapping_add(1))?;
            }
        }

        self.0.set_local(name, value, activation, this)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(
            crate::avm1::ArrayBuilder::new_with_proto(activation.strings(), this)
                .with([])
                .into(),
        )
    }

    fn as_array_object(&self) -> Option<ArrayObject<'gc>> {
        Some(*self)
    }

    fn set_length(
        &self,
        activation: &mut Activation<'_, 'gc>,
        new_length: i32,
    ) -> Result<(), Error<'gc>> {
        let old_length = self.0.get_data(istr!("length"), activation);
        if let Value::Number(old_length) = old_length {
            for i in new_length.max(0)..f64_to_wrapping_i32(old_length) {
                self.delete_element(activation, i);
            }
        }
        self.0.set_length(activation, new_length)
    }

    fn set_element(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: i32,
        value: Value<'gc>,
    ) -> Result<(), Error<'gc>> {
        let length = self.length(activation)?;
        if index >= length {
            self.set_length(activation, index.wrapping_add(1))?;
        }
        self.0.set_element(activation, index, value)
    }
}
