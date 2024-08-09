//! Active trait definitions

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::metadata::Metadata;
use crate::avm2::method::Method;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;
use bitflags::bitflags;
use gc_arena::Collect;
use std::ops::Deref;
use swf::avm2::types::{
    DefaultValue as AbcDefaultValue, Trait as AbcTrait, TraitKind as AbcTraitKind,
};

bitflags! {
    /// All attributes a trait can have.
    #[derive(Clone, Copy)]
    pub struct TraitAttributes: u8 {
        /// Whether or not traits in downstream classes are allowed to override
        /// this trait.
        const FINAL    = 1 << 0;

        /// Whether or not this trait is intended to override an upstream class's
        /// trait.
        const OVERRIDE = 1 << 1;
    }
}

/// Represents a trait as loaded into the VM.
///
/// A trait is an uninstantiated AVM2 property. Traits are used by objects to
/// track how to construct their properties when first accessed.
///
/// This type exists primarily to support classes with native methods. Adobe's
/// implementation of AVM2 handles native classes by having a special ABC file
/// load before all other code. We instead generate an initial heap in the same
/// manner as we do in AVM1, which means that we need to have a way to
/// dynamically originate traits that do not come from any particular ABC file.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct Trait<'gc> {
    /// The name of this trait.
    name: QName<'gc>,

    /// The attributes set on this trait.
    #[collect(require_static)]
    attributes: TraitAttributes,

    /// The kind of trait in use.
    kind: TraitKind<'gc>,

    /// Metadata on the trait, such as "[Inject]".
    metadata: Option<Box<[Metadata<'gc>]>>,
}

fn trait_attribs_from_abc_traits(abc_trait: &AbcTrait) -> TraitAttributes {
    let mut attributes = TraitAttributes::empty();
    attributes.set(TraitAttributes::FINAL, abc_trait.is_final);
    attributes.set(TraitAttributes::OVERRIDE, abc_trait.is_override);
    attributes
}

