//! XMLNode class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::xml_node_object::XmlNodeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::avm_warn;
use crate::string::AvmString;
use crate::xml;
use crate::xml::{XmlDocument, XmlNode};
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "localName" => property(local_name; READ_ONLY);
    "nodeName" => property(node_name; READ_ONLY);
    "nodeType" => property(node_type; READ_ONLY);
    "nodeValue" => property(node_value; READ_ONLY);
    "prefix" => property(prefix; READ_ONLY);
    "childNodes" => property(child_nodes; READ_ONLY);
    "firstChild" => property(first_child; READ_ONLY);
    "lastChild" => property(last_child; READ_ONLY);
    "parentNode" => property(parent_node; READ_ONLY);
    "previousSibling" => property(previous_sibling; READ_ONLY);
    "nextSibling" => property(next_sibling; READ_ONLY);
    "attributes" => property(attributes; READ_ONLY);
    "namespaceURI" => property(namespace_uri; READ_ONLY);
    "appendChild" => method(append_child);
    "insertBefore" => method(insert_before);
    "cloneNode" => method(clone_node);
    "getNamespaceForPrefix" => method(get_namespace_for_prefix);
    "getPrefixForNamespace" => method(get_prefix_for_namespace);
    "hasChildNodes" => method(has_child_nodes);
    "removeNode" => method(remove_node);
    "toString" => method(to_string);
};

/// XMLNode constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blank_document = XmlDocument::new(activation.context.gc_context);

    match (
        args.get(0)
            .map(|v| v.coerce_to_f64(activation).map(|v| v as u32)),
        args.get(1).map(|v| v.coerce_to_string(activation)),
        this.as_xml_node(),
    ) {
        (Some(Ok(1)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement =
                XmlNode::new_element(activation.context.gc_context, *strval, blank_document);
            xmlelement.introduce_script_object(activation.context.gc_context, this);
            this_node.swap(activation.context.gc_context, xmlelement);
        }
        (Some(Ok(3)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement =
                XmlNode::new_text(activation.context.gc_context, *strval, blank_document);
            xmlelement.introduce_script_object(activation.context.gc_context, this);
            this_node.swap(activation.context.gc_context, xmlelement);
        }
        //Invalid nodetype ID, string value missing, or not an XMLElement
        _ => {}
    };

    Ok(this.into())
}

fn append_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(mut xmlnode), Some(child_xmlnode)) = (
        this.as_xml_node(),
        args.get(0)
            .and_then(|n| n.coerce_to_object(activation).as_xml_node()),
    ) {
        if !xmlnode.has_child(child_xmlnode) {
            let position = xmlnode.children_len();
            if let Err(e) =
                xmlnode.insert_child(activation.context.gc_context, position, child_xmlnode)
            {
                avm_warn!(
                    activation,
                    "Couldn't insert_child inside of XMLNode.appendChild: {}",
                    e
                );
            }
        }
    }

    Ok(Value::Undefined)
}

fn insert_before<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(mut xmlnode), Some(child_xmlnode), Some(insertpoint_xmlnode)) = (
        this.as_xml_node(),
        args.get(0)
            .and_then(|n| n.coerce_to_object(activation).as_xml_node()),
        args.get(1)
            .and_then(|n| n.coerce_to_object(activation).as_xml_node()),
    ) {
        if !xmlnode.has_child(child_xmlnode) {
            if let Some(position) = xmlnode.child_position(insertpoint_xmlnode) {
                if let Err(e) =
                    xmlnode.insert_child(activation.context.gc_context, position, child_xmlnode)
                {
                    avm_warn!(
                        activation,
                        "Couldn't insert_child inside of XMLNode.insertBefore: {}",
                        e
                    );
                }
            }
        }
    }

    Ok(Value::Undefined)
}

fn clone_node<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), deep) = (
        this.as_xml_node(),
        args.get(0)
            .map(|v| v.as_bool(activation.swf_version()))
            .unwrap_or(false),
    ) {
        let mut clone_node = xmlnode.duplicate(activation.context.gc_context, deep);

        return Ok(clone_node
            .script_object(
                activation.context.gc_context,
                Some(activation.context.avm1.prototypes.xml_node),
            )
            .into());
    }

    Ok(Value::Undefined)
}

fn get_namespace_for_prefix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(prefix_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation)),
    ) {
        if let Some(uri) = xmlnode.lookup_uri_for_namespace(&prefix_string?) {
            Ok(uri.into())
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Undefined)
    }
}

