//! XML namespacing support

use crate::xml::Error;
use gc_arena::Collect;
use std::borrow::Cow;
use std::fmt;

/// Represents a scoped name within XML.
///
/// All names in XML are optionally namespaced. Each namespace is represented
/// as a string; the document contains a mapping of namespaces to URIs.
///
/// The special namespace `xmlns` is used to map namespace strings to URIs; it
/// should not be used for user-specified namespaces.
#[derive(Clone, Collect, PartialEq, Eq, PartialOrd, Ord)]
#[collect(no_drop)]
pub struct XMLName {
    /// The name of the XML namespace this name is scoped to.
    ///
    /// Names without a namespace use the default namespace.
    ///
    /// Namespaces may be resolved to a URI by consulting the encapsulating
    /// document.
    namespace: Option<String>,
    name: String,
}

impl XMLName {
    /// Construct an XML name from it's parts (name and namespace).
    pub fn from_parts(namespace: Option<&str>, name: &str) -> Self {
        XMLName {
            namespace: namespace.map(|s| s.to_string()),
            name: name.to_string(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Self::from_bytes_cow(Cow::Borrowed(bytes))
    }

    pub fn from_str(strval: &str) -> Self {
        Self::from_str_cow(Cow::Borrowed(strval))
    }

    pub fn from_bytes_cow(bytes: Cow<[u8]>) -> Result<Self, Error> {
        let full_name = match bytes {
            Cow::Borrowed(ln) => Cow::Borrowed(std::str::from_utf8(ln)?),
            Cow::Owned(ln) => Cow::Owned(String::from_utf8(ln)?),
        };

        Ok(Self::from_str_cow(full_name))
    }

    pub fn from_str_cow(full_name: Cow<str>) -> Self {
        if let Some(colon_index) = full_name.find(':') {
            Self {
                namespace: Some(full_name[0..colon_index].to_owned()),
                name: full_name[colon_index + 1..].to_owned(),
            }
        } else {
            Self {
                namespace: None,
                name: full_name.into_owned(),
            }
        }
    }

    /// Retrieve the local part of this name.
    pub fn local_name(&self) -> &str {
        &self.name
    }

    /// Retrieve the prefix part of this name, if available.
    pub fn prefix(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Return the fully qualified part of the name.
    ///
    /// This consists of the namespace, if present, plus a colon and local name.
    pub fn node_name(&self) -> Cow<str> {
        if let Some(ref ns) = self.namespace {
            Cow::Owned(format!("{}:{}", ns, self.name))
        } else {
            Cow::Borrowed(&self.name)
        }
    }
}

impl fmt::Debug for XMLName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XMLName")
            .field("namespace", &self.namespace)
            .field("name", &self.name)
            .finish()
    }
}
