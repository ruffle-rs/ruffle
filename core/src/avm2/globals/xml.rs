//! XML builtin and prototype

use crate::avm2::e4x::{name_to_multiname, E4XNode, E4XNodeKind};
use crate::avm2::error::type_error;
pub use crate::avm2::object::xml_allocator;
use crate::avm2::object::{
    E4XOrXml, NamespaceObject, QNameObject, TObject, XmlListObject, XmlObject,
};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::string::AvmString;
use crate::avm2::{Activation, Error, Multiname, Object, Value};
use crate::avm2_stub_method;

fn ill_formed_markup_err<'gc>(
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    type_error(
        activation,
        "Error #1088: The markup in the document following the root element must be well-formed.",
        1088,
    )
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_xml_object().unwrap();
    let value = args[0];
    let ignore_comments = args.get_bool(1);
    let ignore_processing_instructions = args.get_bool(2);
    let ignore_whitespace = args.get_bool(3);

    if let Some(obj) = value.as_object() {
        if let Some(xml_list) = obj.as_xml_list_object() {
            // Note - 'new XML(new XMLList())' throws an error, even though
            // 'new XML("")' does not. We need this special case to ensure that we return
            // an error, since E4XNode::parse would otherwise return an empty array
            // (which would be accepted)
            if xml_list.length() != 1 {
                return Err(Error::AvmError(ill_formed_markup_err(activation)?));
            }
        }
    }

    let nodes = E4XNode::parse(
        value,
        activation,
        ignore_comments,
        ignore_processing_instructions,
        ignore_whitespace,
    )?;

    let node = match nodes.as_slice() {
        // XML defaults to an empty text node when nothing was parsed
        [] => E4XNode::text(activation.context.gc_context, AvmString::default(), None),
        [node] => *node,
        _ => {
            return Err(Error::AvmError(ill_formed_markup_err(activation)?));
        }
    };
    this.set_node(activation.context.gc_context, node);

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.as_xml_object().unwrap();
    if let Some(local_name) = node.local_name() {
        avm2_stub_method!(activation, "XML", "name", "namespaces");
        // FIXME - use namespace
        let namespace = activation.avm2().public_namespace;
        Ok(QNameObject::from_name(activation, Multiname::new(namespace, local_name))?.into())
    } else {
        Ok(Value::Null)
    }
}

pub fn namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Implement namespace support (including prefix)
    avm2_stub_method!(activation, "XML", "namespace");
    let namespace = activation.avm2().public_namespace;
    Ok(NamespaceObject::from_namespace(activation, namespace)?.into())
}

pub fn local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.as_xml_object().unwrap();
    Ok(node.local_name().map_or(Value::Null, Value::String))
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_string(activation)?))
}

pub fn to_xml_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_xml_string(activation)?))
}

pub fn child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;
    // FIXME: Support numerical indexes.
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| node.matches_name(&multiname))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn child_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();

    let parent = if let Some(parent) = node.parent() {
        parent
    } else {
        return Ok(Value::Number(-1.0));
    };

    if let E4XNodeKind::Attribute(_) = &*node.kind() {
        return Ok(Value::Number(-1.0));
    }

    if let E4XNodeKind::Element { children, .. } = &*parent.kind() {
        let index = children
            .iter()
            .position(|child| E4XNode::ptr_eq(*child, *node))
            .unwrap();
        return Ok(Value::Number(index as f64));
    }

    unreachable!("parent must be an element")
}

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn copy<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(XmlObject::new(node.deep_copy(activation.context.gc_context), activation).into())
}

pub fn parent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let node = xml.node();
    Ok(node.parent().map_or(Value::Undefined, |parent| {
        XmlObject::new(parent, activation).into()
    }))
}

pub fn elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = if args[0] == Value::Undefined {
        Multiname::any(activation.context.gc_context)
    } else {
        name_to_multiname(activation, &args[0], false)?
    };
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| {
                matches!(&*node.kind(), E4XNodeKind::Element { .. })
                    && node.matches_name(&multiname)
            })
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let attributes = if let E4XNodeKind::Element { attributes, .. } = &*xml.node().kind() {
        attributes.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, attributes, Some(xml.into())).into())
}

