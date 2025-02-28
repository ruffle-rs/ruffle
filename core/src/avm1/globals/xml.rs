//! XML class

use std::cell::Cell;

use crate::avm1::function::{Executable, ExecutionReason, FunctionObject};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{
    Activation, Attribute, Error, NativeObject, Object, ScriptObject, TObject, Value,
};
use crate::avm_warn;
use crate::backend::navigator::Request;
use crate::string::{AvmString, StringContext, WStr, WString};
use crate::xml::{custom_unescape, XmlNode, ELEMENT_NODE, TEXT_NODE};
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc};
use quick_xml::errors::IllFormedError;
use quick_xml::events::attributes::AttrError;
use quick_xml::{events::Event, Reader};
use ruffle_macros::istr;

#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub enum XmlStatus {
    /// No error; parse was completed successfully.
    NoError = 0,

    /// A CDATA section was not properly terminated.
    #[allow(dead_code)]
    CdataNotTerminated = -2,

    /// The XML declaration was not properly terminated.
    DeclNotTerminated = -3,

    /// The DOCTYPE declaration was not properly terminated.
    #[allow(dead_code)]
    DoctypeNotTerminated = -4,

    /// A comment was not properly terminated.
    #[allow(dead_code)]
    CommentNotTerminated = -5,

    /// An XML element was malformed.
    ElementMalformed = -6,

    /// Out of memory.
    OutOfMemory = -7,

    /// An attribute value was not properly terminated.
    AttributeNotTerminated = -8,

    /// A start-tag was not matched with an end-tag.
    #[allow(dead_code)]
    MismatchedStart = -9,

    /// An end-tag was encountered without a matching start-tag.
    MismatchedEnd = -10,
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct Xml<'gc>(Gc<'gc, XmlData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct XmlData<'gc> {
    /// The root node of the XML document.
    root: XmlNode<'gc>,

    /// The XML declaration, if set.
    xml_decl: Lock<Option<AvmString<'gc>>>,

    /// The XML doctype, if set.
    doctype: Lock<Option<AvmString<'gc>>>,

    /// The document's ID map.
    ///
    /// When nodes are parsed into the document by way of `parseXML` or the
    /// document constructor, they get put into this object, which is accessible
    /// through the document's `idMap`.
    id_map: ScriptObject<'gc>,

    /// The last parse error encountered, if any.
    status: Cell<XmlStatus>,
}

impl<'gc> Xml<'gc> {
    /// Associate an object with a new XML document.
    fn empty(context: &StringContext<'gc>, object: Object<'gc>) -> Self {
        let gc_context = context.gc();

        let mut root = XmlNode::new(gc_context, ELEMENT_NODE, None);
        root.introduce_script_object(gc_context, object);

        let xml = Self(Gc::new(
            gc_context,
            XmlData {
                root,
                xml_decl: Lock::new(None),
                doctype: Lock::new(None),
                id_map: ScriptObject::new(context, None),
                status: Cell::new(XmlStatus::NoError),
            },
        ));
        object.set_native(gc_context, NativeObject::Xml(xml));
        xml
    }

