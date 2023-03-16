use crate::avm2::script::TranslationUnit;
use crate::avm2::Activation;
use crate::avm2::Error;
use crate::avm2::{Namespace, NamespaceData};
use crate::context::GcContext;
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
#[derive(Clone, Copy, Collect)]
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

    /// Pull a `QName` from the multiname pool.
    ///
    /// This function returns an Err if the multiname does not exist or is not
    /// a `QName`.
    pub fn from_abc_multiname(
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
        context: &mut GcContext<'_, 'gc>,
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
                ns: translation_unit.pool_namespace(*namespace, context)?,
                name: translation_unit.pool_string(name.0, context)?,
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
    pub fn from_qualified_name(name: AvmString<'gc>, activation: &mut Activation<'_, 'gc>) -> Self {
        let mc = activation.context.gc_context;
        // If we have a type like 'some::namespace::Vector.<other::namespace::MyType>',
        // we want to look at 'some::namespace::Vector' when splitting out the namespace
        let before_type_param = if let Some(type_param_start) = name.find(WStr::from_units(b".<")) {
            &name[..type_param_start]
        } else {
            &name
        };

        // We unfortunately can't use 'rsplit' here, because we would need to split
        // the entire string, but only before the first '.<'
        //
        // Get the last '::' or '.', only considering the string before '.<' (if any).
        // This will ignore any namespaces that are part of a type parameter (e.g 'Vector.<other::namespace::MyType>').
        // The type parameter will stay combined with the type name (so we'll have 'Vector.<other::namespace::MyType>')
        // in some namespace, depending on whether or not anything comes before 'Vector')
        let parts = if let Some(last_separator) = before_type_param.rfind(WStr::from_units(b"::")) {
            Some((&name[..last_separator], &name[(last_separator + 2)..]))
        } else if let Some(last_separator) = before_type_param.rfind(b".".as_slice()) {
            Some((&name[..last_separator], &name[(last_separator + 1)..]))
        } else {
            None
        };

        if let Some((package_name, local_name)) = parts {
            Self {
                ns: Namespace::package(AvmString::new(mc, package_name), mc),
                name: AvmString::new(mc, local_name),
            }
        } else {
            Self {
                ns: activation.avm2().public_namespace,
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

    // Like `to_qualified_name`, but uses a `.` instead of `::` separate
    // the namespace and local name. This matches the output produced by
    // Flash Player in error messages
    pub fn to_qualified_name_err_message(self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let mut buf = WString::new();
        let uri = self.namespace().as_uri();
        if !uri.is_empty() {
            buf.push_str(&uri);
            buf.push_char('.');
        }
        buf.push_str(&self.local_name());
        AvmString::new(mc, buf)
    }

    pub fn local_name(&self) -> AvmString<'gc> {
        self.name
    }

    pub fn namespace(self) -> Namespace<'gc> {
        self.ns
    }

    /// Get the string value of this QName, including the namespace URI.
    pub fn as_uri(&self, mc: MutationContext<'gc, '_>) -> AvmString<'gc> {
        let ns = match &*self.ns.0 {
            NamespaceData::Namespace(s) => s,
            NamespaceData::PackageInternal(s) => s,
            NamespaceData::Protected(s) => s,
            NamespaceData::Explicit(s) => s,
            NamespaceData::StaticProtected(s) => s,
            NamespaceData::Private(s) => s,
            NamespaceData::Any => WStr::from_units(b"*"),
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
            Either::Left(name) => write!(f, "{name}"),
            Either::Right(name) => write!(f, "{name}"),
        }
    }
}
