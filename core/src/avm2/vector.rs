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
/// evaluated to a default value based on the type of the vector.
///
/// A vector may also be configured to have a fixed size; when this is enabled,
/// attempts to modify the length fail.
#[derive(Collect, Debug, Clone)]
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
    pub fn new(length: usize, is_fixed: bool, value_type: Object<'gc>) -> Self {
        let mut storage = Vec::new();

        storage.resize(length, None);

        VectorStorage {
            storage,
            is_fixed,
            value_type,
        }
    }

    pub fn set_is_fixed(&mut self, is_fixed: bool) {
        self.is_fixed = is_fixed;
    }

    pub fn length(&self) -> usize {
        self.storage.len()
    }

    pub fn resize(&mut self, new_length: usize) -> Result<(), Error> {
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

    /// Get the value type this vector coerces things to.
    pub fn value_type(&self) -> Object<'gc> {
        self.value_type
    }

    /// Check if a vector index is in bounds.
    pub fn is_in_range(&self, pos: usize) -> bool {
        pos < self.storage.len()
    }

    /// Retrieve a value from the vector.
    ///
    /// If the value is `None`, the type default value will be substituted.
    pub fn get(
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
    /// This function does no coercion as calling it requires mutably borrowing
    /// the vector (and thus it is unwise to reenter the AVM2 runtime to coerce
    /// things). You must use the associated `coerce` fn before storing things
    /// in the vector.
    ///
    /// This function yields an error if the position is outside the length of
    /// the vector.
    pub fn set(&mut self, pos: usize, value: Option<Value<'gc>>) -> Result<(), Error> {
        self.storage
            .get_mut(pos)
            .map(|v| *v = value)
            .ok_or_else(|| format!("RangeError: {} is outside the range of the vector", pos).into())
    }
}
