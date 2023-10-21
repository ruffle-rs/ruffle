use std::{
    cell::{Ref, RefMut},
    fmt::{self, Debug},
};

use gc_arena::{Collect, GcCell, Mutation};
use quick_xml::{
    events::{attributes::AttrError as XmlAttrError, BytesStart, Event},
    name::ResolveResult,
    Error as XmlError, NsReader,
};

use crate::{avm2::TObject, xml::custom_unescape};

use super::{
    error::{make_error_1010, make_error_1118, type_error},
    object::E4XOrXml,
    string::AvmString,
    Activation, Error, Multiname, Value,
};
use crate::string::{WStr, WString};

mod is_xml_name;
mod iterators;

pub use is_xml_name::is_xml_name;

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

fn make_xml_error<'gc>(activation: &mut Activation<'_, 'gc>, err: XmlError) -> Error<'gc> {
    let error = match err {
        XmlError::InvalidAttr(XmlAttrError::Duplicated(_, _)) => type_error(
            activation,
            "Error #1104: Attribute was already specified for element.",
            1104,
        ),
        XmlError::UnexpectedEof(currently_parsing) => match currently_parsing.as_str() {
            "CData" => type_error(
                activation,
                "Error #1091: XML parser failure: Unterminated CDATA section.",
                1091,
            ),
            "DOCTYPE" => type_error(
                activation,
                "Error #1093: XML parser failure: Unterminated DOCTYPE declaration.",
                1093,
            ),
            "Comment" => type_error(
                activation,
                "Error #1094: XML parser failure: Unterminated comment.",
                1094,
            ),
            "XmlDecl" => type_error(
                activation,
                "Error #1097: XML parser failure: Unterminated processing instruction.",
                1097,
            ),
            _ => type_error(
                activation,
                "Error #1090: XML parser failure: element is malformed.",
                1090,
            ),
        },
        _ => type_error(
            activation,
            "Error #1090: XML parser failure: element is malformed.",
            1090,
        ),
    };

    match error {
        Ok(err) => Error::AvmError(err),
        Err(err) => err,
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
        parent: Option<Self>,
    ) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent,
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
        parent: Option<E4XNode<'gc>>,
    ) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent,
                namespace: None,
                local_name: Some(name),
                kind: E4XNodeKind::Attribute(value),
            },
        ))
    }

    /// Returns an iterator that yields ancestor nodes (including itself).
    pub fn ancestors(self) -> impl Iterator<Item = E4XNode<'gc>> {
        iterators::AnscIter::for_node(self)
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

    /// Returns the amount of children in this node if this node is of Element kind, otherwise returns [None].
    pub fn length(&self) -> Option<usize> {
        if let E4XNodeKind::Element { children, .. } = &*self.kind() {
            Some(children.len())
        } else {
            None
        }
    }

    /// Removes all matching children matching provided name, returns the first child removed along with its index (if any).
    pub fn remove_matching_children(
        &self,
        gc_context: &Mutation<'gc>,
        name: &Multiname<'gc>,
    ) -> Option<(usize, E4XNode<'gc>)> {
        let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(gc_context) else {
            return None;
        };

        let index = children
            .iter()
            .position(|x| name.is_any_name() || x.matches_name(name));

        let val = if let Some(index) = index {
            Some((index, children[index]))
        } else {
            None
        };

        children.retain(|x| {
            if name.is_any_name() || x.matches_name(name) {
                // Remove parent.
                x.set_parent(None, gc_context);
                false
            } else {
                true
            }
        });

        val
    }

    pub fn insert_at(&self, gc_context: &Mutation<'gc>, index: usize, node: E4XNode<'gc>) {
        let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(gc_context) else {
            return;
        };

        node.set_parent(Some(*self), gc_context);
        children.insert(index, node);
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

    // ECMA-357 9.1.1.4 [[DeleteByIndex]] (P)
    pub fn delete_by_index(&self, index: usize, activation: &mut Activation<'_, 'gc>) {
        let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(activation.gc()) else {
            return;
        };

        // 2.a. If i is less than x.[[Length]]
        if index < children.len() {
            // 2.a.i. If x has a property with name P
            // 2.a.i.2. Remove the property with the name P from x
            let element = children.remove(index);
            // 2.a.i.1. Let x[P].[[Parent]] = null
            element.set_parent(None, activation.gc());
        }
    }

    // ECMA-357 9.1.1.11 [[Insert]] (P, V)
    pub fn insert(
        &self,
        index: usize,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return
        if !matches!(*self.kind(), E4XNodeKind::Element { .. }) {
            return Ok(());
        }

        // 4. If Type(V) is XML and (V is x or an ancestor of x) throw an Error exception
        if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
            if self.ancestors().any(|x| E4XNode::ptr_eq(x, *xml.node())) {
                return Err(make_error_1118(activation));
            }
        }

        // 10. If Type(V) is XMLList
        if let Some(list) = value.as_object().and_then(|x| x.as_xml_list_object()) {
            let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(activation.gc()) else {
                unreachable!("E4XNode should be of element kind");
            };

            // 10.a. For j = 0 to V.[[Length-1]]
            for (child_index, child) in list.children().iter().enumerate() {
                let child = child.node();
                // 10.a.i. V[j].[[Parent]] = x
                child.set_parent(Some(*self), activation.gc());
                // 10.a.ii. x[i + j] = V[j]
                children.insert(index + child_index, *child);
            }
        // 11. Else
        } else {
            // 11.a. Call the [[Replace]] method of x with arguments i and V
            if let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(activation.gc()) {
                // NOTE: Make room for the replace operation.
                children.insert(index, E4XNode::dummy(activation.gc()))
            }
            self.replace(index, value, activation)?;
        }

        // 12. Return
        Ok(())
    }

    // ECMA-357 9.1.1.12 [[Replace]] (P, V)
    pub fn replace(
        &self,
        index: usize,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<(), Error<'gc>> {
        // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", "attribute"}, return
        if !matches!(*self.kind(), E4XNodeKind::Element { .. }) {
            return Ok(());
        }

        // 5. If Type(V) is XML and V.[[Class]] ∈ {"element", "comment", "processing-instruction", "text"}
        if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
            if matches!(*xml.node().kind(), E4XNodeKind::Attribute(_)) {
                return Ok(());
            }

            // 5.a. If V.[[Class]] is “element” and (V is x or an ancestor of x) throw an Error exception
            if matches!(*xml.node().kind(), E4XNodeKind::Element { .. })
                && self.ancestors().any(|x| E4XNode::ptr_eq(x, *xml.node()))
            {
                return Err(make_error_1118(activation));
            }

            // 5.b. Let V.[[Parent]] = x
            xml.node().set_parent(Some(*self), activation.gc());

            let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(activation.gc()) else {
                unreachable!("E4XNode should be of element kind");
            };

            // 5.c. If x has a property with name P
            if let Some(node) = children.get(index) {
                // 5.c.i. Let x[P].[[Parent]] = null
                node.set_parent(None, activation.gc());
            }

            // 5.d. Let x[P] = V
            if index >= children.len() {
                children.push(*xml.node());
            } else {
                children[index] = *xml.node();
            }
        // 6. Else if Type(V) is XMLList
        } else if value
            .as_object()
            .and_then(|x| x.as_xml_list_object())
            .is_some()
        {
            // 6.a. Call the [[DeleteByIndex]] method of x with argument P
            self.delete_by_index(index, activation);
            // 6.b. Call the [[Insert]] method of x with arguments P and V
            self.insert(index, value, activation)?;
        // 7. Else
        } else {
            // 7.a. Let s = ToString(V)
            let s: AvmString<'_> = value.coerce_to_string(activation)?;
            // 7.b. Create a new XML object t with t.[[Class]] = "text", t.[[Parent]] = x and t.[[Value]] = s
            let text_node = E4XNode::text(activation.gc(), s, Some(*self));

            let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(activation.gc()) else {
                unreachable!("E4XNode should be of element kind");
            };

            // 7.c. If x has a property with name P
            if let Some(node) = children.get(index) {
                // 7.c.i. Let x[P].[[Parent]] = null
                node.set_parent(None, activation.gc());
            }

            // 7.d. Let the value of property P of x be t
            if index >= children.len() {
                children.push(text_node);
            } else {
                children[index] = text_node;
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
                .map_err(|e| make_xml_error(activation, e))?;

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
                            .map_err(|e| make_xml_error(activation, e))?
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
                        .map_err(|e| make_xml_error(activation, e))?;
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
                        .map_err(|e| make_xml_error(activation, e))?;
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
        for attribute in
            attributes.map_err(|e| make_xml_error(activation, XmlError::InvalidAttr(e)))?
        {
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
                .map_err(|e| make_xml_error(activation, e))?;
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

    pub fn set_namespace(&self, namespace: AvmString<'gc>, mc: &Mutation<'gc>) {
        self.0.write(mc).namespace = Some(namespace);
    }

    pub fn namespace(&self) -> Option<AvmString<'gc>> {
        self.0.read().namespace
    }

    pub fn set_local_name(&self, name: AvmString<'gc>, mc: &Mutation<'gc>) {
        self.0.write(mc).local_name = Some(name);
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

// 10.6.1. ToXMLName Applied to the String Type
pub fn string_to_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> Multiname<'gc> {
    if let Some(name) = name.strip_prefix(b'@') {
        let name = AvmString::new(activation.context.gc_context, name);
        Multiname::attribute(activation.avm2().public_namespace, name)
    } else if &*name == b"*" {
        Multiname::any(activation.context.gc_context)
    } else {
        Multiname::new(activation.avm2().public_namespace, name)
    }
}

// 10.6 ToXMLName
pub fn name_to_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &Value<'gc>,
    force_attribute: bool,
) -> Result<Multiname<'gc>, Error<'gc>> {
    if matches!(name, Value::Undefined | Value::Null) {
        return Err(make_error_1010(activation, None));
    }

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
    let mut multiname = string_to_multiname(activation, name);
    if force_attribute {
        multiname.set_is_attribute(true);
    };
    Ok(multiname)
}

// Based on https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/XMLObject.cpp#L1543
// This is needed to reproduce some weird behavior in SWFv9.
pub fn maybe_escape_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    child: Value<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // NOTE: This depends on root SWF version, not caller movie version.
    if activation.context.swf.version() <= 9 {
        if child.as_object().map_or(false, |x| {
            x.as_xml_object().is_some() || x.as_xml_list_object().is_some()
        }) {
            Ok(child)
        } else {
            let string = child.coerce_to_string(activation)?;
            let xml = activation
                .avm2()
                .classes()
                .xml
                .construct(activation, &[string.into()])?;
            Ok(xml.into())
        }
    } else {
        Ok(child)
    }
}
