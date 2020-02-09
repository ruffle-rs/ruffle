//! AVM2 names & namespacing

use crate::avm2::value::abc_string;
use crate::avm2::{Avm2, Error};
use gc_arena::Collect;
use swf::avm2::types::{
    AbcFile, Index, Multiname as AbcMultiname, Namespace as AbcNamespace,
    NamespaceSet as AbcNamespaceSet,
};

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
    ) -> Result<Self, Error> {
        let abc_namespace: Result<&AbcNamespace, Error> = file
            .constant_pool
            .namespaces
            .get(namespace_index.0 as usize)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        Ok(match abc_namespace? {
            AbcNamespace::Namespace(idx) => Self::Namespace(abc_string(file, idx.clone())?),
            AbcNamespace::Package(idx) => Self::Package(abc_string(file, idx.clone())?),
            AbcNamespace::PackageInternal(idx) => {
                Self::PackageInternal(abc_string(file, idx.clone())?)
            }
            AbcNamespace::Protected(idx) => Self::Protected(abc_string(file, idx.clone())?),
            AbcNamespace::Explicit(idx) => Self::Explicit(abc_string(file, idx.clone())?),
            AbcNamespace::StaticProtected(idx) => {
                Self::StaticProtected(abc_string(file, idx.clone())?)
            }
            AbcNamespace::Private(idx) => Self::Private(abc_string(file, idx.clone())?),
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
    /// Read a namespace set from the ABC constant pool, and return a list of
    /// copied namespaces.
    fn abc_namespace_set(
        file: &AbcFile,
        namespace_set_index: Index<AbcNamespaceSet>,
    ) -> Result<Vec<Namespace>, Error> {
        let ns_set: Result<&AbcNamespaceSet, Error> = file
            .constant_pool
            .namespace_sets
            .get(namespace_set_index.0 as usize)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let mut result = vec![];

        for ns in ns_set? {
            result.push(Namespace::from_abc_namespace(file, ns.clone())?)
        }

        Ok(result)
    }

    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    pub fn from_abc_multiname(
        file: &AbcFile,
        multiname_index: Index<Multiname>,
        avm: &mut Avm2<'_>,
    ) -> Result<Self, Error> {
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(multiname_index.0 as usize)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(file, namespace.clone())?],
                    name: abc_string(file, name.clone())?,
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => {
                let ns = avm.pop().as_namespace()?.clone();
                Self {
                    ns: vec![ns],
                    name: abc_string(file, name.clone())?,
                }
            }
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => {
                let ns = avm.pop().as_namespace()?.clone();
                let name = avm.pop().as_string()?.clone();
                Self {
                    ns: vec![ns],
                    name: name,
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(file, namespace_set.clone())?,
                name: abc_string(file, name.clone())?,
            },
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => {
                let name = avm.pop().as_string()?.clone();
                Self {
                    ns: Self::abc_namespace_set(file, namespace_set.clone())?,
                    name,
                }
            }
        })
    }
}
