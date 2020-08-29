//! Array support types

use crate::avm2::value::Value;
use gc_arena::Collect;
use std::iter::ExactSizeIterator;

/// Trait which exists purely so that we can reverse the iterators that come
/// out of `ArrayStorage.iter`.
///
/// Not to be confused with the `ArrayIterator` struct in `globals::array`.
pub trait ArrayIterator: DoubleEndedIterator + ExactSizeIterator {}

impl<T> ArrayIterator for T where T: DoubleEndedIterator + ExactSizeIterator {}

/// The array storage portion of an array object.
///
/// Array values may consist of either standard `Value`s or "holes": values
/// which are not properties of the associated object and must be resolved in
/// the prototype.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ArrayStorage<'gc> {
    storage: Vec<Option<Value<'gc>>>,
}

impl<'gc> ArrayStorage<'gc> {
    /// Construct new array storage.
    ///
    /// The length parameter indicates how big the array storage should start
    /// out as. All array storage consists of holes.
    pub fn new(length: usize) -> Self {
        let mut storage = Vec::new();

        storage.resize(length, None);

        Self { storage }
    }

    pub fn from_args(values: &[Value<'gc>]) -> Self {
        let storage = values
            .iter()
            .map(|v| Some(v.clone()))
            .collect::<Vec<Option<Value<'gc>>>>();

        Self { storage }
    }

    /// Retrieve a value from array storage by index.
    ///
    /// Array holes and out of bounds values will be treated the same way, by
    /// yielding `None`.
    pub fn get(&self, item: usize) -> Option<Value<'gc>> {
        self.storage.get(item).cloned().unwrap_or(None)
    }

    /// Set an array storage slot to a particular value.
    ///
    /// If the item index extends beyond the length of the array, then the
    /// array will be extended with holes.
    pub fn set(&mut self, item: usize, value: Value<'gc>) {
        if self.storage.len() < (item + 1) {
            self.storage.resize(item + 1, None)
        }

        *self.storage.get_mut(item).unwrap() = Some(value)
    }

    /// Delete an array storage slot, leaving a hole.
    pub fn delete(&mut self, item: usize) {
        if let Some(i) = self.storage.get_mut(item) {
            *i = None;
        }
    }

    /// Get the length of the array.
    pub fn length(&self) -> usize {
        self.storage.len()
    }

    /// Set the length of the array.
    pub fn set_length(&mut self, size: usize) {
        self.storage.resize(size, None)
    }

    /// Append the contents of another array into this one.
    ///
    /// The values in the other array remain there and are merely copied into
    /// this one.
    ///
    /// Holes are copied as holes and not resolved at append time.
    pub fn append(&mut self, other_array: &Self) {
        for other_item in other_array.storage.iter() {
            self.storage.push(other_item.clone())
        }
    }

    /// Push a single value onto the end of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn push(&mut self, item: Value<'gc>) {
        self.storage.push(Some(item))
    }

    /// Iterate over array values.
    pub fn iter<'a>(&'a self) -> impl ArrayIterator<Item = Option<Value<'gc>>> + 'a {
        self.storage.iter().cloned()
    }
}
