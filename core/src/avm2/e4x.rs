use std::{
    cell::{Ref, RefMut},
    fmt::{self, Debug},
};

use gc_arena::{Collect, GcCell, Mutation};
use quick_xml::{
    errors::{IllFormedError, SyntaxError as XmlSyntaxError},
    events::{attributes::AttrError as XmlAttrError, BytesStart, Event},
    name::ResolveResult,
    Error as XmlError, NsReader,
};

use crate::{avm2::TObject, xml::custom_unescape};

use super::{
    error::{make_error_1010, make_error_1085, make_error_1118, type_error},
    object::{E4XOrXml, FunctionObject, NamespaceObject},
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
    namespace: Option<Box<E4XNamespace<'gc>>>,
    local_name: Option<AvmString<'gc>>,
    kind: E4XNodeKind<'gc>,
    notification: Option<FunctionObject<'gc>>,
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

        XmlError::Syntax(syntax_error) => match syntax_error {
            XmlSyntaxError::UnclosedCData => type_error(
                activation,
                "Error #1091: XML parser failure: Unterminated CDATA section.",
                1091,
            ),
            XmlSyntaxError::UnclosedDoctype => type_error(
                activation,
                "Error #1093: XML parser failure: Unterminated DOCTYPE declaration.",
                1093,
            ),
            XmlSyntaxError::UnclosedComment => type_error(
                activation,
                "Error #1094: XML parser failure: Unterminated comment.",
                1094,
            ),
            XmlSyntaxError::UnclosedPIOrXmlDecl => type_error(
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

#[derive(Copy, Clone, Collect, PartialEq, Debug)]
#[collect(no_drop)]
pub struct E4XNamespace<'gc> {
    pub uri: AvmString<'gc>,
    pub prefix: Option<AvmString<'gc>>,
}

impl<'gc> E4XNamespace<'gc> {
    pub fn new_uri(uri: AvmString<'gc>) -> Self {
        E4XNamespace { prefix: None, uri }
    }

    pub fn default_namespace() -> Self {
        E4XNamespace {
            prefix: None,
            uri: "".into(),
        }
    }
}

impl<'gc> E4XNamespace<'gc> {
    pub fn as_namespace_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<NamespaceObject<'gc>, Error<'gc>> {
        let args = if let Some(prefix) = self.prefix {
            vec![prefix.into(), self.uri.into()]
        } else {
            vec![self.uri.into()]
        };
        let obj = activation
            .avm2()
            .classes()
            .namespace
            .construct(activation, &args)?;
        Ok(obj
            .as_namespace_object()
            .expect("just constructed a namespace"))
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
        namespaces: Vec<E4XNamespace<'gc>>,
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
                    namespaces: vec![],
                },
                notification: None,
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
                notification: None,
            },
        ))
    }

    pub fn element(
        mc: &Mutation<'gc>,
        namespace: Option<E4XNamespace<'gc>>,
        name: AvmString<'gc>,
        parent: Option<Self>,
    ) -> Self {
        E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent,
                namespace: namespace.map(Box::new),
                local_name: Some(name),
                kind: E4XNodeKind::Element {
                    attributes: vec![],
                    children: vec![],
                    namespaces: vec![],
                },
                notification: None,
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
                notification: None,
            },
        ))
    }

    /// Returns true when the node is an attribute (E4XNodeKind::Attribute)
    pub fn is_attribute(&self) -> bool {
        matches!(self.0.read().kind, E4XNodeKind::Attribute(_))
    }

    /// Returns true when the node is an element (E4XNodeKind::Element)
    pub fn is_element(&self) -> bool {
        matches!(self.0.read().kind, E4XNodeKind::Element { .. })
    }

    /// Returns true when the node is text (E4XNodeKind::Text or E4XNodeKind::CData)
    pub fn is_text(&self) -> bool {
        matches!(
            self.0.read().kind,
            E4XNodeKind::Text(_) | E4XNodeKind::CData(_)
        )
    }

    /// Returns true when the node is a comment (E4XNodeKind::Comment)
    pub fn is_comment(&self) -> bool {
        matches!(self.0.read().kind, E4XNodeKind::Comment(_))
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
                    ..
                },
                E4XNodeKind::Element {
                    children: children_b,
                    attributes: attributes_b,
                    ..
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

        // TODO: FP actually respects ignoreComments and ignoreProcessingInstructions here.
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
                namespaces,
            } => E4XNodeKind::Element {
                attributes: attributes.iter().map(|attr| attr.deep_copy(mc)).collect(),
                children: children.iter().map(|child| child.deep_copy(mc)).collect(),
                namespaces: namespaces.clone(),
            },
        };

        let node = E4XNode(GcCell::new(
            mc,
            E4XNodeData {
                parent: None,
                namespace: this.namespace.clone(),
                local_name: this.local_name,
                kind,
                notification: None,
            },
        ));

        if let E4XNodeKind::Element {
            attributes,
            children,
            ..
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

    pub fn child_index(&self) -> Option<usize> {
        let parent = self.parent()?;

        if self.is_attribute() {
            return None;
        }

        if let E4XNodeKind::Element { children, .. } = &*parent.kind() {
            let index = children
                .iter()
                .position(|child| E4XNode::ptr_eq(*child, *self))
                .unwrap();
            return Some(index);
        }

        unreachable!("parent must be an element")
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
        if !self.is_element() {
            return Ok(());
        }

        // 4. If Type(V) is XML and (V is x or an ancestor of x) throw an Error exception
        if let Some(xml) = value.as_object().and_then(|x| x.as_xml_object()) {
            if self.ancestors().any(|x| E4XNode::ptr_eq(x, xml.node())) {
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
                children.insert(index + child_index, child);
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
        if !self.is_element() {
            return Ok(());
        }

        // 5. If Type(V) is XML and V.[[Class]] ∈ {"element", "comment", "processing-instruction", "text"}
        if let Some(xml) = value
            .as_object()
            .and_then(|x| x.as_xml_object())
            .filter(|x| !x.node().is_attribute())
        {
            // 5.a. If V.[[Class]] is “element” and (V is x or an ancestor of x) throw an Error exception
            if xml.node().is_element() && self.ancestors().any(|x| E4XNode::ptr_eq(x, xml.node())) {
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
                children.push(xml.node());
            } else {
                children[index] = xml.node();
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

    // ECMA-357 9.1.1.6 [[HasProperty]] (P)
    pub fn has_property(&self, name: &Multiname<'gc>) -> bool {
        if !name.has_explicit_namespace() {
            if let Some(local_name) = name.local_name() {
                // 1. If ToString(ToUint32(P)) == P
                if let Ok(index) = local_name.parse::<usize>() {
                    // 1.a. Return (P == "0")
                    return index == 0;
                }
            }

            // FIXME: 2. Let n = ToXMLName(P).

            // 3. - 4.
            if let E4XNodeKind::Element {
                children,
                attributes,
                ..
            } = &*self.kind()
            {
                let search_children = if name.is_attribute() {
                    attributes
                } else {
                    children
                };

                return search_children.iter().any(|child| child.matches_name(name));
            }
        }

        // 5. Return false
        false
    }

    // ECMA-357 13.4.4.26 XML.prototype.normalize ()
    pub fn normalize(&self, mc: &Mutation<'gc>) {
        if let E4XNodeKind::Element { children, .. } = &mut *self.kind_mut(mc) {
            // 1. Let i = 0
            let mut index = 0;

            // 2. While i < x.[[Length]]
            while index < children.len() {
                let child = children[index];

                // 2.a. If x[i].[[Class]] == "element"
                if child.is_element() {
                    // 2.a.i. Call the normalize method of x[i]
                    child.normalize(mc);
                    // 2.a.ii. Let i = i + 1
                    index += 1;
                // 2.b. Else if x[i].[[Class]] == "text"
                } else if child.is_text() {
                    let is_whitespace_text = {
                        let (E4XNodeKind::Text(text) | E4XNodeKind::CData(text)) =
                            &mut *child.kind_mut(mc)
                        else {
                            unreachable!()
                        };

                        // 2.b.i. While ((i+1) < x.[[Length]]) and (x[i + 1].[[Class]] == "text")
                        while index + 1 < children.len() && children[index + 1].is_text() {
                            {
                                let (E4XNodeKind::Text(other) | E4XNodeKind::CData(other)) =
                                    &*children[index + 1].kind()
                                else {
                                    unreachable!()
                                };

                                // 2.b.i.1. Let x[i].[[Value]] be the result of concatenating x[i].[[Value]] and x[i + 1].[[Value]]
                                *text = AvmString::concat(mc, *text, *other);
                            }

                            // 2.b.i.2. Call the [[DeleteByIndex]] method of x with argument ToString(i + 1)
                            // NOTE: We cannot call [[DeleteByIndex]] directly because of borrow errors, so we do it manually.
                            let child = children.remove(index + 1);
                            child.set_parent(None, mc);
                        }

                        // NOTE: Non-standard avmplus behavior, spec says to check if length is 0, but avmplus
                        //       checks if the string is made out of whitespace characters.
                        let mut chars = text.chars();
                        chars.all(|c| {
                            if let Ok(c) = c {
                                matches!(c, '\t' | '\n' | '\r' | ' ')
                            } else {
                                false
                            }
                        })
                    };

                    // 2.b.ii. If x[i].[[Value]].length == 0
                    if is_whitespace_text {
                        // 2.b.ii.1. Call the [[DeleteByIndex]] method of x with argument ToString(i)
                        // NOTE: We cannot call [[DeleteByIndex]] directly because of borrow errors, so we do it manually.
                        let child = children.remove(index);
                        child.set_parent(None, mc);
                    // 2.b.iii. Else
                    } else {
                        // 2.b.iii.1. Let i = i + 1
                        index += 1
                    }
                // 2.c. Else
                } else {
                    // 2.c.i. Let i = i + 1
                    index += 1;
                }
            }
        }
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
                        notification: None,
                    },
                ));
                push_childless_node(node, open_tags, top_level, activation)?;
            }
            Ok(())
        }

        loop {
            let event = match parser.read_event() {
                Ok(event) => event,
                Err(XmlError::IllFormed(IllFormedError::MismatchedEndTag { expected, found })) => {
                    // We must accept </a/>, </a />, and </a b="c">
                    // TODO: Reject </a bc>, </a//>, <a //> etc.
                    if let Some(rest) = found.strip_prefix(&expected) {
                        if rest.starts_with([' ', '\t', '/']) {
                            let node = open_tags.pop().unwrap();
                            if open_tags.is_empty() {
                                top_level.push(node);
                            }
                            continue;
                        }
                    }
                    return Err(make_error_1085(activation, &expected));
                }
                Err(err) => return Err(make_xml_error(activation, err)),
            };

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
                            notification: None,
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
                            notification: None,
                        },
                    ));

                    push_childless_node(node, &mut open_tags, &mut top_level, activation)?;
                }
                // These are completely ignored by AVM2
                Event::Decl(_) | Event::DocType(_) => {}
                Event::Eof => break,
            }
        }

        // Throw an error for unclosed tags.
        if let Some(current_tag) = open_tags.last() {
            return Err(make_error_1085(
                activation,
                &current_tag.local_name().unwrap().to_utf8_lossy(),
            ));
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
        let mut namespaces = Vec::new();

        let attributes: Result<Vec<_>, _> = bs.attributes().collect();
        for attribute in
            attributes.map_err(|e| make_xml_error(activation, XmlError::InvalidAttr(e)))?
        {
            let value_str = custom_unescape(&attribute.value, decoder)
                .map_err(|e| make_xml_error(activation, e))?;
            let value = AvmString::new_utf8_bytes(activation.gc(), value_str.as_bytes());

            let (ns, local_name) = parser.resolve_attribute(attribute.key);

            let local_name = ruffle_wstr::from_utf8_bytes(local_name.into_inner());
            let name = activation
                .context
                .interner
                .intern_wstr(activation.gc(), local_name)
                .into();

            let namespace = match ns {
                ResolveResult::Bound(ns) if ns.into_inner() == b"http://www.w3.org/2000/xmlns/" => {
                    namespaces.push(E4XNamespace {
                        uri: value,
                        prefix: Some(name),
                    });
                    continue;
                }
                ResolveResult::Bound(ns) => {
                    let prefix = attribute.key.prefix().map(|prefix| {
                        AvmString::new_utf8_bytes(activation.gc(), prefix.into_inner())
                    });
                    let uri = AvmString::new_utf8_bytes(activation.gc(), ns.into_inner());
                    Some(E4XNamespace { prefix, uri })
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
                ResolveResult::Unbound => {
                    // The default XML namespace declaration
                    if &*name == b"xmlns" {
                        namespaces.push(E4XNamespace {
                            uri: value,
                            prefix: Some("".into()),
                        });
                        continue;
                    }
                    None
                }
            };

            let attribute_data = E4XNodeData {
                parent: None,
                namespace: namespace.map(Box::new),
                local_name: Some(name),
                kind: E4XNodeKind::Attribute(value),
                notification: None,
            };
            let attribute = E4XNode(GcCell::new(activation.context.gc_context, attribute_data));
            attribute_nodes.push(attribute);
        }

        let (ns, local_name) = parser.resolve_element(bs.name());

        let local_name = ruffle_wstr::from_utf8_bytes(local_name.into_inner());
        let name = activation
            .context
            .interner
            .intern_wstr(activation.gc(), local_name)
            .into();

        let namespace = match ns {
            ResolveResult::Bound(ns) => {
                let prefix = bs
                    .name()
                    .prefix()
                    .map(|prefix| AvmString::new_utf8_bytes(activation.gc(), prefix.into_inner()));
                let uri = AvmString::new_utf8_bytes(activation.gc(), ns.into_inner());
                Some(E4XNamespace { prefix, uri })
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
            namespace: namespace.map(Box::new),
            local_name: Some(name),
            kind: E4XNodeKind::Element {
                attributes: attribute_nodes,
                children: Vec::new(),
                namespaces,
            },
            notification: None,
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

    pub fn set_namespace(&self, namespace: Option<E4XNamespace<'gc>>, mc: &Mutation<'gc>) {
        self.0.write(mc).namespace = namespace.map(Box::new);
    }

    pub fn namespace(&self) -> Option<E4XNamespace<'gc>> {
        self.0.read().namespace.as_deref().copied()
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

    pub fn set_notification(&self, notification: Option<FunctionObject<'gc>>, mc: &Mutation<'gc>) {
        self.0.write(mc).notification = notification;
    }

    pub fn notification(&self) -> Option<FunctionObject<'gc>> {
        self.0.read().notification
    }

    pub fn in_scope_namespaces(&self) -> Vec<E4XNamespace<'gc>> {
        let mut result: Vec<E4XNamespace<'gc>> = Vec::new();

        let mut next_node = Some(*self);
        while let Some(node) = next_node {
            if let E4XNodeKind::Element { namespaces, .. } = &*node.kind() {
                for new_ns in namespaces {
                    let found = result.iter().any(|ns| {
                        if new_ns.prefix.is_some() {
                            new_ns.prefix == ns.prefix
                        } else {
                            // XXX check ns.prefix == None?
                            new_ns.uri == ns.uri
                        }
                    });
                    if !found {
                        result.push(*new_ns);
                    }
                }
            }
            next_node = node.parent();
        }

        result
    }

    // ECMA-357 9.1.1.13 [[AddInScopeNamespace]] (N)
    pub fn add_in_scope_namespace(&self, gc: &Mutation<'gc>, namespace: E4XNamespace<'gc>) {
        // 1. If x.[[Class]] ∈ {"text", "comment", "processing-instruction", “attribute”}, return
        if !self.is_element() {
            return;
        }

        // 2. If N.prefix != undefined
        let Some(prefix) = namespace.prefix else {
            // 3. Return
            return;
        };

        // 2.a. If N.prefix == "" and x.[[Name]].uri == "", return
        if prefix.is_empty() && self.namespace().map_or(true, |ns| ns.uri.is_empty()) {
            return;
        }

        {
            let E4XNodeKind::Element {
                ref mut namespaces, ..
            } = &mut *self.kind_mut(gc)
            else {
                unreachable!("must be an element");
            };

            // 2.b. Let match be null
            // 2.c. For each ns in x.[[InScopeNamespaces]]
            // 2.c.i. If N.prefix == ns.prefix, let match = ns
            let found_index = namespaces.iter().position(|ns| Some(prefix) == ns.prefix);

            // 2.d. If match is not null and match.uri is not equal to N.uri
            if let Some(found_index) = found_index {
                if namespaces[found_index].uri != namespace.uri {
                    // 2.d.i. Remove match from x.[[InScopeNamespaces]]
                    namespaces.remove(found_index);
                }
            }

            // 2.e. Let x.[[InScopeNamespaces]] = x.[[InScopeNamespaces]] ∪ { N }
            namespaces.push(namespace);
        }

        // 2.f. If x.[[Name]].[[Prefix]] == N.prefix
        match self.namespace() {
            Some(self_ns) if self_ns.prefix == Some(prefix) => {
                // 2.f.i. Let x.[[Name]].prefix = undefined
                self.set_namespace(Some(E4XNamespace::new_uri(self_ns.uri)), gc);
            }
            _ => {}
        }

        // 2.g. For each attr in x.[[Attributes]]
        if let E4XNodeKind::Element {
            ref mut attributes, ..
        } = &mut *self.kind_mut(gc)
        {
            for attr in attributes.iter_mut() {
                // 2.g.i. If attr.[[Name]].[[Prefix]] == N.prefix, let attr.[[Name]].prefix = undefined
                match attr.namespace() {
                    Some(attr_ns) if attr_ns.prefix == Some(prefix) => {
                        attr.set_namespace(Some(E4XNamespace::new_uri(attr_ns.uri)), gc);
                    }
                    _ => {}
                }
            }
        }
    }

    // FIXME - avmplus constructs an actual QName here, and does the normal
    // Multiname matching logic. We should do the same.
    pub fn matches_name(&self, name: &Multiname<'gc>) -> bool {
        if self.is_attribute() != name.is_attribute() {
            return false;
        }

        // A non-qname Any name matches all nodes, including Text etc.
        // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/Multiname.cpp#L59
        if name.is_any_name() && !name.is_qname() {
            return true;
        }

        if !name.is_any_name() && self.local_name() != name.local_name() {
            return false;
        }

        if self.local_name().is_none() {
            return false;
        }

        if name.is_any_namespace() {
            return true;
        }

        let self_ns = self.namespace().map(|ns| ns.uri).unwrap_or_default();
        // FIXME: For cases where we don't have *any* explicit namespace
        // we just give up and assume we should match the default public namespace.
        if !name.namespace_set().iter().any(|ns| ns.is_namespace()) {
            return self_ns.is_empty();
        }

        name.namespace_set().iter().any(|ns| ns.as_uri() == self_ns)
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
            E4XNodeKind::Element { children, .. } => {
                children.iter().any(|child| child.is_element())
            }
            E4XNodeKind::Text(_) | E4XNodeKind::CData(_) => false,
            E4XNodeKind::Attribute(_) => false,
            E4XNodeKind::Comment(_) => false,
            E4XNodeKind::ProcessingInstruction(_) => false,
        }
    }

    pub fn has_simple_content(&self) -> bool {
        match &self.0.read().kind {
            E4XNodeKind::Element { children, .. } => {
                children.iter().all(|child| !child.is_element())
            }
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

fn to_xml_string_inner<'gc>(
    xml: E4XOrXml<'gc>,
    buf: &mut WString,
    ancestor_namespaces: &[E4XNamespace<'gc>],
    pretty: Option<(u32, u32)>,
) {
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
            ..
        } => (children, attributes),
    };

    // 9. Let namespaceDeclarations = { }
    let mut namespace_declarations = Vec::new();

    // 10. For each ns in x.[[InScopeNamespaces]]
    for ns in node.in_scope_namespaces() {
        // 10.a If there is no ans ∈ AncestorNamespaces, such that ans.uri == ns.uri
        //      and ans.prefix == ns.prefix
        if !ancestor_namespaces.contains(&ns) {
            // 10.a.i. Let ns1 be a copy of ns
            // 10.a.ii. Let namespaceDeclarations = namespaceDeclarations ∪ { ns1 }
            namespace_declarations.push(ns)
        }
    }

    // 11. For each name in the set of names consisting of x.[[Name]] and
    //     the name of each attribute in x.[[Attributes]]

    // TODO: Generate fake namespace prefixes when required.
    let get_namespace = |namespace_declarations: &[E4XNamespace<'gc>], ns: &E4XNamespace<'gc>| {
        ancestor_namespaces
            .iter()
            .chain(namespace_declarations.iter())
            .find(|ancestor_ns| ancestor_ns.uri == ns.uri)
            .copied()
    };

    if let Some(ns) = node.namespace() {
        if get_namespace(&namespace_declarations, &ns).is_none() {
            namespace_declarations.push(ns);
        }
    }
    for attribute in attributes {
        if let Some(ns) = attribute.namespace() {
            if get_namespace(&namespace_declarations, &ns).is_none() {
                namespace_declarations.push(ns);
            }
        }
    }

    let get_prefix = |node: &E4XNode<'gc>| {
        node.namespace().and_then(|ns| {
            get_namespace(&namespace_declarations, &ns)
                .and_then(|ns| ns.prefix)
                .filter(|p| !p.is_empty())
        })
    };

    buf.push_char('<');
    if let Some(prefix) = get_prefix(&node) {
        buf.push_str(&prefix);
        buf.push_char(':');
    }
    buf.push_str(&node.local_name().unwrap());

    for attribute in attributes {
        if let E4XNodeKind::Attribute(value) = &*attribute.kind() {
            buf.push_char(' ');
            if let Some(prefix) = get_prefix(attribute) {
                buf.push_str(&prefix);
                buf.push_char(':');
            }
            buf.push_str(&attribute.local_name().unwrap());
            buf.push_char('=');
            buf.push_char('"');
            buf.push_str(&escape_attribute_value(*value));
            buf.push_char('"');
        }
    }

    for ns in &namespace_declarations {
        buf.push_utf8(" xmlns");
        if let Some(prefix) = ns.prefix.filter(|p| !p.is_empty()) {
            buf.push_char(':');
            buf.push_str(&prefix);
        }
        buf.push_char('=');
        buf.push_char('"');
        buf.push_str(&escape_attribute_value(ns.uri));
        buf.push_char('"');
    }

    if children.is_empty() {
        buf.push_utf8("/>");
        return;
    }

    buf.push_char('>');

    let indent_children = children.len() > 1 || children.len() == 1 && !children[0].is_text();
    let child_pretty = if let Some((indent_level, pretty_indent)) = pretty {
        if indent_children {
            Some((indent_level + pretty_indent, pretty_indent))
        } else {
            None
        }
    } else {
        None
    };

    let mut all_namespaces = ancestor_namespaces.to_vec();
    all_namespaces.extend_from_slice(&namespace_declarations);

    for child in children {
        if pretty.is_some() && indent_children {
            buf.push_char('\n');
        }
        to_xml_string_inner(E4XOrXml::E4X(*child), buf, &all_namespaces, child_pretty);
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
    if let Some(prefix) = get_prefix(&node) {
        buf.push_str(&prefix);
        buf.push_char(':');
    }
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
            .expect("shouldn't error");

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
    let ancestor_namespaces = Vec::new();
    to_xml_string_inner(xml, &mut buf, &ancestor_namespaces, pretty);
    AvmString::new(activation.context.gc_context, buf)
}

// 10.6.1. ToXMLName Applied to the String Type
pub fn string_to_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> Multiname<'gc> {
    if let Some(name) = name.strip_prefix(b'@') {
        if name == b"*" {
            return Multiname::any_attribute(activation.gc());
        }

        let name = AvmString::new(activation.context.gc_context, name);
        Multiname::attribute(activation.avm2().public_namespace_base_version, name)
    } else if &*name == b"*" {
        Multiname::any(activation.context.gc_context)
    } else {
        Multiname::new(activation.avm2().public_namespace_base_version, name)
    }
}

// 10.6 ToXMLName
// note: the coercion rules in FP are slightly more complex.
// in FP there are 2 layers:
// - ToXMLName()
// - CoerceE4XMultiname()
// for example, the first layer doesn't propagate IS_QNAME on QNames, but latter does
// TODO: figure out if this matters for us, maybe there are some edge cases
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
            return Ok(child);
        } else {
            let string = child.coerce_to_string(activation)?;
            let xml = activation
                .avm2()
                .classes()
                .xml
                .construct(activation, &[string.into()])?;
            return Ok(xml.into());
        }
    }

    if activation.caller_movie_or_root().version() >= 21 {
        if let Some(xml) = child.as_object().and_then(|x| x.as_xml_object()) {
            let node = xml.node();
            let parent = node.parent();

            let index = node.child_index();

            if let Some(parent) = parent {
                if let Some(index) = index {
                    parent.delete_by_index(index, activation);
                }
            }
        }
    }

    Ok(child)
}
