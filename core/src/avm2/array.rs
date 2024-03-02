//! Array support types

use crate::avm2::value::Value;
use gc_arena::Collect;
use std::{cmp::max, ops::RangeBounds};
use std::hash::Hash;
use std::iter::Cloned;
use std::ops::Range;
use crate::avm2::dynamic_map::DynamicMap;

/// The array storage portion of an array object.
///
/// Array values may consist of either standard `Value`s or "holes": values
/// which are not properties of the associated object and must be resolved in
/// the prototype.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ArrayStorage<'gc> {
    storage_type: ArrayStorageType<'gc>,
}

struct ArrayStorageIterator<'a, 'gc> {
    storage: &'a ArrayStorage<'gc>,
    index: usize,
    index_back: usize,
}

impl<'a, 'gc> Iterator for ArrayStorageIterator<'a, 'gc> {
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.storage.storage_type {
            ArrayStorageType::Dense(storage) => {
                if self.index < storage.len() {
                    let value = storage[self.index].clone();
                    self.index += 1;
                    Some(value)
                } else {
                    None
                }
            }
            ArrayStorageType::Sparse(storage, length) => {
                if self.index < *length {
                    let next = storage.next(self.index).unwrap();
                    self.index = next;
                    let value = storage.value_at(next).cloned();
                    Some(value)
                } else {
                    None
                }
            }
        }
    }

    /*fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.storage.length() - self.index;
        (remaining, Some(remaining))
    }*/
}

impl<'a, 'gc> DoubleEndedIterator for ArrayStorageIterator<'a, 'gc> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &self.storage.storage_type {
            ArrayStorageType::Dense(storage) => {
                if self.index_back > 0 {
                    self.index_back -= 1;
                    let value = storage[self.index_back].clone();
                    Some(value)
                } else {
                    None
                }
            }
            ArrayStorageType::Sparse(storage, length) => {
                if self.index_back > 0 {
                    let next = storage.next_back(self.index_back).unwrap();
                    self.index_back = next;
                    let value = storage.value_at(next).cloned();
                    Some(value)
                } else {
                    None
                }
            }
        }
    }
}

impl<'a, 'gc> ExactSizeIterator for ArrayStorageIterator<'a, 'gc> {}

//doubleended exact size iterator
impl<'a, 'gc> DoubleEndedExactSizeIterator for ArrayStorageIterator<'a, 'gc> {}

