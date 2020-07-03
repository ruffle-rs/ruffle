//! Active trait definitions

use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Multiname, QName};
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use swf::avm2::types::{Trait as AbcTrait, TraitKind as AbcTraitKind};

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
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Trait<'gc> {
    /// The name of this trait.
    name: QName,

    /// Whether or not traits in downstream classes are allowed to override
    /// this trait.
    is_final: bool,

    /// Whether or not this trait is intended to override an upstream class's
    /// trait.
    is_override: bool,

    /// The kind of trait in use.
    kind: TraitKind<'gc>,
}

/// The fields for a particular kind of trait.
///
/// The kind of a trait also determines how it's instantiated on the object.
/// See each individual variant for more information.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum TraitKind<'gc> {
    /// A data field on an object instance that can be read from and written
    /// to.
    Slot {
        slot_id: u32,
        type_name: Multiname,
        default_value: Option<Value<'gc>>,
    },

    /// A method on an object that can be called.
    Method { disp_id: u32, method: Method<'gc> },

    /// A getter property on an object that can be read.
    Getter { disp_id: u32, method: Method<'gc> },

    /// A setter property on an object that can be written.
    Setter { disp_id: u32, method: Method<'gc> },

    /// A class property on an object that can be used to construct more
    /// objects.
    Class {
        slot_id: u32,
        class: GcCell<'gc, Class<'gc>>,
    },

    /// A free function (not an instance method) that can be called.
    Function { slot_id: u32, function: Method<'gc> },

    /// A data field on an object that is always a particular value, and cannot
    /// be overridden.
    Const {
        slot_id: u32,
        type_name: Multiname,
        default_value: Option<Value<'gc>>,
    },
}

impl<'gc> Trait<'gc> {
    /// Convert an ABC trait into a loaded trait.
    pub fn from_abc_trait(
        unit: TranslationUnit<'gc>,
        abc_trait: &AbcTrait,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let name = QName::from_abc_multiname(&unit.abc(), abc_trait.name.clone())?;

        Ok(match &abc_trait.kind {
            AbcTraitKind::Slot {
                slot_id,
                type_name,
                value,
            } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Slot {
                    slot_id: *slot_id,
                    type_name: if type_name.0 == 0 {
                        Multiname::any()
                    } else {
                        Multiname::from_abc_multiname_static(&unit.abc(), type_name.clone())?
                    },
                    default_value: if let Some(dv) = value {
                        Some(abc_default_value(&unit.abc(), &dv)?)
                    } else {
                        None
                    },
                },
            },
            AbcTraitKind::Method { disp_id, method } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Method {
                    disp_id: *disp_id,
                    method: unit.load_method(method.0, mc)?,
                },
            },
            AbcTraitKind::Getter { disp_id, method } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Getter {
                    disp_id: *disp_id,
                    method: unit.load_method(method.0, mc)?,
                },
            },
            AbcTraitKind::Setter { disp_id, method } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Setter {
                    disp_id: *disp_id,
                    method: unit.load_method(method.0, mc)?,
                },
            },
            AbcTraitKind::Class { slot_id, class } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Class {
                    slot_id: *slot_id,
                    class: unit.load_class(class.0, mc)?,
                },
            },
            AbcTraitKind::Function { slot_id, function } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Function {
                    slot_id: *slot_id,
                    function: unit.load_method(function.0, mc)?,
                },
            },
            AbcTraitKind::Const {
                slot_id,
                type_name,
                value,
            } => Trait {
                name,
                is_final: abc_trait.is_final,
                is_override: abc_trait.is_override,
                kind: TraitKind::Const {
                    slot_id: *slot_id,
                    type_name: if type_name.0 == 0 {
                        Multiname::any()
                    } else {
                        Multiname::from_abc_multiname_static(&unit.abc(), type_name.clone())?
                    },
                    default_value: if let Some(dv) = value {
                        Some(abc_default_value(&unit.abc(), &dv)?)
                    } else {
                        None
                    },
                },
            },
        })
    }

    pub fn name(&self) -> &QName {
        &self.name
    }

    pub fn kind(&self) -> &TraitKind<'gc> {
        &self.kind
    }

    pub fn is_final(&self) -> bool {
        self.is_final
    }

    pub fn is_override(&self) -> bool {
        self.is_override
    }
}
