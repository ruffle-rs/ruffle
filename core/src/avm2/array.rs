//! Array support types

use crate::avm2::value::Value;
use gc_arena::Collect;
use std::collections::BTreeMap;
use std::{cmp::max, ops::RangeBounds};

/// The array storage portion of an array object.
///
/// Array values may consist of either standard `Value`s or "holes": values
/// which are not properties of the associated object and must be resolved in
/// the prototype.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum ArrayStorage<'gc> {
    Dense(Vec<Option<Value<'gc>>>),
    Sparse(BTreeMap<usize, Value<'gc>>, usize),
}

struct ArrayStorageIterator<'a, 'gc> {
    storage: &'a ArrayStorage<'gc>,
    index: usize,
    index_back: usize,
}

impl<'a, 'gc> Iterator for ArrayStorageIterator<'a, 'gc> {
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.storage {
            ArrayStorage::Dense(storage) => {
                if self.index >= self.index_back {
                    None
                } else {
                    let value = storage[self.index];
                    self.index += 1;
                    Some(value)
                }
            }
            ArrayStorage::Sparse(storage, _length) => {
                if self.index >= self.index_back {
                    None
                } else {
                    let value = storage.get(&self.index).cloned();
                    self.index += 1;
                    Some(value)
                }
            }
        }
    }
}

impl<'a, 'gc> DoubleEndedIterator for ArrayStorageIterator<'a, 'gc> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &self.storage {
            ArrayStorage::Dense(storage) => {
                if self.index >= self.index_back || self.index_back == 0 {
                    None
                } else {
                    self.index_back -= 1;
                    let value = storage[self.index_back];
                    Some(value)
                }
            }
            ArrayStorage::Sparse(storage, _length) => {
                if self.index >= self.index_back || self.index_back == 0 {
                    None
                } else {
                    self.index_back -= 1;
                    let value = storage.get(&self.index_back).cloned();
                    Some(value)
                }
            }
        }
    }
}

impl<'a, 'gc> ExactSizeIterator for ArrayStorageIterator<'a, 'gc> {
    fn len(&self) -> usize {
        match &self.storage {
            ArrayStorage::Dense(_) => self.index_back - self.index,
            ArrayStorage::Sparse(_, _) => self.index_back - self.index,
        }
    }
}

impl<'gc> ArrayStorage<'gc> {
    /// Construct new array storage.
    ///
    /// The length parameter indicates how big the array storage should start
    /// out as. All array storage consists of holes.
    pub fn new(length: usize) -> Self {
        if length > (1 << 28) {
            let storage = BTreeMap::new();
            let storage_type = ArrayStorage::Sparse(storage, 0);
            return storage_type;
        }
        let storage = Vec::with_capacity(length);
        let storage_type = ArrayStorage::Dense(storage);

        storage_type
    }

    /// Convert a set of arguments into array storage.
    pub fn from_args(values: &[Value<'gc>]) -> Self {
        let storage = values
            .iter()
            .map(|v| Some(*v))
            .collect::<Vec<Option<Value<'gc>>>>();

        let storage_type = ArrayStorage::Dense(storage);

        storage_type
    }

    /// Wrap an existing storage Vec in an `ArrayStorage`.
    pub fn from_storage(storage: Vec<Option<Value<'gc>>>) -> Self {
        let storage_type = ArrayStorage::Dense(storage);
        storage_type
    }

    /// Retrieve a value from array storage by index.
    ///
    /// Array holes and out of bounds values will be treated the same way, by
    /// yielding `None`.
    pub fn get(&self, item: usize) -> Option<Value<'gc>> {
        match &self {
            ArrayStorage::Dense(storage) => {
                return storage.get(item).cloned().unwrap_or(None);
            }
            ArrayStorage::Sparse(storage, ..) => storage.get(&item).cloned(),
        }
    }

    pub fn get_next_enumerant(&self, last_index: usize) -> Option<usize> {
        match &self {
            ArrayStorage::Dense(storage) => {
                let mut last_index = last_index;
                while last_index < storage.len() {
                    if storage[last_index].is_some() {
                        return Some(last_index + 1);
                    }
                    last_index += 1;
                }
                None
            }
            ArrayStorage::Sparse(storage, _length) => {
                if let Some((&key, &_value)) = storage.range(last_index..).next() {
                    return Some(key + 1);
                }
                None
            }
        }
    }

    pub fn convert_to_sparse(&mut self) {
        match self {
            ArrayStorage::Dense(storage) => {
                let mut new_storage = BTreeMap::new();
                for (i, v) in storage.iter().enumerate() {
                    if let Some(v) = v {
                        new_storage.insert(i, *v);
                    }
                }
                *self = ArrayStorage::Sparse(new_storage, storage.len());
            }
            ArrayStorage::Sparse(..) => {}
        }
    }

