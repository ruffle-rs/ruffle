//! XML Tree structure

use crate::xml;
use crate::xml::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::{BytesStart, BytesText};
use std::borrow::Cow;
use std::collections::BTreeMap;
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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Self::from_bytes_cow(Cow::Borrowed(bytes))
    }

    pub fn from_str(strval: &str) -> Result<Self, Error> {
        Self::from_str_cow(Cow::Borrowed(strval))
    }

    pub fn from_bytes_cow(bytes: Cow<[u8]>) -> Result<Self, Error> {
        let full_name = match bytes {
            Cow::Borrowed(ln) => Cow::Borrowed(std::str::from_utf8(ln)?),
            Cow::Owned(ln) => Cow::Owned(String::from_utf8(ln)?),
        };

        Self::from_str_cow(full_name)
    }

    pub fn from_str_cow(full_name: Cow<str>) -> Result<Self, Error> {
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

impl fmt::Debug for XMLName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XMLName")
            .field("namespace", &self.namespace)
            .field("name", &self.name)
            .finish()
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
    /// Construct a new XML text node.
    pub fn new_text(mc: MutationContext<'gc, '_>, contents: &str) -> Self {
        XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Text {
                contents: contents.to_string(),
            },
        ))
    }

    /// Construct a new XML element node.
    pub fn new_element(mc: MutationContext<'gc, '_>, element_name: &str) -> Result<Self, Error> {
        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Element {
                tag_name: XMLName::from_str(element_name)?,
                attributes: BTreeMap::new(),
                children: Vec::new(),
            },
        )))
    }

    /// Construct an XML Element node from a `quick_xml` `BytesStart` event.
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

    /// Construct an XML Text node from a `quick_xml` `BytesText` event.
    ///
    /// The returned node will always be `Text`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
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

    /// Construct an XML Comment node from a `quick_xml` `BytesText` event.
    ///
    /// The returned node will always be `Comment`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
    pub fn comment_from_text_event<'a>(
        mc: MutationContext<'gc, '_>,
        bt: BytesText<'a>,
    ) -> Result<Self, Error> {
        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Comment {
                contents: match bt.unescaped()? {
                    Cow::Borrowed(ln) => Cow::Borrowed(std::str::from_utf8(ln)?),
                    Cow::Owned(ln) => Cow::Owned(String::from_utf8(ln)?),
                }
                .to_owned()
                .to_string(),
            },
        )))
    }

    /// Append a child element to an Element node.
    ///
    /// This function yields an error if appending to a Node that cannot accept
    /// children. In that case, no modification will be made to the node.
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

    /// Returns the type of this node as an integer.
    ///
    /// This is primarily intended to match W3C DOM L1 specifications and
    /// should not be used in lieu of a proper `match` statement.
    pub fn node_type(&self) -> u8 {
        match &*self.0.read() {
            XMLNodeData::Element { .. } => xml::ELEMENT_NODE,
            XMLNodeData::Text { .. } => xml::TEXT_NODE,
            XMLNodeData::Comment { .. } => xml::COMMENT_NODE,
        }
    }

    /// Returns the tagname, if the element has one.
    pub fn tag_name(&self) -> Option<XMLName> {
        match &*self.0.read() {
            XMLNodeData::Element { ref tag_name, .. } => Some(tag_name.clone()),
            _ => None,
        }
    }

    /// Returns the string contents of the node, if the element has them.
    pub fn node_value(&self) -> Option<String> {
        match &*self.0.read() {
            XMLNodeData::Text { ref contents } => Some(contents.clone()),
            XMLNodeData::Comment { ref contents } => Some(contents.clone()),
            _ => None,
        }
    }

    /// Returns an iterator that yields child nodes.
    ///
    /// Yields None if this node cannot accept children.
    pub fn children(&self) -> Option<impl Iterator<Item = XMLNode<'gc>>> {
        struct ChildIter<'gc> {
            base: XMLNode<'gc>,
            index: usize,
        };

        impl<'gc> ChildIter<'gc> {
            fn for_node(base: XMLNode<'gc>) -> Self {
                Self { base, index: 0 }
            }
        }

        impl<'gc> Iterator for ChildIter<'gc> {
            type Item = XMLNode<'gc>;

            fn next(&mut self) -> Option<Self::Item> {
                match &*self.base.0.read() {
                    XMLNodeData::Element { ref children, .. } if self.index < children.len() => {
                        let item = children.get(self.index).cloned();
                        self.index += 1;

                        item
                    }
                    _ => None,
                }
            }
        }

        match &*self.0.read() {
            XMLNodeData::Element { .. } => Some(ChildIter::for_node(*self)),
            _ => return None,
        }
    }
}

impl<'gc> fmt::Debug for XMLNode<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self.0.read() {
            XMLNodeData::Text { ref contents } => f
                .debug_struct("XMLNodeData::Text")
                .field("contents", contents)
                .finish(),
            XMLNodeData::Comment { ref contents } => f
                .debug_struct("XMLNodeData::Comment")
                .field("contents", contents)
                .finish(),
            XMLNodeData::Element {
                ref tag_name,
                ref attributes,
                ref children,
            } => f
                .debug_struct("XMLNodeData::Element")
                .field("tag_name", tag_name)
                .field("attributes", attributes)
                .field("children", children)
                .finish(),
        }
    }
}