    /// Yield the document in node form.
    pub fn root(self) -> XmlNode<'gc> {
        self.0.root
    }

    /// Retrieve the XML declaration of this document.
    fn xml_decl(self) -> Option<AvmString<'gc>> {
        self.0.xml_decl.get()
    }

    /// Retrieve the first DocType node in the document.
    fn doctype(self) -> Option<AvmString<'gc>> {
        self.0.doctype.get()
    }

    /// Obtain the script object for the document's `idMap` property.
    fn id_map(self) -> ScriptObject<'gc> {
        self.0.id_map
    }

    fn status(self) -> XmlStatus {
        self.0.status.get()
    }

    /// Replace the contents of this document with the result of parsing a string.
    ///
    /// This method does not yet actually remove existing node contents.
    fn parse(
        self,
        activation: &mut Activation<'_, 'gc>,
        data: &WStr,
        ignore_white: bool,
    ) -> Result<(), quick_xml::Error> {
        let data_utf8 = data.to_utf8_lossy();
        let mut parser = Reader::from_str(&data_utf8);
        let mut open_tags = vec![self.root()];

        self.0.status.set(XmlStatus::NoError);

        loop {
            let event = parser.read_event().map_err(|error| {
                self.0.status.set(match error {
                    quick_xml::Error::Syntax(_)
                    | quick_xml::Error::InvalidAttr(AttrError::ExpectedEq(_))
                    | quick_xml::Error::InvalidAttr(AttrError::Duplicated(_, _)) => {
                        XmlStatus::ElementMalformed
                    }
                    quick_xml::Error::IllFormed(
                        IllFormedError::MismatchedEndTag { .. }
                        | IllFormedError::UnmatchedEndTag { .. },
                    ) => XmlStatus::MismatchedEnd,
                    quick_xml::Error::IllFormed(IllFormedError::MissingDeclVersion(_)) => {
                        XmlStatus::DeclNotTerminated
                    }
                    quick_xml::Error::InvalidAttr(AttrError::UnquotedValue(_)) => {
                        XmlStatus::AttributeNotTerminated
                    }
                    _ => XmlStatus::OutOfMemory,
                    // Not accounted for:
                    // quick_xml::Error::UnexpectedToken(_)
                    // quick_xml::Error::UnexpectedBang
                    // quick_xml::Error::TextNotFound
                    // quick_xml::Error::EscapeError(_)
                });
                error
            })?;

            match event {
                Event::Start(bs) => {
                    let child =
                        XmlNode::from_start_event(activation, bs, self.id_map(), parser.decoder())?;
                    open_tags
                        .last_mut()
                        .unwrap()
                        .append_child(activation.gc(), child);
                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let child =
                        XmlNode::from_start_event(activation, bs, self.id_map(), parser.decoder())?;
                    open_tags
                        .last_mut()
                        .unwrap()
                        .append_child(activation.gc(), child);
                }
                Event::End(_) => {
                    open_tags.pop();
                }
                Event::Text(bt) => {
                    Self::handle_text_cdata(
                        custom_unescape(&bt.into_inner(), parser.decoder())?.as_bytes(),
                        ignore_white,
                        &mut open_tags,
                        activation,
                    );
                }
                Event::CData(bt) => {
                    // This is already unescaped
                    Self::handle_text_cdata(
                        &bt.into_inner(),
                        ignore_white,
                        &mut open_tags,
                        activation,
                    );
                }
                Event::Decl(bd) => {
                    let mut xml_decl = WString::from_buf(b"<?".to_vec());
                    xml_decl.push_str(WStr::from_units(&*bd));
                    xml_decl.push_str(WStr::from_units(b"?>"));
                    let xml_decl = Some(AvmString::new(activation.gc(), xml_decl));
                    unlock!(Gc::write(activation.gc(), self.0), XmlData, xml_decl).set(xml_decl);
                }
                Event::DocType(bt) => {
                    // TODO: `quick-xml` is case-insensitive for DOCTYPE declarations,
                    // but it doesn't expose the whole tag, only the inner portion of it.
                    // Flash is also case-insensitive for DOCTYPE declarations. However,
                    // the `.docTypeDecl` property preserves the original case.
                    let mut doctype = WString::from_buf(b"<!DOCTYPE ".to_vec());
                    doctype.push_str(WStr::from_units(&*bt.escape_ascii().collect::<Vec<_>>()));
                    doctype.push_byte(b'>');
                    let doctype = Some(AvmString::new(activation.gc(), doctype));
                    unlock!(Gc::write(activation.gc(), self.0), XmlData, doctype).set(doctype);
                }
                Event::Eof => break,
                _ => {}
            }
        }

        self.root().refresh_cached_child_nodes(activation).unwrap(); // :(

        Ok(())
    }

    fn handle_text_cdata(
        text: &[u8],
        ignore_white: bool,
        open_tags: &mut [XmlNode<'gc>],
        activation: &mut Activation<'_, 'gc>,
    ) {
        let is_whitespace_char = |c: &u8| matches!(*c, b'\t' | b'\n' | b'\r' | b' ');
        let is_whitespace_text = text.iter().all(is_whitespace_char);
        if !(text.is_empty() || ignore_white && is_whitespace_text) {
            let text = AvmString::new_utf8_bytes(activation.gc(), text);
            let child = XmlNode::new(activation.gc(), TEXT_NODE, Some(text));
            open_tags
                .last_mut()
                .unwrap()
                .append_child(activation.gc(), child);
        }
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "docTypeDecl" => property(doc_type_decl; READ_ONLY);
    "ignoreWhite" => bool(false);
    "contentType" => string("application/x-www-form-urlencoded"; READ_ONLY);
    "xmlDecl" => property(xml_decl);
    "idMap" => property(id_map);
    "status" => property(status);
    "createElement" => method(create_element);
    "createTextNode" => method(create_text_node);
    "getBytesLoaded" => method(get_bytes_loaded);
    "getBytesTotal" => method(get_bytes_total);
    "parseXML" => method(parse_xml);
    "load" => method(load);
    "sendAndLoad" => method(send_and_load);
    "onData" => method(on_data);
};

/// XML (document) constructor
fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = Xml::empty(&activation.context.strings, this);

    if let [text, ..] = args {
        let text = text.coerce_to_string(activation)?;

        let ignore_whitespace = this
            .get(istr!("ignoreWhite"), activation)?
            .as_bool(activation.swf_version());

        if let Err(e) = xml.parse(activation, &text, ignore_whitespace) {
            avm_warn!(activation, "XML parsing error: {}", e);
        }
    }

    Ok(this.into())
}

fn create_element<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (NativeObject::Xml(_), [name, ..]) = (this.native(), args) {
        let name = name.coerce_to_string(activation)?;
        let mut node = XmlNode::new(activation.gc(), ELEMENT_NODE, Some(name));
        return Ok(node.script_object(activation).into());
    }

    Ok(Value::Undefined)
}

