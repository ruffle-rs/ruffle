//! XML Document

use crate::avm1::object::xml_idmap_object::XmlIdMapObject;
use crate::avm1::Object;
use crate::string::{AvmString, WStr};
use crate::xml::{Error, ParseError, XmlNode};
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::{BytesDecl, Event};
use quick_xml::{Error as QXError, Reader, Writer};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::io::Cursor;

/// The entirety of an XML document.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XmlDocument<'gc>(GcCell<'gc, XmlDocumentData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlDocumentData<'gc> {
    /// The root node of the XML document.
    root: Option<XmlNode<'gc>>,

    /// Whether or not the document has a document declaration.
    has_xmldecl: bool,

    /// The XML version string, if set.
    version: String,

    /// The XML document encoding, if set.
    encoding: Option<String>,

    /// The XML standalone flag, if set.
    standalone: Option<String>,

    /// The XML doctype, if set.
    doctype: Option<XmlNode<'gc>>,

    /// The document's ID map.
    ///
    /// When nodes are parsed into the document by way of `parseXML` or the
    /// document constructor, they get put into this list here, which is used
    /// to populate the document's `idMap`.
    idmap: BTreeMap<AvmString<'gc>, XmlNode<'gc>>,

    /// The script object associated with this XML node, if any.
    idmap_script_object: Option<Object<'gc>>,

    /// The last parse error encountered, if any.
    last_parse_error: Option<ParseError>,
}

impl<'gc> XmlDocument<'gc> {
    /// Construct a new, empty XML document.
    pub fn new(mc: MutationContext<'gc, '_>) -> Self {
        let document = Self(GcCell::allocate(
            mc,
            XmlDocumentData {
                root: None,
                has_xmldecl: false,
                version: "1.0".to_string(),
                encoding: None,
                standalone: None,
                doctype: None,
                idmap: BTreeMap::new(),
                idmap_script_object: None,
                last_parse_error: None,
            },
        ));
        let root = XmlNode::new_document_root(mc, document);

        document.0.write(mc).root = Some(root);

        document
    }

