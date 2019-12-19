//! User-defined properties

use self::Attribute::*;
use crate::avm1::function::Executable;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, UpdateContext, Value};
use core::fmt;
use enumset::{EnumSet, EnumSetType};
use std::mem::replace;

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
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Property::Virtual { get, .. } => get.exec(avm, context, this, &[]),
            Property::Stored { value, .. } => Ok(value.to_owned().into()),
        }
    }

    /// Set a property slot.
    ///
    /// This function returns `true` if the set has completed, or `false` if
    /// it has not yet occured. If `false`, and you need to run code after the
    /// set has occured, you must recursively execute the top-most frame via
    /// `run_current_frame`.
    pub fn set(
        &mut self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        new_value: impl Into<Value<'gc>>,
    ) -> Result<bool, Error> {
        match self {
            Property::Virtual { set, .. } => {
                if let Some(function) = set {
                    let return_value = function.exec(avm, context, this, &[new_value.into()])?;
                    Ok(return_value.is_immediate())
                } else {
                    Ok(true)
                }
            }
            Property::Stored {
                value, attributes, ..
            } => {
                if !attributes.contains(ReadOnly) {
                    replace::<Value<'gc>>(value, new_value.into());
                }

                Ok(true)
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
