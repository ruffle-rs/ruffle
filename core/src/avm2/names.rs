//! AVM2 names & namespacing

use crate::avm1::AvmString;
use crate::avm2::value::{abc_string_copy, abc_string_option};
use crate::avm2::{Avm2, Error};
use gc_arena::{Collect, MutationContext};
use swf::avm2::types::{
    AbcFile, Index, Multiname as AbcMultiname, Namespace as AbcNamespace,
    NamespaceSet as AbcNamespaceSet,
};

/// Represents the name of a namespace.
#[derive(Clone, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub enum Namespace<'gc> {
    Namespace(AvmString<'gc>),
    Package(AvmString<'gc>),
    PackageInternal(AvmString<'gc>),
    Protected(AvmString<'gc>),
    Explicit(AvmString<'gc>),
    StaticProtected(AvmString<'gc>),
    Private(AvmString<'gc>),
    Any,
}

impl<'gc> Namespace<'gc> {
    /// Read a namespace declaration from the ABC constant pool and copy it to
    /// a namespace value.
    pub fn from_abc_namespace(
        file: &AbcFile,
        namespace_index: Index<AbcNamespace>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        if namespace_index.0 == 0 {
            return Ok(Self::Any);
        }

        let actual_index = namespace_index.0 as usize - 1;
        let abc_namespace: Result<&AbcNamespace, Error> = file
            .constant_pool
            .namespaces
            .get(actual_index)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        Ok(match abc_namespace? {
            AbcNamespace::Namespace(idx) => {
                Self::Namespace(abc_string_copy(file, idx.clone(), mc)?)
            }
            AbcNamespace::Package(idx) => Self::Package(abc_string_copy(file, idx.clone(), mc)?),
            AbcNamespace::PackageInternal(idx) => {
                Self::PackageInternal(abc_string_copy(file, idx.clone(), mc)?)
            }
            AbcNamespace::Protected(idx) => {
                Self::Protected(abc_string_copy(file, idx.clone(), mc)?)
            }
            AbcNamespace::Explicit(idx) => Self::Explicit(abc_string_copy(file, idx.clone(), mc)?),
            AbcNamespace::StaticProtected(idx) => {
                Self::StaticProtected(abc_string_copy(file, idx.clone(), mc)?)
            }
            AbcNamespace::Private(idx) => Self::Private(abc_string_copy(file, idx.clone(), mc)?),
        })
    }

    pub fn public_namespace() -> Self {
        Namespace::Package("".into())
    }

    pub fn as3_namespace() -> Self {
        Namespace::Namespace("http://adobe.com/AS3/2006/builtin".into())
    }

    pub fn package(package_name: impl Into<AvmString<'gc>>) -> Self {
        Namespace::Package(package_name.into())
    }

    pub fn is_any(&self) -> bool {
        match self {
            Self::Any => true,
            _ => false,
        }
    }

    pub fn is_private(&self) -> bool {
        match self {
            Self::Private(_) => true,
            _ => false,
        }
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
pub struct QName<'gc> {
    ns: Namespace<'gc>,
    name: AvmString<'gc>,
}

impl<'gc> QName<'gc> {
    pub fn new(ns: Namespace<'gc>, name: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns,
            name: name.into(),
        }
    }

    pub fn dynamic_name(local_part: impl Into<AvmString<'gc>>) -> Self {
        Self {
            ns: Namespace::public_namespace(),
            name: local_part.into(),
        }
    }

    /// Pull a `QName` from the multiname pool.
    ///
    /// This function returns an Err if the multiname does not exist or is not
    /// a `QName`.
    pub fn from_abc_multiname(
        file: &AbcFile,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index = multiname_index.0 as usize - 1;
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } => Self {
                ns: Namespace::from_abc_namespace(file, namespace.clone(), mc)?,
                name: abc_string_copy(file, name.clone(), mc)?,
            },
            _ => return Err("Attempted to pull QName from non-QName multiname".into()),
        })
    }

    pub fn local_name(&self) -> AvmString<'gc> {
        self.name
    }

    pub fn namespace(&self) -> &Namespace<'gc> {
        &self.ns
    }
}

/// A `Multiname` consists of a name which could be resolved in one or more
/// potential namespaces.
///
/// All unresolved names are of the form `Multiname`, and the name resolution
/// process consists of searching each name space for a given name.
///
/// The existence of a `name` of `None` indicates the `Any` name.
#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct Multiname<'gc> {
    ns: Vec<Namespace<'gc>>,
    name: Option<AvmString<'gc>>,
}