fn create_text_node<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (NativeObject::Xml(_), [text, ..]) = (this.native(), args) {
        let text = text.coerce_to_string(activation)?;
        let mut node = XmlNode::new(activation.gc(), TEXT_NODE, Some(text));
        return Ok(node.script_object(activation).into());
    }

    Ok(Value::Undefined)
}

fn get_bytes_loaded<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Forwards to undocumented property on the object.
    this.get(istr!("_bytesLoaded"), activation)
}

fn get_bytes_total<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Forwards to undocumented property on the object.
    this.get(istr!("_bytesTotal"), activation)
}

fn parse_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Xml(xml) = this.native() {
        for mut child in xml.root().children().rev() {
            child.remove_node(activation.gc());
        }

        if let [text, ..] = args {
            let text = text.coerce_to_string(activation)?;

            let ignore_whitespace = this
                .get(istr!("ignoreWhite"), activation)?
                .as_bool(activation.swf_version());

            let result = xml.parse(activation, &text, ignore_whitespace);
            if let Err(e) = result {
                avm_warn!(activation, "XML parsing error: {}", e);
            }
        }
    }

    Ok(Value::Undefined)
}

fn send_and_load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Null = url_val {
        return Ok(Value::Undefined);
    }

    let target = match args.get(1) {
        Some(&Value::Object(o)) => o,
        _ => return Ok(Value::Undefined),
    };

    if let NativeObject::Xml(xml) = this.native() {
        let url = url_val.coerce_to_string(activation)?;
        spawn_xml_fetch(activation, this, target, url, Some(xml.root()))?;
    }
    Ok(Value::Undefined)
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Null = url_val {
        return Ok(false.into());
    }

    if let NativeObject::Xml(_) = this.native() {
        let url = url_val.coerce_to_string(activation)?;
        spawn_xml_fetch(activation, this, this, url, None)?;

        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

fn on_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let src = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Undefined = src {
        this.call_method(
            istr!("onLoad"),
            &[false.into()],
            activation,
            ExecutionReason::FunctionCall,
        )?;
    } else {
        let src = src.coerce_to_string(activation)?;
        this.call_method(
            istr!("parseXML"),
            &[src.into()],
            activation,
            ExecutionReason::FunctionCall,
        )?;

        this.set(istr!("loaded"), true.into(), activation)?;

        this.call_method(
            istr!("onLoad"),
            &[true.into()],
            activation,
            ExecutionReason::FunctionCall,
        )?;
    }

    Ok(Value::Undefined)
}

fn doc_type_decl<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Xml(xml) = this.native() {
        if let Some(doctype) = xml.doctype() {
            return Ok(doctype.into());
        }
    }

    Ok(Value::Undefined)
}

fn xml_decl<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Xml(xml) = this.native() {
        if let Some(xml_decl) = xml.xml_decl() {
            return Ok(xml_decl.into());
        }
    }

    Ok(Value::Undefined)
}

fn id_map<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Xml(xml) = this.native() {
        return Ok(xml.id_map().into());
    }

    Ok(Value::Undefined)
}

fn status<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::Xml(xml) = this.native() {
        return Ok((xml.status() as i8).into());
    }

    Ok(Value::Undefined)
}

fn spawn_xml_fetch<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    loader_object: Object<'gc>,
    url: AvmString<'gc>,
    send_object: Option<XmlNode<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    let url = url.to_utf8_lossy().into_owned();

    let request = if let Some(node) = send_object {
        // Send `node` as string.
        let string = node.into_string(activation)?;
        Request::post(
            url,
            Some((
                string.to_utf8_lossy().into_owned().into_bytes(),
                "application/x-www-form-urlencoded".to_string(),
            )),
        )
    } else {
        // Not sending any parameters.
        Request::get(url)
    };

    // Create hidden properties on object.
    let bytes_loaded_string = istr!("_bytesLoaded");

    if !this.has_property(activation, bytes_loaded_string) {
        this.define_value(
            activation.gc(),
            bytes_loaded_string,
            0.into(),
            Attribute::DONT_DELETE | Attribute::DONT_ENUM,
        );
    } else {
        this.set(bytes_loaded_string, 0.into(), activation)?;
    }

    let bytes_total_string = istr!("_bytesTotal");

    if !this.has_property(activation, bytes_total_string) {
        this.define_value(
            activation.gc(),
            bytes_total_string,
            Value::Undefined,
            Attribute::DONT_DELETE | Attribute::DONT_ENUM,
        );
    } else {
        this.set(bytes_total_string, Value::Undefined, activation)?;
    }

    let loaded_string = istr!("loaded");

    if !this.has_property(activation, loaded_string) {
        this.define_value(
            activation.gc(),
            loaded_string,
            false.into(),
            Attribute::DONT_DELETE | Attribute::DONT_ENUM,
        );
    } else {
        this.set(loaded_string, false.into(), activation)?;
    }

    let future = activation.context.load_manager.load_form_into_load_vars(
        activation.context.player.clone(),
        loader_object,
        request,
    );
    activation.context.navigator.spawn_future(future);

    Ok(true.into())
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_proto = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, xml_proto, fn_proto);
    FunctionObject::constructor(
        context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        xml_proto.into(),
    )
}
