//! XML Tree structure

use crate::avm1::activation::Activation;
use crate::avm1::object::xml_attributes_object::XmlAttributesObject;
use crate::avm1::object::xml_node_object::XmlNodeObject;
use crate::avm1::{Object, TObject};
use crate::string::{AvmString, WStr, WString};
use crate::xml;
use crate::xml::Error;
use gc_arena::{Collect, GcCell, MutationContext};
use quick_xml::escape::escape;
use quick_xml::events::{BytesStart, BytesText};
use smallvec::alloc::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::mem::swap;

/// Represents a node in the XML tree.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct XmlNode<'gc>(GcCell<'gc, XmlNodeData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub enum XmlNodeData<'gc> {
    /// The root level of an XML document. Has no parent.
    DocumentRoot {
        /// The script object associated with this XML node, if any.
        script_object: Option<Object<'gc>>,

        /// The script object associated with this XML node's attributes, if any.
        attributes_script_object: Option<Object<'gc>>,

        /// Child nodes of this element.
        children: Vec<XmlNode<'gc>>,
    },

    /// An element node in the XML tree.
    ///
    /// Element nodes are non-leaf nodes: they can store additional data as
    /// either attributes (for key/value pairs) or child nodes (for more
    /// structured data).
    Element {
        /// The script object associated with this XML node, if any.
        script_object: Option<Object<'gc>>,

        /// The script object associated with this XML node's attributes, if any.
        attributes_script_object: Option<Object<'gc>>,

        /// The parent node of this one.
        parent: Option<XmlNode<'gc>>,

        /// The previous sibling node to this one.
        prev_sibling: Option<XmlNode<'gc>>,

        /// The next sibling node to this one.
        next_sibling: Option<XmlNode<'gc>>,

        /// The tag name of this element.
        tag_name: AvmString<'gc>,

        /// Attributes of the element.
        attributes: BTreeMap<AvmString<'gc>, AvmString<'gc>>,

        /// Child nodes of this element.
        children: Vec<XmlNode<'gc>>,
    },

    /// A text node in the XML tree.
    Text {
        /// The script object associated with this XML node, if any.
        script_object: Option<Object<'gc>>,

        /// The script object associated with this XML node's attributes, if any.
        attributes_script_object: Option<Object<'gc>>,

        /// The parent node of this one.
        parent: Option<XmlNode<'gc>>,

        /// The previous sibling node to this one.
        prev_sibling: Option<XmlNode<'gc>>,

        /// The next sibling node to this one.
        next_sibling: Option<XmlNode<'gc>>,

        /// The string representation of the text.
        contents: AvmString<'gc>,
    },
}

impl<'gc> XmlNode<'gc> {
    /// Construct a new XML text node.
    pub fn new_text(mc: MutationContext<'gc, '_>, contents: AvmString<'gc>) -> Self {
        XmlNode(GcCell::allocate(
            mc,
            XmlNodeData::Text {
                script_object: None,
                attributes_script_object: None,
                parent: None,
                prev_sibling: None,
                next_sibling: None,
                contents,
            },
        ))
    }

    /// Construct a new XML element node.
    pub fn new_element(mc: MutationContext<'gc, '_>, element_name: AvmString<'gc>) -> Self {
        XmlNode(GcCell::allocate(
            mc,
            XmlNodeData::Element {
                script_object: None,
                parent: None,
                prev_sibling: None,
                next_sibling: None,
                tag_name: element_name,
                attributes: BTreeMap::new(),
                attributes_script_object: None,
                children: Vec::new(),
            },
        ))
    }

    /// Construct a new XML root node.
    pub fn new_document_root(mc: MutationContext<'gc, '_>) -> Self {
        XmlNode(GcCell::allocate(
            mc,
            XmlNodeData::DocumentRoot {
                script_object: None,
                attributes_script_object: None,
                children: Vec::new(),
            },
        ))
    }