    /// Set an array storage slot to a particular value.
    ///
    /// If the item index extends beyond the length of the array, then the
    /// array will be extended with holes.
    pub fn set(&mut self, item: usize, value: Value<'gc>) {
        match self {
            ArrayStorage::Dense(storage) => {
                if storage.len() + 1 < (item) {
                    let mut new_storage = BTreeMap::new();
                    for (i, v) in storage.iter().enumerate() {
                        if let Some(v) = v {
                            new_storage.insert(i, *v);
                        }
                    }
                    new_storage.insert(item, value);
                    *self = ArrayStorage::Sparse(new_storage, item + 1);
                } else {
                    if storage.len() < (item + 1) {
                        storage.resize(item + 1, None)
                    }
                    *storage.get_mut(item).unwrap() = Some(value);
                }
            }
            ArrayStorage::Sparse(storage, length) => {
                storage.insert(item, value);
                if item >= *length {
                    *length = item + 1;
                }
            }
        }
    }

    /// Delete an array storage slot, leaving a hole.
    pub fn delete(&mut self, item: usize) {
        match self {
            ArrayStorage::Dense(storage) => {
                if let Some(i) = storage.get_mut(item) {
                    *i = None;
                }
            }
            ArrayStorage::Sparse(storage, ..) => {
                storage.remove(&item);
            }
        }
    }

    /// Get the length of the array.
    pub fn length(&self) -> usize {
        match &self {
            ArrayStorage::Dense(storage) => storage.len(),
            ArrayStorage::Sparse(_storage, length) => *length,
        }
    }

    /// Set the length of the array.
    pub fn set_length(&mut self, size: usize) {
        match self {
            ArrayStorage::Dense(storage) => {
                if size < 1 << 28 {
                    storage.resize(size, None);
                } else {
                    let mut new_storage = BTreeMap::new();
                    for (i, v) in storage.iter().enumerate() {
                        if let Some(v) = v {
                            new_storage.insert(i, *v);
                        }
                    }
                    *self = ArrayStorage::Sparse(new_storage, size);
                }
            }
            ArrayStorage::Sparse(storage, length) => {
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
        match self {
            ArrayStorage::Dense(storage) => match &other_array {
                ArrayStorage::Dense(other_storage) => {
                    for other_item in other_storage.iter() {
                        storage.push(*other_item)
                    }
                }
                ArrayStorage::Sparse(other_storage, length) => {
                    for i in 0..*length {
                        storage.push(other_storage.get(&i).cloned());
                    }
                }
            },
            ArrayStorage::Sparse(storage, length) => match &other_array {
                ArrayStorage::Dense(other_storage) => {
                    for (i, v) in other_storage.iter().enumerate() {
                        match v {
                            Some(v) => {
                                storage.insert(i + *length, *v);
                            }
                            None => {}
                        }
                    }
                    *length += other_storage.len();
                }
                ArrayStorage::Sparse(other_storage, other_length) => {
                    for i in 0..*other_length {
                        let value = other_storage.get(&i).cloned();
                        if let Some(value) = value {
                            storage.insert(i + *length, value);
                        }
                    }
                    *length += *other_length;
                }
            },
        }
    }

    /// Push a single value onto the end of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn push(&mut self, item: Value<'gc>) {
        match self {
            ArrayStorage::Dense(storage) => {
                storage.push(Some(item));
            }
            ArrayStorage::Sparse(storage, length) => {
                storage.insert(*length, item);
                *length += 1;
            }
        }
    }

    /// Push an array hole onto the end of this array.
    pub fn push_hole(&mut self) {
        match self {
            ArrayStorage::Dense(storage) => {
                let mut new_storage = BTreeMap::new();
                for (i, v) in storage.iter().enumerate() {
                    if let Some(v) = v {
                        new_storage.insert(i, *v);
                    }
                }
                *self = ArrayStorage::Sparse(new_storage, storage.len() + 1);
            }
            ArrayStorage::Sparse(_storage, length) => {
                *length += 1;
            }
        }
    }

    /// Pop a value from the back of the array.
    ///
    /// This method preferrentially pops non-holes from the array first. If a
    /// hole is popped, it will become `undefined`.
    pub fn pop(&mut self) -> Value<'gc> {
        match self {
            ArrayStorage::Dense(storage) => {
                let mut non_hole = None;

                for (i, item) in storage.iter().enumerate().rev() {
                    if item.is_some() {
                        non_hole = Some(i);
                        break;
                    }
                }

                if let Some(non_hole) = non_hole {
                    storage.remove(non_hole).unwrap()
                } else {
                    storage.pop().unwrap_or(None).unwrap_or(Value::Undefined)
                }
            }
            ArrayStorage::Sparse(storage, length) => {
                let mut non_hole = None;

                let storage_clone = storage.clone();
                if let Some((i, _item)) = storage_clone.iter().next_back() {
                    non_hole = Some(i);
                }
                if let Some(non_hole) = non_hole {
                    *length -= 1;
                    storage.remove(non_hole).unwrap()
                } else if *length == 0 {
                    Value::Undefined
                } else {
                    let value = storage
                        .get(&(*length - 1))
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    storage.remove(&(*length - 1));
                    *length -= 1;
                    value
                }
            }
        }
    }

