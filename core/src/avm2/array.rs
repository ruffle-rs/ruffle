//! Array support types

use crate::avm2::value::Value;
use gc_arena::Collect;
use std::iter::{ExactSizeIterator, FromIterator};
use std::ops::RangeBounds;

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

    /// Convert a set of arguments into array storage.
    pub fn from_args(values: &[Value<'gc>]) -> Self {
        let storage = values
            .iter()
            .map(|v| Some(*v))
            .collect::<Vec<Option<Value<'gc>>>>();

        Self { storage }
    }

    /// Wrap an existing storage Vec in an `ArrayStorage`.
    pub fn from_storage(storage: Vec<Option<Value<'gc>>>) -> Self {
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
            self.storage.push(*other_item)
        }
    }

    /// Push a single value onto the end of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn push(&mut self, item: Value<'gc>) {
        self.storage.push(Some(item))
    }

    /// Push an array hole onto the end of this array.
    pub fn push_hole(&mut self) {
        self.storage.push(None)
    }

    /// Pop a value from the back of the array.
    ///
    /// This method preferrentially pops non-holes from the array first. If a
    /// hole is popped, it will become `undefined`.
    pub fn pop(&mut self) -> Value<'gc> {
        let mut non_hole = None;

        for (i, item) in self.storage.iter().enumerate().rev() {
            if item.is_some() {
                non_hole = Some(i);
            }
        }

        if let Some(non_hole) = non_hole {
            self.storage.remove(non_hole).unwrap()
        } else {
            self.storage
                .pop()
                .unwrap_or(None)
                .unwrap_or(Value::Undefined)
        }
    }

    /// Shift a value from the front of the array.
    ///
    /// This method preferrentially pops non-holes from the array first. If a
    /// hole is popped, it will become `undefined`.
    pub fn shift(&mut self) -> Value<'gc> {
        if !self.storage.is_empty() {
            self.storage.remove(0).unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }
    }

    /// Unshift a single value onto the start of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn unshift(&mut self, item: Value<'gc>) {
        self.storage.insert(0, Some(item))
    }

    /// Iterate over array values.
    pub fn iter<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = Option<Value<'gc>>>
           + ExactSizeIterator<Item = Option<Value<'gc>>>
           + 'a {
        self.storage.iter().cloned()
    }

    pub fn splice<'a, R, I>(
        &'a mut self,
        range: R,
        replace_with: I,
    ) -> impl Iterator<Item = Option<Value<'gc>>> + 'a
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = Value<'gc>>,
        <I as IntoIterator>::IntoIter: 'a,
    {
        self.storage
            .splice(range, replace_with.into_iter().map(Some))
    }
}

impl<'gc, V> FromIterator<V> for ArrayStorage<'gc>
where
    V: Into<Value<'gc>>,
{
    fn from_iter<I>(values: I) -> Self
    where
        I: IntoIterator<Item = V>,
    {
        let storage = values.into_iter().map(|v| Some(v.into())).collect();

        Self { storage }
    }
}