/// The fields for a particular kind of trait.
///
/// The kind of a trait also determines how it's instantiated on the object.
/// See each individual variant for more information.
#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum TraitKind<'gc> {
    /// A data field on an object instance that can be read from and written
    /// to.
    Slot {
        slot_id: u32,
        type_name: Multiname<'gc>,
        default_value: Value<'gc>,
        unit: Option<TranslationUnit<'gc>>,
    },

    /// A method on an object that can be called.
    Method { disp_id: u32, method: Method<'gc> },

    /// A getter property on an object that can be read.
    Getter { disp_id: u32, method: Method<'gc> },

    /// A setter property on an object that can be written.
    Setter { disp_id: u32, method: Method<'gc> },

    /// A class property on an object that can be used to construct more
    /// objects.
    Class { slot_id: u32, class: Class<'gc> },

    /// A free function (not an instance method) that can be called.
    Function { slot_id: u32, function: Method<'gc> },

    /// A data field on an object that is always a particular value, and cannot
    /// be overridden.
    Const {
        slot_id: u32,
        type_name: Multiname<'gc>,
        default_value: Value<'gc>,
        unit: Option<TranslationUnit<'gc>>,
    },
}

impl<'gc> Trait<'gc> {
    pub fn from_class(name: QName<'gc>, class: Class<'gc>) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Class { slot_id: 0, class },
            metadata: None,
        }
    }

    pub fn from_method(name: QName<'gc>, method: Method<'gc>) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Method { disp_id: 0, method },
            metadata: None,
        }
    }

    pub fn from_getter(name: QName<'gc>, method: Method<'gc>) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Getter { disp_id: 0, method },
            metadata: None,
        }
    }

    pub fn from_setter(name: QName<'gc>, method: Method<'gc>) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Setter { disp_id: 0, method },
            metadata: None,
        }
    }

    pub fn from_function(name: QName<'gc>, function: Method<'gc>) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Function {
                slot_id: 0,
                function,
            },
            metadata: None,
        }
    }

    pub fn from_slot(
        name: QName<'gc>,
        type_name: Multiname<'gc>,
        default_value: Option<Value<'gc>>,
    ) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Slot {
                slot_id: 0,
                default_value: default_value.unwrap_or_else(|| default_value_for_type(&type_name)),
                type_name,
                unit: None,
            },
            metadata: None,
        }
    }

    pub fn from_const(
        name: QName<'gc>,
        type_name: Multiname<'gc>,
        default_value: Option<Value<'gc>>,
    ) -> Self {
        Trait {
            name,
            attributes: TraitAttributes::empty(),
            kind: TraitKind::Const {
                slot_id: 0,
                default_value: default_value.unwrap_or_else(|| default_value_for_type(&type_name)),
                type_name,
                unit: None,
            },
            metadata: None,
        }
    }

    /// Convert an ABC trait into a loaded trait.
    pub fn from_abc_trait(
        unit: TranslationUnit<'gc>,
        abc_trait: &AbcTrait,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Self, Error<'gc>> {
        let name = QName::from_abc_multiname(unit, abc_trait.name, activation.context)?;

        Ok(match &abc_trait.kind {
            AbcTraitKind::Slot {
                slot_id,
                type_name,
                value,
            } => {
                let type_name = unit
                    .pool_multiname_static_any(*type_name, activation.context)?
                    .deref()
                    .clone();
                let default_value = slot_default_value(unit, value, &type_name, activation)?;
                Trait {
                    name,
                    attributes: trait_attribs_from_abc_traits(abc_trait),
                    kind: TraitKind::Slot {
                        slot_id: *slot_id,
                        type_name,
                        default_value,
                        unit: Some(unit),
                    },
                    metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
                }
            }
            AbcTraitKind::Method { disp_id, method } => Trait {
                name,
                attributes: trait_attribs_from_abc_traits(abc_trait),
                kind: TraitKind::Method {
                    disp_id: *disp_id,
                    method: unit.load_method(*method, false, activation)?,
                },
                metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
            },
            AbcTraitKind::Getter { disp_id, method } => Trait {
                name,
                attributes: trait_attribs_from_abc_traits(abc_trait),
                kind: TraitKind::Getter {
                    disp_id: *disp_id,
                    method: unit.load_method(*method, false, activation)?,
                },
                metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
            },
            AbcTraitKind::Setter { disp_id, method } => Trait {
                name,
                attributes: trait_attribs_from_abc_traits(abc_trait),
                kind: TraitKind::Setter {
                    disp_id: *disp_id,
                    method: unit.load_method(*method, false, activation)?,
                },
                metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
            },
            AbcTraitKind::Class { slot_id, class } => Trait {
                name,
                attributes: trait_attribs_from_abc_traits(abc_trait),
                kind: TraitKind::Class {
                    slot_id: *slot_id,
                    class: unit.load_class(class.0, activation)?,
                },
                metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
            },
            AbcTraitKind::Function { slot_id, function } => Trait {
                name,
                attributes: trait_attribs_from_abc_traits(abc_trait),
                kind: TraitKind::Function {
                    slot_id: *slot_id,
                    function: unit.load_method(*function, true, activation)?,
                },
                metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
            },
            AbcTraitKind::Const {
                slot_id,
                type_name,
                value,
            } => {
                let type_name = unit
                    .pool_multiname_static_any(*type_name, activation.context)?
                    .deref()
                    .clone();
                let default_value = slot_default_value(unit, value, &type_name, activation)?;
                Trait {
                    name,
                    attributes: trait_attribs_from_abc_traits(abc_trait),
                    kind: TraitKind::Const {
                        slot_id: *slot_id,
                        type_name,
                        default_value,
                        unit: Some(unit),
                    },
                    metadata: Metadata::from_abc_index(activation, unit, &abc_trait.metadata)?,
                }
            }
        })
    }

    pub fn name(&self) -> QName<'gc> {
        self.name
    }

    pub fn kind(&self) -> &TraitKind<'gc> {
        &self.kind
    }

    pub fn metadata(&self) -> Option<Box<[Metadata<'gc>]>> {
        self.metadata.clone()
    }

    pub fn is_final(&self) -> bool {
        self.attributes.contains(TraitAttributes::FINAL)
    }

    pub fn is_override(&self) -> bool {
        self.attributes.contains(TraitAttributes::OVERRIDE)
    }

    pub fn set_attributes(&mut self, attribs: TraitAttributes) {
        self.attributes = attribs;
    }

    /// Convenience chaining method that adds the override flag to a trait.
    pub fn with_override(mut self) -> Self {
        self.attributes |= TraitAttributes::OVERRIDE;

        self
    }

    /// Get the slot ID of this trait.
    pub fn slot_id(&self) -> Option<u32> {
        match self.kind {
            TraitKind::Slot { slot_id, .. } => Some(slot_id),
            TraitKind::Method { .. } => None,
            TraitKind::Getter { .. } => None,
            TraitKind::Setter { .. } => None,
            TraitKind::Class { slot_id, .. } => Some(slot_id),
            TraitKind::Function { slot_id, .. } => Some(slot_id),
            TraitKind::Const { slot_id, .. } => Some(slot_id),
        }
    }

    /// Set the slot ID of this trait.
    pub fn set_slot_id(&mut self, id: u32) {
        match &mut self.kind {
            TraitKind::Slot { slot_id, .. } => *slot_id = id,
            TraitKind::Method { .. } => {}
            TraitKind::Getter { .. } => {}
            TraitKind::Setter { .. } => {}
            TraitKind::Class { slot_id, .. } => *slot_id = id,
            TraitKind::Function { slot_id, .. } => *slot_id = id,
            TraitKind::Const { slot_id, .. } => *slot_id = id,
        }
    }

    /// Get the dispatch ID of this trait.
    pub fn disp_id(&self) -> Option<u32> {
        match self.kind {
            TraitKind::Slot { .. } => None,
            TraitKind::Method { disp_id, .. } => Some(disp_id),
            TraitKind::Getter { disp_id, .. } => Some(disp_id),
            TraitKind::Setter { disp_id, .. } => Some(disp_id),
            TraitKind::Class { .. } => None,
            TraitKind::Function { .. } => None,
            TraitKind::Const { .. } => None,
        }
    }

    /// Set the dispatch ID of this trait.
    pub fn set_disp_id(&mut self, id: u32) {
        match &mut self.kind {
            TraitKind::Slot { .. } => {}
            TraitKind::Method { disp_id, .. } => *disp_id = id,
            TraitKind::Getter { disp_id, .. } => *disp_id = id,
            TraitKind::Setter { disp_id, .. } => *disp_id = id,
            TraitKind::Class { .. } => {}
            TraitKind::Function { .. } => {}
            TraitKind::Const { .. } => {}
        }
    }

    /// Get the method contained within this trait, if it has one.
    pub fn as_method(&self) -> Option<Method<'gc>> {
        match &self.kind {
            TraitKind::Method { method, .. } => Some(*method),
            TraitKind::Getter { method, .. } => Some(*method),
            TraitKind::Setter { method, .. } => Some(*method),
            TraitKind::Function { function, .. } => Some(*function),
            _ => None,
        }
    }
}

