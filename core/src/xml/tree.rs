//! XML Tree structure

use crate::xml;
use crate::xml::{Error, XMLDocument, XMLName};
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::{BytesStart, BytesText};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

/// Represents a node in the XML tree.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XMLNode<'gc>(GcCell<'gc, XMLNodeData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum XMLNodeData<'gc> {
    /// A text node in the XML tree.
    Text {
        /// The document that this tree node currently belongs to.
        document: XMLDocument<'gc>,

        /// The parent node of this one.
        parent: Option<XMLNode<'gc>>,

        /// The string representation of the text.
        contents: String,
    },

    /// A comment node in the XML tree.
    Comment {
        /// The document that this tree node currently belongs to.
        document: XMLDocument<'gc>,

        /// The parent node of this one.
        parent: Option<XMLNode<'gc>>,

        /// The string representation of the comment.
        contents: String,
    },

    /// An element node in the XML tree.
    ///
    /// Element nodes are non-leaf nodes: they can store additional data as
    /// either attributes (for key/value pairs) or child nodes (for more
    /// structured data).
    Element {
        /// The document that this tree node currently belongs to.
        document: XMLDocument<'gc>,

        /// The parent node of this one.
        parent: Option<XMLNode<'gc>>,

        /// The tag name of this element.
        tag_name: XMLName,

        /// Attributes of the element.
        attributes: BTreeMap<XMLName, String>,

        /// Child nodes of this element.
        children: Vec<XMLNode<'gc>>,
    },

    /// The root level of an XML document. Has no parent.
    DocumentRoot {
        /// The document that this is the root of.
        document: XMLDocument<'gc>,

        /// Child nodes of this element.
        children: Vec<XMLNode<'gc>>,
    },
}

