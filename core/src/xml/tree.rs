//! XML Tree structure

use crate::xml::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, BytesText};
use std::borrow::Cow;
use std::collections::BTreeMap;

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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Self::from_bytes_cow(Cow::Borrowed(bytes))
    }

    pub fn from_bytes_cow(bytes: Cow<[u8]>) -> Result<Self, Error> {
        let full_name = match bytes {
            Cow::Borrowed(ln) => Cow::Borrowed(std::str::from_utf8(ln)?),
            Cow::Owned(ln) => Cow::Owned(String::from_utf8(ln)?),
        };

        if let Some(colon_index) = full_name.find(':') {
            Ok(Self {
                namespace: Some(full_name[0..colon_index].to_owned()),
                name: full_name[colon_index + 1..].to_owned(),
            })
        } else {
            Ok(Self {
                namespace: None,
                name: full_name.into_owned(),
            })
        }
    }
}

/// Represents a node in the XML tree.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XMLNode<'gc>(GcCell<'gc, XMLNodeData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum XMLNodeData<'gc> {
    /// A text node in the XML tree.
    Text {
        /// The string representation of the text.
        contents: String,
    },

    /// A comment node in the XML tree.
    Comment {
        /// The string representation of the comment.
        contents: String,
    },

    /// An element node in the XML tree.
    ///
    /// Element nodes are non-leaf nodes: they can store additional data as
    /// either attributes (for key/value pairs) or child nodes (for more
    /// structured data).
    Element {
        /// The tag name of this element.
        tag_name: XMLName,

        /// Attributes of the element.
        attributes: BTreeMap<XMLName, String>,

        /// Child nodes of this element.
        children: Vec<XMLNode<'gc>>,
    },
}

impl<'gc> XMLNode<'gc> {
    /// Construct an XML node from a `quick_xml` `BytesStart` event.
    ///
    /// The returned node will always be an `Element`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
    pub fn from_start_event<'a>(
        mc: MutationContext<'gc, '_>,
        bs: BytesStart<'a>,
    ) -> Result<Self, Error> {
        let tag_name = XMLName::from_bytes_cow(bs.unescaped()?)?;
        let mut attributes = BTreeMap::new();

        for a in bs.attributes() {
            let attribute = a?;
            attributes.insert(
                XMLName::from_bytes(attribute.key)?,
                String::from_utf8(attribute.value.to_owned().to_vec())?,
            );
        }

        let children = Vec::new();

        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Element {
                tag_name,
                attributes,
                children,
            },
        )))
    }

    pub fn text_from_text_event<'a>(
        mc: MutationContext<'gc, '_>,
        bt: BytesText<'a>,
    ) -> Result<Self, Error> {
        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Text {
                contents: match bt.unescaped()? {
                    Cow::Borrowed(ln) => Cow::Borrowed(std::str::from_utf8(ln)?),
                    Cow::Owned(ln) => Cow::Owned(String::from_utf8(ln)?),
                }
                .to_owned()
                .to_string(),
            },
        )))
    }

    pub fn append_child(
        &mut self,
        mc: MutationContext<'gc, '_>,
        child: XMLNode<'gc>,
    ) -> Result<(), Error> {
        match &mut *self.0.write(mc) {
            XMLNodeData::Element {
                ref mut children, ..
            } => children.push(child),
            _ => return Err("Not an Element".into()),
        };

        Ok(())
    }
}