impl<'gc> Multiname<'gc> {
    /// Read a namespace set from the ABC constant pool, and return a list of
    /// copied namespaces.
    fn abc_namespace_set(
        file: &AbcFile,
        namespace_set_index: Index<AbcNamespaceSet>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Vec<Namespace<'gc>>, Error> {
        if namespace_set_index.0 == 0 {
            //TODO: What is namespace set zero?
            return Ok(vec![]);
        }

        let actual_index = namespace_set_index.0 as usize - 1;
        let ns_set: Result<&AbcNamespaceSet, Error> = file
            .constant_pool
            .namespace_sets
            .get(actual_index)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let mut result = vec![];

        for ns in ns_set? {
            result.push(Namespace::from_abc_namespace(file, ns.clone(), mc)?)
        }

        Ok(result)
    }

    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    pub fn from_abc_multiname(
        file: &AbcFile,
        multiname_index: Index<AbcMultiname>,
        avm: &mut Avm2<'gc>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index: Result<usize, Error> = (multiname_index.0 as usize)
            .checked_sub(1)
            .ok_or_else(|| "Attempted to resolve a multiname at index zero. This is a bug.".into());
        let actual_index = actual_index?;
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(file, namespace.clone(), mc)?],
                    name: abc_string_option(file, name.clone(), mc)?,
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => {
                let ns = avm.pop().as_namespace()?.clone();
                Self {
                    ns: vec![ns],
                    name: abc_string_option(file, name.clone(), mc)?,
                }
            }
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => {
                let ns = avm.pop().as_namespace()?.clone();
                let name = avm.pop().as_string()?.clone();
                Self {
                    ns: vec![ns],
                    name: Some(name),
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
                ns: Self::abc_namespace_set(file, namespace_set.clone(), mc)?,
                name: abc_string_option(file, name.clone(), mc)?,
            },
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => {
                let name = avm.pop().as_string()?.clone();
                Self {
                    ns: Self::abc_namespace_set(file, namespace_set.clone(), mc)?,
                    name: Some(name),
                }
            }
        })
    }

    /// Read a static multiname from the ABC constant pool
    ///
    /// This function prohibits the use of runtime-qualified and late-bound
    /// names. Runtime multinames will instead result in an error.
    pub fn from_abc_multiname_static(
        file: &AbcFile,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index: Result<usize, Error> =
            (multiname_index.0 as usize).checked_sub(1).ok_or_else(|| {
                "Attempted to resolve a (static) multiname at index zero. This is a bug.".into()
            });
        let actual_index = actual_index?;
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(file, namespace.clone(), mc)?],
                    name: abc_string_option(file, name.clone(), mc)?,
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
                ns: Self::abc_namespace_set(file, namespace_set.clone(), mc)?,
                name: abc_string_option(file, name.clone(), mc)?,
            },
            _ => return Err(format!("Multiname {} is not static", multiname_index.0).into()),
        })
    }

    /// Indicates the any type (any name in any namespace).
    pub fn any() -> Self {
        Self {
            ns: vec![Namespace::Any],
            name: None,
        }
    }

    pub fn namespace_set(&self) -> impl Iterator<Item = &Namespace<'gc>> {
        self.ns.iter()
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.name
    }
}