#[derive(Clone, Collect)]
#[collect(no_drop)]
enum ArrayStorageType<'gc>  {
    Dense(Vec<Option<Value<'gc>>>),
    Sparse(DynamicMap<usize, Value<'gc>>, usize)
}

impl<'gc> ArrayStorage<'gc> {
    /// Construct new array storage.
    ///
    /// The length parameter indicates how big the array storage should start
    /// out as. All array storage consists of holes.
    pub fn new(length: usize) -> Self {
        let storage = Vec::with_capacity(length);
        let storage_type = ArrayStorageType::Dense(storage);
        Self { storage_type }
    }

    /// Convert a set of arguments into array storage.
    pub fn from_args(values: &[Value<'gc>]) -> Self {
        let storage = values
            .iter()
            .map(|v| Some(*v))
            .collect::<Vec<Option<Value<'gc>>>>();

        let storage_type = ArrayStorageType::Dense(storage);
        Self { storage_type }
    }

    /// Wrap an existing storage Vec in an `ArrayStorage`.
    pub fn from_storage(storage: Vec<Option<Value<'gc>>>) -> Self {
        let storage_type = ArrayStorageType::Dense(storage);
        Self { storage_type }
    }

    /// Retrieve a value from array storage by index.
    ///
    /// Array holes and out of bounds values will be treated the same way, by
    /// yielding `None`.
    pub fn get(&self, item: usize) -> Option<Value<'gc>> {
        match &self.storage_type {
            ArrayStorageType::Dense(storage) => {
                return storage.get(item).cloned().unwrap_or(None);
            }
            ArrayStorageType::Sparse(storage, ..) => {
                storage.value_at(item).cloned()
            }
        }
        //self.storage.get(item).cloned().unwrap_or(None)
    }

    /// Set an array storage slot to a particular value.
    ///
    /// If the item index extends beyond the length of the array, then the
    /// array will be extended with holes.
    pub fn set(&mut self, item: usize, value: Value<'gc>) {

        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                if storage.len() < (item + 1) {
                    //convert self to sparse
                    let mut new_storage = DynamicMap::new();
                    for (i, v) in storage.iter().enumerate() {
                        if let Some(v) = v {
                            new_storage.insert(i, *v);
                        }
                    }
                    new_storage.insert(item, value);
                    self.storage_type = ArrayStorageType::Sparse(new_storage, item + 1);
                    return;
                }
                *storage.get_mut(item).unwrap() = Some(value);
            }
            ArrayStorageType::Sparse(storage, ..) => {
                storage.insert(item, value);
            }
        }
    }

    /// Delete an array storage slot, leaving a hole.
    pub fn delete(&mut self, item: usize) {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                if let Some(i) = storage.get_mut(item) {
                    *i = None;
                }
            }
            ArrayStorageType::Sparse(storage, ..) => {
                storage.remove(&item);
            }
        }
    }

    /// Get the length of the array.
    pub fn length(&self) -> usize {
        match &self.storage_type {
            ArrayStorageType::Dense(storage) => {
                storage.len()
            }
            ArrayStorageType::Sparse(storage, length) => {
                *length
            }
        }
        //self.storage.len()
    }

    /// Set the length of the array.
    pub fn set_length(&mut self, size: usize) {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                storage.resize(size, None);
            }
            ArrayStorageType::Sparse(storage, length) => {
                if size > *length {
                    *length = size;
                } else {
                    let mut to_remove = Vec::new();
                    for i in size..*length {
                        to_remove.push(i);
                    }
                    for i in to_remove {
                        storage.remove(&i);
                    }
                    *length = size;
                }
            }
        }
    }

    /// Append the contents of another array into this one.
    ///
    /// The values in the other array remain there and are merely copied into
    /// this one.
    ///
    /// Holes are copied as holes and not resolved at append time.
    pub fn append(&mut self, other_array: &Self) {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                match &other_array.storage_type {
                    ArrayStorageType::Dense(other_storage) => {
                        for other_item in other_storage.iter() {
                            storage.push(*other_item)
                        }
                    }
                    ArrayStorageType::Sparse(other_storage, length) => {
                        for i in 0..*length {
                            storage.push(other_storage.value_at(i).cloned());
                        }
                    }
                }
            }
            ArrayStorageType::Sparse(storage, length) => {
                match &other_array.storage_type {
                    ArrayStorageType::Dense(other_storage) => {
                        for (i, v) in other_storage.iter().enumerate() {
                            storage.insert(i + *length, v.unwrap());
                        }
                    }
                    ArrayStorageType::Sparse(other_storage, other_length) => {
                        for i in 0..*other_length {
                            storage.insert(i + *length, other_storage.value_at(i).cloned().unwrap());
                        }
                    }
                }
            }
        }
        /*for other_item in other_array.storage.iter() {
            self.storage.push(*other_item)
        }*/
    }

    /// Push a single value onto the end of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn push(&mut self, item: Value<'gc>) {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                storage.push(Some(item));
            }
            ArrayStorageType::Sparse(storage, length) => {
                storage.insert(*length, item);
                *length += 1;
            }
        }
        //self.storage.push(Some(item))
    }

    /// Push an array hole onto the end of this array.
    pub fn push_hole(&mut self) {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                let mut new_storage = DynamicMap::new();
                for (i, v) in storage.iter().enumerate() {
                    if let Some(v) = v {
                        new_storage.insert(i, *v);
                    }
                }
                self.storage_type = ArrayStorageType::Sparse(new_storage, storage.len() + 1);

            }
            ArrayStorageType::Sparse(storage, length) => {
                //storage.insert(*length, Value::Undefined);
                *length += 1;
            }
        }
        //self.storage.push(None)
    }

    /// Pop a value from the back of the array.
    ///
    /// This method preferrentially pops non-holes from the array first. If a
    /// hole is popped, it will become `undefined`.
    pub fn pop(&mut self) -> Value<'gc> {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                storage.pop().unwrap_or(None).unwrap_or(Value::Undefined)
            }
            ArrayStorageType::Sparse(storage, length) => {
                let value = storage.value_at(*length - 1).cloned().unwrap_or(Value::Undefined);
                storage.remove(&(*length - 1));
                *length -= 1;
                value
            }
        }

        /*let mut non_hole = None;

        for (i, item) in self.storage.iter().enumerate().rev() {
            if item.is_some() {
                non_hole = Some(i);
                break;
            }
        }

        if let Some(non_hole) = non_hole {
            self.storage.remove(non_hole).unwrap()
        } else {
            self.storage
                .pop()
                .unwrap_or(None)
                .unwrap_or(Value::Undefined)
        }*/
    }

    /// Shift a value from the front of the array.
    ///
    /// This method preferrentially pops non-holes from the array first. If a
    /// hole is popped, it will become `undefined`.
    pub fn shift(&mut self) -> Value<'gc> {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                if !storage.is_empty() {
                    return storage.remove(0).unwrap_or(Value::Undefined);
                }
                Value::Undefined
            }
            ArrayStorageType::Sparse(storage, length) => {
                let value = storage.value_at(0).cloned().unwrap_or(Value::Undefined);
                storage.remove(&0);
                *length -= 1;
                value
            }
        }
        /*if !self.storage.is_empty() {
            self.storage.remove(0).unwrap_or(Value::Undefined)
        } else {
            Value::Undefined
        }*/
    }

    /// Unshift a single value onto the start of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn unshift(&mut self, item: Value<'gc>) {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                storage.insert(0, Some(item));
            }
            ArrayStorageType::Sparse(storage, length) => {
                let mut new_storage = DynamicMap::new();
                new_storage.insert(0, item);
                for i in 0..*length {
                    new_storage.insert(i + 1, storage.value_at(i).cloned().unwrap());
                }
                *length += 1;
                self.storage_type = ArrayStorageType::Sparse(new_storage, *length);
            }
        }
        //self.storage.insert(0, Some(item))
    }

    /// Iterate over array values.
    pub fn iter<'a>(
        &'a self,
    ) -> impl DoubleEndedExactSizeIterator<Item = Option<Value<'gc>>> + 'a {
        ArrayStorageIterator {
            storage: self,
            index: 0,
            index_back: self.length(),
        }
    }

    /// Remove a value from a specific position in the array.
    ///
    /// This function returns None if the index is out of bonds.
    /// Otherwise, it returns the removed value.
    ///
    /// Negative bounds are supported and treated as indexing from the end of
    /// the array, backwards.
    pub fn remove(&mut self, position: i32) -> Option<Value<'gc>> {
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                let position = if position < 0 {
                    max(position + storage.len() as i32, 0) as usize
                } else {
                    position as usize
                };

                if position >= storage.len() {
                    None
                } else {
                    storage.remove(position)
                }
            }
            ArrayStorageType::Sparse(storage, length) => {
                let position = if position < 0 {
                    max(position + *length as i32, 0) as usize
                } else {
                    position as usize
                };

                if position >= *length {
                    None
                } else {
                    let value = storage.value_at(position).cloned();
                    storage.remove(&position);
                    *length -= 1;
                    value
                }
            }
        }
        /*let position = if position < 0 {
            max(position + self.storage.len() as i32, 0) as usize
        } else {
            position as usize
        };

        if position >= self.storage.len() {
            None
        } else {
            self.storage.remove(position)
        }*/
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
        match &mut self.storage_type {
            ArrayStorageType::Dense(storage) => {
                let start = match range.start_bound() {
                    std::ops::Bound::Included(&start) => start,
                    std::ops::Bound::Excluded(&start) => start + 1,
                    std::ops::Bound::Unbounded => 0,
                };
                let end = match range.end_bound() {
                    std::ops::Bound::Included(&end) => end + 1,
                    std::ops::Bound::Excluded(&end) => end,
                    std::ops::Bound::Unbounded => storage.len(),
                };
                let replace_with = replace_with.into_iter().map(Some);
                storage.splice(range, replace_with)
            }
            ArrayStorageType::Sparse(storage, length) => {
                let start = match range.start_bound() {
                    std::ops::Bound::Included(&start) => start,
                    std::ops::Bound::Excluded(&start) => start + 1,
                    std::ops::Bound::Unbounded => 0,
                };
                let end = match range.end_bound() {
                    std::ops::Bound::Included(&end) => end + 1,
                    std::ops::Bound::Excluded(&end) => end,
                    std::ops::Bound::Unbounded => *length,
                };
                let replace_with = replace_with.into_iter().map(Some);
                //storage.splice(range, replace_with)
                todo!("splice sparse")
            }
        }
        /*self.storage
            .splice(range, replace_with.into_iter().map(Some))*/
    }
}

pub trait DoubleEndedExactSizeIterator: DoubleEndedIterator + ExactSizeIterator {}

impl<'gc, V> FromIterator<V> for ArrayStorage<'gc>
    where
        V: Into<Value<'gc>>,
{
    fn from_iter<I>(values: I) -> Self
        where
            I: IntoIterator<Item = V>,
    {
        let storage = values.into_iter().map(|v| Some(v.into())).collect();

        //Self { storage }
        let storage_type = ArrayStorageType::Dense(storage);
        Self { storage_type }
    }
}
