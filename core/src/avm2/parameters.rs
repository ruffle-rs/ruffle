use crate::avm2::error::type_error;
use crate::avm2::object::PrimitiveObject;
use crate::avm2::Object;
use crate::avm2::{Activation, Error, Value};
use crate::string::AvmString;

pub trait ParametersExt<'gc> {
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
    fn get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
        name: &'static str,
    ) -> Result<Object<'gc>, Error<'gc>> {
        match self
            .get(index)
            .expect("Value must be Some() from AS definition")
        {
            Value::Null | Value::Undefined => Err(null_parameter_error(activation, name)),
            Value::Object(o) => Ok(*o),
            primitive => Ok(PrimitiveObject::from_primitive(*primitive, activation)
                .expect("Primitive object is infallible at this point")),
        }
    }

    fn try_get_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Option<Object<'gc>> {
        match self
            .get(index)
            .expect("Value must be Some() from AS definition")
        {
            Value::Null | Value::Undefined => None,
            Value::Object(o) => Some(*o),
            primitive => Some(
                PrimitiveObject::from_primitive(*primitive, activation)
                    .expect("Primitive object is infallible at this point"),
            ),
        }
    }

    fn get_f64(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<f64, Error<'gc>> {
        self.get(index)
            .expect("Value must be Some() from AS definition")
            .coerce_to_number(activation)
    }

    fn get_u32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<u32, Error<'gc>> {
        self.get(index)
            .expect("Value must be Some() from AS definition")
            .coerce_to_u32(activation)
    }

    fn get_i32(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<i32, Error<'gc>> {
        self.get(index)
            .expect("Value must be Some() from AS definition")
            .coerce_to_i32(activation)
    }

    fn get_bool(&self, index: usize) -> bool {
        self.get(index)
            .expect("Value must be Some() from AS definition")
            .coerce_to_boolean()
    }

    fn get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        self.get(index)
            .expect("Value must be Some() from AS definition")
            .coerce_to_string(activation)
    }

    fn try_get_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
        index: usize,
    ) -> Result<Option<AvmString<'gc>>, Error<'gc>> {
        match self
            .get(index)
            .expect("Value must be Some() from AS definition")
        {
            Value::Null | Value::Undefined => Ok(None),
            other => Ok(Some(other.coerce_to_string(activation)?)),
        }
    }
}

fn null_parameter_error<'gc>(activation: &mut Activation<'_, 'gc>, name: &str) -> Error<'gc> {
    let error = type_error(
        activation,
        &format!("Parameter {name} must be non-null."),
        2007,
    );
    match error {
        Err(e) => e,
        Ok(e) => Error::AvmError(e),
    }
}
