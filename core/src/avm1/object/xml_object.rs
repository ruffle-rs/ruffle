//! AVM1 object type to represent XML documents

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use crate::string::{AvmString, WStr, WString};
use crate::xml::{Error as XmlError, ParseError, XmlNode};
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::events::{BytesDecl, Event};
use quick_xml::{Error as QXError, Reader, Writer};
use std::fmt;
use std::io::Cursor;

/// A ScriptObject that is inherently tied to an XML document.
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct XmlObject<'gc>(GcCell<'gc, XmlObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlObjectData<'gc> {
    base: ScriptObject<'gc>,

    /// The root node of the XML document.
    root: XmlNode<'gc>,

    /// Whether or not the document has a document declaration.
    has_xmldecl: bool,

    /// The XML version string, if set.
    version: String,

    /// The XML document encoding, if set.
    encoding: Option<String>,

    /// The XML standalone flag, if set.
    standalone: Option<String>,

    /// The XML doctype, if set.
    doctype: Option<AvmString<'gc>>,

    /// The document's ID map.
    ///
    /// When nodes are parsed into the document by way of `parseXML` or the
    /// document constructor, they get put into this object, which is accessible
    /// through the document's `idMap`.
    id_map: ScriptObject<'gc>,

    /// The last parse error encountered, if any.
    last_parse_error: Option<ParseError>,
}

impl<'gc> XmlObject<'gc> {
    /// Construct a new XML document and object pair.
    pub fn empty(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        let mut root = XmlNode::new_document_root(gc_context);
        let object = Self(GcCell::allocate(
            gc_context,
            XmlObjectData {
                base: ScriptObject::object(gc_context, proto),
                root,
                has_xmldecl: false,
                version: "1.0".to_string(),
                encoding: None,
                standalone: None,
                doctype: None,
                id_map: ScriptObject::bare_object(gc_context),
                last_parse_error: None,
            },
        ));
        root.introduce_script_object(gc_context, object.into());
        object
    }

    /// Yield the document in node form.
    pub fn as_node(self) -> XmlNode<'gc> {
        self.0.read().root
    }

    /// Retrieve the first DocType node in the document.
    pub fn doctype(self) -> Option<AvmString<'gc>> {
        self.0.read().doctype
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
        activation: &mut Activation<'_, 'gc, '_>,
        data: &WStr,
        process_entity: bool,
        ignore_white: bool,
    ) -> Result<(), XmlError> {
        let data_utf8 = data.to_utf8_lossy();
        let mut parser = Reader::from_str(&data_utf8);
        let mut buf = Vec::new();
        let mut open_tags = vec![self.as_node()];

        self.clear_parse_error(activation.context.gc_context);

        loop {
            let event =
                self.log_parse_result(activation.context.gc_context, parser.read_event(&mut buf))?;

            match event {
                Event::Start(bs) => {
                    let child = XmlNode::from_start_event(
                        activation.context.gc_context,
                        bs,
                        process_entity,
                    )?;
                    self.update_id_map(activation, child);
                    open_tags
                        .last_mut()
                        .unwrap()
                        .append_child(activation.context.gc_context, child)?;
                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let child = XmlNode::from_start_event(
                        activation.context.gc_context,
                        bs,
                        process_entity,
                    )?;
                    self.update_id_map(activation, child);
                    open_tags
                        .last_mut()
                        .unwrap()
                        .append_child(activation.context.gc_context, child)?;
                }
                Event::End(_) => {
                    open_tags.pop();
                }
                Event::Text(bt) | Event::CData(bt) => {
                    let child = XmlNode::text_from_text_event(
                        activation.context.gc_context,
                        bt,
                        process_entity,
                    )?;
                    if child.node_value() != Some(AvmString::default())
                        && (!ignore_white || !child.is_whitespace_text())
                    {
                        open_tags
                            .last_mut()
                            .unwrap()
                            .append_child(activation.context.gc_context, child)?;
                    }
                }
                Event::DocType(bt) => {
                    // TODO: `quick-xml` is case-insensitive for DOCTYPE declarations,
                    // but it doesn't expose the whole tag, only the inner portion of it.
                    // Flash is also case-insensitive for DOCTYPE declarations. However,
                    // the `.docTypeDecl` property preserves the original case.
                    let mut doctype = WString::from_buf(b"<!DOCTYPE".to_vec());
                    doctype.push_str(WStr::from_units(bt.escaped()));
                    doctype.push_byte(b'>');
                    self.0.write(activation.context.gc_context).doctype =
                        Some(AvmString::new(activation.context.gc_context, doctype));
                }
                Event::Decl(bd) => {
                    let mut self_write = self.0.write(activation.context.gc_context);

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
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// Generate a string matching the XML document declaration, if there is
    /// one.
    pub fn xmldecl_string(self) -> Result<Option<String>, XmlError> {
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

    /// Obtain the script object for the document's `idMap` property.
    pub fn id_map(self) -> ScriptObject<'gc> {
        self.0.read().id_map
    }

    /// Update the ID map object with a given new node.
    fn update_id_map(&mut self, activation: &mut Activation<'_, 'gc, '_>, mut node: XmlNode<'gc>) {
        if let Some(id) = node.attribute_value(WStr::from_units(b"id")) {
            self.0
                .write(activation.context.gc_context)
                .id_map
                .define_value(
                    activation.context.gc_context,
                    id,
                    node.script_object(activation).into(),
                    Attribute::empty(),
                );
        }
    }

    /// Log the result of an XML parse, saving the error for later inspection
    /// if necessary.
    fn log_parse_result<O>(
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
    fn clear_parse_error(self, gc_context: MutationContext<'gc, '_>) {
        self.0.write(gc_context).last_parse_error = None;
    }
}

impl fmt::Debug for XmlObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("XmlObject")
            .field("base", &this.base)
            .field("root", &self.0.read().root)
            .finish()
    }
}

impl<'gc> TObject<'gc> for XmlObject<'gc> {
    impl_custom_object!(base);

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(Self::empty(activation.context.gc_context, Some(this)).into())
    }

    fn as_xml(&self) -> Option<XmlObject<'gc>> {
        Some(*self)
    }

    fn as_xml_node(&self) -> Option<XmlNode<'gc>> {
        Some(self.as_node())
    }
}