pub fn attribute<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], true)?;
    let attributes = if let E4XNodeKind::Element { attributes, .. } = &*xml.node().kind() {
        attributes
            .iter()
            .filter(|node| node.matches_name(&multiname))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, attributes, Some(xml.into())).into())
}

pub fn call_handler<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(obj) = args.try_get_object(activation, 0) {
        // We do *not* create a new object when AS does 'XML(someXML)'
        if let Some(xml) = obj.as_xml_object() {
            return Ok(xml.into());
        }
        // This re-uses the XML object stored in the list
        if let Some(xml_list) = obj.as_xml_list_object() {
            if xml_list.length() == 1 {
                return Ok(xml_list.children_mut(activation.context.gc_context)[0]
                    .get_or_create_xml(activation)
                    .into());
            }
            return Err(Error::AvmError(ill_formed_markup_err(activation)?));
        }
    }

    Ok(activation
        .avm2()
        .classes()
        .xml
        .construct(activation, args)?
        .into())
}

pub fn node_kind<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let name = match &*xml.node().kind() {
        E4XNodeKind::Text(_) => "text",
        E4XNodeKind::CData(_) => "text", // cdata pretends to be text here
        E4XNodeKind::Comment(_) => "comment",
        E4XNodeKind::ProcessingInstruction(_) => "processing-instruction",
        E4XNodeKind::Attribute(_) => "attribute",
        E4XNodeKind::Element { .. } => "element",
    };
    Ok(name.into())
}

pub fn append_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();

    let child = args.get_value(0);
    if let Some(child) = child.as_object().and_then(|o| o.as_xml_object()) {
        xml.node()
            .append_child(activation.context.gc_context, *child.node())?;
    } else if let Some(list) = child.as_object().and_then(|o| o.as_xml_list_object()) {
        for child in &*list.children() {
            xml.node()
                .append_child(activation.context.gc_context, *child.node())?;
        }
    } else {
        // Appending a non-XML/XMLList object
        let last_child_name = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
            let num_children = children.len();

            match num_children {
                0 => None,
                _ => children[num_children - 1].local_name(),
            }
        } else {
            // FIXME - figure out exactly when appending is allowed in FP,
            // and throw the proper AVM error.
            return Err(Error::RustError(
                format!(
                    "Cannot append child {child:?} to node {:?}",
                    xml.node().kind()
                )
                .into(),
            ));
        };

        let text = child.coerce_to_string(activation)?;
        if let Some(last_child_name) = last_child_name {
            let element_node =
                E4XNode::element(activation.context.gc_context, last_child_name, *xml.node()); // Creating an element requires passing a parent node, unlike creating a text node

            let text_node = E4XNode::text(activation.context.gc_context, text, None);

            element_node
                .append_child(activation.context.gc_context, text_node)
                .expect("Appending to an element node should succeed");

            xml.node()
                .append_child(activation.context.gc_context, element_node)?;
        } else {
            let node = E4XNode::text(activation.context.gc_context, text, None);
            // The text node will be parented in the append_child operation

            xml.node()
                .append_child(activation.context.gc_context, node)?;
        }
    };

    Ok(this.into())
}

pub fn descendants<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0], false)?;
    let mut descendants = Vec::new();
    xml.node().descendants(&multiname, &mut descendants);
    Ok(XmlListObject::new(activation, descendants, Some(xml.into())).into())
}

pub fn text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.as_xml_object().unwrap();
    let nodes = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| matches!(&*node.kind(), E4XNodeKind::Text(_)))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };
    Ok(XmlListObject::new(activation, nodes, Some(xml.into())).into())
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Integer(1))
}

pub fn has_complex_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_obj = this.as_xml_object().unwrap();
    let result = xml_obj.node().has_complex_content();
    Ok(result.into())
}

pub fn has_simple_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml_obj = this.as_xml_object().unwrap();
    let result = xml_obj.node().has_simple_content();
    Ok(result.into())
}
