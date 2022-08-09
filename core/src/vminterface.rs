//! Any-AVM object references

use crate::avm1::Object as Avm1Object;
use crate::avm2::Object as Avm2Object;
use gc_arena::Collect;
use ruffle_types::vminterface::ClaimError;

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
    #[allow(dead_code)]
    pub fn is_avm1_object(&self) -> bool {
        matches!(self, Self::Avm1(_))
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
    #[allow(dead_code)]
    pub fn is_avm2_object(&self) -> bool {
        matches!(self, Self::Avm2(_))
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
