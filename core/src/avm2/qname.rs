use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_1033;
use crate::avm2::script::TranslationUnit;
use crate::avm2::{Error, Namespace};
use crate::context::UpdateContext;
use crate::string::{AvmString, WStr, WString};
use either::Either;
use gc_arena::{Collect, Mutation};
use std::fmt::Debug;
use swf::avm2::types::{Index, Multiname as AbcMultiname};

use super::api_version::ApiVersion;
use super::Multiname;

/// Qualified name.
/// NOTE: this struct doesn't actually directly correspond to an AVM2 QName concept.
/// Currently, we mostly use this struct simply to wrap a name+namespace pair,
/// and try to reduce its future use if possible.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct QName<'gc> {
    ns: Namespace<'gc>,
    name: AvmString<'gc>,
}

impl<'gc> PartialEq for QName<'gc> {
    fn eq(&self, other: &Self) -> bool {
        // Implemented by hand to enforce order of comparisons for perf
        self.name == other.name && self.ns.exact_version_match(other.ns)
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
    /// This function returns an Err if the multiname is not a `QName`.
    pub fn from_abc_multiname(
        activation: &mut Activation<'_, 'gc>,
        translation_unit: TranslationUnit<'gc>,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Self, Error<'gc>> {
        let name = Multiname::from_abc_index(activation, translation_unit, multiname_index)?;

        if name.is_any_namespace()
            || name.is_any_name()
            || name.has_lazy_component()
            || name.is_attribute()
        {
            return Err(make_error_1033(activation));
        }

        // For any non-QName not in the playerglobals domain, return an error.
        if !name.is_qname()
            && !translation_unit
                .domain()
                .is_playerglobals_domain(activation.avm2())
        {
            return Err(make_error_1033(activation));
        }

        // Now we know that the Multiname must have an explicit namespace and a local name.

        Ok(Self {
            ns: name.namespace_set()[0],
            name: name.local_name().unwrap(),
        })
    }

    /// Constructs a `QName` from a fully qualified name.
    ///
    /// A fully qualified name can be any of the following formats:
    /// NAMESPACE::LOCAL_NAME
    /// NAMESPACE.LOCAL_NAME (Where the LAST dot is used to split the namespace & local_name)
    /// LOCAL_NAME (Use the public namespace)
    ///
    /// This does *not* handle `Vector.<SomeTypeParam>` - use `get_defined_value_handling_vector` for that
    pub fn from_qualified_name(
        name: AvmString<'gc>,
        api_version: ApiVersion,
        context: &mut UpdateContext<'gc>,
    ) -> Self {
        let parts = name
            .rsplit_once(WStr::from_units(b"::"))
            .or_else(|| name.rsplit_once(WStr::from_units(b".")));

        if let Some((package_name, local_name)) = parts {
            let package_name = context.strings.intern_wstr(package_name);

            Self {
                ns: Namespace::package(package_name, api_version, &mut context.strings),
                name: AvmString::new(context.gc(), local_name),
            }
        } else {
            Self {
                ns: context.avm2.namespaces.public_for(api_version),
                name,
            }
        }
    }

    /// Converts this `QName` to a fully qualified name.
    pub fn to_qualified_name(self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        match self.to_qualified_name_no_mc() {
            Either::Left(avm_string) => avm_string,
            Either::Right(wstring) => AvmString::new(mc, wstring),
        }
    }

    /// Like `to_qualified_name`, but avoids the need for a `Mutation`
    /// by returning `Either::Right(wstring)` when it would otherwise
    /// be necessary to allocate a new `AvmString`.
    ///
    /// This method is intended for contexts like `Debug` impls where
    /// a `Mutation` is not available. Normally, you should
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
    pub fn to_qualified_name_err_message(self, mc: &Mutation<'gc>) -> AvmString<'gc> {
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
    pub fn as_uri(&self, mc: &Mutation<'gc>) -> AvmString<'gc> {
        let ns_uri = self.ns.as_uri_opt();
        let ns = match &ns_uri {
            Some(s) if s.is_empty() => return self.name,
            Some(s) => s,
            None => WStr::from_units(b"*"),
        };

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
