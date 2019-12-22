//! XML Document

use crate::xml::Error;
use crate::xml::XMLNode;
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fmt;

/// The entirety of an XML document.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XMLDocument<'gc>(GcCell<'gc, XMLDocumentData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XMLDocumentData<'gc> {
    /// The root node of the XML document.
    root: Option<XMLNode<'gc>>,
}

impl<'gc> XMLDocument<'gc> {
    /// Construct a new, empty XML document.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let document = Self(GcCell::allocate(mc, XMLDocumentData { root: None }));
        let root = XMLNode::new_document_root(mc, document);

        document.0.write(mc).root = Some(root);

        document
    }

    /// Ensure that a newly-encountered node is added to an ongoing parsing
    /// stack, or to the document itself if the parsing stack is empty.
    fn add_child_to_tree(
        &mut self,
        mc: MutationContext<'gc, '_>,
        open_tags: &mut Vec<XMLNode<'gc>>,
        child: XMLNode<'gc>,
    ) -> Result<(), Error> {
        if let Some(node) = open_tags.last_mut() {
            node.append_child(mc, child)?;
        } else {
            self.as_node().append_child(mc, child)?;
        }

        Ok(())
    }

    pub fn from_str(mc: MutationContext<'gc, '_>, data: &str) -> Result<Self, Error> {
        let mut parser = Reader::from_str(data);
        let mut buf = Vec::new();
        let mut document = Self::new(mc);
        let mut open_tags: Vec<XMLNode<'gc>> = Vec::new();

        loop {
            match parser.read_event(&mut buf)? {
                Event::Start(bs) => {
                    let child = XMLNode::from_start_event(mc, bs, document)?;
                    document.add_child_to_tree(mc, &mut open_tags, child)?;
                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let child = XMLNode::from_start_event(mc, bs, document)?;
                    document.add_child_to_tree(mc, &mut open_tags, child)?;
                }
                Event::End(_) => {
                    open_tags.pop();
                }
                Event::Text(bt) => {
                    let child = XMLNode::text_from_text_event(mc, bt, document)?;
                    if child.node_value().as_deref() != Some("") {
                        document.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::Comment(bt) => {
                    let child = XMLNode::comment_from_text_event(mc, bt, document)?;
                    if child.node_value().as_deref() != Some("") {
                        document.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(document)
    }

    /// Returns an iterator that yields the document's root nodes.
    pub fn roots(self) -> impl Iterator<Item = XMLNode<'gc>> {
        self.as_node()
            .children()
            .expect("Document root node must always be capable of holding children")
    }

    /// Yield the document in node form.
    pub fn as_node(self) -> XMLNode<'gc> {
        self.0
            .read()
            .root
            .expect("Document must always have a root node")
    }
}

impl<'gc> fmt::Debug for XMLDocument<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XMLDocument")
            .field("root", &self.0.read().root)
            .finish()
    }
}
