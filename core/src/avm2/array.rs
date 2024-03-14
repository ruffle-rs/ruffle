//! Array support types

use crate::avm2::value::Value;
use gc_arena::Collect;
use std::collections::BTreeMap;
use std::{cmp::max, ops::RangeBounds};

const MIN_SPARSE_LENGTH: usize = 32;

/// The array storage portion of an array object.
///
/// Array values may consist of either standard `Value`s or "holes": values
/// which are not properties of the associated object and must be resolved in
/// the prototype.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum ArrayStorage<'gc> {
    Dense(Vec<Option<Value<'gc>>>, usize),
    Sparse(BTreeMap<usize, Value<'gc>>, usize),
}

struct ArrayStorageIterator<'a, 'gc> {
    storage: &'a ArrayStorage<'gc>,
    index: usize,
    index_back: usize,
}

struct ArrayStorageMutableIterator<'a, 'gc> {
    storage: &'a mut ArrayStorage<'gc>,
    index: usize,
    index_back: usize,
}

impl<'a, 'gc> Iterator for ArrayStorageIterator<'a, 'gc> {
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.storage {
            ArrayStorage::Dense(storage, ..) => {
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
            ArrayStorage::Dense(storage, ..) => {
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
            ArrayStorage::Dense(_, _) => self.index_back - self.index,
            ArrayStorage::Sparse(_, _) => self.index_back - self.index,
        }
    }
}

impl<'a, 'gc> Iterator for ArrayStorageMutableIterator<'a, 'gc> {
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.storage {
            ArrayStorage::Dense(storage, ..) => {
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

impl<'a, 'gc> DoubleEndedIterator for ArrayStorageMutableIterator<'a, 'gc> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &self.storage {
            ArrayStorage::Dense(storage, ..) => {
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

impl<'a, 'gc> ExactSizeIterator for ArrayStorageMutableIterator<'a, 'gc> {
    fn len(&self) -> usize {
        match &self.storage {
            ArrayStorage::Dense(_, _) => self.index_back - self.index,
            ArrayStorage::Sparse(_, _) => self.index_back - self.index,
        }
    }
}

struct ArrayStorageDrainIterator<'a, 'gc> {
    iter: ArrayStorageMutableIterator<'a, 'gc>,
    start: usize,
    end: usize,
    has_to_drain: bool,
}

impl<'a, 'gc> Drop for ArrayStorageDrainIterator<'a, 'gc> {
    fn drop(&mut self) {
        let iter = &mut self.iter;
        let len = iter.len();
        for _ in 0..len {
            iter.next().unwrap();
        }
        if !self.has_to_drain {
            return;
        }
        match self.iter.storage {
            ArrayStorage::Dense(storage, mut length) => {
                let mut storage_clone = storage.clone();
                storage_clone.drain(self.start..self.end);
                *self.iter.storage =
                    ArrayStorage::Dense(storage_clone, length - (self.end - self.start));
            }
            ArrayStorage::Sparse(storage, mut length) => {
                let storage_clone = storage.clone();
                let storage_range = storage_clone.range(self.start..self.end);
                for (index, _) in storage_range {
                    storage.remove(index);
                }
                *self.iter.storage =
                    ArrayStorage::Sparse(storage_clone, length - (self.end - self.start));
            }
        }
    }
}

struct ArrayStorageSpliceIterator<'a, 'gc, I>
where
    I: Iterator<Item = Option<Value<'gc>>> + 'a,
{
    drain: ArrayStorageDrainIterator<'a, 'gc>,
    replace_with: I,
}

impl<'a, 'gc, I> Drop for ArrayStorageSpliceIterator<'a, 'gc, I>
where
    I: Iterator<Item = Option<Value<'gc>>> + 'a,
{
    fn drop(&mut self) {
        let iter = &mut self.drain.iter;
        let len = iter.len();
        for _ in 0..len {
            iter.next().unwrap();
        }
        match self.drain.iter.storage {
            ArrayStorage::Dense(storage, mut length) => {
                storage.splice(self.drain.start..self.drain.end, self.replace_with.by_ref());
                let new_length = storage.len();
                length = new_length;
                *self.drain.iter.storage = ArrayStorage::Dense(storage.clone(), length);
            }
            ArrayStorage::Sparse(storage, mut length) => {
                let storage_clone = storage.clone();
                let storage_range = storage_clone.range(self.drain.start..self.drain.end);
                for (index, _) in storage_range {
                    storage.remove(index);
                }
                let mut replace_with_tree: BTreeMap<usize, Value<'gc>> = BTreeMap::new();
                let mut replace_with_length = 0;
                for value in self.replace_with.by_ref() {
                    match value {
                        Some(value) => {
                            replace_with_tree.insert(replace_with_length, value);
                            replace_with_length += 1;
                        }
                        None => {
                            replace_with_length += 1;
                        }
                    }
                }
                let offset =
                    self.drain.start as i32 - self.drain.end as i32 + replace_with_length as i32;
                let storage_after_offset = storage_clone
                    .range(self.drain.end..)
                    .collect::<BTreeMap<_, _>>();
                for (index, value) in storage_after_offset {
                    storage.remove(index);
                    storage.insert((*index as i32 + offset) as usize, *value);
                }

                length -= self.drain.end - self.drain.start;
                length += replace_with_length;
                for (index, value) in replace_with_tree {
                    storage.insert(self.drain.start + index, value);
                }
                *self.drain.iter.storage = ArrayStorage::Sparse(storage.clone(), length);
            }
        }
    }
}

impl<'a, 'gc, I> Iterator for ArrayStorageSpliceIterator<'a, 'gc, I>
where
    I: Iterator<Item = Option<Value<'gc>>> + 'a,
{
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.iter.next()
    }
}

impl<'a, 'gc, I> DoubleEndedIterator for ArrayStorageSpliceIterator<'a, 'gc, I>
where
    I: Iterator<Item = Option<Value<'gc>>> + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.iter.next_back()
    }
}

impl<'a, 'gc, I> ExactSizeIterator for ArrayStorageSpliceIterator<'a, 'gc, I>
where
    I: Iterator<Item = Option<Value<'gc>>> + 'a,
{
    fn len(&self) -> usize {
        self.drain.iter.len()
    }
}

impl<'a, 'gc> ArrayStorageIterator<'a, 'gc> {}

impl<'a, 'gc> Iterator for ArrayStorageDrainIterator<'a, 'gc> {
    type Item = Option<Value<'gc>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, 'gc> DoubleEndedIterator for ArrayStorageDrainIterator<'a, 'gc> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, 'gc> ExactSizeIterator for ArrayStorageDrainIterator<'a, 'gc> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'gc> ArrayStorage<'gc> {
    /// Construct new array storage.
    ///
    /// The length parameter indicates how big the array storage should start
    /// out as. All array storage consists of holes.
    pub fn new(length: usize) -> Self {
        if length > (1 << 28) {
            ArrayStorage::Sparse(BTreeMap::new(), 0)
        } else {
            ArrayStorage::Dense(Vec::with_capacity(length), 0)
        }
    }

    /// Convert a set of arguments into array storage.
    pub fn from_args(values: &[Value<'gc>]) -> Self {
        let storage = values
            .iter()
            .map(|v| Some(*v))
            .collect::<Vec<Option<Value<'gc>>>>();

        ArrayStorage::Dense(storage, values.len())
    }

    /// Wrap an existing storage Vec in an `ArrayStorage`.
    pub fn from_storage(storage: Vec<Option<Value<'gc>>>) -> Self {
        let dense_used = storage.iter().filter(|v| v.is_some()).count();
        let storage_type = ArrayStorage::Dense(storage, dense_used);
        storage_type
    }

    /// Retrieve a value from array storage by index.
    ///
    /// Array holes and out of bounds values will be treated the same way, by
    /// yielding `None`.
    pub fn get(&self, item: usize) -> Option<Value<'gc>> {
        match &self {
            ArrayStorage::Dense(storage, ..) => {
                return storage.get(item).cloned().unwrap_or(None);
            }
            ArrayStorage::Sparse(storage, ..) => storage.get(&item).cloned(),
        }
    }

    pub fn get_next_enumerant(&self, last_index: usize) -> Option<usize> {
        match &self {
            ArrayStorage::Dense(storage, ..) => {
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

    fn convert_to_sparse(&mut self) {
        match self {
            ArrayStorage::Dense(storage, ..) => {
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

    fn convert_to_dense(&mut self) {
        match self {
            ArrayStorage::Dense(..) => {}
            ArrayStorage::Sparse(storage, length) => {
                let mut new_storage = Vec::new();
                for i in 0..*length {
                    let value = storage.get(&i).cloned();
                    new_storage.push(value);
                }
                let dense_used = new_storage.iter().filter(|v| v.is_some()).count();
                *self = ArrayStorage::Dense(new_storage, dense_used);
            }
        }
    }

    /// Set an array storage slot to a particular value.
    ///
    /// If the item index extends beyond the length of the array, then the
    /// array will be extended with holes.
    pub fn set(&mut self, item: usize, value: Value<'gc>) {
        match self {
            ArrayStorage::Dense(storage, dense_used) => {
                if storage.len() < (item + 1) {
                    //check if dense_used is less than quarter of item
                    if *dense_used < (item / 4) && MIN_SPARSE_LENGTH < item {
                        self.convert_to_sparse();
                        if let ArrayStorage::Sparse(storage, length) = self {
                            *length = item + 1;
                            storage.insert(item, value);
                        }
                    } else {
                        storage.resize(item + 1, None);
                        *storage.get_mut(item).unwrap() = Some(value);
                        let new_holes = item as i32 - storage.len() as i32 + 1;
                        *dense_used = (*dense_used + new_holes as usize) + 1;
                    }
                } else {
                    if storage[item].is_none() {
                        *dense_used += 1;
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
            ArrayStorage::Dense(storage, dense_used) => {
                if let Some(i) = storage.get_mut(item) {
                    *i = None;
                    if *dense_used != 0 {
                        *dense_used -= 1;
                        if *dense_used == 0 {
                            self.convert_to_dense();
                        }
                        self.maybe_convert_to_sparse();
                    }
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
            ArrayStorage::Dense(storage, ..) => storage.len(),
            ArrayStorage::Sparse(_storage, length) => *length,
        }
    }

    /// Set the length of the array.
    pub fn set_length(&mut self, size: usize) {
        match self {
            ArrayStorage::Dense(storage, dense_used) => {
                if size < 1 << 28 {
                    let num_of_new_holes = (size as i32 - storage.len() as i32).max(0) as usize;
                    if *dense_used + num_of_new_holes < (size / 4)
                        && num_of_new_holes > 0
                        && MIN_SPARSE_LENGTH < size
                    {
                        self.convert_to_sparse();
                        if let ArrayStorage::Sparse(_storage, length) = self {
                            *length = size;
                        }
                    } else {
                        storage.resize(size, None);
                    }
                } else {
                    self.convert_to_sparse();
                    if let ArrayStorage::Sparse(_storage, length) = self {
                        *length = size;
                    }
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
            ArrayStorage::Dense(storage, dense_used) => match &other_array {
                ArrayStorage::Dense(other_storage, other_dense_used) => {
                    for other_item in other_storage.iter() {
                        storage.push(*other_item)
                    }
                    *dense_used += other_dense_used;
                    self.maybe_convert_to_sparse();
                }
                ArrayStorage::Sparse(other_storage, length) => {
                    for i in 0..*length {
                        storage.push(other_storage.get(&i).cloned());
                    }
                    *dense_used += other_storage.len();
                    self.maybe_convert_to_sparse();
                }
            },
            ArrayStorage::Sparse(storage, length) => match &other_array {
                ArrayStorage::Dense(other_storage, ..) => {
                    for (i, v) in other_storage.iter().enumerate() {
                        if let Some(v) = v {
                            storage.insert(i + *length, *v);
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
            ArrayStorage::Dense(storage, dense_used) => {
                storage.push(Some(item));
                *dense_used += 1;
            }
            ArrayStorage::Sparse(storage, length) => {
                storage.insert(*length, item);
                *length += 1;
            }
        }
    }

    fn maybe_convert_to_sparse(&mut self) {
        match self {
            ArrayStorage::Dense(storage, dense_used) => {
                if *dense_used < (storage.len() / 4) && MIN_SPARSE_LENGTH < storage.len() {
                    self.convert_to_sparse();
                }
            }
            ArrayStorage::Sparse(..) => {}
        }
    }

    /// Push an array hole onto the end of this array.
    pub fn push_hole(&mut self) {
        match self {
            ArrayStorage::Dense(storage, ..) => {
                storage.push(None);
                self.maybe_convert_to_sparse();
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
            ArrayStorage::Dense(storage, dense_used) => {
                let non_hole = storage
                    .iter()
                    .enumerate()
                    .rposition(|(_, item)| item.is_some());

                if let Some(non_hole) = non_hole {
                    *dense_used -= 1;
                    let value = storage.remove(non_hole).unwrap();
                    self.maybe_convert_to_sparse();
                    value
                } else {
                    storage.pop().unwrap_or(None).unwrap_or(Value::Undefined)
                }
            }
            ArrayStorage::Sparse(storage, length) => {
                if *length == 0 {
                    return Value::Undefined;
                }

                let mut non_hole = None;

                let storage_clone = storage.clone();
                if let Some((i, _item)) = storage_clone.iter().next_back() {
                    non_hole = Some(i);
                }
                if let Some(non_hole) = non_hole {
                    *length -= 1;
                    storage.remove(non_hole).unwrap()
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
            ArrayStorage::Dense(storage, dense_used) => {
                if !storage.is_empty() {
                    let value = storage.remove(0);
                    if value.is_some() {
                        *dense_used -= 1;
                    }
                    self.maybe_convert_to_sparse();
                    return value.unwrap_or(Value::Undefined);
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
            ArrayStorage::Dense(storage, dense_used) => {
                storage.insert(0, Some(item));
                *dense_used += 1;
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
    pub fn iter_mut<'a>(&'a mut self) -> ArrayStorageMutableIterator<'a, 'gc> {
        let index_back = self.length();
        ArrayStorageMutableIterator {
            storage: self,
            index: 0,
            index_back: index_back,
        }
    }

    pub fn iter<'a>(
        &'a self,
    ) -> impl DoubleEndedIterator<Item = Option<Value<'gc>>>
           + ExactSizeIterator<Item = Option<Value<'gc>>>
           + 'a {
        let index_back = self.length();
        ArrayStorageIterator {
            storage: self,
            index: 0,
            index_back: index_back,
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
            ArrayStorage::Dense(storage, dense_used) => {
                let position = if position < 0 {
                    max(position + storage.len() as i32, 0) as usize
                } else {
                    position as usize
                };

                if position >= storage.len() {
                    None
                } else {
                    let value = storage.remove(position);
                    if value.is_some() {
                        *dense_used -= 1;
                    }
                    self.maybe_convert_to_sparse();
                    value
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

    fn slice(&mut self, start: usize, end: usize) -> ArrayStorageMutableIterator<'_, 'gc> {
        let mut iter = self.iter_mut();
        iter.index = start;
        iter.index_back = end;
        iter
    }

    fn drain(&mut self, start: usize, end: usize) -> ArrayStorageDrainIterator<'_, 'gc> {
        //slice and clone
        let iter = self.slice(start, end);

        ArrayStorageDrainIterator {
            iter,
            start,
            end,
            has_to_drain: true,
        }
    }

    fn drain_without_drop(
        &mut self,
        start: usize,
        end: usize,
    ) -> ArrayStorageDrainIterator<'_, 'gc> {
        //slice and clone
        let iter = self.slice(start, end);

        ArrayStorageDrainIterator {
            iter,
            start,
            end,
            has_to_drain: false,
        }
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
        let start = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => match self {
                ArrayStorage::Dense(storage, ..) => storage.len(),
                ArrayStorage::Sparse(storage, length) => *length,
            },
        };
        let drain = self.drain_without_drop(start, end);
        let replace_with = replace_with.into_iter().map(Some);
        ArrayStorageSpliceIterator {
            drain,
            replace_with,
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
        let storage: Vec<Option<Value>> = values.into_iter().map(|v| Some(v.into())).collect();

        let dense_used = storage.iter().filter(|v| v.is_some()).count();
        let storage_type = ArrayStorage::Dense(storage, dense_used);
        storage_type
    }
}
