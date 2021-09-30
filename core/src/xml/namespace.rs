//! XML namespacing support

use crate::string::{AvmString, BorrowWStr, WStr, WString};
use gc_arena::{Collect, MutationContext};
use std::fmt;

/// Represents a scoped name within XML.
///
/// All names in XML are optionally namespaced. Each namespace is represented
/// as a string; the document contains a mapping of namespaces to URIs.
///
/// Names without a namespace use the default namespace.
///
/// The special namespace `xmlns` is used to map namespace strings to URIs; it
/// should not be used for user-specified namespaces.
#[derive(Copy, Clone, Collect, PartialEq, Eq, PartialOrd, Ord)]
#[collect(no_drop)]
pub struct XmlName<'gc> {
    /// The position of the namespace separator in the name, if the name is namespaced.
    namespace_sep: Option<usize>,
    name: AvmString<'gc>,
}

impl<'gc> XmlName<'gc> {
    pub fn in_namespace(
        gc_context: MutationContext<'gc, '_>,
        namespace: WStr<'_>,
        name: WStr<'_>,
    ) -> Self {
        let mut full_name = WString::from(namespace);
        full_name.push_byte(b':');
        full_name.push_str(name);
        Self {
            namespace_sep: Some(namespace.len()),
            name: AvmString::new_ucs2(gc_context, full_name),
        }
    }

    pub fn in_default_namespace(name: AvmString<'gc>) -> Self {
        Self {
            namespace_sep: None,
            name,
        }
    }

    pub fn from_str(full_name: impl Into<AvmString<'gc>>) -> Self {
        let full_name = full_name.into();
        Self {
            namespace_sep: full_name.find(b':'),
            name: full_name,
        }
    }

    /// Retrieve the local part of this name.
    pub fn local_name(&self) -> WStr<'_> {
        match self.namespace_sep {
            Some(sep) => self.name.slice(sep + 1..),
            None => self.name.borrow(),
        }
    }

    /// Retrieve the prefix part of this name, if available.
    pub fn prefix(&self) -> Option<WStr<'_>> {
        self.namespace_sep.map(|sep| self.name.slice(..sep))
    }

    /// Return the fully qualified part of the name.
    ///
    /// This consists of the namespace, if present, plus a colon and local name.
    pub fn node_name(&self) -> AvmString<'gc> {
        self.name
    }

    /// Compares both names as case-insensitve (for use in HTML parsing).
    /// TODO: We shouldn't need this when we have a proper HTML parser.
    pub fn eq_ignore_case(&self, other: XmlName<'gc>) -> bool {
        self.name.eq_ignore_case(other.name.borrow())
    }
}

impl<'gc> fmt::Debug for XmlName<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XmlName")
            .field("namespace", &self.prefix())
            .field("name", &self.local_name())
            .finish()
    }
}
