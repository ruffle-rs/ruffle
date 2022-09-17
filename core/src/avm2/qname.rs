use crate::avm2::script::TranslationUnit;
use crate::avm2::Error;
use crate::avm2::Namespace;
use crate::either::Either;
use crate::string::{AvmString, WStr, WString};
use gc_arena::{Collect, MutationContext};
use std::fmt::Debug;
use swf::avm2::types::{Index, Multiname as AbcMultiname};

/// A `QName`, likely "qualified name", consists of a namespace and name string.
///
/// This is technically interchangeable with `xml::XMLName`, as they both
/// implement `QName`; however, AVM2 and XML have separate representations.
///
/// A property cannot be retrieved or set without first being resolved into a
/// `QName`. All other forms of names and multinames are either versions of
/// `QName` with unspecified parameters, or multiple names to be checked in
/// order.
#[derive(Clone, Copy, Collect, Hash)]
#[collect(no_drop)]
pub struct QName<'gc> {
    ns: Namespace<'gc>,
    name: AvmString<'gc>,
}

impl<'gc> PartialEq for QName<'gc> {
    fn eq(&self, other: &Self) -> bool {
        // Implemented by hand to enforce order of comparisons for perf
        self.name == other.name && self.ns == other.ns
    }
}

impl<'gc> Eq for QName<'gc> {}

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
    ) -> Result<Self, Error<'gc>> {
        if multiname_index.0 == 0 {
            return Err("Attempted to load a trait name of index zero".into());
        }

        let actual_index = multiname_index.0 as usize - 1;
        let abc = translation_unit.abc();
        let abc_multiname: Result<_, Error<'gc>> = abc
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } => Self {
                ns: Namespace::from_abc_namespace(translation_unit, *namespace, mc)?,
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
    pub fn from_qualified_name(name: AvmString<'gc>, mc: MutationContext<'gc, '_>) -> Self {
        let parts = name
            .rsplit_once(WStr::from_units(b"::"))
            .or_else(|| name.rsplit_once(WStr::from_units(b".")));

        if let Some((package_name, local_name)) = parts {
            Self {
                ns: Namespace::Package(AvmString::new(mc, package_name)),
                name: AvmString::new(mc, local_name),
            }
        } else {
            Self {
                ns: Namespace::public(),
                name,
            }
        }
    }

    /// Converts this `QName` to a fully qualified name.
    pub fn to_qualified_name(self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        match self.to_qualified_name_no_mc() {
            Either::Left(avm_string) => avm_string,
            Either::Right(wstring) => AvmString::new(mc, wstring),
        }
    }

    /// Like `to_qualified_name`, but avoids the need for a `MutationContext`
    /// by returning `Either::Right(wstring)` when it would otherwise
    /// be necessary to allocate a new `AvmString`.
    ///
    /// This method is intended for contexts like `Debug` impls where
    /// a `MutationContext` is not available. Normally, you should
    /// use `to_qualified_name`
    pub fn to_qualified_name_no_mc(self) -> Either<AvmString<'gc>, WString> {
        let uri = self.namespace().as_uri();
        let name = self.local_name();
        if uri.is_empty() {
            Either::Left(name)
        } else {
            Either::Right({
                let mut buf = WString::from(uri.as_wstr());
                buf.push_str(WStr::from_units(b"::"));
                buf.push_str(&name);
                buf
            })
        }
    }

    pub fn local_name(&self) -> AvmString<'gc> {
        self.name
    }

    pub fn namespace(self) -> Namespace<'gc> {
        self.ns
    }

    /// Get the string value of this QName, including the namespace URI.
    pub fn as_uri(&self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let ns = match &self.ns {
            Namespace::Namespace(s) => s,
            Namespace::Package(s) => s,
            Namespace::PackageInternal(s) => s,
            Namespace::Protected(s) => s,
            Namespace::Explicit(s) => s,
            Namespace::StaticProtected(s) => s,
            Namespace::Private(s) => s,
            Namespace::Any => WStr::from_units(b"*"),
        };

        if ns.is_empty() {
            return self.name;
        }

        let mut uri = WString::from(ns);
        uri.push_str(WStr::from_units(b"::"));
        uri.push_str(&self.name);
        AvmString::new(mc, uri)
    }
}

impl<'gc> Debug for QName<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.to_qualified_name_no_mc() {
            Either::Left(name) => write!(f, "{}", name),
            Either::Right(name) => write!(f, "{}", name),
        }
    }
}
