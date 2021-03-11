//! Storage for AS3 Vectors

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::Collect;

/// The vector storage portion of a vector object.
///
/// Vector values are restricted to a single type, decided at the time of the
/// construction of the vector's storage. The type is determined by the type
/// argument associated with the class of the vector. Vector holes are
/// evaluated to a default value based on the
///
/// A vector may also be configured to have a fixed size; when this is enabled,
/// attempts to modify the length fail.
#[derive(Collect)]
#[collect(no_drop)]
pub struct VectorStorage<'gc> {
    /// The storage for vector values.
    ///
    /// While this is structured identically to `ArrayStorage`, the role of
    /// `None` values is significantly different. Instead of being treated as a
    /// hole to be resolved by reference to the prototype chain, vector holes
    /// are treated as default values. The actual default changes based on the
    /// type contained in the array.
    storage: Vec<Option<Value<'gc>>>,

    /// Whether or not the array length is fixed.
    is_fixed: bool,

    /// The allowed type of the contents of the vector, in the form of a class
    /// object.
    ///
    /// Vector typing is enforced by one of two ways: either by generating
    /// exceptions on values that are not of the given type, or by coercing
    /// incorrectly typed values to the given type if possible. Values that do
    /// not coerce would then be treated as vector holes and retrieved as a
    /// default value.
    value_type: Object<'gc>,
}

impl<'gc> VectorStorage<'gc> {
    fn new(length: usize, is_fixed: bool, value_type: Object<'gc>) -> Self {
        let mut storage = Vec::new();

        storage.resize(length, None);

        VectorStorage {
            storage,
            is_fixed,
            value_type,
        }
    }

    fn resize(&mut self, new_length: usize) -> Result<(), Error> {
        if self.is_fixed {
            return Err("RangeError: Vector is fixed".into());
        }

        self.storage.resize(new_length, None);

        Ok(())
    }

    /// Get the default value for this vector.
    fn default(&self, activation: &mut Activation<'_, 'gc, '_>) -> Value<'gc> {
        if Object::ptr_eq(self.value_type, activation.avm2().classes().int) {
            Value::Integer(0)
        } else if Object::ptr_eq(self.value_type, activation.avm2().classes().uint) {
            Value::Unsigned(0)
        } else if Object::ptr_eq(self.value_type, activation.avm2().classes().number) {
            Value::Number(0.0)
        } else {
            Value::Null
        }
    }

    /// Retrieve a value from the vector.
    ///
    /// If the value is `None`, the type default value will be substituted.
    fn get(
        &self,
        pos: usize,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        self.storage
            .get(pos)
            .cloned()
            .map(|v| v.unwrap_or_else(|| self.default(activation)))
            .ok_or_else(|| format!("RangeError: {} is outside the range of the vector", pos).into())
    }

    /// Store a value into the vector.
    ///
    /// If the value is not of the vector's type, then the value will be
    /// coerced to fit as per `coerce`. This function yields an error if:
    ///
    ///  * The coercion fails
    ///  * The vector is of a non-coercible type, and the value is not an
    ///    instance or subclass instance of the vector's type
    ///  * The position is outside the length of the vector
    fn set(
        &mut self,
        pos: usize,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        let coerced_value = match value.coerce_to_type(activation, self.value_type)? {
            Value::Undefined => None,
            Value::Null => None,
            v => Some(v),
        };

        self.storage
            .get_mut(pos)
            .map(|v| *v = coerced_value)
            .ok_or_else(|| format!("RangeError: {} is outside the range of the vector", pos).into())
    }
}
