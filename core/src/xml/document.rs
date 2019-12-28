//! XML Document

use crate::xml::XMLNode;
use gc_arena::{Collect, GcCell, MutationContext};
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