    /// Yield the document in node form.
    ///
    /// If the document does not have a node, then this function will panic.
    pub fn as_node(self) -> XmlNode<'gc> {
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
            XmlDocumentData {
                root: None,
                has_xmldecl: self_read.has_xmldecl,
                version: self_read.version.clone(),
                encoding: self_read.encoding.clone(),
                standalone: self_read.standalone.clone(),
                doctype: None,
                idmap: BTreeMap::new(),
                idmap_script_object: None,
                last_parse_error: None,
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
        proposed_root: XmlNode<'gc>,
    ) {
        match (
            &mut *self.0.write(gc_context),
            proposed_root.is_document_root(),
        ) {
            (XmlDocumentData { root, .. }, true) if root.is_none() => {
                *root = Some(proposed_root);
            }
            (XmlDocumentData { root, .. }, false) if root.is_none() => {
                *root = Some(XmlNode::new_document_root(gc_context, *self));
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
        proposed_doctype: XmlNode<'gc>,
    ) {
        let mut self_write = self.0.write(gc_context);

        if self_write.doctype.is_none() && proposed_doctype.is_doctype() {
            self_write.doctype = Some(proposed_doctype);
        }
    }

    /// Retrieve the first DocType node in the document.
    pub fn doctype(self) -> Option<XmlNode<'gc>> {
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

    /// Ensure that a newly-encountered node is added to an ongoing parsing
    /// stack, or to the document root itself if the parsing stack is empty.
    fn add_child_to_tree(
        &mut self,
        mc: MutationContext<'gc, '_>,
        open_tags: &mut Vec<XmlNode<'gc>>,
        child: XmlNode<'gc>,
    ) -> Result<(), Error> {
        if let Some(node) = open_tags.last_mut() {
            node.append_child(mc, child)?;
        } else {
            self.as_node().append_child(mc, child)?;
        }

        Ok(())
    }

    /// Replace the contents of this document with the result of parsing a string.
    ///
    /// This method does not yet actually remove existing node contents.
    ///
    /// If `process_entity` is `true`, then entities will be processed by this
    /// function. Invalid or unrecognized entities will cause parsing to fail
    /// with an `Err`.
    pub fn replace_with_str(
        &mut self,
        mc: MutationContext<'gc, '_>,
        data: &WStr,
        process_entity: bool,
        ignore_white: bool,
    ) -> Result<(), Error> {
        let data_utf8 = data.to_utf8_lossy();
        let mut parser = Reader::from_str(&data_utf8);
        let mut buf = Vec::new();
        let mut open_tags: Vec<XmlNode<'gc>> = Vec::new();

        self.clear_parse_error(mc);

        loop {
            let event = self.log_parse_result(mc, parser.read_event(&mut buf))?;

            self.process_event(mc, &event)?;

            match event {
                Event::Start(bs) => {
                    let child = XmlNode::from_start_event(mc, bs, *self, process_entity)?;
                    self.update_idmap(mc, child);
                    self.add_child_to_tree(mc, &mut open_tags, child)?;
                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let child = XmlNode::from_start_event(mc, bs, *self, process_entity)?;
                    self.update_idmap(mc, child);
                    self.add_child_to_tree(mc, &mut open_tags, child)?;
                }
                Event::End(_) => {
                    open_tags.pop();
                }
                Event::Text(bt) | Event::CData(bt) => {
                    let child = XmlNode::text_from_text_event(mc, bt, *self, process_entity)?;
                    if child.node_value() != Some(AvmString::default())
                        && (!ignore_white || !child.is_whitespace_text())
                    {
                        self.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::Comment(bt) => {
                    let child = XmlNode::comment_from_text_event(mc, bt, *self)?;
                    if child.node_value() != Some(AvmString::default()) {
                        self.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::DocType(bt) => {
                    let child = XmlNode::doctype_from_text_event(mc, bt, *self)?;
                    if child.node_value() != Some(AvmString::default()) {
                        self.add_child_to_tree(mc, &mut open_tags, child)?;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
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
                self_read.version.as_bytes(),
                self_read.encoding.as_ref().map(|s| s.as_bytes()),
                self_read.standalone.as_ref().map(|s| s.as_bytes()),
            );
            writer.write_event(Event::Decl(bd))?;

            Ok(Some(String::from_utf8(result)?))
        } else {
            Ok(None)
        }
    }

    /// Obtain the script object for the document's `idMap` property, or create
    /// one if it doesn't exist
    pub fn idmap_script_object(&mut self, gc_context: MutationContext<'gc, '_>) -> Object<'gc> {
        let mut object = self.0.read().idmap_script_object;
        if object.is_none() {
            object = Some(XmlIdMapObject::from_xml_document(gc_context, *self));
            self.0.write(gc_context).idmap_script_object = object;
        }

        object.unwrap()
    }

    /// Update the idmap object with a given new node.
    pub fn update_idmap(&mut self, mc: MutationContext<'gc, '_>, node: XmlNode<'gc>) {
        if let Some(id) = node.attribute_value(WStr::from_units(b"id")) {
            self.0.write(mc).idmap.insert(id, node);
        }
    }

    /// Retrieve a node from the idmap.
    ///
    /// This only retrieves nodes that had this `id` *at the time of string
    /// parsing*. Nodes which obtained the `id` after the fact, or nodes with
    /// the `id` that were added to the document after the fact, will not be
    /// returned by this function.
    pub fn get_node_by_id(self, id: AvmString<'gc>) -> Option<XmlNode<'gc>> {
        self.0.read().idmap.get(&id).copied()
    }

    /// Retrieve all IDs currently present in the idmap.
    pub fn get_node_ids(self) -> HashSet<AvmString<'gc>> {
        let mut result = HashSet::new();

        for key in self.0.read().idmap.keys() {
            result.insert(*key);
        }

        result
    }

    /// Log the result of an XML parse, saving the error for later inspection
    /// if necessary.
    pub fn log_parse_result<O>(
        self,
        gc_context: MutationContext<'gc, '_>,
        maybe_error: Result<O, QXError>,
    ) -> Result<O, ParseError> {
        match maybe_error {
            Ok(v) => Ok(v),
            Err(e) => {
                let new_error = ParseError::from_quickxml_error(e);

                self.0.write(gc_context).last_parse_error = Some(new_error.clone());

                Err(new_error)
            }
        }
    }

    /// Get the last parse error within this document, if any.
    pub fn last_parse_error(self) -> Option<ParseError> {
        self.0.read().last_parse_error.clone()
    }

    /// Clear the previous parse error.
    pub fn clear_parse_error(self, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).last_parse_error = None;
    }
}

impl<'gc> fmt::Debug for XmlDocument<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("XmlDocument")
            .field("root", &self.0.read().root)
            .finish()
    }
}
