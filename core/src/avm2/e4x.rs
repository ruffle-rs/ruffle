use std::{
    cell::{Ref, RefMut},
    fmt::{self, Debug},
};

use gc_arena::{Collect, GcCell, Mutation};
use quick_xml::{
    events::{BytesStart, Event},
    name::ResolveResult,
    NsReader,
};

use crate::{avm2::TObject, xml::custom_unescape};

use super::{
    error::type_error, object::E4XOrXml, string::AvmString, Activation, Error, Multiname, Value,
};
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
    namespace: Option<AvmString<'gc>>,
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

fn malformed_element<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    Error::AvmError(
        type_error(
            activation,
            "Error #1090: XML parser failure: element is malformed.",
            1090,
        )
        .expect("Failed to construct XML TypeError"),
    )
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
    pub fn dummy(mc: &Mutation<'gc>) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent: None,
                namespace: None,
                local_name: None,
                kind: E4XNodeKind::Element {
                    attributes: vec![],
                    children: vec![],
                },
            },
        ))
    }

    pub fn text(mc: &Mutation<'gc>, text: AvmString<'gc>, parent: Option<Self>) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent,
                namespace: None,
                local_name: None,
                kind: E4XNodeKind::Text(text),
            },
        ))
    }

    pub fn element(
        mc: &Mutation<'gc>,
        namespace: Option<AvmString<'gc>>,
        name: AvmString<'gc>,
        parent: Self,
    ) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent: Some(parent),
                namespace,
                local_name: Some(name),
                kind: E4XNodeKind::Element {
                    attributes: vec![],
                    children: vec![],
                },
            },
        ))
    }

    pub fn attribute(
        mc: &Mutation<'gc>,
        name: AvmString<'gc>,
        value: AvmString<'gc>,
        parent: E4XNode<'gc>,
    ) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent: Some(parent),
                namespace: None,
                local_name: Some(name),
                kind: E4XNodeKind::Attribute(value),
            },
        ))
    }

    pub fn equals(&self, other: &Self) -> bool {
        if self.local_name() != other.local_name() {
            return false;
        }

        let this = self.0.read();
        let other = other.0.read();

        match (&this.kind, &other.kind) {
            (
                E4XNodeKind::Text(a) | E4XNodeKind::CData(a),
                E4XNodeKind::Text(b) | E4XNodeKind::CData(b),
            ) => a == b,
            (E4XNodeKind::Comment(a), E4XNodeKind::Comment(b)) => a == b,
            (E4XNodeKind::ProcessingInstruction(a), E4XNodeKind::ProcessingInstruction(b)) => {
                a == b
            }
            (E4XNodeKind::Attribute(a), E4XNodeKind::Attribute(b)) => a == b,
            (
                E4XNodeKind::Element {
                    children: children_a,
                    attributes: attributes_a,
                },
                E4XNodeKind::Element {
                    children: children_b,
                    attributes: attributes_b,
                },
            ) => {
                if children_a.len() != children_b.len() || attributes_a.len() != attributes_b.len()
                {
                    return false;
                }

                // The attributes can be in a different order.
                for attr_a in attributes_a {
                    if !attributes_b.iter().any(|attr_b| attr_a.equals(attr_b)) {
                        return false;
                    }
                }

                children_a
                    .iter()
                    .zip(children_b.iter())
                    .all(|(a, b)| a.equals(b))
            }
            _ => false,
        }
    }

    pub fn deep_copy(&self, mc: &Mutation<'gc>) -> Self {
        let this = self.0.read();

        let kind = match &this.kind {
            E4XNodeKind::Text(string) => E4XNodeKind::Text(*string),
            E4XNodeKind::CData(string) => E4XNodeKind::CData(*string),
            E4XNodeKind::Comment(string) => E4XNodeKind::Comment(*string),
            E4XNodeKind::ProcessingInstruction(string) => {
                E4XNodeKind::ProcessingInstruction(*string)
            }
            E4XNodeKind::Attribute(string) => E4XNodeKind::Attribute(*string),
            E4XNodeKind::Element {
                attributes,
                children,
            } => E4XNodeKind::Element {
                attributes: attributes.iter().map(|attr| attr.deep_copy(mc)).collect(),
                children: children.iter().map(|child| child.deep_copy(mc)).collect(),
            },
        };

        let node = E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent: None,
                namespace: this.namespace,
                local_name: this.local_name,
                kind,
            },
        ));

        if let E4XNodeKind::Element {
            attributes,
            children,
        } = &mut node.0.write(mc).kind
        {
            for attr in attributes.iter_mut() {
                let mut data = attr.0.write(mc);
                data.parent = Some(node);
            }

            for child in children.iter_mut() {
                let mut data = child.0.write(mc);
                data.parent = Some(node);
            }
        }

        node
    }

    pub fn remove_all_children(&self, gc_context: &Mutation<'gc>) {
        let mut this = self.0.write(gc_context);
        if let E4XNodeKind::Element { children, .. } = &mut this.kind {
            for child in children.iter_mut() {
                let mut child_data = child.0.write(gc_context);
                child_data.parent = None;
            }
            children.clear()
        }
    }

    pub fn remove_child(&self, gc_context: &Mutation<'gc>, child: &Self) {
        let mut this = self.0.write(gc_context);
        if let E4XNodeKind::Element { children, .. } = &mut this.kind {
            children.retain(|c| !GcCell::ptr_eq(c.0, child.0));
        }
    }

    pub fn remove_attribute(&self, gc_context: &Mutation<'gc>, attribute: &Self) {
        let mut this = self.0.write(gc_context);
        if let E4XNodeKind::Element { attributes, .. } = &mut this.kind {
            attributes.retain(|a| !GcCell::ptr_eq(a.0, attribute.0));
        }
    }

    pub fn append_child(&self, gc_context: &Mutation<'gc>, child: Self) -> Result<(), Error<'gc>> {
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
        ignore_comments: bool,
        ignore_processing_instructions: bool,
        ignore_white: bool,
    ) -> Result<Vec<Self>, Error<'gc>> {
        let string = match &value {
            // The docs claim that this throws a TypeError, but it actually doesn't
            Value::Null | Value::Undefined => AvmString::default(),
            // The docs claim that only String, Number or Boolean are accepted, but that's also a lie
            val => {
                if let Some(obj) = val.as_object() {
                    if obj.as_xml_object().is_some() || obj.as_xml_list_object().is_some() {
                        value = obj.call_public_property("toXMLString", &[], activation)?;
                    }
                }
                value.coerce_to_string(activation)?
            }
        };

        let data_utf8 = string.to_utf8_lossy();
        let mut parser = NsReader::from_str(&data_utf8);
        let mut open_tags: Vec<E4XNode<'gc>> = vec![];

        let mut top_level = vec![];

        // This can't be a closure that captures these variables, because we need to modify them
        // outside of this body.
        fn push_childless_node<'gc>(
            node: E4XNode<'gc>,
            open_tags: &mut [E4XNode<'gc>],
            top_level: &mut Vec<E4XNode<'gc>>,
            activation: &mut Activation<'_, 'gc>,
        ) -> Result<(), Error<'gc>> {
            if let Some(current_tag) = open_tags.last_mut() {
                current_tag.append_child(activation.context.gc_context, node)?;
            }

            if open_tags.is_empty() {
                top_level.push(node);
            }
            Ok(())
        }

        // Inbuilt trim_ascii is behind an unstable feature
        // So the method was moved out in order to use it for the time being
        const fn trim_ascii(bytes: &[u8]) -> &[u8] {
            let mut bytes = bytes;
            while let [first, rest @ ..] = bytes {
                if first.is_ascii_whitespace() {
                    bytes = rest;
                } else {
                    break;
                }
            }
            while let [rest @ .., last] = bytes {
                if last.is_ascii_whitespace() {
                    bytes = rest;
                } else {
                    break;
                }
            }

            bytes
        }

        fn handle_text_cdata<'gc>(
            text: &[u8],
            ignore_white: bool,
            open_tags: &mut [E4XNode<'gc>],
            top_level: &mut Vec<E4XNode<'gc>>,
            is_text: bool,
            activation: &mut Activation<'_, 'gc>,
        ) -> Result<(), Error<'gc>> {
            let is_whitespace_char = |c: &u8| matches!(*c, b'\t' | b'\n' | b'\r' | b' ');
            let is_whitespace_text = text.iter().all(is_whitespace_char);
            if !(is_text && ignore_white && is_whitespace_text) {
                let text = AvmString::new_utf8_bytes(
                    activation.context.gc_context,
                    if is_text && ignore_white {
                        trim_ascii(text)
                    } else {
                        text
                    },
                );
                let node = E4XNode(GcCell::new(
                    activation.context.gc_context,
                    E4XNodeData {
                        parent: None,
                        namespace: None,
                        local_name: None,
                        kind: if is_text {
                            E4XNodeKind::Text(text)
                        } else {
                            E4XNodeKind::CData(text)
                        },
                    },
                ));
                push_childless_node(node, open_tags, top_level, activation)?;
            }
            Ok(())
        }

        loop {
            let event = parser
                .read_event()
                .map_err(|_| malformed_element(activation))?;

            match &event {
                Event::Start(bs) => {
                    let child =
                        E4XNode::from_start_event(activation, &parser, bs, parser.decoder())?;

                    if let Some(current_tag) = open_tags.last_mut() {
                        current_tag.append_child(activation.context.gc_context, child)?;
                    }
                    open_tags.push(child);
                }
                Event::Empty(bs) => {
                    let node =
                        E4XNode::from_start_event(activation, &parser, bs, parser.decoder())?;
                    push_childless_node(node, &mut open_tags, &mut top_level, activation)?;
                }
                Event::End(_) => {
                    let node = open_tags.pop().unwrap();
                    if open_tags.is_empty() {
                        top_level.push(node);
                    }
                }
                Event::Text(bt) => {
                    handle_text_cdata(
                        custom_unescape(bt, parser.decoder())
                            .map_err(|_| malformed_element(activation))?
                            .as_bytes(),
                        ignore_white,
                        &mut open_tags,
                        &mut top_level,
                        true,
                        activation,
                    )?;
                }
                Event::CData(bt) => {
                    // This is already unescaped
                    handle_text_cdata(
                        bt,
                        ignore_white,
                        &mut open_tags,
                        &mut top_level,
                        false,
                        activation,
                    )?;
                }

                Event::Comment(bt) => {
                    if ignore_comments {
                        continue;
                    }
                    let text = custom_unescape(bt, parser.decoder())
                        .map_err(|_| malformed_element(activation))?;
                    let text =
                        AvmString::new_utf8_bytes(activation.context.gc_context, text.as_bytes());
                    let node = E4XNode(GcCell::new(
                        activation.context.gc_context,
                        E4XNodeData {
                            parent: None,
                            namespace: None,
                            local_name: None,
                            kind: E4XNodeKind::Comment(text),
                        },
                    ));

                    push_childless_node(node, &mut open_tags, &mut top_level, activation)?;
                }
                Event::PI(bt) => {
                    if ignore_processing_instructions {
                        continue;
                    }
                    let text = custom_unescape(bt, parser.decoder())
                        .map_err(|_| malformed_element(activation))?;
                    let (name, value) = if let Some((name, value)) = text.split_once(' ') {
                        (
                            AvmString::new_utf8_bytes(
                                activation.context.gc_context,
                                name.as_bytes(),
                            ),
                            AvmString::new_utf8_bytes(
                                activation.context.gc_context,
                                value.trim_start().as_bytes(),
                            ),
                        )
                    } else {
                        (
                            AvmString::new_utf8_bytes(
                                activation.context.gc_context,
                                text.as_bytes(),
                            ),
                            AvmString::default(),
                        )
                    };
                    let node = E4XNode(GcCell::new(
                        activation.context.gc_context,
                        E4XNodeData {
                            parent: None,
                            namespace: None,
                            local_name: Some(name),
                            kind: E4XNodeKind::ProcessingInstruction(value),
                        },
                    ));

                    push_childless_node(node, &mut open_tags, &mut top_level, activation)?;
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
        parser: &NsReader<&[u8]>,
        bs: &BytesStart<'_>,
        decoder: quick_xml::Decoder,
    ) -> Result<Self, Error<'gc>> {
        let mut attribute_nodes = Vec::new();

        let attributes: Result<Vec<_>, _> = bs.attributes().collect();
        for attribute in attributes.map_err(|_| malformed_element(activation))? {
            let (ns, local_name) = parser.resolve_attribute(attribute.key);
            let name =
                AvmString::new_utf8_bytes(activation.context.gc_context, local_name.into_inner());
            let namespace = match ns {
                ResolveResult::Bound(ns) => Some(AvmString::new_utf8_bytes(
                    activation.context.gc_context,
                    ns.into_inner(),
                )),
                ResolveResult::Unknown(ns) if ns == b"xmlns" => continue,
                // https://www.w3.org/TR/xml-names/#xmlReserved
                // The prefix xml is by definition bound to the namespace name http://www.w3.org/XML/1998/namespace.
                ResolveResult::Unknown(ns) if ns == b"xml" => {
                    Some("http://www.w3.org/XML/1998/namespace".into())
                }
                ResolveResult::Unknown(ns) => {
                    return Err(Error::AvmError(type_error(
                        activation,
                        &format!(
                            "Error #1083: The prefix \"{}\" for element \"{}\" is not bound.",
                            String::from_utf8_lossy(&ns),
                            name
                        ),
                        1083,
                    )?))
                }
                ResolveResult::Unbound => None,
            };

            let value_str = custom_unescape(&attribute.value, decoder)
                .map_err(|_| malformed_element(activation))?;
            let value =
                AvmString::new_utf8_bytes(activation.context.gc_context, value_str.as_bytes());

            let attribute_data = E4XNodeData {
                parent: None,
                namespace,
                local_name: Some(name),
                kind: E4XNodeKind::Attribute(value),
            };
            let attribute = E4XNode(GcCell::new(activation.context.gc_context, attribute_data));
            attribute_nodes.push(attribute);
        }

        let (ns, local_name) = parser.resolve_element(bs.name());
        let name =
            AvmString::new_utf8_bytes(activation.context.gc_context, local_name.into_inner());
        let namespace = match ns {
            ResolveResult::Bound(ns) => Some(AvmString::new_utf8_bytes(
                activation.context.gc_context,
                ns.into_inner(),
            )),
            ResolveResult::Unknown(ns) if ns == b"xml" => {
                Some("http://www.w3.org/XML/1998/namespace".into())
            }
            ResolveResult::Unknown(ns) => {
                return Err(Error::AvmError(type_error(
                    activation,
                    &format!(
                        "Error #1083: The prefix \"{}\" for element \"{}\" is not bound.",
                        String::from_utf8_lossy(&ns),
                        name
                    ),
                    1083,
                )?))
            }
            ResolveResult::Unbound => None,
        };

        let data = E4XNodeData {
            parent: None,
            namespace,
            local_name: Some(name),
            kind: E4XNodeKind::Element {
                attributes: attribute_nodes,
                children: Vec::new(),
            },
        };

        let result = E4XNode(GcCell::new(activation.context.gc_context, data));

        let mut result_kind = result.kind_mut(activation.context.gc_context);
        if let E4XNodeKind::Element { attributes, .. } = &mut *result_kind {
            for attribute in attributes {
                attribute.set_parent(Some(result), activation.context.gc_context);
            }
        }

        Ok(result)
    }

    pub fn namespace(&self) -> Option<AvmString<'gc>> {
        self.0.read().namespace
    }

    pub fn local_name(&self) -> Option<AvmString<'gc>> {
        self.0.read().local_name
    }

    pub fn set_parent(&self, parent: Option<E4XNode<'gc>>, mc: &Mutation<'gc>) {
        self.0.write(mc).parent = parent;
    }

    pub fn parent(&self) -> Option<E4XNode<'gc>> {
        self.0.read().parent
    }

    pub fn matches_name(&self, name: &Multiname<'gc>) -> bool {
        let self_is_attr = matches!(self.0.read().kind, E4XNodeKind::Attribute(_));
        if self_is_attr != name.is_attribute() {
            return false;
        }

        if !name.is_any_name() && self.local_name() != name.local_name() {
            return false;
        }

        if name.is_any_namespace() {
            return true;
        }

        if name.has_explicit_namespace() {
            return self.namespace() == name.explict_namespace();
        }

        // By default `xml.*` matches in all namespaces, unless an explicit
        // namespace is given (`xml.ns::*`).
        // However normal properties like "xml.prop" match only in the
        // default namespace.
        // TODO: Implement this by better handling default namespaces.
        // See also "The QName Constructor Called as a Function".
        name.is_any_name() || self.namespace().is_none()
    }

    pub fn descendants(&self, name: &Multiname<'gc>, out: &mut Vec<E4XOrXml<'gc>>) {
        if let E4XNodeKind::Element {
            children,
            attributes,
            ..
        } = &self.0.read().kind
        {
            if name.is_attribute() {
                for attribute in attributes {
                    if attribute.matches_name(name) {
                        out.push(E4XOrXml::E4X(*attribute));
                    }
                }
            }
            for child in children {
                if child.matches_name(name) {
                    out.push(E4XOrXml::E4X(*child));
                }
                child.descendants(name, out)
            }
        }
    }

    pub fn has_complex_content(&self) -> bool {
        match &self.0.read().kind {
            E4XNodeKind::Element { children, .. } => children
                .iter()
                .any(|child| matches!(&*child.kind(), E4XNodeKind::Element { .. })),
            E4XNodeKind::Text(_) | E4XNodeKind::CData(_) => false,
            E4XNodeKind::Attribute(_) => false,
            E4XNodeKind::Comment(_) => false,
            E4XNodeKind::ProcessingInstruction(_) => false,
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

    pub fn xml_to_string(&self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        match &self.0.read().kind {
            E4XNodeKind::Text(text) | E4XNodeKind::CData(text) => *text,
            E4XNodeKind::Attribute(text) => *text,
            E4XNodeKind::Element { children, .. } => {
                if self.has_simple_content() {
                    return simple_content_to_string(
                        children.iter().map(|node| E4XOrXml::E4X(*node)),
                        activation,
                    );
                }

                return to_xml_string(E4XOrXml::E4X(*self), activation);
            }
            E4XNodeKind::Comment(_) | E4XNodeKind::ProcessingInstruction(_) => {
                return to_xml_string(E4XOrXml::E4X(*self), activation);
            }
        }
    }

    pub fn xml_to_xml_string(&self, activation: &mut Activation<'_, 'gc>) -> AvmString<'gc> {
        return to_xml_string(E4XOrXml::E4X(*self), activation);
    }

    pub fn kind(&self) -> Ref<'_, E4XNodeKind<'gc>> {
        Ref::map(self.0.read(), |r| &r.kind)
    }

    pub fn kind_mut(&self, mc: &Mutation<'gc>) -> RefMut<'_, E4XNodeKind<'gc>> {
        RefMut::map(self.0.write(mc), |r| &mut r.kind)
    }

    pub fn ptr_eq(first: E4XNode<'gc>, second: E4XNode<'gc>) -> bool {
        GcCell::ptr_eq(first.0, second.0)
    }
}

