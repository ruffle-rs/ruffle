use gc_arena::Collect;

/// The VM (or lack thereof) that created a given object.
///
/// This primary purpose of this type is to communicate if a movie clip is
/// being created on an AVM stack, and if so, which type. If it is on-stack,
/// then it needs to be constructed immediately before user code can continue
/// running. Otherwise, its constructor should be queued to run later.
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
    #[inline]
    pub fn is_avm(self) -> bool {
        matches!(self, Self::Avm1 | Self::Avm2)
    }
}

/// Represents an error generated due to a failure to convert or claim an
/// object for a given VM.
#[derive(Copy, Clone, Debug, Collect)]
#[collect(require_static)]
pub struct ClaimError();

impl From<Instantiator> for Option<AvmType> {
    #[inline]
    fn from(instantiator: Instantiator) -> Option<AvmType> {
        match instantiator {
            Instantiator::Avm1 => Some(AvmType::Avm1),
            Instantiator::Avm2 => Some(AvmType::Avm2),
            _ => None,
        }
    }
}

/// Denotes an AVM type.
#[derive(Copy, Clone, Debug, Collect, PartialEq, Eq)]
#[collect(no_drop)]
pub enum AvmType {
    Avm1,
    Avm2,
}

impl AvmType {
    #[inline]
    pub fn into_avm2_loader_version(self) -> u32 {
        match self {
            AvmType::Avm1 => 2,
            AvmType::Avm2 => 3,
        }
    }
}
