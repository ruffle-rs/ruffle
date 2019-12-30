//! XML Document

use crate::xml::{Error, XMLNode};
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::{BytesDecl, Event};
use quick_xml::Writer;
use std::fmt;
use std::io::Cursor;

/// The entirety of an XML document.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XMLDocument<'gc>(GcCell<'gc, XMLDocumentData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XMLDocumentData<'gc> {
    /// The root node of the XML document.
    root: Option<XMLNode<'gc>>,

    /// Whether or not the document has a document declaration.
    has_xmldecl: bool,

    /// The XML version string, if set.
    version: String,

    /// The XML document encoding, if set.
    encoding: Option<String>,

    /// The XML standalone flag, if set.
    standalone: Option<String>,

    /// The XML doctype, if set.
    doctype: Option<XMLNode<'gc>>,
}

impl<'gc> XMLDocument<'gc> {
    /// Construct a new, empty XML document.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let document = Self(GcCell::allocate(
            mc,
            XMLDocumentData {
                root: None,
                has_xmldecl: false,
                version: "1.0".to_string(),
                encoding: None,
                standalone: None,
                doctype: None,
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
                has_xmldecl: self_read.has_xmldecl,
                version: self_read.version.clone(),
                encoding: self_read.encoding.clone(),
                standalone: self_read.standalone.clone(),
                doctype: None,
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

    /// Set the DOCTYPE of the document, if possible.
    ///
    /// If the proposed doctype is not an `XMLNode::DocType`, or the document
    /// already has a doctype, nothing happens.
    pub fn link_doctype(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        proposed_doctype: XMLNode<'gc>,
    ) {
        let mut self_write = self.0.write(gc_context);

        if self_write.doctype.is_none() && proposed_doctype.is_doctype() {
            self_write.doctype = Some(proposed_doctype);
        }
    }

    /// Retrieve the first DocType node in the document.
    pub fn doctype(self) -> Option<XMLNode<'gc>> {
        self.0.read().doctype
    }

    /// Process events being passed into some node of the document.
    ///
    /// There are certain nodes which have document-wide implications if parsed
    /// into any node within the document. These are processed here.
    pub fn process_event(self, mc: MutationContext<'gc, '_>, event: &Event) -> Result<(), Error> {
        if let Event::Decl(bd) = event {
            let mut self_write = self.0.write(mc);

            self_write.has_xmldecl = true;
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

    /// Generate a string matching the XML document declaration, if there is
    /// one.
    pub fn xmldecl_string(self) -> Result<Option<String>, Error> {
        let self_read = self.0.read();

        if self_read.has_xmldecl {
            let mut result = Vec::new();
            let mut writer = Writer::new(Cursor::new(&mut result));
            let bd = BytesDecl::new(
                &self_read.version.as_bytes(),
                self_read.encoding.as_ref().map(|s| s.as_bytes()),
                self_read.standalone.as_ref().map(|s| s.as_bytes()),
            );
            writer.write_event(Event::Decl(bd))?;

            Ok(Some(String::from_utf8(result)?))
        } else {
            Ok(None)
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
