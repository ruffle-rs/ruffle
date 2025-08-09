use crate::avm2::error::make_error_2007;
use crate::avm2::Object;
use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

use ruffle_macros::istr;

/// Extensions over parameters that are passed into AS-defined, Rust-implemented methods.
///
/// It is expected that the AS signature is correct and you only operate on values defined from it.
/// These values will be `expect()`ed to exist, and any method here will panic if they're missing.
///
/// The rules for ActionScript type coercion may be surprising. Here is a table mapping
/// ParametersExt functions to the corresponding ActionScript types:
///
/// `get_value`: All parameter types work
/// `get_f64`: `Number`, `int`, or `uint` type
/// `get_i32`: `Number`, `int`, or `uint` type
/// `get_u32`: `Number`, `int`, or `uint` type
/// `get_bool`: `Boolean` type only
/// `get_string` and family: `String` type only
/// `get_object` and family: Any non-primitive type; i.e. any type *except* the following:
///   - `*` (aka "any") type
///   - `Object` type
///   - `Boolean` type
///   - `int` type
///   - `uint` type
///   - `Number` type
///   - `String` type
pub trait ParametersExt<'gc> {
    /// Gets the value at the given index.
    fn get_value(&self, index: usize) -> Value<'gc>;

    /// Gets the value at the given index as an Object. It is expected that the
    /// value is either Object or Null.
    ///
    /// If the value is null, a TypeError 2007 is raised.
    fn get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<Object<'gc>, Error<'gc>> {
        self.try_get_object(index)
            .ok_or_else(|| make_error_2007(activation, name))
    }

    /// Gets the value at the given index as an Object. It is expected that the
    /// value is either Object or Null.
    ///
    /// If the value is null, None is returned.
    fn try_get_object(&self, index: usize) -> Option<Object<'gc>> {
        match self.get_value(index) {
            Value::Null => None,
            Value::Object(o) => Some(o),
            _ => panic!("Expected Object or null as parameter"),
        }
    }

    /// Gets the value at the given index and coerces it to an f64.
    ///
    /// If the value is null or is undefined, 0.0 is returned.
    /// If the object cannot be coerced to an f64, a TypeError 1050 is raised.
    fn get_f64(&self, index: usize) -> f64 {
        self.get_value(index).as_f64()
    }

    /// Gets the uint-typed value at the given index.
    fn get_u32(&self, index: usize) -> u32 {
        self.get_value(index).as_u32()
    }

    /// Gets the int-typed value at the given index.
    fn get_i32(&self, index: usize) -> i32 {
        self.get_value(index).as_i32()
    }

    /// Gets the Boolean-typed value at the given index.
    fn get_bool(&self, index: usize) -> bool {
        match self.get_value(index) {
            Value::Bool(b) => b,
            _ => unreachable!("Expected Boolean-typed parameter"),
        }
    }

    /// Gets the String-typed value at the given index.
    ///
    /// If the value is null, None is returned.
    fn try_get_string(&self, index: usize) -> Option<AvmString<'gc>> {
        match self.get_value(index) {
            Value::Null => None,
            Value::String(s) => Some(s),
            _ => unreachable!("Expected String-typed parameter"),
        }
    }

    /// Like `try_get_string`, but returns "null" for null values instead
    /// of returning `None`.
    fn get_string(&self, activation: &mut Activation<'_, 'gc>, index: usize) -> AvmString<'gc> {
        self.try_get_string(index).unwrap_or_else(|| istr!("null"))
    }

    /// Like `try_get_string`, but throws TypeError 2007 for null values instead
    /// of returning `None`.
    fn get_string_non_null(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        self.try_get_string(index)
            .ok_or_else(|| make_error_2007(activation, name))
    }
}

impl<'gc> ParametersExt<'gc> for &[Value<'gc>] {
    #[inline]
    fn get_value(&self, index: usize) -> Value<'gc> {
        self[index]
    }
}
