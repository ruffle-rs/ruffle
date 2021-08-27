//! AVM2 names & namespacing

use crate::avm2::activation::Activation;
use crate::avm2::script::TranslationUnit;
use crate::avm2::string::AvmString;
use crate::avm2::Error;
use gc_arena::{Collect, MutationContext};
use std::fmt::Write;
use swf::avm2::types::{
    Index, Multiname as AbcMultiname, Namespace as AbcNamespace, NamespaceSet as AbcNamespaceSet,
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
        translation_unit: TranslationUnit<'gc>,
        namespace_index: Index<AbcNamespace>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        if namespace_index.0 == 0 {
            return Ok(Self::Any);
        }

        let actual_index = namespace_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_namespace: Result<_, Error> = abc
            .constant_pool
            .namespaces
            .get(actual_index)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        Ok(match abc_namespace? {
            AbcNamespace::Namespace(idx) => {
                Self::Namespace(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Package(idx) => Self::Package(translation_unit.pool_string(idx.0, mc)?),
            AbcNamespace::PackageInternal(idx) => {
                Self::PackageInternal(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Protected(idx) => {
                Self::Protected(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Explicit(idx) => Self::Explicit(translation_unit.pool_string(idx.0, mc)?),
            AbcNamespace::StaticProtected(idx) => {
                Self::StaticProtected(translation_unit.pool_string(idx.0, mc)?)
            }
            AbcNamespace::Private(idx) => Self::Private(translation_unit.pool_string(idx.0, mc)?),
        })
    }

    pub fn public() -> Self {
        Self::Package("".into())
    }

    pub fn as3_namespace() -> Self {
        Self::Namespace("http://adobe.com/AS3/2006/builtin".into())
    }

    pub fn package(package_name: impl Into<AvmString<'gc>>) -> Self {
        Self::Package(package_name.into())
    }

    pub fn private(name: impl Into<AvmString<'gc>>) -> Self {
        Self::Private(name.into())
    }

    pub fn is_public(&self) -> bool {
        *self == Self::public()
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }

    pub fn is_dynamic(&self) -> bool {
        self.is_public() || self.is_any()
    }

    /// Get the string value of this namespace, ignoring its type.
    ///
    /// TODO: Is this *actually* the namespace URI?
    pub fn as_uri(&self) -> AvmString<'gc> {
        match self {
            Self::Namespace(s) => *s,
            Self::Package(s) => *s,
            Self::PackageInternal(s) => *s,
            Self::Protected(s) => *s,
            Self::Explicit(s) => *s,
            Self::StaticProtected(s) => *s,
            Self::Private(s) => *s,
            Self::Any => "".into(),
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
            ns: Namespace::public(),
            name: local_part.into(),
        }
    }

    /// Pull a `QName` from the multiname pool.
    ///
    /// This function returns an Err if the multiname does not exist or is not
    /// a `QName`.
    pub fn from_abc_multiname(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        if multiname_index.0 == 0 {
            return Err("Attempted to load a trait name of index zero".into());
        }

        let actual_index = multiname_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } => Self {
                ns: Namespace::from_abc_namespace(translation_unit, namespace.clone(), mc)?,
                name: translation_unit.pool_string(name.0, mc)?,
            },
            _ => return Err("Attempted to pull QName from non-QName multiname".into()),
        })
    }

    /// Constructs a `QName` from a fully qualified name.
    ///
    /// A fully qualified name can be any of the following formats:
    /// NAMESPACE::LOCAL_NAME
    /// NAMESPACE.LOCAL_NAME (Where the LAST dot is used to split the namespace & local_name)
    /// LOCAL_NAME (Use the public namespace)
    pub fn from_qualified_name(name: &str, mc: MutationContext<'gc, '_>) -> Self {
        if let Some((package_name, local_name)) = name.split_once("::") {
            Self {
                ns: Namespace::Package(AvmString::new(mc, package_name.to_string())),
                name: AvmString::new(mc, local_name.to_string()),
            }
        } else if let Some((package_name, local_name)) = name.rsplit_once('.') {
            Self {
                ns: Namespace::Package(AvmString::new(mc, package_name.to_string())),
                name: AvmString::new(mc, local_name.to_string()),
            }
        } else {
            Self {
                ns: Namespace::public(),
                name: AvmString::new(mc, name.to_string()),
            }
        }
    }

    /// Converts this `QName` to a fully qualified name.
    pub fn to_qualified_name(&self) -> String {
        let mut result = String::new();
        let uri = self.namespace().as_uri();
        if !uri.is_empty() {
            write!(result, "{}::", uri).expect("Write failed");
        }
        result.push_str(&self.local_name());
        result
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
        translation_unit: TranslationUnit<'gc>,
        namespace_set_index: Index<AbcNamespaceSet>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Vec<Namespace<'gc>>, Error> {
        if namespace_set_index.0 == 0 {
            //TODO: What is namespace set zero?
            return Ok(vec![]);
        }

        let actual_index = namespace_set_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let ns_set: Result<_, Error> = abc
            .constant_pool
            .namespace_sets
            .get(actual_index)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let mut result = vec![];

        for ns in ns_set? {
            result.push(Namespace::from_abc_namespace(
                translation_unit,
                ns.clone(),
                mc,
            )?)
        }

        Ok(result)
    }

    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    ///
    /// Multiname index zero is also treated as an error, you must check for it
    /// and substitute it with whatever default is called for by AVM2.
    pub fn from_abc_multiname(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index: Result<usize, Error> = (multiname_index.0 as usize)
            .checked_sub(1)
            .ok_or_else(|| "Attempted to resolve a multiname at index zero. This is a bug.".into());
        let actual_index = actual_index?;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(
                        translation_unit,
                        namespace.clone(),
                        activation.context.gc_context,
                    )?],
                    name: translation_unit
                        .pool_string_option(name.0, activation.context.gc_context)?,
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => {
                let ns = activation.avm2().pop().as_namespace()?.clone();
                Self {
                    ns: vec![ns],
                    name: translation_unit
                        .pool_string_option(name.0, activation.context.gc_context)?,
                }
            }
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => {
                let ns = activation.avm2().pop().as_namespace()?.clone();
                let name = activation.avm2().pop().coerce_to_string(activation)?;
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
                ns: Self::abc_namespace_set(
                    translation_unit,
                    namespace_set.clone(),
                    activation.context.gc_context,
                )?,
                name: translation_unit.pool_string_option(name.0, activation.context.gc_context)?,
            },
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => {
                let name = activation.avm2().pop().coerce_to_string(activation)?;
                Self {
                    ns: Self::abc_namespace_set(
                        translation_unit,
                        namespace_set.clone(),
                        activation.context.gc_context,
                    )?,
                    name: Some(name),
                }
            }
        })
    }

    /// Read a static multiname from the ABC constant pool
    ///
    /// This function prohibits the use of runtime-qualified and late-bound
    /// names. Runtime multinames will instead result in an error.
    ///
    /// Multiname index zero is also treated as an error, you must check for it
    /// and substitute it with whatever default is called for by AVM2.
    pub fn from_abc_multiname_static(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        mc: MutationContext<'gc, '_>,
    ) -> Result<Self, Error> {
        let actual_index: Result<usize, Error> =
            (multiname_index.0 as usize).checked_sub(1).ok_or_else(|| {
                "Attempted to resolve a (static) multiname at index zero. This is a bug.".into()
            });
        let actual_index = actual_index?;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(
                        translation_unit,
                        namespace.clone(),
                        mc,
                    )?],
                    name: translation_unit.pool_string_option(name.0, mc)?,
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
                ns: Self::abc_namespace_set(translation_unit, namespace_set.clone(), mc)?,
                name: translation_unit.pool_string_option(name.0, mc)?,
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

    pub fn includes_dynamic_namespace(&self) -> bool {
        for ns in self.ns.iter() {
            if ns.is_dynamic() {
                return true;
            }
        }

        false
    }

    /// Indicates if this multiname matches any type in any namespace.
    pub fn is_any(&self) -> bool {
        self.ns.contains(&Namespace::Any) && self.name.is_none()
    }

    /// Determine if this multiname matches a given QName.
    pub fn contains_name(&self, name: &QName<'gc>) -> bool {
        let ns_match = self
            .ns
            .iter()
            .any(|ns| ns == &Namespace::Any || ns == name.namespace());
        let name_match = self.name.map(|n| n == name.local_name()).unwrap_or(true);

        ns_match && name_match
    }
}

impl<'gc> From<QName<'gc>> for Multiname<'gc> {
    fn from(q: QName<'gc>) -> Self {
        Self {
            ns: vec![q.ns],
            name: Some(q.name),
        }
    }
}
