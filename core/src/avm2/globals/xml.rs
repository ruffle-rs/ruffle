//! XML builtin and prototype

use crate::avm2::e4x::{E4XNode, E4XNodeKind};
pub use crate::avm2::object::xml_allocator;
use crate::avm2::object::{E4XOrXml, QNameObject, TObject, XmlListObject};
use crate::avm2::{Activation, Error, Object, QName, Value};
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
            if nodes.len() != 1 {
                return Err(Error::RustError(
                    format!(
                        "XML constructor must be called with a single node: found {:?}",
                        nodes
                    )
                    .into(),
                ));
            }
            this.set_node(activation.context.gc_context, nodes[0])
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
        Ok(QNameObject::from_qname(activation, QName::new(namespace, local_name))?.into())
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

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let xml = this.unwrap().as_xml_object().unwrap();
    let children = if let E4XNodeKind::Element { children, .. } = &*xml.node().kind() {
        // FIXME - avoid clone
        children.clone()
    } else {
        Vec::new()
    };

    Ok(XmlListObject::new(
        activation,
        children.iter().map(|node| E4XOrXml::E4X(*node)).collect(),
    )
    .into())
}