fn get_prefix_for_namespace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(uri_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation)),
    ) {
        if let Some(prefix) = xmlnode.lookup_namespace_for_uri(&uri_string?) {
            Ok(AvmString::new(activation.context.gc_context, prefix).into())
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Undefined)
    }
}

fn has_child_nodes<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(xmlnode) = this.as_xml_node() {
        Ok((xmlnode.children_len() > 0).into())
    } else {
        Ok(Value::Undefined)
    }
}

fn remove_node<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(mut parent) = node.parent() {
            if let Err(e) = parent.remove_child(activation.context.gc_context, node) {
                avm_warn!(activation, "Error in XML.removeNode: {}", e);
            }
        }
    }

    Ok(Value::Undefined)
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let result = node.into_string(&XmlNode::is_as2_compatible);

        return Ok(AvmString::new_utf8(
            activation.context.gc_context,
            result.unwrap_or_else(|e| {
                avm_warn!(activation, "XMLNode toString failed: {}", e);
                "".to_string()
            }),
        )
        .into());
    }

    Ok("".into())
}

fn local_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.local_name(activation.context.gc_context))
        .map_or(Value::Null, Value::from))
}

fn node_name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map_or(Value::Null, Value::from))
}

fn node_type<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .map(|n| {
            match n.node_type() {
                xml::DOCUMENT_NODE => xml::ELEMENT_NODE,
                xml::DOCUMENT_TYPE_NODE => xml::TEXT_NODE,
                xml::COMMENT_NODE => xml::TEXT_NODE,
                n => n,
            }
            .into()
        })
        .unwrap_or_else(|| Value::Undefined))
}

fn node_value<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.node_value())
        .map(|v| v.into())
        .unwrap_or_else(|| Value::Null))
}

fn prefix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.prefix(activation.context.gc_context))
        .map_or(Value::Null, Value::from))
}

fn child_nodes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            node.children()
                .filter(XmlNode::is_as2_compatible)
                .map(|mut child| {
                    child
                        .script_object(
                            activation.context.gc_context,
                            Some(activation.context.avm1.prototypes.xml_node),
                        )
                        .into()
                }),
        )
        .into());
    }

    Ok(Value::Undefined)
}

fn first_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut children = node.children();
        let mut next = children.next();
        while let Some(my_next) = next {
            if my_next.is_as2_compatible() {
                break;
            }

            next = my_next.next_sibling();
        }

        return Ok(next
            .map(|mut child| {
                child
                    .script_object(
                        activation.context.gc_context,
                        Some(activation.context.avm1.prototypes.xml_node),
                    )
                    .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn last_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut children = node.children();
        let mut prev = children.next_back();
        while let Some(my_prev) = prev {
            if my_prev.is_as2_compatible() {
                break;
            }

            prev = my_prev.prev_sibling();
        }
        return Ok(prev
            .map(|mut child| {
                child
                    .script_object(
                        activation.context.gc_context,
                        Some(activation.context.avm1.prototypes.xml_node),
                    )
                    .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn parent_node<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .parent()
            .map(|mut parent| {
                parent
                    .script_object(
                        activation.context.gc_context,
                        Some(activation.context.avm1.prototypes.xml_node),
                    )
                    .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn previous_sibling<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut prev = node.prev_sibling();
        while let Some(my_prev) = prev {
            if my_prev.is_as2_compatible() {
                break;
            }

            prev = my_prev.prev_sibling();
        }

        return Ok(prev
            .map(|mut prev| {
                prev.script_object(
                    activation.context.gc_context,
                    Some(activation.context.avm1.prototypes.xml_node),
                )
                .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn next_sibling<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut next = node.next_sibling();
        while let Some(my_next) = next {
            if my_next.is_as2_compatible() {
                break;
            }

            next = my_next.next_sibling();
        }

        return Ok(next
            .map(|mut next| {
                next.script_object(
                    activation.context.gc_context,
                    Some(activation.context.avm1.prototypes.xml_node),
                )
                .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn attributes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut node) = this.as_xml_node() {
        return Ok(node
            .attribute_script_object(activation.context.gc_context)
            .map(|o| o.into())
            .unwrap_or_else(|| Value::Undefined));
    }

    Ok(Value::Undefined)
}

fn namespace_uri<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(prefix) = node.prefix(activation.context.gc_context) {
            return Ok(node
                .lookup_uri_for_namespace(&prefix)
                .unwrap_or_default()
                .into());
        }

        return Ok(Value::Null);
    }

    Ok(Value::Undefined)
}

/// Construct the prototype for `XMLNode`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xmlnode_proto = XmlNodeObject::empty_node(gc_context, Some(proto));
    let object = xmlnode_proto.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    xmlnode_proto
}
