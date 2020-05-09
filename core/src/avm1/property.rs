//! User-defined properties

use self::Attribute::*;
use crate::avm1::function::Executable;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, UpdateContext, Value};
use core::fmt;
use enumset::{EnumSet, EnumSetType};

/// Attributes of properties in the AVM runtime.
/// The order is significant and should match the order used by `object::as_set_prop_flags`.
#[derive(EnumSetType, Debug)]
pub enum Attribute {
    DontEnum,
    DontDelete,
    ReadOnly,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum Property<'gc> {
    Virtual {
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    },
    Stored {
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    },
}

impl<'gc> Property<'gc> {
    /// Get the value of a property slot.
    ///
    /// This function yields `ReturnValue` because some properties may be
    /// user-defined.
    pub fn get(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Property::Virtual { get, .. } => get.exec(avm, context, this, base_proto, &[]),
            Property::Stored { value, .. } => Ok(value.to_owned().into()),
        }
    }

    /// Set a property slot.
    ///
    /// This function returns the `ReturnValue` of the property's virtual
    /// function, if any happen to exist. It should be resolved, and it's value
    /// discarded.
    pub fn set(
        &mut self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        base_proto: Option<Object<'gc>>,
        new_value: impl Into<Value<'gc>>,
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Property::Virtual { set, .. } => {
                if let Some(function) = set {
                    function.exec(avm, context, this, base_proto, &[new_value.into()])
                } else {
                    Ok(Value::Undefined.into())
                }
            }
            Property::Stored {
                value, attributes, ..
            } => {
                if !attributes.contains(ReadOnly) {
                    *value = new_value.into();
                }

                Ok(Value::Undefined.into())
            }
        }
    }

    /// List this property's attributes.
    pub fn attributes(&self) -> EnumSet<Attribute> {
        match self {
            Property::Virtual { attributes, .. } => *attributes,
            Property::Stored { attributes, .. } => *attributes,
        }
    }

    /// Re-define this property's attributes.
    pub fn set_attributes(&mut self, new_attributes: EnumSet<Attribute>) {
        match self {
            Property::Virtual {
                ref mut attributes, ..
            } => *attributes = new_attributes,
            Property::Stored {
                ref mut attributes, ..
            } => *attributes = new_attributes,
        }
    }

    pub fn can_delete(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(DontDelete),
            Property::Stored { attributes, .. } => !attributes.contains(DontDelete),
        }
    }

    pub fn is_enumerable(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(DontEnum),
            Property::Stored { attributes, .. } => !attributes.contains(DontEnum),
        }
    }

    pub fn is_overwritable(&self) -> bool {
        match self {
            Property::Virtual {
                attributes, set, ..
            } => !attributes.contains(ReadOnly) && !set.is_none(),
            Property::Stored { attributes, .. } => !attributes.contains(ReadOnly),
        }
    }

    pub fn is_virtual(&self) -> bool {
        match self {
            Property::Virtual { .. } => true,
            Property::Stored { .. } => false,
        }
    }
}

unsafe impl<'gc> gc_arena::Collect for Property<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        match self {
            Property::Virtual { get, set, .. } => {
                get.trace(cc);
                set.trace(cc);
            }
            Property::Stored { value, .. } => value.trace(cc),
        }
    }
}

impl fmt::Debug for Property<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Property::Virtual {
                get: _,
                set,
                attributes,
            } => f
                .debug_struct("Property::Virtual")
                .field("get", &true)
                .field("set", &set.is_some())
                .field("attributes", &attributes)
                .finish(),
            Property::Stored { value, attributes } => f
                .debug_struct("Property::Stored")
                .field("value", &value)
                .field("attributes", &attributes)
                .finish(),
        }
    }
}