impl<'gc> XMLNode<'gc> {
    /// Construct a new XML text node.
    pub fn new_text(
        mc: MutationContext<'gc, '_>,
        contents: &str,
        document: XMLDocument<'gc>,
    ) -> Self {
        XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Text {
                document,
                parent: None,
                contents: contents.to_string(),
            },
        ))
    }

    /// Construct a new XML element node.
    pub fn new_element(
        mc: MutationContext<'gc, '_>,
        element_name: &str,
        document: XMLDocument<'gc>,
    ) -> Result<Self, Error> {
        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Element {
                document,
                parent: None,
                tag_name: XMLName::from_str(element_name)?,
                attributes: BTreeMap::new(),
                children: Vec::new(),
            },
        )))
    }

    /// Construct a new XML root node.
    pub fn new_document_root(mc: MutationContext<'gc, '_>, document: XMLDocument<'gc>) -> Self {
        XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::DocumentRoot {
                document,
                children: Vec::new(),
            },
        ))
    }

    /// Construct an XML Element node from a `quick_xml` `BytesStart` event.
    ///
    /// The returned node will always be an `Element`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
    pub fn from_start_event<'a>(
        mc: MutationContext<'gc, '_>,
        bs: BytesStart<'a>,
        document: XMLDocument<'gc>,
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
                document,
                parent: None,
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
        document: XMLDocument<'gc>,
    ) -> Result<Self, Error> {
        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Text {
                document,
                parent: None,
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
        document: XMLDocument<'gc>,
    ) -> Result<Self, Error> {
        Ok(XMLNode(GcCell::allocate(
            mc,
            XMLNodeData::Comment {
                document,
                parent: None,
                contents: match bt.unescaped()? {
                    Cow::Borrowed(ln) => Cow::Borrowed(std::str::from_utf8(ln)?),
                    Cow::Owned(ln) => Cow::Owned(String::from_utf8(ln)?),
                }
                .to_owned()
                .to_string(),
            },
        )))
    }

    /// Return the XML document that this tree node belongs to.
    ///
    /// Every XML node belongs to a document object (see `XMLDocument`) which
    /// stores global information about the document, such as namespace URIs.
    pub fn document(&self) -> XMLDocument<'gc> {
        match &*self.0.read() {
            XMLNodeData::Text { document, .. } => *document,
            XMLNodeData::Comment { document, .. } => *document,
            XMLNodeData::Element { document, .. } => *document,
            XMLNodeData::DocumentRoot { document, .. } => *document,
        }
    }

    /// Adopt a child element into the current node.
    ///
    /// This does not add the node to any internal lists; it merely updates the
    /// child to ensure that it considers this node it's parent. This function
    /// should always be called after a child node is added to this one.
    pub fn adopt(
        &mut self,
        mc: MutationContext<'gc, '_>,
        child: XMLNode<'gc>,
    ) -> Result<(), Error> {
        let mut write = child.0.write(mc);
        let (child_document, child_parent) = match &mut *write {
            XMLNodeData::Element {
                document, parent, ..
            } => Ok((document, parent)),
            XMLNodeData::Text {
                document, parent, ..
            } => Ok((document, parent)),
            XMLNodeData::Comment {
                document, parent, ..
            } => Ok((document, parent)),
            XMLNodeData::DocumentRoot { .. } => Err("Cannot adopt other document roots"),
        }?;

        *child_document = self.document();
        *child_parent = Some(*self);

        Ok(())
    }

    /// Append a child element to an Element node.
    ///
    /// The child will be adopted into the current tree: all child references
    /// to other nodes or documents will be adjusted to reflect it's new
    /// position in the tree. This may remove it from any existing trees or
    /// documents.
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
            } => {
                children.push(child);
            }
            _ => return Err("Not an Element".into()),
        };

        self.adopt(mc, child)?;

        Ok(())
    }

    /// Returns the type of this node as an integer.
    ///
    /// This is primarily intended to match W3C DOM L1 specifications and
    /// should not be used in lieu of a proper `match` statement.
    pub fn node_type(&self) -> u8 {
        match &*self.0.read() {
            XMLNodeData::Element { .. } => xml::ELEMENT_NODE,
            XMLNodeData::DocumentRoot { .. } => xml::ELEMENT_NODE,
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
            XMLNodeData::Text { ref contents, .. } => Some(contents.clone()),
            XMLNodeData::Comment { ref contents, .. } => Some(contents.clone()),
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
                let read = self.base.0.read();
                let children = match &*read {
                    XMLNodeData::Element { children, .. }
                    | XMLNodeData::DocumentRoot { children, .. } => Some(children),
                    _ => None,
                };

                if let Some(children) = children {
                    if self.index < children.len() {
                        let item = children.get(self.index).cloned();
                        self.index += 1;

                        return item;
                    }
                }

                None
            }
        }

        match &*self.0.read() {
            XMLNodeData::Element { .. } | XMLNodeData::DocumentRoot { .. } => {
                Some(ChildIter::for_node(*self))
            }
            _ => return None,
        }
    }
}

impl<'gc> fmt::Debug for XMLNode<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self.0.read() {
            XMLNodeData::Text {
                document,
                parent,
                contents,
            } => f
                .debug_struct("XMLNodeData::Text")
                .field("document", document)
                .field("parent", parent)
                .field("contents", contents)
                .finish(),
            XMLNodeData::Comment {
                document,
                parent,
                contents,
            } => f
                .debug_struct("XMLNodeData::Comment")
                .field("document", document)
                .field("parent", parent)
                .field("contents", contents)
                .finish(),
            XMLNodeData::Element {
                document,
                parent,
                tag_name,
                attributes,
                children,
            } => f
                .debug_struct("XMLNodeData::Element")
                .field("document", document)
                .field("parent", parent)
                .field("tag_name", tag_name)
                .field("attributes", attributes)
                .field("children", children)
                .finish(),
            XMLNodeData::DocumentRoot { document, children } => f
                .debug_struct("XMLNodeData::DocumentRoot")
                .field("document", document)
                .field("children", children)
                .finish(),
        }
    }
}
