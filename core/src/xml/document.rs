//! XML Document

use crate::xml::{Error, XMLNode};
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::Event;
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

    /// The XML version string, if set.
    version: String,

    /// The XML document encoding, if set.
    encoding: Option<String>,

    /// The XML standalone flag, if set.
    standalone: Option<String>,
}

impl<'gc> XMLDocument<'gc> {
    /// Construct a new, empty XML document.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let document = Self(GcCell::allocate(
            mc,
            XMLDocumentData {
                root: None,
                version: "1.0".to_string(),
                encoding: None,
                standalone: None,
            },
        ));
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
        let self_read = self.0.read();
        Self(GcCell::allocate(
            gc_context,
            XMLDocumentData {
                root: None,
                version: self_read.version.clone(),
                encoding: self_read.encoding.clone(),
                standalone: self_read.standalone.clone(),
            },
        ))
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
            (XMLDocumentData { root, .. }, true) if root.is_none() => {
                *root = Some(proposed_root);
            }
            (XMLDocumentData { root, .. }, false) if root.is_none() => {
                *root = Some(XMLNode::new_document_root(gc_context, *self));
            }
            _ => {}
        }
    }

    /// Process events being passed into some node of the document.
    ///
    /// There are certain nodes which have document-wide implications if parsed
    /// into any node within the document. These are processed here.
    pub fn process_event(self, mc: MutationContext<'gc, '_>, event: &Event) -> Result<(), Error> {
        if let Event::Decl(bd) = event {
            let mut self_write = self.0.write(mc);

            self_write.version = String::from_utf8(bd.version()?.into_owned())?;
            self_write.encoding = if let Some(encoding) = bd.encoding() {
                Some(String::from_utf8(encoding?.into_owned())?)
            } else {
                None
            };
            self_write.standalone = if let Some(standalone) = bd.standalone() {
                Some(String::from_utf8(standalone?.into_owned())?)
            } else {
                None
            };
        }

        Ok(())
    }
}

impl<'gc> fmt::Debug for XMLDocument<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XMLDocument")
            .field("root", &self.0.read().root)
            .finish()
    }
}