pub fn simple_content_to_string<'gc>(
    children: impl Iterator<Item = E4XOrXml<'gc>>,
    activation: &mut Activation<'_, 'gc>,
) -> AvmString<'gc> {
    let mut out = AvmString::default();
    for child in children {
        if matches!(
            &*child.node().kind(),
            E4XNodeKind::Comment(_) | E4XNodeKind::ProcessingInstruction { .. }
        ) {
            continue;
        }
        let child_str = child.node().xml_to_string(activation);
        out = AvmString::concat(activation.context.gc_context, out, child_str);
    }
    out
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

fn to_xml_string_inner(xml: E4XOrXml, buf: &mut WString, pretty: Option<(u32, u32)>) {
    // FIXME: Namespace support.

    let node = xml.node();
    let node_kind = node.kind();

    if let Some((indent_level, _)) = pretty {
        for _ in 0..indent_level {
            buf.push_char(' ');
        }
    }

    let (children, attributes) = match &*node_kind {
        E4XNodeKind::Text(text) => {
            // FIXME: Spec says to trim XMLWhitespace characters here
            buf.push_str(&escape_element_value(*text));
            return;
        }
        E4XNodeKind::ProcessingInstruction(value) => {
            buf.push_utf8("<?");
            buf.push_str(&node.local_name().unwrap());
            buf.push_char(' ');
            buf.push_str(value);
            buf.push_utf8("?>");
            return;
        }
        E4XNodeKind::Comment(data) => {
            buf.push_utf8("<!--");
            buf.push_str(data);
            buf.push_utf8("-->");
            return;
        }
        E4XNodeKind::Attribute(data) => {
            buf.push_str(&escape_attribute_value(*data));
            return;
        }
        E4XNodeKind::CData(data) => {
            buf.push_utf8("<![CDATA[");
            buf.push_str(data);
            buf.push_utf8("]]>");
            return;
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
        return;
    }

    buf.push_char('>');

    let indent_children = children.len() > 1
        || children.len() == 1 && !matches!(*children[0].kind(), E4XNodeKind::Text(_));
    let child_pretty = if let Some((indent_level, pretty_indent)) = pretty {
        if indent_children {
            Some((indent_level + pretty_indent, pretty_indent))
        } else {
            None
        }
    } else {
        None
    };

    for child in children {
        if pretty.is_some() && indent_children {
            buf.push_char('\n');
        }
        to_xml_string_inner(E4XOrXml::E4X(*child), buf, child_pretty);
    }

    if let Some((indent_level, _)) = pretty {
        if indent_children {
            buf.push_char('\n');
            for _ in 0..indent_level {
                buf.push_char(' ');
            }
        }
    }

    buf.push_utf8("</");
    buf.push_str(&node.local_name().unwrap());
    buf.push_char('>');
}

// Implementation of `ToXMLString` from ECMA-357 (10.2.1)
pub fn to_xml_string<'gc>(
    xml: E4XOrXml<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> AvmString<'gc> {
    let pretty_printing = activation
        .avm2()
        .classes()
        .xml
        .get_public_property("prettyPrinting", activation)
        .expect("prettyPrinting should be set")
        .coerce_to_boolean();
    let pretty = if pretty_printing {
        let pretty_indent = activation
            .avm2()
            .classes()
            .xml
            .get_public_property("prettyIndent", activation)
            .expect("prettyIndent should be set")
            .coerce_to_i32(activation)
            .expect("shouldnt error");

        // NOTE: Negative values are invalid and are ignored.
        if pretty_indent < 0 {
            None
        } else {
            Some((0, pretty_indent as u32))
        }
    } else {
        None
    };

    let mut buf = WString::new();
    to_xml_string_inner(xml, &mut buf, pretty);
    AvmString::new(activation.context.gc_context, buf)
}

pub fn name_to_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &Value<'gc>,
    force_attribute: bool,
) -> Result<Multiname<'gc>, Error<'gc>> {
    if let Value::Object(o) = name {
        if let Some(qname) = o.as_qname_object() {
            let mut name = qname.name().clone();
            if force_attribute {
                name.set_is_attribute(true);
            }
            return Ok(name);
        }
    }

    let name = name.coerce_to_string(activation)?;

    let mut multiname = if let Some(name) = name.strip_prefix(b'@') {
        let name = AvmString::new(activation.context.gc_context, name);
        Multiname::attribute(activation.avm2().public_namespace, name)
    } else if &*name == b"*" {
        Multiname::any(activation.context.gc_context)
    } else {
        Multiname::new(activation.avm2().public_namespace, name)
    };
    if force_attribute {
        multiname.set_is_attribute(true);
    };
    Ok(multiname)
}
