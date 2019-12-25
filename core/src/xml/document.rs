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

    /// Yield the document in node form.
    ///
    /// If the document does not have a node, then this function will panic.
    pub fn as_node(self) -> XMLNode<'gc> {
        self.0
            .read()
            .root
            .expect("Document must always have a root node")
    }

    /// Create a duplicate copy of this document.
    ///
    /// The contents of the document will not be duplicated. This results in a
    /// rootless document that is not safe to use without first linking another
    /// root node into it. (See `link_root_node`.)
    pub fn duplicate(self, gc_context: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(gc_context, XMLDocumentData { root: None }))
    }

    /// Set the root node of the document, if possible.
    ///
    /// If the proposed root is not an `XMLNode::DocumentRoot`, then a fresh
    /// document root node will be created and the root will be adopted into
    /// it.
    ///
    /// If the document already has a root node, nothing happens.
    pub fn link_root_node(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        proposed_root: XMLNode<'gc>,
    ) {
        match (
            &mut *self.0.write(gc_context),
            proposed_root.is_document_root(),
        ) {
            (XMLDocumentData { root }, true) if root.is_none() => {
                *root = Some(proposed_root);
            }
            (XMLDocumentData { root }, false) if root.is_none() => {
                *root = Some(XMLNode::new_document_root(gc_context, *self));
            }
            _ => {}
        }
    }
}

impl<'gc> fmt::Debug for XMLDocument<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XMLDocument")
            .field("root", &self.0.read().root)
            .finish()
    }
}
