//! User-defined properties

use crate::avm1::{Object, Value};
use bitflags::bitflags;
use core::fmt;
use gc_arena::Collect;

bitflags! {
    /// Attributes of properties in the AVM runtime.
    /// The values are significant and should match the order used by `object::as_set_prop_flags`.
    #[derive(Collect)]
    #[collect(require_static)]
    pub struct Attribute: u16 {
        const DONT_ENUM     = 1 << 0;
        const DONT_DELETE   = 1 << 1;
        const READ_ONLY     = 1 << 2;
        const VERSION_MASK  = 0xf << 9;
    }
}

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Property<'gc> {
    data: Value<'gc>,
    getter: Option<Object<'gc>>,
    setter: Option<Object<'gc>>,
    attributes: Attribute,
}

impl<'gc> Property<'gc> {
    pub fn new_stored(data: Value<'gc>, attributes: Attribute) -> Self {
        Self {
            data,
            getter: None,
            setter: None,
            attributes,
        }
    }

    pub fn new_virtual(
        getter: Object<'gc>,
        setter: Option<Object<'gc>>,
        attributes: Attribute,
    ) -> Self {
        Self {
            data: Value::Undefined,
            getter: Some(getter),
            setter,
            attributes,
        }
    }

    pub fn data(&self) -> Value<'gc> {
        self.data
    }

    pub fn getter(&self) -> Option<Object<'gc>> {
        self.getter
    }

    pub fn setter(&self) -> Option<Object<'gc>> {
        self.setter
    }

    /// Store data on this property, ignoring virtual setters.
    ///
    /// Read-only properties are not affected.
    pub fn set_data(&mut self, data: Value<'gc>) {
        if self.is_overwritable() {
            self.data = data;
        }
    }

    /// Make this property virtual by attaching a getter/setter to it.
    pub fn set_virtual(&mut self, getter: Object<'gc>, setter: Option<Object<'gc>>) {
        self.getter = Some(getter);
        self.setter = setter;
    }

    /// List this property's attributes.
    pub fn attributes(&self) -> Attribute {
        self.attributes
    }

    /// Re-define this property's attributes.
    pub fn set_attributes(&mut self, attributes: Attribute) {
        self.attributes = attributes;
    }

    pub fn is_enumerable(&self) -> bool {
        !self.attributes.contains(Attribute::DONT_ENUM)
    }

    pub fn can_delete(&self) -> bool {
        !self.attributes.contains(Attribute::DONT_DELETE)
    }

    pub fn is_overwritable(&self) -> bool {
        !self.attributes.contains(Attribute::READ_ONLY)
    }

    pub fn is_virtual(&self) -> bool {
        self.getter.is_some()
    }
}

impl fmt::Debug for Property<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Property")
            .field("data", &self.data)
            .field("getter", &self.getter)
            .field("setter", &self.setter)
            .field("attributes", &self.attributes)
            .finish()
    }
}
