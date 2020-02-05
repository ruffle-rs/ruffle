//! AVM2 names & namespacing

use gc_arena::Collect;
use swf::avm2::types::{AbcFile, Index, Namespace as AbcNamespace};

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
    pub fn from_abc_namespace(name: &AbcNamespace, file: &AbcFile) -> Option<Self> {
        Some(match name {
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
        })
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