/// Returns the default value for a slot/const trait.
///
/// If no default value is supplied, the "null" value for the type's trait is returned.
fn slot_default_value<'gc>(
    translation_unit: TranslationUnit<'gc>,
    value: &Option<AbcDefaultValue>,
    type_name: &Multiname<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(value) = value {
        // TODO: This should verify that the default value is compatible with the type.
        abc_default_value(translation_unit, value, activation)
    } else {
        Ok(default_value_for_type(type_name))
    }
}

/// Returns the default "null" value for the given type.
/// (`0` for ints, `null` for objects, etc.)
fn default_value_for_type<'gc>(type_name: &Multiname<'gc>) -> Value<'gc> {
    // TODO: It's technically possible to have a multiname in here, so this should go through something
    // like `Activation::resolve_type` to get an actual `Class` object, and then check something like `Class::built_in_type`.
    // The Multiname is guaranteed to be static by `pool.pool_multiname_static` earlier.
    if type_name.is_any_name() {
        Value::Undefined
    } else if type_name.contains_public_namespace() {
        let name = type_name.local_name().unwrap_or_default();
        if &name == b"Boolean" {
            false.into()
        } else if &name == b"Number" {
            f64::NAN.into()
        } else if &name == b"int" {
            0.into()
        } else if &name == b"String" {
            Value::Null
        } else if &name == b"uint" {
            0.into()
        } else {
            Value::Null // Object type
        }
    } else {
        // Object type
        Value::Null
    }
}
