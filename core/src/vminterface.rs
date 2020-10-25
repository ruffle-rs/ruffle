//! Any-AVM object references

use crate::avm1::Object as Avm1Object;
use crate::avm2::Object as Avm2Object;
use gc_arena::Collect;

/// The VM (or lack thereof) that created a given object.
///
/// This primary purpose of this type is to communicate if a movie clip is
/// being created on an AVM stack, and if so, which type. If it is on-stack,
/// then it needs to be constructed immediately before user code can continue
/// running. Otherwise, it's constructor should be queued to run later.
///
/// A secondary purpose of this type is to flag which VM is creating an object,
/// which can be used to ensure the object is instantiated as tied to the
/// correct VM.
#[derive(Copy, Clone, Debug, Collect)]
#[collect(require_static)]
pub enum Instantiator {
    /// This object was instantiated by a tag in a given SWF movie, or by a VM
    /// action which does not implicitly instantiate a given object. Acceptable
    /// situations in which one might flag a VM action as movie-triggered
    /// includes:
    ///
    ///  * When the object instantiation is a side effect of the command (e.g.
    ///    `ActionGoto` needs to recreate movie clips that fell off the
    ///    timeline)
    ///  * When the VM is creating objects that it does not expect to have a
    ///    custom constructor for. (e.g. top-level movie layers that it needs
    ///    to load into)
    ///
    /// TODO: This should actually link the movie so we can inspect it for a
    /// file attributes tag.
    Movie,

    /// This object was instantiated by AVM1 code constructing the object. All
    /// objects should be created as AVM1 objects and any custom constructors
    /// should resolve on-stack.
    Avm1,

    /// This object was instantiated by AVM2 code constructing the object. All
    /// objects should be created as AVM1 objects and any custom constructors
    /// should resolve on-stack.
    Avm2,
}

impl Instantiator {
    /// Returns true if the instantiation happened on an AVM stack (either kind).
    ///
    /// If that is the case, then any constructor calls necessary to finish the
    /// object must happen on-stack.
    pub fn is_avm(self) -> bool {
        match self {
            Self::Avm1 => true,
            Self::Avm2 => true,
            _ => false,
        }
    }
}

impl Into<Option<AvmType>> for Instantiator {
    fn into(self) -> Option<AvmType> {
        match self {
            Self::Avm1 => Some(AvmType::Avm1),
            Self::Avm2 => Some(AvmType::Avm2),
            _ => None,
        }
    }
}

/// Represents an error generated due to a failure to convert or claim an
/// object for a given VM.
#[derive(Copy, Clone, Debug, Collect)]
#[collect(require_static)]
pub struct ClaimError();

/// Denotes an AVM type.
#[derive(Copy, Clone, Debug, Collect, PartialEq, Eq)]
#[collect(no_drop)]
pub enum AvmType {
    Avm1,
    Avm2,
}

impl From<AvmObject<'_>> for AvmType {
    fn from(vmo: AvmObject<'_>) -> Self {
        match vmo {
            AvmObject::Avm1(_) => AvmType::Avm1,
            AvmObject::Avm2(_) => AvmType::Avm2,
        }
    }
}

/// A reference to either an AVM1 or AVM2 object.
///
/// Used by non-AVM code to retain VM objects that may have been customized or
/// altered by user code. Non-AVM structures may be held by either VM, and thus
/// those structures must also hold the VM side of themselves as well.
///
/// This structure is specifically designed to only store one VM's
/// representation of the object. Objects cannot be shared across multiple VMs
/// and attempting to do so will generate a runtime error. Dual-representation
/// objects are prohibited.
#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum AvmObject<'gc> {
    /// An object that is exclusively represented as an AVM1 object. Attempts
    /// to access it from AVM2 will fail.
    Avm1(Avm1Object<'gc>),

    /// An object that is exclusively represented as an AVM2 object. Attempts
    /// to access it from AVM1 will fail.
    Avm2(Avm2Object<'gc>),
}

impl<'gc> AvmObject<'gc> {
    /// Determine if this object is an AVM1 object.
    pub fn is_avm1_object(&self) -> bool {
        match self {
            Self::Avm1(_) => true,
            Self::Avm2(_) => false,
        }
    }

    /// Attempt to access the AVM1 claim to this object, generating an error if
    /// the object cannot be accessed by the VM.
    pub fn as_avm1_object(&self) -> Result<Avm1Object<'gc>, ClaimError> {
        match self {
            Self::Avm1(o) => Ok(*o),
            Self::Avm2(_) => Err(ClaimError()),
        }
    }

    /// Determine if this object is an AVM2 object.
    pub fn is_avm2_object(&self) -> bool {
        match self {
            Self::Avm1(_) => false,
            Self::Avm2(_) => true,
        }
    }

    /// Attempt to access the AVM2 claim to this object, generating an error if
    /// the object cannot be accessed by the VM.
    pub fn as_avm2_object(&self) -> Result<Avm2Object<'gc>, ClaimError> {
        match self {
            Self::Avm1(_) => Err(ClaimError()),
            Self::Avm2(o) => Ok(*o),
        }
    }
}

impl<'gc> From<Avm1Object<'gc>> for AvmObject<'gc> {
    fn from(t: Avm1Object<'gc>) -> Self {
        Self::Avm1(t)
    }
}

impl<'gc> From<Avm2Object<'gc>> for AvmObject<'gc> {
    fn from(t: Avm2Object<'gc>) -> Self {
        Self::Avm2(t)
    }
}
