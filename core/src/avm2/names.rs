//! AVM2 names & namespacing

use crate::avm2::Avm2;
use gc_arena::Collect;
use swf::avm2::types::{AbcFile, Index, Multiname as AbcMultiname, Namespace as AbcNamespace};

/// Represents the name of a namespace.
#[derive(Clone, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub enum Namespace {
    Namespace(String),
    Package(String),
    PackageInternal(String),
    Protected(String),
    Explicit(String),
    StaticProtected(String),
    Private(String),
}

impl Namespace {
    /// Read a namespace declaration from the ABC constant pool and copy it to
    /// a namespace value.
    pub fn from_abc_namespace(
        file: &AbcFile,
        namespace_index: Index<AbcNamespace>,
    ) -> Option<Self> {
        Some(
            match file
                .constant_pool
                .namespaces
                .get(namespace_index.0 as usize)?
            {
                AbcNamespace::Namespace(Index(idx, ..)) => {
                    Self::Namespace(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
                AbcNamespace::Package(Index(idx, ..)) => {
                    Self::Package(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
                AbcNamespace::PackageInternal(Index(idx, ..)) => {
                    Self::PackageInternal(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
                AbcNamespace::Protected(Index(idx, ..)) => {
                    Self::Protected(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
                AbcNamespace::Explicit(Index(idx, ..)) => {
                    Self::Explicit(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
                AbcNamespace::StaticProtected(Index(idx, ..)) => {
                    Self::StaticProtected(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
                AbcNamespace::Private(Index(idx, ..)) => {
                    Self::Private(file.constant_pool.strings.get(*idx as usize)?.clone())
                }
            },
        )
    }
}

/// A `QName`, likely "qualified name", consists of a namespace and name string.
///
/// This is technically interchangeable with `xml::XMLName`, as they both
/// implement `QName`; however, AVM2 and XML have separate representations.
///
/// A property cannot be retrieved or set without first being resolved into a
/// `QName`. All other forms of names and multinames are either versions of
/// `QName` with unspecified parameters, or multiple names to be checked in
/// order.
#[derive(Clone, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub struct QName {
    ns: Namespace,
    name: String,
}

/// A `Multiname` consists of a name which could be resolved in one or more
/// potential namespaces.
///
/// All unresolved names are of the form `Multiname`, and the name resolution
/// process consists of searching each name space for a given name.
pub struct Multiname {
    ns: Vec<Namespace>,
    name: String,
}

impl Multiname {
    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    ///
    /// This does not yet support late-bound or runtime multinames.
    pub fn from_abc_multiname(
        file: &AbcFile,
        multiname_index: Index<Multiname>,
        avm: &mut Avm2<'_>,
    ) -> Option<Self> {
        Some(
            match file
                .constant_pool
                .multinames
                .get(multiname_index.0 as usize)?
            {
                AbcMultiname::QName { namespace, name }
                | AbcMultiname::QNameA { namespace, name } => Self {
                    ns: vec![Namespace::from_abc_namespace(file, namespace.clone())?],
                    name: file.constant_pool.strings.get(name.0 as usize)?.clone(),
                },
                AbcMultiname::Multiname {
                    namespace_set,
                    name,
                }
                | AbcMultiname::MultinameA {
                    namespace_set,
                    name,
                } => Self {
                    ns: file
                        .constant_pool
                        .namespace_sets
                        .get(namespace_set.0 as usize)?
                        .iter()
                        .map(|ns| Namespace::from_abc_namespace(file, ns.clone()))
                        .filter(|ns| ns.is_some())
                        .map(|ns| ns.unwrap())
                        .collect(),
                    name: file.constant_pool.strings.get(name.0 as usize)?.clone(),
                },
                _ => return None,
            },
        )
    }
}
