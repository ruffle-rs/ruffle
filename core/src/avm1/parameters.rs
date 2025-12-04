use crate::avm1::{Activation, Error, Object, Value};
use crate::string::AvmString;
use std::cmp::PartialEq;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Controls the behavior of explicit `undefined` when passed to a `try_get_*` method of [`ParametersExt`].
pub enum UndefinedAs {
    /// An explicit `undefined` will be treated the same as an implicit `undefined` - the result will be `None.`
    None,

    /// An explicit `undefined` will be treated as if the value did exist - the result will be `Some`.
    Some,
}

pub trait ParametersExt<'gc> {
    /// Gets the value at the given index.
    /// If the value does not exist, it will return Undefined.
    fn get_value(&self, index: usize) -> Value<'gc>;

    /// Gets the value at the given index, if it exists.
    fn get_optional(&self, index: usize) -> Option<Value<'gc>>;

    /// Gets the value at the given index as an Object.
    /// The value will be coerced to an Object, even if it's undefined/missing.
    fn get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<Object<'gc>, Error<'gc>> {
        self.get_value(index).coerce_to_object_or_bare(activation)
    }

    /// Tries to get the value at the given index as an Object.
    /// The value will be coerced to an Object if it exists and is coercible.
    fn try_get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<Option<Object<'gc>>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            value.coerce_to_object(activation)
        } else {
            Ok(None)
        }
    }

    /// Get the value at the given index as a String.
    /// The value will be coerced to a String, even if it's undefined/missing.
    fn get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        self.get_value(index).coerce_to_string(activation)
    }

    /// Tries to get the value at the given index as a String.
    /// The value will be coerced to a String if it exists.
    fn try_get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<AvmString<'gc>>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_string(activation)?))
        } else {
            Ok(None)
        }
    }

    /// Get the value at the given index as a bool.
    /// The value will be coerced to a bool, even if it's undefined/missing.
    fn get_bool(&self, activation: &mut Activation<'_, 'gc>, index: usize) -> bool {
        self.get_value(index).as_bool(activation.swf_version())
    }

    /// Tries to get the value at the given index as a bool.
    /// The value will be coerced to a bool if it exists.
    fn try_get_bool(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Option<bool> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return None;
            }
            Some(value.as_bool(activation.swf_version()))
        } else {
            None
        }
    }

    /// Gets the value at the given index as an u16.
    /// The value will be coerced to an u16, even if it's undefined/missing.
    #[expect(dead_code)]
    fn get_u16(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<u16, Error<'gc>> {
        self.get_value(index).coerce_to_u16(activation)
    }

    /// Gets the value at the given index as an u16.
    /// The value will be coerced to an u16 if it exists.
    #[expect(dead_code)]
    fn try_get_u16(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<u16>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_u16(activation)?))
        } else {
            Ok(None)
        }
    }

    /// Gets the value at the given index as an i16.
    /// The value will be coerced to an i16, even if it's undefined/missing.
    #[expect(dead_code)]
    fn get_i16(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<i16, Error<'gc>> {
        self.get_value(index).coerce_to_i16(activation)
    }

    /// Gets the value at the given index as an i16.
    /// The value will be coerced to an i16 if it exists.
    #[expect(dead_code)]
    fn try_get_i16(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<i16>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_i16(activation)?))
        } else {
            Ok(None)
        }
    }

    /// Gets the value at the given index as an u8.
    /// The value will be coerced to an u8, even if it's undefined/missing.
    #[expect(dead_code)]
    fn get_u8(&self, activation: &mut Activation<'_, 'gc>, index: usize) -> Result<u8, Error<'gc>> {
        self.get_value(index).coerce_to_u8(activation)
    }

    /// Gets the value at the given index as an u8.
    /// The value will be coerced to an u8 if it exists.
    fn try_get_u8(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<u8>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_u8(activation)?))
        } else {
            Ok(None)
        }
    }

    /// Gets the value at the given index as an i32.
    /// The value will be coerced to an i32, even if it's undefined/missing.
    fn get_i32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<i32, Error<'gc>> {
        self.get_value(index).coerce_to_i32(activation)
    }

    /// Gets the value at the given index as an i32.
    /// The value will be coerced to an i32 if it exists.
    fn try_get_i32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<i32>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_i32(activation)?))
        } else {
            Ok(None)
        }
    }

    /// Gets the value at the given index as an u32.
    /// The value will be coerced to an u32, even if it's undefined/missing.
    fn get_u32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<u32, Error<'gc>> {
        self.get_value(index).coerce_to_u32(activation)
    }

    /// Gets the value at the given index as an u32.
    /// The value will be coerced to an u32 if it exists.
    fn try_get_u32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<u32>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_u32(activation)?))
        } else {
            Ok(None)
        }
    }

    /// Gets the value at the given index as an f64.
    /// The value will be coerced to an f64, even if it's undefined/missing.
    fn get_f64(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<f64, Error<'gc>> {
        self.get_value(index).coerce_to_f64(activation)
    }

    /// Gets the value at the given index as an f64.
    /// The value will be coerced to an f64 if it exists.
    #[expect(dead_code)]
    fn try_get_f64(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        undefined_behaviour: UndefinedAs,
    ) -> Result<Option<f64>, Error<'gc>> {
        if let Some(value) = self.get_optional(index) {
            if undefined_behaviour == UndefinedAs::None && value == Value::Undefined {
                return Ok(None);
            }
            Ok(Some(value.coerce_to_f64(activation)?))
        } else {
            Ok(None)
        }
    }
}

impl<'gc> ParametersExt<'gc> for &[Value<'gc>] {
    #[inline]
    fn get_value(&self, index: usize) -> Value<'gc> {
        self.get(index).copied().unwrap_or(Value::Undefined)
    }

    #[inline]
    fn get_optional(&self, index: usize) -> Option<Value<'gc>> {
        self.get(index).copied()
    }
}