    /// Shift a value from the front of the array.
    ///
    /// This method preferrentially pops non-holes from the array first. If a
    /// hole is popped, it will become `undefined`.
    pub fn shift(&mut self) -> Value<'gc> {
        match self {
            ArrayStorage::Dense(storage) => {
                if !storage.is_empty() {
                    return storage.remove(0).unwrap_or(Value::Undefined);
                }
                Value::Undefined
            }
            ArrayStorage::Sparse(storage, length) => {
                let value = storage.get(&0).cloned().unwrap_or(Value::Undefined);
                storage.remove(&0);

                let storage_clone = storage.clone();
                for (&key, &value) in storage_clone.range(0..) {
                    storage.insert(key - 1, value);
                    storage.remove(&key);
                }

                if *length > 0 {
                    *length -= 1;
                }
                value
            }
        }
    }

    /// Unshift a single value onto the start of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn unshift(&mut self, item: Value<'gc>) {
        match self {
            ArrayStorage::Dense(storage) => {
                storage.insert(0, Some(item));
            }
            ArrayStorage::Sparse(storage, length) => {
                let mut new_storage = BTreeMap::new();
                new_storage.insert(0, item);
                for i in 0..*length {
                    let item_from_storage = storage.get(&i).cloned();
                    if let Some(item) = item_from_storage {
                        new_storage.insert(i + 1, item);
                    }
                }
                *length += 1;
                *self = ArrayStorage::Sparse(new_storage, *length);
            }
        }
    }

    /// Iterate over array values.
    pub fn iter<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = Option<Value<'gc>>>
           + ExactSizeIterator<Item = Option<Value<'gc>>>
           + 'a {
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
        match self {
            ArrayStorage::Dense(storage) => {
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
            ArrayStorage::Sparse(storage, length) => {
                let position = if position < 0 {
                    max(position + *length as i32, 0) as usize
                } else {
                    position as usize
                };

                if position >= *length {
                    None
                } else {
                    let value = storage.get(&position).cloned();
                    storage.remove(&position);
                    *length -= 1;
                    let storage_clone = storage.clone();
                    for (&key, &value) in storage_clone.range(position..) {
                        storage.insert(key - 1, value);
                        storage.remove(&key);
                    }
                    value
                }
            }
        }
    }

    pub fn splice<'a, R, I>(
        &'a mut self,
        range: R,
        replace_with: I,
    ) -> Box<dyn Iterator<Item = Option<Value<'gc>>> + 'a>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = Value<'gc>>,
        <I as IntoIterator>::IntoIter: 'a,
    {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };

        match self {
            ArrayStorage::Dense(storage) => {
                let end = match range.end_bound() {
                    std::ops::Bound::Included(&end) => end + 1,
                    std::ops::Bound::Excluded(&end) => end,
                    std::ops::Bound::Unbounded => storage.len(),
                };
                let replace_with = replace_with.into_iter().map(Some);
                Box::new(storage.splice(start..end, replace_with))
            }
            ArrayStorage::Sparse(storage, length) => {
                let end = match range.end_bound() {
                    std::ops::Bound::Included(&end) => end + 1,
                    std::ops::Bound::Excluded(&end) => end,
                    std::ops::Bound::Unbounded => *length,
                };
                let replace_with = replace_with.into_iter().map(Some);
                let storage_clone = storage.clone();
                let mut new_storage = BTreeMap::new();
                for (i, v) in storage_clone.iter() {
                    if i < &start || i >= &end {
                        new_storage.insert(i, *v);
                    }
                }
                for (i, v) in replace_with.enumerate() {
                    storage.insert(i, v.unwrap());
                }

                //I have no idea what I'm doing (I honestly don't know how to refactor this)
                Box::new(
                    new_storage
                        .into_values()
                        .map(Some)
                        .collect::<Vec<Option<Value<'gc>>>>()
                        .into_iter(),
                )
            }
        }
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

        let storage_type = ArrayStorage::Dense(storage);
        storage_type
    }
}
