//! Property data structures

use self::Attribute::*;
use crate::avm2::function::Executable;
use crate::avm2::object::{Object, TObject};
use crate::avm2::return_value::ReturnValue;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use enumset::{EnumSet, EnumSetType};
use gc_arena::{Collect, CollectionContext};

/// Attributes of properties in the AVM runtime.
///
/// TODO: Replace with AVM2 properties for traits
#[derive(EnumSetType, Debug)]
pub enum Attribute {
    DontEnum,
    DontDelete,
    ReadOnly,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Property<'gc> {
    Virtual {
        get: Option<Executable<'gc>>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    },
    Stored {
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    },
    Slot {
        slot_id: u32,
        attributes: EnumSet<Attribute>,
    },
}

unsafe impl<'gc> Collect for Property<'gc> {
    fn trace(&self, cc: CollectionContext) {
        match self {
            Property::Virtual { get, set, .. } => {
                get.trace(cc);
                set.trace(cc);
            }
            Property::Stored { value, .. } => value.trace(cc),
            Property::Slot { .. } => {}
        }
    }
}

impl<'gc> Property<'gc> {
    /// Create a new stored property.
    pub fn new_stored(value: impl Into<Value<'gc>>) -> Self {
        Property::Stored {
            value: value.into(),
            attributes: EnumSet::from(Attribute::DontDelete),
        }
    }

    /// Create a new stored property.
    pub fn new_const(value: impl Into<Value<'gc>>) -> Self {
        Property::Stored {
            value: value.into(),
            attributes: Attribute::ReadOnly | Attribute::DontDelete,
        }
    }

    /// Convert a value into a dynamic property.
    pub fn new_dynamic_property(value: impl Into<Value<'gc>>) -> Self {
        Property::Stored {
            value: value.into(),
            attributes: EnumSet::empty(),
        }
    }

    /// Convert a function into a method.
    ///
    /// This applies ReadOnly/DontDelete to the property.
    pub fn new_method(fn_obj: Object<'gc>) -> Self {
        Property::Stored {
            value: fn_obj.into(),
            attributes: Attribute::ReadOnly | Attribute::DontDelete,
        }
    }

    /// Create a new, unconfigured virtual property item.
    pub fn new_virtual() -> Self {
        Property::Virtual {
            get: None,
            set: None,
            attributes: Attribute::ReadOnly | Attribute::DontDelete,
        }
    }

    /// Create a new slot property.
    pub fn new_slot(slot_id: u32) -> Self {
        Property::Slot {
            slot_id,
            attributes: EnumSet::from(Attribute::DontDelete),
        }
    }

    /// Install a getter into this property.
    ///
    /// This function errors if attempting to install executables into a
    /// non-virtual property.
    pub fn install_virtual_getter(&mut self, getter_impl: Executable<'gc>) -> Result<(), Error> {
        match self {
            Property::Virtual { get, .. } => *get = Some(getter_impl),
            Property::Stored { .. } => return Err("Not a virtual property".into()),
            Property::Slot { .. } => return Err("Not a virtual property".into()),
        };

        Ok(())
    }

    /// Install a setter into this property.
    ///
    /// This function errors if attempting to install executables into a
    /// non-virtual property.
    pub fn install_virtual_setter(&mut self, setter_impl: Executable<'gc>) -> Result<(), Error> {
        match self {
            Property::Virtual { set, .. } => *set = Some(setter_impl),
            Property::Stored { .. } => return Err("Not a virtual property".into()),
            Property::Slot { .. } => return Err("Not a virtual property".into()),
        };

        Ok(())
    }

    /// Get the value of a property slot.
    ///
    /// This function yields `ReturnValue` because some properties may be
    /// user-defined.
    pub fn get(
        &self,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        match self {
            Property::Virtual { get: Some(get), .. } => get.exec(avm, context, this, &[]),
            Property::Virtual { get: None, .. } => Ok(Value::Undefined.into()),
            Property::Stored { value, .. } => Ok(value.to_owned().into()),
            Property::Slot { slot_id, .. } => this.get_slot(*slot_id).map(|v| v.into()),
        }
    }

    /// Set a property slot.
    ///
    /// This function returns `true` if the set has completed, or `false` if
    /// it has not yet occured. If `false`, and you need to run code after the
    /// set has occured, you must recursively execute the top-most frame via
    /// `run_current_frame`.
    ///
    /// This function cannot set slot properties and will panic if one
    /// is encountered.
    pub fn set(
        &mut self,
        avm: &mut Avm2<'gc>,
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
                    *value = new_value.into();
                }

                Ok(true)
            }
            Property::Slot { .. } => panic!("Cannot recursively set slots"),
        }
    }

    /// Init a property slot.
    ///
    /// The difference between `set` and `init` is that this function does not
    /// respect `ReadOnly` and will allow initializing nominally `const`
    /// properties, at least once. Virtual properties with no setter cannot be
    /// initialized.
    ///
    /// This function returns `true` if the set has completed, or `false` if
    /// it has not yet occured. If `false`, and you need to run code after the
    /// set has occured, you must recursively execute the top-most frame via
    /// `run_current_frame`.
    ///
    /// This function cannot initialize slot properties and will panic if one
    /// is encountered.
    pub fn init(
        &mut self,
        avm: &mut Avm2<'gc>,
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
            Property::Stored { value, .. } => {
                *value = new_value.into();

                Ok(true)
            }
            Property::Slot { .. } => panic!("Cannot recursively init slots"),
        }
    }

    /// Retrieve the slot ID of a property.
    ///
    /// This function yields `None` if this property is not a slot.
    pub fn slot_id(&self) -> Option<u32> {
        match self {
            Property::Slot { slot_id, .. } => Some(*slot_id),
            _ => None,
        }
    }

    /// List this property's attributes.
    pub fn attributes(&self) -> EnumSet<Attribute> {
        match self {
            Property::Virtual { attributes, .. } => *attributes,
            Property::Stored { attributes, .. } => *attributes,
            Property::Slot { attributes, .. } => *attributes,
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
            Property::Slot {
                ref mut attributes, ..
            } => *attributes = new_attributes,
        }
    }

    pub fn can_delete(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(DontDelete),
            Property::Stored { attributes, .. } => !attributes.contains(DontDelete),
            Property::Slot { attributes, .. } => !attributes.contains(DontDelete),
        }
    }

    pub fn is_enumerable(&self) -> bool {
        match self {
            Property::Virtual { attributes, .. } => !attributes.contains(DontEnum),
            Property::Stored { attributes, .. } => !attributes.contains(DontEnum),
            Property::Slot { attributes, .. } => !attributes.contains(DontEnum),
        }
    }

    pub fn is_overwritable(&self) -> bool {
        match self {
            Property::Virtual {
                attributes, set, ..
            } => !attributes.contains(ReadOnly) && !set.is_none(),
            Property::Stored { attributes, .. } => !attributes.contains(ReadOnly),
            Property::Slot { attributes, .. } => !attributes.contains(ReadOnly),
        }
    }
}
