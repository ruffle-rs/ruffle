//! Array support types

use crate::avm2::activation::Activation;
use crate::avm2::names::QName;
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::Collect;

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

    /// Retrieve a value from array storage by index.
    ///
    /// Array holes will be resolved on the prototype. No reference to
    /// class traits will be made.
    fn get(
        &self,
        item: usize,
        proto: Option<Object<'gc>>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        Ok(self
            .storage
            .get(item)
            .cloned()
            .unwrap_or(None)
            .map(Ok)
            .unwrap_or_else(|| {
                if let Some(mut proto) = proto {
                    proto.get_property(
                        proto,
                        &QName::dynamic_name(AvmString::new(
                            activation.context.gc_context,
                            format!("{}", item),
                        )),
                        activation,
                    )
                } else {
                    Ok(Value::Undefined)
                }
            })?)
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
}