    /// Construct an XML Element node from a `quick_xml` `BytesStart` event.
    ///
    /// The returned node will always be an `Element`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
    pub fn from_start_event<'a>(
        mc: MutationContext<'gc, '_>,
        bs: BytesStart<'a>,
        process_entity: bool,
    ) -> Result<Self, Error> {
        let tag_name = AvmString::new_utf8_bytes(mc, bs.name())?;
        let mut attributes = BTreeMap::new();

        for a in bs.attributes() {
            let attribute = a?;
            let value_bytes = if process_entity {
                attribute.unescaped_value()?
            } else {
                attribute.value
            };

            let value = AvmString::new_utf8_bytes(mc, value_bytes)?;
            let attr_key = AvmString::new_utf8_bytes(mc, attribute.key)?;
            attributes.insert(attr_key, value);
        }

        Ok(XmlNode(GcCell::allocate(
            mc,
            XmlNodeData::Element {
                script_object: None,
                parent: None,
                prev_sibling: None,
                next_sibling: None,
                tag_name,
                attributes,
                attributes_script_object: None,
                children: Vec::new(),
            },
        )))
    }

    /// Construct an XML Text node from a `quick_xml` `BytesText` event.
    ///
    /// The returned node will always be `Text`, and it must only contain
    /// valid encoded UTF-8 data. (Other encoding support is planned later.)
    pub fn text_from_text_event<'a>(
        mc: MutationContext<'gc, '_>,
        bt: BytesText<'a>,
        process_entity: bool,
    ) -> Result<Self, Error> {
        let contents = if process_entity {
            bt.unescaped()?
        } else {
            Cow::Borrowed(bt.escaped())
        };

        Ok(XmlNode(GcCell::allocate(
            mc,
            XmlNodeData::Text {
                script_object: None,
                attributes_script_object: None,
                parent: None,
                prev_sibling: None,
                next_sibling: None,
                contents: AvmString::new_utf8_bytes(mc, contents)?,
            },
        )))
    }

    /// Adopt a new child node into the current node.
    ///
    /// This does not add the node to any internal lists; it merely updates the
    /// child to ensure that it considers this node its parent. This function
    /// should always be called after a child node is added to this one. If
    /// you adopt a node that is NOT already added to the children list, bad
    /// things may happen.
    ///
    /// The `new_child_position` parameter is the position of the new child in
    /// this node's child list. This is used to find and link the child's
    /// siblings to each other.
    fn adopt_child(
        &mut self,
        mc: MutationContext<'gc, '_>,
        mut child: XmlNode<'gc>,
        new_child_position: usize,
    ) -> Result<(), Error> {
        if GcCell::ptr_eq(self.0, child.0) {
            return Err(Error::CannotAdoptSelf);
        }

        let (new_prev, new_next) = match &mut *self.0.write(mc) {
            XmlNodeData::Element { children, .. } | XmlNodeData::DocumentRoot { children, .. } => {
                let mut write = child.0.write(mc);
                let child_parent = match &mut *write {
                    XmlNodeData::Element { parent, .. } | XmlNodeData::Text { parent, .. } => {
                        parent
                    }
                    XmlNodeData::DocumentRoot { .. } => return Err(Error::CannotAdoptRoot),
                };

                if let Some(parent) = child_parent {
                    if !GcCell::ptr_eq(self.0, parent.0) {
                        parent.orphan_child(mc, child)?;
                    }
                }

                *child_parent = Some(*self);

                let new_prev = new_child_position
                    .checked_sub(1)
                    .and_then(|p| children.get(p).cloned());
                let new_next = new_child_position
                    .checked_add(1)
                    .and_then(|p| children.get(p).cloned());

                (new_prev, new_next)
            }
            _ => return Err(Error::CannotAdoptHere),
        };

        child.disown_siblings(mc)?;

        child.adopt_siblings(mc, new_prev, new_next)?;

        Ok(())
    }

    /// Get the parent, if this node has one.
    pub fn parent(self) -> Option<XmlNode<'gc>> {
        match *self.0.read() {
            XmlNodeData::DocumentRoot { .. } => None,
            XmlNodeData::Element { parent, .. } | XmlNodeData::Text { parent, .. } => parent,
        }
    }

    /// Get the previous sibling, if this node has one.
    pub fn prev_sibling(self) -> Option<XmlNode<'gc>> {
        match *self.0.read() {
            XmlNodeData::DocumentRoot { .. } => None,
            XmlNodeData::Element { prev_sibling, .. } | XmlNodeData::Text { prev_sibling, .. } => {
                prev_sibling
            }
        }
    }

    /// Set this node's previous sibling.
    fn set_prev_sibling(
        &mut self,
        mc: MutationContext<'gc, '_>,
        new_prev: Option<XmlNode<'gc>>,
    ) -> Result<(), Error> {
        match &mut *self.0.write(mc) {
            XmlNodeData::DocumentRoot { .. } => return Err(Error::RootCantHaveSiblings),
            XmlNodeData::Element { prev_sibling, .. } | XmlNodeData::Text { prev_sibling, .. } => {
                *prev_sibling = new_prev
            }
        };

        Ok(())
    }

    /// Get the next sibling, if this node has one.
    pub fn next_sibling(self) -> Option<XmlNode<'gc>> {
        match *self.0.read() {
            XmlNodeData::DocumentRoot { .. } => None,
            XmlNodeData::Element { next_sibling, .. } | XmlNodeData::Text { next_sibling, .. } => {
                next_sibling
            }
        }
    }

    /// Set this node's next sibling.
    fn set_next_sibling(
        &mut self,
        mc: MutationContext<'gc, '_>,
        new_next: Option<XmlNode<'gc>>,
    ) -> Result<(), Error> {
        match &mut *self.0.write(mc) {
            XmlNodeData::DocumentRoot { .. } => return Err(Error::RootCantHaveSiblings),
            XmlNodeData::Element { next_sibling, .. } | XmlNodeData::Text { next_sibling, .. } => {
                *next_sibling = new_next
            }
        };

        Ok(())
    }

    /// Remove node from its current siblings list.
    ///
    /// If a former sibling exists, we will also adopt it to the opposing side
    /// of this node, so as to maintain a coherent sibling list.
    ///
    /// This is the opposite of `adopt_siblings` - the former adds a node to a
    /// new sibling list, and this removes it from the current one.
    fn disown_siblings(&mut self, mc: MutationContext<'gc, '_>) -> Result<(), Error> {
        let old_prev = self.prev_sibling();
        let old_next = self.next_sibling();

        if let Some(mut prev) = old_prev {
            prev.set_next_sibling(mc, old_next)?;
        }

        if let Some(mut next) = old_next {
            next.set_prev_sibling(mc, old_prev)?;
        }

        self.set_prev_sibling(mc, None)?;
        self.set_next_sibling(mc, None)?;

        Ok(())
    }

    /// Unset the parent of this node.
    fn disown_parent(&mut self, mc: MutationContext<'gc, '_>) -> Result<(), Error> {
        match &mut *self.0.write(mc) {
            XmlNodeData::DocumentRoot { .. } => return Err(Error::RootCantHaveParent),
            XmlNodeData::Element { parent, .. } | XmlNodeData::Text { parent, .. } => {
                *parent = None
            }
        };

        Ok(())
    }

    /// Add node to a new siblings list.
    ///
    /// If a given sibling exists, we will also ensure this node is adopted as
    /// its sibling, so as to maintain a coherent sibling list.
    ///
    /// This is the opposite of `disown_siblings` - the former removes a
    /// sibling from its current list, and this adds the sibling to a new one.
    fn adopt_siblings(
        &mut self,
        mc: MutationContext<'gc, '_>,
        new_prev: Option<XmlNode<'gc>>,
        new_next: Option<XmlNode<'gc>>,
    ) -> Result<(), Error> {
        if let Some(mut prev) = new_prev {
            prev.set_next_sibling(mc, Some(*self))?;
        }

        if let Some(mut next) = new_next {
            next.set_prev_sibling(mc, Some(*self))?;
        }

        self.set_prev_sibling(mc, new_prev)?;
        self.set_next_sibling(mc, new_next)?;

        Ok(())
    }

    /// Remove node from this node's child list.
    ///
    /// This function yields Err if this node cannot accept child nodes.
    fn orphan_child(
        &mut self,
        mc: MutationContext<'gc, '_>,
        child: XmlNode<'gc>,
    ) -> Result<(), Error> {
        if let Some(position) = self.child_position(child) {
            match &mut *self.0.write(mc) {
                XmlNodeData::DocumentRoot { children, .. }
                | XmlNodeData::Element { children, .. } => children.remove(position),
                XmlNodeData::Text { .. } => return Err(Error::TextNodeCantHaveChildren),
            };
        }

        Ok(())
    }

    /// Insert a child element into the child list of an Element node.
    ///
    /// The child will be adopted into the current tree: all child references
    /// to other nodes or documents will be adjusted to reflect its new
    /// position in the tree. This may remove it from any existing trees or
    /// documents.
    ///
    /// This function yields an error if appending to a Node that cannot accept
    /// children. In that case, no modification will be made to the node.
    pub fn insert_child(
        &mut self,
        mc: MutationContext<'gc, '_>,
        position: usize,
        child: XmlNode<'gc>,
    ) -> Result<(), Error> {
        let is_cyclic = self
            .ancestors()
            .any(|ancestor| GcCell::ptr_eq(ancestor.0, child.0));
        if is_cyclic {
            return Err(Error::CannotInsertIntoSelf);
        }

        match &mut *self.0.write(mc) {
            XmlNodeData::Element {
                ref mut children, ..
            }
            | XmlNodeData::DocumentRoot {
                ref mut children, ..
            } => {
                children.insert(position, child);
            }
            _ => return Err(Error::NotAnElement),
        };

        self.adopt_child(mc, child, position)?;

        Ok(())
    }

    /// Append a child element into the end of the child list of an Element
    /// node.
    pub fn append_child(
        &mut self,
        mc: MutationContext<'gc, '_>,
        child: XmlNode<'gc>,
    ) -> Result<(), Error> {
        self.insert_child(mc, self.children_len(), child)
    }

    /// Remove a previously added node from this tree.
    ///
    /// If the node is not a child of this one, or this node cannot accept
    /// children, then this function yields an error.
    pub fn remove_child(
        &mut self,
        mc: MutationContext<'gc, '_>,
        mut child: XmlNode<'gc>,
    ) -> Result<(), Error> {
        if let Some(position) = self.child_position(child) {
            match &mut *self.0.write(mc) {
                XmlNodeData::Element { children, .. }
                | XmlNodeData::DocumentRoot { children, .. } => children.remove(position),
                XmlNodeData::Text { .. } => return Err(Error::TextNodeCantHaveChildren),
            };

            child.disown_siblings(mc)?;
            child.disown_parent(mc)?;
        } else {
            return Err(Error::CantRemoveNonChild);
        }

        Ok(())
    }

    /// Returns the type of this node as an integer.
    ///
    /// This is primarily intended to match W3C DOM L1 specifications and
    /// should not be used in lieu of a proper `match` statement.
    pub fn node_type(self) -> u8 {
        match &*self.0.read() {
            XmlNodeData::DocumentRoot { .. } => xml::DOCUMENT_NODE,
            XmlNodeData::Element { .. } => xml::ELEMENT_NODE,
            XmlNodeData::Text { .. } => xml::TEXT_NODE,
        }
    }

    /// Returns the tagname, if the element has one.
    pub fn tag_name(self) -> Option<AvmString<'gc>> {
        match &*self.0.read() {
            XmlNodeData::Element { ref tag_name, .. } => Some(*tag_name),
            _ => None,
        }
    }

    pub fn local_name(self, gc_context: MutationContext<'gc, '_>) -> Option<AvmString<'gc>> {
        self.tag_name().map(|name| match name.find(b':') {
            Some(i) if i + 1 < name.len() => AvmString::new(gc_context, &name[i + 1..]),
            _ => name,
        })
    }

    pub fn prefix(self, gc_context: MutationContext<'gc, '_>) -> Option<AvmString<'gc>> {
        self.tag_name().map(|name| match name.find(b':') {
            Some(i) if i + 1 < name.len() => AvmString::new(gc_context, &name[..i]),
            _ => "".into(),
        })
    }

    /// Returns the string contents of the node, if the element has them.
    pub fn node_value(self) -> Option<AvmString<'gc>> {
        match &*self.0.read() {
            XmlNodeData::Text { ref contents, .. } => Some(*contents),
            _ => None,
        }
    }

    /// Returns the number of children of the current tree node.
    ///
    /// Nodes that cannot hold children always yield `0`.
    pub fn children_len(self) -> usize {
        match &*self.0.read() {
            XmlNodeData::Element { children, .. } | XmlNodeData::DocumentRoot { children, .. } => {
                children.len()
            }
            _ => 0,
        }
    }

    /// Get the position of a child of this node.
    ///
    /// This function yields None if the node cannot accept children or if the
    /// child node is not a child of this node.
    pub fn child_position(self, child: XmlNode<'gc>) -> Option<usize> {
        self.children()
            .position(|other| GcCell::ptr_eq(child.0, other.0))
    }

    /// Checks if `child` is a direct descendant of `self`.
    pub fn has_child(self, child: XmlNode<'gc>) -> bool {
        child
            .parent()
            .filter(|p| GcCell::ptr_eq(self.0, p.0))
            .is_some()
    }

    /// Retrieve a given child by index (e.g. position in the document).
    pub fn get_child_by_index(self, index: usize) -> Option<XmlNode<'gc>> {
        match &*self.0.read() {
            XmlNodeData::Element { children, .. } | XmlNodeData::DocumentRoot { children, .. } => {
                Some(children)
            }
            _ => None,
        }
        .and_then(|children| children.get(index))
        .cloned()
    }

    /// Returns if the node can yield children.
    ///
    /// Document roots and elements can yield children, while all other
    /// elements are structurally prohibited from adopting child `XMLNode`s.
    pub fn has_children(self) -> bool {
        matches!(
            *self.0.read(),
            XmlNodeData::Element { .. } | XmlNodeData::DocumentRoot { .. }
        )
    }

    /// Returns an iterator that yields child nodes.
    pub fn children(self) -> impl DoubleEndedIterator<Item = XmlNode<'gc>> {
        xml::iterators::ChildIter::for_node(self)
    }

    /// Returns an iterator that yields ancestor nodes (including itself).
    pub fn ancestors(self) -> impl Iterator<Item = XmlNode<'gc>> {
        xml::iterators::AnscIter::for_node(self)
    }

    /// Get the already-instantiated script object from the current node.
    fn get_script_object(self) -> Option<Object<'gc>> {
        match &*self.0.read() {
            XmlNodeData::DocumentRoot { script_object, .. }
            | XmlNodeData::Element { script_object, .. }
            | XmlNodeData::Text { script_object, .. } => *script_object,
        }
    }

    /// Introduce this node to a new script object.
    ///
    /// This internal function *will* overwrite already extant objects, so only
    /// call this if you need to instantiate the script object for the first
    /// time. Attempting to call it a second time will panic.
    pub fn introduce_script_object(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        new_object: Object<'gc>,
    ) {
        assert!(self.get_script_object().is_none(), "An attempt was made to change the already-established link between script object and XML node. This has been denied and is likely a bug.");

        match &mut *self.0.write(gc_context) {
            XmlNodeData::DocumentRoot { script_object, .. }
            | XmlNodeData::Element { script_object, .. }
            | XmlNodeData::Text { script_object, .. } => *script_object = Some(new_object),
        }
    }

    /// Obtain the script object for a given XML tree node, constructing a new
    /// script object if one does not exist.
    pub fn script_object(&mut self, activation: &mut Activation<'_, 'gc, '_>) -> Object<'gc> {
        match self.get_script_object() {
            Some(object) => object,
            None => {
                let proto = activation.context.avm1.prototypes().xml_node;
                XmlNodeObject::from_xml_node(activation.context.gc_context, *self, Some(proto))
                    .into()
            }
        }
    }

    /// Obtain the script object for a given XML tree node's attributes,
    /// constructing a new script object if one does not exist.
    pub fn attribute_script_object(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
    ) -> Option<Object<'gc>> {
        match &mut *self.0.write(gc_context) {
            XmlNodeData::Element {
                attributes_script_object,
                ..
            }
            | XmlNodeData::DocumentRoot {
                attributes_script_object,
                ..
            }
            | XmlNodeData::Text {
                attributes_script_object,
                ..
            } => {
                if attributes_script_object.is_none() {
                    *attributes_script_object =
                        Some(XmlAttributesObject::from_xml_node(gc_context, *self));
                }

                *attributes_script_object
            }
        }
    }

    /// Swap the contents of this node with another one.
    ///
    /// After this function completes, the current `XMLNode` will contain all
    /// data present in the `other` node, and vice versa. References to the node
    /// will *not* be updated: it is a logic error to swap nodes that have
    /// existing referents.
    pub fn swap(&mut self, gc_context: MutationContext<'gc, '_>, other: Self) {
        if !GcCell::ptr_eq(self.0, other.0) {
            swap(
                &mut *self.0.write(gc_context),
                &mut *other.0.write(gc_context),
            );
        }
    }

    // Check if this XML node is constitutes text and only contains whitespace.
    pub fn is_whitespace_text(self) -> bool {
        const WHITESPACE_CHARS: &[u16] = &[b' ' as u16, b'\t' as u16, b'\r' as u16, b'\n' as u16];
        matches!(&*self.0.read(), XmlNodeData::Text { contents, .. } if contents.iter().all(|c| WHITESPACE_CHARS.contains(&c)))
    }

    /// Create a duplicate copy of this node.
    ///
    /// If the `deep` flag is set true, then the entire node tree will be
    /// cloned.
    pub fn duplicate(self, gc_context: MutationContext<'gc, '_>, deep: bool) -> XmlNode<'gc> {
        let mut clone = XmlNode(GcCell::allocate(
            gc_context,
            match &*self.0.read() {
                XmlNodeData::DocumentRoot { .. } => XmlNodeData::DocumentRoot {
                    script_object: None,
                    attributes_script_object: None,
                    children: Vec::new(),
                },
                XmlNodeData::Element {
                    tag_name,
                    attributes,
                    ..
                } => XmlNodeData::Element {
                    script_object: None,
                    parent: None,
                    prev_sibling: None,
                    next_sibling: None,
                    tag_name: *tag_name,
                    attributes: attributes.clone(),
                    attributes_script_object: None,
                    children: Vec::new(),
                },
                XmlNodeData::Text { contents, .. } => XmlNodeData::Text {
                    script_object: None,
                    attributes_script_object: None,
                    parent: None,
                    prev_sibling: None,
                    next_sibling: None,
                    contents: *contents,
                },
            },
        ));

        if deep {
            for (position, child) in self.children().enumerate() {
                clone
                    .insert_child(gc_context, position, child.duplicate(gc_context, deep))
                    .expect("If I can see my children then my clone should accept children");
            }
        }

        clone
    }

    /// Retrieve the value of a single attribute on this node.
    ///
    /// If the node does not contain attributes, then this function always
    /// yields None.
    pub fn attribute_value(self, name: &WStr) -> Option<AvmString<'gc>> {
        match &*self.0.read() {
            XmlNodeData::Element { attributes, .. } => attributes.get(name).copied(),
            _ => None,
        }
    }

    /// Retrieve all keys defined on this node.
    pub fn attribute_keys(self) -> Vec<AvmString<'gc>> {
        match &*self.0.read() {
            XmlNodeData::Element { attributes, .. } => {
                attributes.keys().cloned().collect::<Vec<_>>()
            }
            _ => Vec::new(),
        }
    }

    /// Retrieve the value of a single attribute on this node, case-insensitively.
    ///
    /// TODO: Probably won't need this when we have a proper HTML parser.
    pub fn attribute_value_ignore_case(self, name: &WStr) -> Option<AvmString<'gc>> {
        match &*self.0.read() {
            XmlNodeData::Element { attributes, .. } => attributes
                .iter()
                .find(|(k, _)| k.eq_ignore_case(name))
                .map(|(_, v)| *v),
            _ => None,
        }
    }

    /// Set the value of a single attribute on this node.
    ///
    /// If the node does not contain attributes, then this function silently fails.
    pub fn set_attribute_value(
        self,
        gc_context: MutationContext<'gc, '_>,
        name: AvmString<'gc>,
        value: AvmString<'gc>,
    ) {
        if let XmlNodeData::Element { attributes, .. } = &mut *self.0.write(gc_context) {
            attributes.insert(name, value);
        }
    }

    /// Delete the value of a single attribute on this node.
    ///
    /// If the node does not contain attributes, then this function silently fails.
    pub fn delete_attribute(self, gc_context: MutationContext<'gc, '_>, name: &WStr) {
        if let XmlNodeData::Element { attributes, .. } = &mut *self.0.write(gc_context) {
            attributes.remove(name);
        }
    }

    /// Look up the URI for the given namespace.
    ///
    /// XML namespaces are determined by `xmlns:` namespace attributes on the
    /// current node, or its parent.
    pub fn lookup_uri_for_namespace(self, namespace: &WStr) -> Option<AvmString<'gc>> {
        let mut attribute_name = WString::from_buf(b"xmlns:".to_vec());
        attribute_name.push_str(namespace);

        for node in self.ancestors() {
            if namespace.is_empty() {
                if let Some(url) = node.attribute_value(WStr::from_units(b"xmlns")) {
                    return Some(url);
                }
            }

            if let Some(url) = node.attribute_value(&attribute_name) {
                return Some(url);
            }
        }

        None
    }

    /// Look up the namespace for the given URI.
    ///
    /// XML namespaces are determined by `xmlns:` namespace attributes on the
    /// current node, or its parent.
    ///
    /// If there are multiple namespaces that match the URI, the first
    /// mentioned on the closest node will be returned.
    pub fn lookup_namespace_for_uri(self, uri: &WStr) -> Option<WString> {
        for node in self.ancestors() {
            if let XmlNodeData::Element { attributes, .. } = &*node.0.read() {
                for (attr, attr_value) in attributes.iter() {
                    if attr_value == uri {
                        if attr == b"xmlns" {
                            return Some(WString::new());
                        } else if attr.starts_with(WStr::from_units(b"xmlns:")) {
                            return Some(attr[b"xmlns:".len()..].into());
                        }
                    }
                }
            }
        }

        None
    }

    /// Convert the given node to a string of UTF-8 encoded XML.
    pub fn into_string(self) -> WString {
        let mut result = WString::new();
        self.write_node_to_string(&mut result);
        result
    }

    /// Write the contents of this node, including its children, to the given string.
    fn write_node_to_string(self, result: &mut WString) {
        // TODO: we convert some strings to utf8, replacing unpaired surrogates by the replacement char.
        // It is correct?

        let children: Vec<_> = self.children().collect();

        match &*self.0.read() {
            XmlNodeData::DocumentRoot { .. } => {}
            XmlNodeData::Element {
                tag_name,
                attributes,
                ..
            } => {
                result.push_byte(b'<');
                result.push_str(tag_name);

                for (key, value) in attributes {
                    result.push_byte(b' ');
                    result.push_str(key);
                    result.push_str(WStr::from_units(b"=\""));
                    let encoded_value = value.to_utf8_lossy();
                    let escaped_value = escape(encoded_value.as_bytes());
                    result.push_str(WStr::from_units(&*escaped_value));
                    result.push_byte(b'"');
                }

                if children.is_empty() {
                    result.push_str(WStr::from_units(b" />"));
                } else {
                    result.push_byte(b'>');
                }
            }
            XmlNodeData::Text { contents, .. } => {
                let encoded = contents.to_utf8_lossy();
                let escaped = escape(encoded.as_bytes());
                result.push_str(WStr::from_units(&*escaped));
            }
        }

        for child in &children {
            child.write_node_to_string(result);
        }

        match &*self.0.read() {
            XmlNodeData::DocumentRoot { .. } => {}
            XmlNodeData::Element { tag_name, .. } => {
                if !children.is_empty() {
                    result.push_str(WStr::from_units(b"</"));
                    result.push_str(tag_name);
                    result.push_byte(b'>');
                }
            }
            XmlNodeData::Text { .. } => {}
        }
    }
}

