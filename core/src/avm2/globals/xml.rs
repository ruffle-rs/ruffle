//! XML builtin and prototype

use crate::avm2::e4x::{E4XNode, E4XNodeKind};
pub use crate::avm2::object::xml_allocator;
use crate::avm2::object::{E4XOrXml, QNameObject, TObject, XmlListObject};
use crate::avm2::string::AvmString;
use crate::avm2::{Activation, Error, Multiname, Object, Value};
use crate::avm2_stub_method;

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap().as_xml_object().unwrap();
    let value = args[0];

    match E4XNode::parse(value, activation) {
        Ok(nodes) => {
            let node = match nodes.as_slice() {
                // XML defaults to an empty text node when nothing was parsed
                [] => E4XNode::text(activation.context.gc_context, AvmString::default()),
                [node] => *node,
                _ => {
                    return Err(Error::RustError(
                        format!(
                            "XML constructor must be called with a single node: found {:?}",
                            nodes
                        )
                        .into(),
                    ))
                }
            };
            this.set_node(activation.context.gc_context, node);
        }
        Err(e) => {
            return Err(Error::RustError(
                format!("Failed to parse XML: {e:?}").into(),
            ))
        }
    }

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.unwrap().as_xml_object().unwrap();
    if let Some(local_name) = node.local_name() {
        avm2_stub_method!(activation, "XML", "name", "namespaces");
        // FIXME - use namespace
        let namespace = activation.avm2().public_namespace;
        Ok(QNameObject::from_name(activation, Multiname::new(namespace, local_name))?.into())
    } else {
        Ok(Value::Null)
    }
}

pub fn local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let node = this.unwrap().as_xml_object().unwrap();
    Ok(node.local_name().map_or(Value::Null, Value::String))
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_string(activation)?))
}

pub fn to_xml_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let node = xml.node();
    Ok(Value::String(node.xml_to_xml_string(activation)?))
}

fn name_to_multiname<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &Value<'gc>,
) -> Result<Multiname<'gc>, Error<'gc>> {
    if let Value::Object(o) = name {
        if let Some(qname) = o.as_qname_object() {
            return Ok(qname.name().clone());
        }
    }

    let name = name.coerce_to_string(activation)?;
    Ok(if &*name == b"*" {
        Multiname::any(activation.context.gc_context)
    } else {
        Multiname::new(activation.avm2().public_namespace, name)
    })
}

pub fn child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0])?;
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

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children.iter().map(|node| E4XOrXml::E4X(*node)).collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn elements<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        children
            .iter()
            .filter(|node| matches!(&*node.kind(), E4XNodeKind::Element { .. }))
            .map(|node| E4XOrXml::E4X(*node))
            .collect()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(activation, children, Some(xml.into())).into())
}

pub fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
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
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let xml = this.as_xml_object().unwrap();
    let multiname = name_to_multiname(activation, &args[0])?;
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
