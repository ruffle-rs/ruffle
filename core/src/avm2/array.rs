//! Array support types

use crate::avm2::value::Value;
use gc_arena::Collect;
use std::collections::BTreeMap;

const MIN_SPARSE_LENGTH: usize = 32;
const MAX_DENSE_LENGTH: usize = 1 << 28;

/// The array storage portion of an array object.
///
/// Array values may consist of either standard `Value`s or "holes": values
/// which are not properties of the associated object and must be resolved in
/// the prototype.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum ArrayStorage<'gc> {
    /// Dense arrays store a vector of values and a count of non-holes in the vector (m_denseUsed in avmplus).
    Dense {
        storage: Vec<Option<Value<'gc>>>,
        occupied_count: usize,
    },
    /// Sparse arrays store a BTreeMap of values and an explicit ECMAScript "Array.length".
    Sparse {
        storage: BTreeMap<usize, Value<'gc>>,
        length: usize,
    },
}

/// An iterator over array storage. This iterator will yield `Some(None)` for holes.
struct ArrayStorageIterator<'a, 'gc> {
    storage: &'a ArrayStorage<'gc>,
    index: usize,
    index_back: usize,
}

impl<'gc> Iterator for ArrayStorageIterator<'_, 'gc> {
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.index_back {
            None
        } else {
            let value = match &self.storage {
                ArrayStorage::Dense { storage, .. } => storage[self.index],
                ArrayStorage::Sparse { storage, .. } => storage.get(&self.index).copied(),
            };
            self.index += 1;
            Some(value)
        }
    }
}

impl DoubleEndedIterator for ArrayStorageIterator<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.index_back || self.index_back == 0 {
            None
        } else {
            self.index_back -= 1;
            let value = match &self.storage {
                ArrayStorage::Dense { storage, .. } => storage[self.index_back],
                ArrayStorage::Sparse { storage, .. } => storage.get(&self.index_back).copied(),
            };
            Some(value)
        }
    }
}

impl ExactSizeIterator for ArrayStorageIterator<'_, '_> {
    fn len(&self) -> usize {
        self.index_back - self.index
    }
}

impl<'gc> ArrayStorage<'gc> {
    /// Construct new array storage.
    ///
    /// The length parameter indicates how big the array storage should start
    /// out as. All array storage consists of holes.
    pub fn new(length: usize) -> Self {
        if length > MAX_DENSE_LENGTH {
            ArrayStorage::Sparse {
                storage: BTreeMap::new(),
                length,
            }
        } else {
            ArrayStorage::Dense {
                storage: Vec::with_capacity(length),
                occupied_count: 0,
            }
        }
    }

    /// Convert a set of arguments into array storage.
    pub fn from_args(values: &[Value<'gc>]) -> Self {
        let storage = values
            .iter()
            .map(|v| Some(*v))
            .collect::<Vec<Option<Value<'gc>>>>();

        ArrayStorage::Dense {
            storage,
            occupied_count: values.len(),
        }
    }

    /// Wrap an existing storage Vec in an `ArrayStorage`.
    pub fn from_storage(storage: Vec<Option<Value<'gc>>>) -> Self {
        let occupied_count = storage.iter().filter(|v| v.is_some()).count();

        ArrayStorage::Dense {
            storage,
            occupied_count,
        }
    }

    /// Replace the existing dense storage with a new dense storage.
    /// Panics if this `ArrayStorage` is sparse.
    pub fn replace_dense_storage(&mut self, new_storage: Vec<Option<Value<'gc>>>) {
        let new_occupied_count = new_storage.iter().filter(|v| v.is_some()).count();

        match self {
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                *occupied_count = new_occupied_count;
                *storage = new_storage;
            }
            ArrayStorage::Sparse { .. } => {
                panic!("Cannot replace dense storage on sparse ArrayStorage");
            }
        }
    }

