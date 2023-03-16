use std::{
    cell::{Ref, RefMut},
    fmt::{self, Debug},
};

use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::avm2::{error::type_error, TObject};

use super::{object::E4XOrXml, string::AvmString, Activation, Error, Multiname, Value};
use crate::string::{WStr, WString};

/// The underlying XML node data, based on E4XNode in avmplus
/// This wrapped by XMLObject when necessary (see `E4XOrXml`)
#[derive(Copy, Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct E4XNode<'gc>(GcCell<'gc, E4XNodeData<'gc>>);

#[derive(Collect)]
#[collect(no_drop)]
pub struct E4XNodeData<'gc> {
    parent: Option<E4XNode<'gc>>,
    local_name: Option<AvmString<'gc>>,
    kind: E4XNodeKind<'gc>,
}

impl<'gc> Debug for E4XNodeData<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("E4XNodeData")
            // Don't print the actual parent, to avoid infinite recursion
            .field("parent", &self.parent.is_some())
            .field("local_name", &self.local_name)
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Collect, Debug)]
#[collect(no_drop)]
pub enum E4XNodeKind<'gc> {
    Text(AvmString<'gc>),
    CData(AvmString<'gc>),
    Comment(AvmString<'gc>),
    ProcessingInstruction(AvmString<'gc>),
    Attribute(AvmString<'gc>),
    Element {
        attributes: Vec<E4XNode<'gc>>,
        children: Vec<E4XNode<'gc>>,
    },
}

impl<'gc> E4XNode<'gc> {
    pub fn dummy(mc: MutationContext<'gc, '_>) -> Self {
        E4XNode(GcCell::allocate(
            mc,
            E4XNodeData {
                parent: None,
                local_name: None,
                kind: E4XNodeKind::Element {
                    attributes: vec![],
                    children: vec![],
                },
            },
        ))
    }

    pub fn text(mc: MutationContext<'gc, '_>, text: AvmString<'gc>) -> Self {
        E4XNode(GcCell::allocate(
            mc,
            E4XNodeData {
                parent: None,
                local_name: None,
                kind: E4XNodeKind::Text(text),
            },
        ))
    }

    pub fn attribute(
        mc: MutationContext<'gc, '_>,
        name: AvmString<'gc>,
        value: AvmString<'gc>,
    ) -> Self {
        E4XNode(GcCell::allocate(
            mc,
            E4XNodeData {
                parent: None,
                local_name: Some(name),
                kind: E4XNodeKind::Attribute(value),
            },
        ))
    }

    pub fn append_child(
        &self,
        gc_context: MutationContext<'gc, '_>,
        child: Self,
    ) -> Result<(), Error<'gc>> {
        let mut this = self.0.write(gc_context);
        let mut child_data = match child.0.try_write(gc_context) {
            Ok(data) => data,
            Err(_) => {
                return Err(Error::RustError(
                    format!(
                        "Circular write in append_child with self={:?} child={:?}",
                        self, child
                    )
                    .into(),
                ))
            }
        };

        child_data.parent = Some(*self);

        match &mut this.kind {
            E4XNodeKind::Element { children, .. } => {
                children.push(child);
            }
            _ => {
                // FIXME - figure out exactly when appending is allowed in FP,
                // and throw the proper AVM error.
                return Err(Error::RustError(
                    format!("Cannot append child {child:?} to node {:?}", this.kind).into(),
                ));
            }
        }
        Ok(())
    }

    /// Parses a value provided to `XML`/`XMLList` into a list of nodes.
    /// The caller is responsible for validating that the number of top-level nodes
    /// is correct (for XML, there should be exactly one.)
    pub fn parse(
        mut value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Vec<Self>, Error<'gc>> {
        let string = match &value {
            // The docs claim that this throws a TypeError, but it actually doesn't
            Value::Null | Value::Undefined => AvmString::default(),
            // The docs claim that only String, Number or Boolean are accepted, but that's also a lie
            val => {
                if let Some(obj) = val.as_object() {
                    if let Some(xml) = obj.as_xml_object() {
                        value = xml.call_public_property("toXMLString", &[], activation)?;
                    } else if let Some(list) = obj.as_xml_list_object() {
                        if list.length() == 1 {
                            value = list.children_mut(activation.context.gc_context)[0]
                                .get_or_create_xml(activation)
                                .call_public_property("toXMLString", &[], activation)?;
                        } else {
                            return Err(Error::AvmError(type_error(activation, "Error #1088: The markup in the document following the root element must be well-formed.", 1088)?));
                        }
                    }
                }
                value.coerce_to_string(activation)?
            }
        };

        let data_utf8 = string.to_utf8_lossy();
        let mut parser = Reader::from_str(&data_utf8);
        let mut buf = Vec::new();
        let mut open_tags: Vec<E4XNode<'gc>> = vec![];

        // FIXME - look these up from static property and settings
        let ignore_comments = true;
        let ignore_processing_instructions = true;
        let ignore_white = true;

        let mut top_level = vec![];
        let mut depth = 0;

        // This can't be a closure that captures these variables, because we need to modify them
        // outside of this body.
        fn push_childless_node<'gc>(
            node: E4XNode<'gc>,
            open_tags: &mut [E4XNode<'gc>],
            top_level: &mut Vec<E4XNode<'gc>>,
            depth: usize,
            activation: &mut Activation<'_, 'gc>,
        ) -> Result<(), Error<'gc>> {
            if let Some(current_tag) = open_tags.last_mut() {
                current_tag.append_child(activation.context.gc_context, node)?;
            }

            if depth == 0 {
                top_level.push(node);
            }
            Ok(())
        }

        loop {
            let event = parser.read_event(&mut buf).map_err(|error| {
                Error::RustError(format!("XML parsing error: {error:?}").into())
            })?;

            match &event {
                Event::Start(bs) => {
                    let child = E4XNode::from_start_event(activation, bs)?;

                    if let Some(current_tag) = open_tags.last_mut() {
                        current_tag.append_child(activation.context.gc_context, child)?;
                    }
                    open_tags.push(child);
                    depth += 1;
                }
                Event::Empty(bs) => {
                    let node = E4XNode::from_start_event(activation, bs)?;
                    push_childless_node(node, &mut open_tags, &mut top_level, depth, activation)?;
                }
                Event::End(_) => {
                    depth -= 1;
                    let node = open_tags.pop().unwrap();
                    if depth == 0 {
                        top_level.push(node);
                    }
                }
                Event::Text(bt) | Event::CData(bt) => {
                    let text = bt.unescaped()?;
                    let is_whitespace_char = |c: &u8| matches!(*c, b'\t' | b'\n' | b'\r' | b' ');
                    let is_whitespace_text = text.iter().all(is_whitespace_char);
                    if !(text.is_empty() || ignore_white && is_whitespace_text) {
                        let text = AvmString::new_utf8_bytes(activation.context.gc_context, &text);
                        let node = E4XNode(GcCell::allocate(
                            activation.context.gc_context,
                            E4XNodeData {
                                parent: None,
                                local_name: None,
                                kind: match &event {
                                    Event::Text(_) => E4XNodeKind::Text(text),
                                    Event::CData(_) => E4XNodeKind::CData(text),
                                    _ => unreachable!(),
                                },
                            },
                        ));
                        push_childless_node(
                            node,
                            &mut open_tags,
                            &mut top_level,
                            depth,
                            activation,
                        )?;
                    }
                }
                Event::Comment(bt) | Event::PI(bt) => {
                    if (matches!(event, Event::Comment(_)) && ignore_comments)
                        || (matches!(event, Event::PI(_)) && ignore_processing_instructions)
                    {
                        continue;
                    }
                    let text = bt.unescaped()?;
                    let text = AvmString::new_utf8_bytes(activation.context.gc_context, &text);
                    let kind = match event {
                        Event::Comment(_) => E4XNodeKind::Comment(text),
                        Event::PI(_) => E4XNodeKind::ProcessingInstruction(text),
                        _ => unreachable!(),
                    };
                    let node = E4XNode(GcCell::allocate(
                        activation.context.gc_context,
                        E4XNodeData {
                            parent: None,
                            local_name: None,
                            kind,
                        },
                    ));

                    push_childless_node(node, &mut open_tags, &mut top_level, depth, activation)?;
                }
                // These are completely ignored by AVM2
                Event::Decl(_) | Event::DocType(_) => {}
                Event::Eof => break,
            }
        }
        Ok(top_level)
    }

    /// Construct an XML Element node from a `quick_xml` `BytesStart` event.
    ///
    /// The returned node will always be an `Element`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
    pub fn from_start_event(
        activation: &mut Activation<'_, 'gc>,
        bs: &BytesStart<'_>,
    ) -> Result<Self, quick_xml::Error> {
        // FIXME - handle namespace
        let name = AvmString::new_utf8_bytes(activation.context.gc_context, bs.local_name());

        let mut attribute_nodes = Vec::new();

        let attributes: Result<Vec<_>, _> = bs.attributes().collect();
        for attribute in attributes? {
            let key = AvmString::new_utf8_bytes(activation.context.gc_context, attribute.key);
            let value_bytes = attribute.unescaped_value()?;
            let value = AvmString::new_utf8_bytes(activation.context.gc_context, &value_bytes);

            let attribute_data = E4XNodeData {
                parent: None,
                local_name: Some(key),
                kind: E4XNodeKind::Attribute(value),
            };
            let attribute = E4XNode(GcCell::allocate(
                activation.context.gc_context,
                attribute_data,
            ));
            attribute_nodes.push(attribute);
        }

        let data = E4XNodeData {
            parent: None,
            local_name: Some(name),
            kind: E4XNodeKind::Element {
                attributes: attribute_nodes,
                children: Vec::new(),
            },
        };

        Ok(E4XNode(GcCell::allocate(
            activation.context.gc_context,
            data,
        )))
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.0.read().local_name
    }

    pub fn matches_name(&self, name: &Multiname<'gc>) -> bool {
        // FIXME - we need to handle namespaces here
        if name.is_any_name() {
            return true;
        }

        if let Some(local_name) = self.local_name() {
            Some(local_name) == name.local_name()
        } else {
            false
        }
    }

    pub fn descendants(&self, name: &Multiname<'gc>, out: &mut Vec<E4XOrXml<'gc>>) {
        if let E4XNodeKind::Element { children, .. } = &self.0.read().kind {
            for child in children {
                if child.matches_name(name) {
                    out.push(E4XOrXml::E4X(*child));
                }
                child.descendants(name, out)
            }
        }
    }

    pub fn has_simple_content(&self) -> bool {
        match &self.0.read().kind {
            E4XNodeKind::Element { children, .. } => children
                .iter()
                .all(|child| !matches!(&*child.kind(), E4XNodeKind::Element { .. })),
            E4XNodeKind::Text(_) | E4XNodeKind::CData(_) => true,
            E4XNodeKind::Attribute(_) => true,
            E4XNodeKind::Comment(_) => false,
            E4XNodeKind::ProcessingInstruction(_) => false,
        }
    }

    pub fn xml_to_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        match &self.0.read().kind {
            E4XNodeKind::Text(text) | E4XNodeKind::CData(text) => Ok(*text),
            E4XNodeKind::Attribute(text) => Ok(*text),
            E4XNodeKind::Element { children, .. } => {
                if self.has_simple_content() {
                    return simple_content_to_string(
                        children.iter().map(|node| E4XOrXml::E4X(*node)),
                        activation,
                    );
                }

                return to_xml_string(E4XOrXml::E4X(*self), activation);
            }
            other => Err(format!("XML.toString(): Not yet implemented for {other:?}").into()),
        }
    }

    pub fn xml_to_xml_string(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<AvmString<'gc>, Error<'gc>> {
        return to_xml_string(E4XOrXml::E4X(*self), activation);
    }

    pub fn kind(&self) -> Ref<'_, E4XNodeKind<'gc>> {
        Ref::map(self.0.read(), |r| &r.kind)
    }

    pub fn kind_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<'_, E4XNodeKind<'gc>> {
        RefMut::map(self.0.write(mc), |r| &mut r.kind)
    }

    pub fn ptr_eq(first: E4XNode<'gc>, second: E4XNode<'gc>) -> bool {
        GcCell::ptr_eq(first.0, second.0)
    }
}

pub fn simple_content_to_string<'gc>(
    children: impl Iterator<Item = E4XOrXml<'gc>>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<AvmString<'gc>, Error<'gc>> {
    let mut out = AvmString::default();
    for child in children {
        if matches!(
            &*child.node().kind(),
            E4XNodeKind::Comment(_) | E4XNodeKind::ProcessingInstruction(_)
        ) {
            continue;
        }
        let child_str = child.node().xml_to_string(activation)?;
        out = AvmString::concat(activation.context.gc_context, out, child_str);
    }
    Ok(out)
}

// Implementation of `EscapeAttributeValue` from ECMA-357 (10.2.1.2)
pub fn escape_attribute_value(s: AvmString) -> WString {
    let mut r = WString::with_capacity(s.len(), s.is_wide());
    for c in &s {
        let escape: &[u8] = match u8::try_from(c) {
            Ok(b'"') => b"&quot;",
            Ok(b'<') => b"&lt;",
            Ok(b'&') => b"&amp;",
            Ok(b'\x0A') => b"&#xA;",
            Ok(b'\x0D') => b"&#xD;",
            Ok(b'\x09') => b"&#x9;",
            _ => {
                r.push(c);
                continue;
            }
        };

        r.push_str(WStr::from_units(escape));
    }
    r
}

// Implementation of `EscapeElementValue` from ECMA-357 (10.2.1.1)
pub fn escape_element_value(s: AvmString) -> WString {
    let mut r = WString::with_capacity(s.len(), s.is_wide());
    for c in &s {
        let escape: &[u8] = match u8::try_from(c) {
            Ok(b'<') => b"&lt;",
            Ok(b'>') => b"&gt;",
            Ok(b'&') => b"&amp;",
            _ => {
                r.push(c);
                continue;
            }
        };

        r.push_str(WStr::from_units(escape));
    }
    r
}

fn to_xml_string_inner<'gc>(xml: E4XOrXml<'gc>, buf: &mut WString) -> Result<(), Error<'gc>> {
    // FIXME: Implement pretty printing and namespace support.

    let node = xml.node();
    let node_kind = node.kind();
    let (children, attributes) = match &*node_kind {
        E4XNodeKind::Text(text) => {
            buf.push_str(&escape_element_value(*text));
            return Ok(());
        }
        E4XNodeKind::Attribute(_)
        | E4XNodeKind::Comment(_)
        | E4XNodeKind::ProcessingInstruction(_)
        | E4XNodeKind::CData(_) => {
            return Err(format!("ToXMLString: Not yet implemented node {:?}", node_kind).into())
        }
        E4XNodeKind::Element {
            children,
            attributes,
        } => (children, attributes),
    };

    buf.push_char('<');
    buf.push_str(&node.local_name().unwrap());

    for attribute in attributes {
        if let E4XNodeKind::Attribute(value) = &*attribute.kind() {
            buf.push_char(' ');
            buf.push_str(&attribute.local_name().unwrap());
            buf.push_char('=');
            buf.push_char('"');
            buf.push_str(&escape_attribute_value(*value));
            buf.push_char('"');
        }
    }

    if children.is_empty() {
        buf.push_utf8("/>");
        return Ok(());
    }

    buf.push_char('>');

    for child in children {
        to_xml_string_inner(E4XOrXml::E4X(*child), buf)?;
    }

    buf.push_utf8("</");
    buf.push_str(&node.local_name().unwrap());
    buf.push_char('>');

    Ok(())
}

// Implementation of `ToXMLString` from ECMA-357 (10.2.1)
pub fn to_xml_string<'gc>(
    xml: E4XOrXml<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<AvmString<'gc>, Error<'gc>> {
    let mut buf = WString::new();
    to_xml_string_inner(xml, &mut buf)?;
    Ok(AvmString::new(activation.context.gc_context, buf))
}
