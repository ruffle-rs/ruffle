use crate::avm2::error::make_error_2007;
use crate::avm2::object::PrimitiveObject;
use crate::avm2::Object;
use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

/// Extensions over parameters that are passed into AS-defined, Rust-implemented methods.
///
/// It is expected that the AS signature is correct and you only operate on values defined from it.
/// These values will be `expect()`ed to exist, and any method here will panic if they're missing.  
pub trait ParametersExt<'gc> {
    /// Gets the value at the given index.
    fn get_value(&self, index: usize) -> Value<'gc>;

    /// Gets the value at the given index and coerces it to an Object.
    ///
    /// If the value is null or is undefined, a TypeError 2007 is raised.
    fn get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<Object<'gc>, Error<'gc>>;

    /// Tries to get the value at the given index and coerce it to an Object.
    ///
    /// If the value is null or is undefined, None is returned.
    fn try_get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Option<Object<'gc>>;

    /// Gets the value at the given index and coerces it to an f64.
    ///
    /// If the value is null or is undefined, 0.0 is returned.
    /// If the object cannot be coerced to an f64, a TypeError 1050 is raised.
    fn get_f64(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<f64, Error<'gc>>;

    /// Gets the value at the given index and coerces it to a u32.
    ///
    /// If the value is null or is undefined, 0 is returned.
    /// If the object cannot be coerced to a u32, a TypeError 1050 is raised.
    fn get_u32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<u32, Error<'gc>>;

    /// Gets the value at the given index and coerces it to a i32.
    ///
    /// If the value is null or is undefined, 0 is returned.
    /// If the object cannot be coerced to an i32, a TypeError 1050 is raised.
    fn get_i32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<i32, Error<'gc>>;

    /// Gets the value at the given index and coerces it to a bool.
    ///
    /// If the value is null or is undefined, false is returned.
    fn get_bool(&self, index: usize) -> bool;

    /// Gets the value at the given index and coerces it to an AvmString.
    ///
    /// If the value is undefined, "undefined" is returned.
    /// If the value is null, "null" is returned.
    /// If the object cannot be coerced to a string, a TypeError 1050 is raised.
    fn get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<AvmString<'gc>, Error<'gc>>;
    fn get_string_non_null(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<AvmString<'gc>, Error<'gc>>;

    /// Gets the value at the given index and coerces it to an AvmString.
    ///
    /// If the value is null or is undefined, Ok(None) is returned.
    /// If the object cannot be coerced to a string, a TypeError 1050 is raised.
    fn try_get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<Option<AvmString<'gc>>, Error<'gc>>;
}

impl<'gc> ParametersExt<'gc> for &[Value<'gc>] {
    #[inline]
    fn get_value(&self, index: usize) -> Value<'gc> {
        self[index]
    }

    fn get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<Object<'gc>, Error<'gc>> {
        match self[index] {
            Value::Null | Value::Undefined => Err(make_error_2007(activation, name)),
            Value::Object(o) => Ok(o),
            primitive => Ok(PrimitiveObject::from_primitive(primitive, activation)
                .expect("Primitive object is infallible at this point")),
        }
    }

    fn try_get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Option<Object<'gc>> {
        match self[index] {
            Value::Null | Value::Undefined => None,
            Value::Object(o) => Some(o),
            primitive => Some(
                PrimitiveObject::from_primitive(primitive, activation)
                    .expect("Primitive object is infallible at this point"),
            ),
        }
    }

    fn get_f64(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<f64, Error<'gc>> {
        self[index].coerce_to_number(activation)
    }

    fn get_u32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<u32, Error<'gc>> {
        self[index].coerce_to_u32(activation)
    }

    fn get_i32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<i32, Error<'gc>> {
        self[index].coerce_to_i32(activation)
    }

    fn get_bool(&self, index: usize) -> bool {
        self[index].coerce_to_boolean()
    }

    fn get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        self[index].coerce_to_string(activation)
    }

    fn get_string_non_null(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        match self[index] {
            Value::Null | Value::Undefined => Err(make_error_2007(activation, name)),
            other => other.coerce_to_string(activation),
        }
    }

    fn try_get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<Option<AvmString<'gc>>, Error<'gc>> {
        match self[index] {
            Value::Null | Value::Undefined => Ok(None),
            other => Ok(Some(other.coerce_to_string(activation)?)),
        }
    }
}