    /// Retrieve a value from array storage by index.
    ///
    /// Array holes and out of bounds values will be treated the same way, by
    /// yielding `None`.
    pub fn get(&self, item: usize) -> Option<Value<'gc>> {
        match &self {
            ArrayStorage::Dense { storage, .. } => storage.get(item).copied().unwrap_or(None),
            ArrayStorage::Sparse { storage, .. } => storage.get(&item).copied(),
        }
    }

    /// Get the next index after the given index that contains a value.
    pub fn get_next_enumerant(&self, last_index: usize) -> Option<usize> {
        match &self {
            ArrayStorage::Dense { storage, .. } => {
                let mut last_index = last_index;
                while last_index < storage.len() {
                    if storage[last_index].is_some() {
                        return Some(last_index + 1);
                    }
                    last_index += 1;
                }
                None
            }
            ArrayStorage::Sparse { storage, .. } => {
                if let Some((&key, _)) = storage.range(last_index..).next() {
                    return Some(key + 1);
                }
                None
            }
        }
    }

    /// Set an array storage slot to a particular value.
    ///
    /// If the item index extends beyond the length of the array, then the
    /// array will be extended with holes.
    pub fn set(&mut self, item: usize, value: Value<'gc>) {
        match self {
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                if item >= storage.len() {
                    if Self::should_convert_to_sparse(item + 1, *occupied_count) {
                        self.convert_to_sparse();
                        if let ArrayStorage::Sparse { storage, length } = self {
                            *length = item + 1;
                            storage.insert(item, value);
                        }
                    } else {
                        storage.resize(item + 1, None);
                        storage[item] = Some(value);
                        *occupied_count += 1;
                    }
                } else {
                    if storage[item].is_none() {
                        *occupied_count += 1;
                    }
                    storage[item] = Some(value);
                }
            }
            ArrayStorage::Sparse { storage, length } => {
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
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                if let Some(i) = storage.get_mut(item) {
                    *i = None;
                    if *occupied_count > 0 {
                        *occupied_count -= 1;
                        self.maybe_convert_to_sparse();
                    }
                }
            }
            ArrayStorage::Sparse { storage, .. } => {
                storage.remove(&item);
                self.maybe_convert_to_dense();
            }
        }
    }

    /// Get the length of the array.
    pub fn length(&self) -> usize {
        match &self {
            ArrayStorage::Dense { storage, .. } => storage.len(),
            ArrayStorage::Sparse { length, .. } => *length,
        }
    }

    /// Set the length of the array.
    pub fn set_length(&mut self, size: usize) {
        match self {
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                if Self::should_convert_to_sparse(size, *occupied_count) {
                    self.convert_to_sparse();
                    if let ArrayStorage::Sparse { .. } = self {
                        self.set_length(size);
                    }
                } else {
                    storage.resize(size, None);
                }
            }
            ArrayStorage::Sparse { storage, length } => {
                if size > *length {
                    *length = size;
                } else {
                    storage.retain(|&k, _| k < size);
                    *length = size;
                    self.maybe_convert_to_dense();
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
        for value in other_array.iter() {
            if let Some(value) = value {
                self.push(value);
            } else {
                self.push_hole();
            }
        }
    }

    /// Push a single value onto the end of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn push(&mut self, item: Value<'gc>) {
        match self {
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                storage.push(Some(item));
                *occupied_count += 1;
            }
            ArrayStorage::Sparse { storage, length } => {
                storage.insert(*length, item);
                *length += 1;
            }
        }
    }

    /// Determine if the array should be converted to a sparse representation based on its size and the number of occupied slots.
    fn should_convert_to_sparse(size: usize, occupied_count: usize) -> bool {
        (occupied_count < (size / 4) && size > MIN_SPARSE_LENGTH) || size > MAX_DENSE_LENGTH
    }

    /// Convert the array storage to a sparse representation.
    fn convert_to_sparse(&mut self) {
        if let ArrayStorage::Dense { storage, .. } = self {
            let mut new_storage = BTreeMap::new();
            for (i, v) in storage.iter().enumerate() {
                if let Some(v) = v {
                    new_storage.insert(i, *v);
                }
            }
            *self = ArrayStorage::Sparse {
                storage: new_storage,
                length: storage.len(),
            };
        }
    }

    /// Convert the array to a sparse representation if it meets the criteria.
    fn maybe_convert_to_sparse(&mut self) {
        if let ArrayStorage::Dense {
            storage,
            occupied_count,
        } = self
        {
            if Self::should_convert_to_sparse(storage.len(), *occupied_count) {
                self.convert_to_sparse();
            }
        }
    }

    /// Convert the array to a dense representation if it meets the criteria.
    fn maybe_convert_to_dense(&mut self) {
        if let ArrayStorage::Sparse { storage, length } = self {
            if storage.is_empty() && *length == 0 {
                *self = ArrayStorage::Dense {
                    storage: Vec::new(),
                    occupied_count: 0,
                };
            }
        }
    }

    /// Push an array hole onto the end of this array.
    pub fn push_hole(&mut self) {
        match self {
            ArrayStorage::Dense { storage, .. } => {
                storage.push(None);
                self.maybe_convert_to_sparse();
            }
            ArrayStorage::Sparse { length, .. } => {
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
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                let non_hole = storage
                    .iter()
                    .enumerate()
                    .rposition(|(_, item)| item.is_some());

                if let Some(non_hole) = non_hole {
                    *occupied_count -= 1;
                    let value = storage.remove(non_hole).unwrap();
                    self.maybe_convert_to_sparse();
                    value
                } else {
                    storage.pop().unwrap_or(None).unwrap_or(Value::Undefined)
                }
            }
            ArrayStorage::Sparse { storage, length } => {
                let non_hole = storage.pop_last();
                if let Some((_index, value)) = non_hole {
                    *length -= 1;
                    value
                } else {
                    if *length > 0 {
                        *length -= 1;
                    }
                    self.maybe_convert_to_dense();
                    Value::Undefined
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
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                if !storage.is_empty() {
                    let value = storage.remove(0);
                    if value.is_some() {
                        *occupied_count -= 1;
                    }
                    self.maybe_convert_to_sparse();
                    return value.unwrap_or(Value::Undefined);
                }
                Value::Undefined
            }
            ArrayStorage::Sparse { storage, length } => {
                let value = storage.get(&0).copied().unwrap_or(Value::Undefined);
                storage.remove(&0);

                let mut new_storage = BTreeMap::new();
                for (&key, &value) in storage.iter() {
                    new_storage.insert(key - 1, value);
                }

                *storage = new_storage;

                if *length > 0 {
                    *length -= 1;
                }
                self.maybe_convert_to_dense();
                value
            }
        }
    }

    /// Unshift a single value onto the start of this array.
    ///
    /// It is not possible to push a hole onto the array.
    pub fn unshift(&mut self, item: Value<'gc>) {
        match self {
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                storage.insert(0, Some(item));
                *occupied_count += 1;
            }
            ArrayStorage::Sparse { storage, length } => {
                let mut new_storage = BTreeMap::new();
                new_storage.insert(0, item);
                for (key, value) in storage.iter() {
                    new_storage.insert(key + 1, *value);
                }
                *storage = new_storage;
                *length += 1;
            }
        }
    }

    /// Iterate over array values.
    pub fn iter<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = Option<Value<'gc>>>
           + ExactSizeIterator<Item = Option<Value<'gc>>>
           + 'a {
        let index_back = self.length();
        ArrayStorageIterator {
            storage: self,
            index: 0,
            index_back,
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
            ArrayStorage::Dense {
                storage,
                occupied_count,
            } => {
                let position = if position < 0 {
                    std::cmp::max(position + storage.len() as i32, 0) as usize
                } else {
                    position as usize
                };

                if position >= storage.len() {
                    None
                } else {
                    let value = storage.remove(position);
                    if value.is_some() {
                        *occupied_count -= 1;
                    }
                    self.maybe_convert_to_sparse();
                    value
                }
            }
            ArrayStorage::Sparse { storage, length } => {
                let position = if position < 0 {
                    std::cmp::max(position + *length as i32, 0) as usize
                } else {
                    position as usize
                };

                if position >= *length {
                    None
                } else {
                    let value = storage.get(&position).copied();
                    storage.remove(&position);
                    *length -= 1;
                    let new_storage = storage.split_off(&position);
                    for (&key, &value) in new_storage.iter() {
                        storage.insert(key - 1, value);
                    }
                    self.maybe_convert_to_dense();
                    value
                }
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
        let storage: Vec<_> = values.into_iter().map(|v| Some(v.into())).collect();

        let occupied_count = storage.iter().filter(|v| v.is_some()).count();

        ArrayStorage::Dense {
            storage,
            occupied_count,
        }
    }
}
