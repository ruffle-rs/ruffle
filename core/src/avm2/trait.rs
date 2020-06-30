//! Active trait definitions

use crate::avm2::class::Class;
use crate::avm2::function::Method;
use crate::avm2::names::{Multiname, QName};
use crate::avm2::value::Value;
use gc_arena::{Collect, Gc};

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
        class: Gc<'gc, Class<'gc>>,
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