impl<'gc> fmt::Debug for XmlNode<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self.0.read() {
            XmlNodeData::DocumentRoot {
                script_object,
                children,
                ..
            } => f
                .debug_struct("XmlNodeData::DocumentRoot")
                .field("0", &self.0.as_ptr())
                .field(
                    "script_object",
                    &script_object
                        .map(|p| format!("{:p}", p.as_ptr()))
                        .unwrap_or_else(|| "None".to_string()),
                )
                .field("children", children)
                .finish(),
            XmlNodeData::Element {
                script_object,
                tag_name,
                attributes,
                children,
                parent,
                ..
            } => f
                .debug_struct("XmlNodeData::Element")
                .field("0", &self.0.as_ptr())
                .field(
                    "script_object",
                    &script_object
                        .map(|p| format!("{:p}", p.as_ptr()))
                        .unwrap_or_else(|| "None".to_string()),
                )
                .field(
                    "parent",
                    &parent
                        .map(|p| format!("{:p}", p.0.as_ptr()))
                        .unwrap_or_else(|| "None".to_string()),
                )
                .field("tag_name", tag_name)
                .field("attributes", attributes)
                .field("children", children)
                .finish(),
            XmlNodeData::Text {
                script_object,
                contents,
                parent,
                ..
            } => f
                .debug_struct("XmlNodeData::Text")
                .field("0", &self.0.as_ptr())
                .field(
                    "script_object",
                    &script_object
                        .map(|p| format!("{:p}", p.as_ptr()))
                        .unwrap_or_else(|| "None".to_string()),
                )
                .field(
                    "parent",
                    &parent
                        .map(|p| format!("{:p}", p.0.as_ptr()))
                        .unwrap_or_else(|| "None".to_string()),
                )
                .field("contents", contents)
                .finish(),
        }
    }
}
