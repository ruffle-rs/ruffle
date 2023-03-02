//! XMLList builtin and prototype

pub use crate::avm2::object::xml_list_allocator;
use crate::{
    avm2::{
        e4x::{simple_content_to_string, E4XNode, E4XNodeKind},
        error::type_error,
        object::{E4XOrXml, XmlListObject},
        Activation, Error, Multiname, Object, TObject, Value,
    },
    avm2_stub_method,
};

fn has_simple_content_inner(children: &[E4XOrXml<'_>]) -> bool {
    match children {
        [] => true,
        [child] => child.node().has_simple_content(),
        _ => children
            .iter()
            .all(|child| !matches!(&*child.node().kind(), E4XNodeKind::Element { .. })),
    }
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap().as_xml_list_object().unwrap();
    let value = args[0];

    match E4XNode::parse(value, activation) {
        Ok(nodes) => {
            this.set_children(
                activation.context.gc_context,
                nodes.into_iter().map(E4XOrXml::E4X).collect(),
            );
        }
        Err(e) => {
            return Err(Error::RustError(
                format!("Failed to parse XML: {e:?}").into(),
            ))
        }
    }

    Ok(Value::Undefined)
}

pub fn has_simple_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list_object().unwrap();
    let children = list.children();
    Ok(has_simple_content_inner(&children).into())
}

pub fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list_object().unwrap();
    let children = list.children();
    if has_simple_content_inner(&children) {
        Ok(simple_content_to_string(children.iter().cloned(), activation)?.into())
    } else {
        avm2_stub_method!(activation, "XMLList", "toString", "non-simple content");
        Err("XMLList.toString() for non-simple content: not yet implemented".into())
    }
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list_object().unwrap();
    let children = list.children();
    Ok(children.len().into())
}

pub fn children<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let list = this.unwrap().as_xml_list_object().unwrap();
    let children = list.children();
    let mut sub_children = Vec::new();
    for child in &*children {
        if let E4XNodeKind::Element { ref children, .. } = &*child.node().kind() {
            sub_children.extend(children.iter().map(|node| E4XOrXml::E4X(*node)));
        }
    }
    Ok(XmlListObject::new(activation, sub_children, Some(list.into())).into())
}

pub fn attribute<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let list = this.as_xml_list_object().unwrap();

    let name = args[0];
    let multiname = match name {
        Value::String(s) => Multiname::new(activation.avm2().public_namespace, s),
        Value::Object(o) => {
            if let Some(qname) = o.as_qname_object() {
                qname.name().clone()
            } else {
                Multiname::new(
                    activation.avm2().public_namespace,
                    name.coerce_to_string(activation)?,
                )
            }
        }
        _ => Multiname::new(
            activation.avm2().public_namespace,
            name.coerce_to_string(activation)?,
        ),
    };

    let children = list.children();
    let mut sub_children = Vec::new();
    for child in &*children {
        if let E4XNodeKind::Element { ref attributes, .. } = &*child.node().kind() {
            if let Some(found) = attributes
                .iter()
                .find(|node| node.matches_name(&multiname))
                .copied()
            {
                sub_children.push(E4XOrXml::E4X(found));
            }
        }
    }
    Ok(XmlListObject::new(activation, sub_children, Some(list.into())).into())
}

pub fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let list = this.as_xml_list_object().unwrap();

    let mut child_attrs = Vec::new();
    for child in list.children().iter() {
        if let E4XNodeKind::Element { ref attributes, .. } = &*child.node().kind() {
            child_attrs.extend(attributes.iter().map(|node| E4XOrXml::E4X(*node)));
        }
    }

    Ok(XmlListObject::new(activation, child_attrs, Some(list.into())).into())
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.unwrap();
    let list = this.as_xml_list_object().unwrap();

    let mut children = list.children_mut(activation.context.gc_context);
    match &mut children[..] {
        [child] => {
            child
                .get_or_create_xml(activation)
                .call_public_property("name", &[], activation)
        }
        _ => Err(Error::AvmError(type_error(
            activation,
            "Error #1086: The name method only works on lists containing one item.",
            1086,
        )?)),
    }
}
