//! XML/XMLNode global classes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::xml_object::XmlObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{ArrayObject, Object, TObject, Value};
use crate::avm_warn;
use crate::backend::navigator::RequestOptions;
use crate::string::{AvmString, BorrowWStr, WStr};
use crate::xml;
use crate::xml::{XmlDocument, XmlNode};
use gc_arena::MutationContext;
use quick_xml::Error as ParseError;

pub const XML_NO_ERROR: f64 = 0.0;
#[allow(dead_code)]
pub const XML_CDATA_NOT_TERMINATED: f64 = -2.0;
pub const XML_DECL_NOT_TERMINATED: f64 = -3.0;
#[allow(dead_code)]
pub const XML_DOCTYPE_NOT_TERMINATED: f64 = -4.0;
#[allow(dead_code)]
pub const XML_COMMENT_NOT_TERMINATED: f64 = -5.0;
pub const XML_ELEMENT_MALFORMED: f64 = -6.0;
pub const XML_OUT_OF_MEMORY: f64 = -7.0;
pub const XML_ATTRIBUTE_NOT_TERMINATED: f64 = -8.0;
#[allow(dead_code)]
pub const XML_MISMATCHED_START: f64 = -9.0;
pub const XML_MISMATCHED_END: f64 = -10.0;

const XMLNODE_PROTO_DECLS: &[Declaration] = declare_properties! {
    "localName" => property(xmlnode_local_name; READ_ONLY);
    "nodeName" => property(xmlnode_node_name; READ_ONLY);
    "nodeType" => property(xmlnode_node_type; READ_ONLY);
    "nodeValue" => property(xmlnode_node_value; READ_ONLY);
    "prefix" => property(xmlnode_prefix; READ_ONLY);
    "childNodes" => property(xmlnode_child_nodes; READ_ONLY);
    "firstChild" => property(xmlnode_first_child; READ_ONLY);
    "lastChild" => property(xmlnode_last_child; READ_ONLY);
    "parentNode" => property(xmlnode_parent_node; READ_ONLY);
    "previousSibling" => property(xmlnode_previous_sibling; READ_ONLY);
    "nextSibling" => property(xmlnode_next_sibling; READ_ONLY);
    "attributes" => property(xmlnode_attributes; READ_ONLY);
    "namespaceURI" => property(xmlnode_namespace_uri; READ_ONLY);
    "appendChild" => method(xmlnode_append_child);
    "insertBefore" => method(xmlnode_insert_before);
    "cloneNode" => method(xmlnode_clone_node);
    "getNamespaceForPrefix" => method(xmlnode_get_namespace_for_prefix);
    "getPrefixForNamespace" => method(xmlnode_get_prefix_for_namespace);
    "hasChildNodes" => method(xmlnode_has_child_nodes);
    "removeNode" => method(xmlnode_remove_node);
    "toString" => method(xmlnode_to_string);
};

const XML_PROTO_DECLS: &[Declaration] = declare_properties! {
    "docTypeDecl" => property(xml_doc_type_decl; READ_ONLY);
    "ignoreWhite" => bool(false);
    "contentType" => string("application/x-www-form-urlencoded"; READ_ONLY);
    "xmlDecl" => property(xml_xml_decl; READ_ONLY);
    "idMap" => property(xml_id_map; READ_ONLY);
    "status" => property(xml_status; READ_ONLY);
    "createElement" => method(xml_create_element);
    "createTextNode" => method(xml_create_text_node);
    "parseXML" => method(xml_parse_xml);
    "load" => method(xml_load);
    "sendAndLoad" => method(xml_send_and_load);
    "onData" => method(xml_on_data);
};

/// Returns true if a particular node can or cannot be exposed to AVM1.
///
/// Our internal XML tree representation supports node types that AVM1 XML did
/// not. Those nodes are filtered from all attributes that return XML nodes to
/// act as if those nodes did not exist. For example, `prevSibling` skips
/// past incompatible nodes, etc.
fn is_as2_compatible(node: &XmlNode<'_>) -> bool {
    node.is_document_root() || node.is_element() || node.is_text()
}

/// XMLNode constructor
pub fn xmlnode_constructor<'gc>(
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

pub fn xmlnode_append_child<'gc>(
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

pub fn xmlnode_insert_before<'gc>(
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

pub fn xmlnode_clone_node<'gc>(
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

pub fn xmlnode_get_namespace_for_prefix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(prefix_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation)),
    ) {
        if let Some(uri) =
            xmlnode.lookup_uri_for_namespace(activation.context.gc_context, prefix_string?.borrow())
        {
            Ok(uri.into())
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Undefined)
    }
}

pub fn xmlnode_get_prefix_for_namespace<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(uri_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation)),
    ) {
        if let Some(prefix) = xmlnode.lookup_namespace_for_uri(uri_string?.borrow()) {
            Ok(AvmString::new(activation.context.gc_context, prefix).into())
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Undefined)
    }
}

pub fn xmlnode_has_child_nodes<'gc>(
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

pub fn xmlnode_remove_node<'gc>(
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

pub fn xmlnode_to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let result = node.into_string(&mut is_as2_compatible);

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

pub fn xmlnode_local_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| AvmString::new(activation.context.gc_context, n.local_name()).into())
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_node_name<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| Value::from(n.node_name()))
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_node_type<'gc>(
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

pub fn xmlnode_node_value<'gc>(
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

pub fn xmlnode_prefix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| {
            n.prefix()
                .map(|n| AvmString::new(activation.context.gc_context, n))
                .unwrap_or_default()
                .into()
        })
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_child_nodes<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            node.children().filter(is_as2_compatible).map(|mut child| {
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

pub fn xmlnode_first_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut children = node.children();
        let mut next = children.next();
        while let Some(my_next) = next {
            if is_as2_compatible(&my_next) {
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

pub fn xmlnode_last_child<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut children = node.children();
        let mut prev = children.next_back();
        while let Some(my_prev) = prev {
            if is_as2_compatible(&my_prev) {
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

pub fn xmlnode_parent_node<'gc>(
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

pub fn xmlnode_previous_sibling<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut prev = node.prev_sibling();
        while let Some(my_prev) = prev {
            if is_as2_compatible(&my_prev) {
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

pub fn xmlnode_next_sibling<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut next = node.next_sibling();
        while let Some(my_next) = next {
            if is_as2_compatible(&my_next) {
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

pub fn xmlnode_attributes<'gc>(
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

pub fn xmlnode_namespace_uri<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(name) = node.tag_name() {
            return Ok(node
                .lookup_uri_for_namespace(
                    activation.context.gc_context,
                    name.prefix().unwrap_or_default(),
                )
                .unwrap_or_default()
                .into());
        }

        return Ok(Value::Null);
    }

    Ok(Value::Undefined)
}

/// Construct the prototype for `XMLNode`.
pub fn create_xmlnode_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xmlnode_proto = XmlObject::empty_node(gc_context, Some(proto));
    let object = xmlnode_proto.as_script_object().unwrap();
    define_properties_on(XMLNODE_PROTO_DECLS, gc_context, object, fn_proto);
    xmlnode_proto
}

/// XML (document) constructor
pub fn xml_constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match (
        args.get(0).map(|v| v.coerce_to_string(activation)),
        this.as_xml_node(),
    ) {
        (Some(Ok(ref string)), Some(ref mut this_node)) => {
            let xmldoc = XmlDocument::new(activation.context.gc_context);
            let mut xmlnode = xmldoc.as_node();
            xmlnode.introduce_script_object(activation.context.gc_context, this);
            this_node.swap(activation.context.gc_context, xmlnode);
            let ignore_whitespace = this
                .get("ignoreWhite", activation)?
                .as_bool(activation.swf_version());

            if let Err(e) = this_node.replace_with_str(
                activation.context.gc_context,
                string.borrow(),
                true,
                ignore_whitespace,
            ) {
                avm_warn!(
                    activation,
                    "Couldn't replace_with_str inside of XML constructor: {}",
                    e
                );
            }
        }
        (None, Some(ref mut this_node)) => {
            let xmldoc = XmlDocument::new(activation.context.gc_context);
            let mut xmlnode = xmldoc.as_node();
            xmlnode.introduce_script_object(activation.context.gc_context, this);
            this_node.swap(activation.context.gc_context, xmlnode);
        }
        //Non-string argument or not an XML document
        _ => {}
    };

    Ok(this.into())
}

pub fn xml_create_element<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let document = if let Some(node) = this.as_xml_node() {
        node.document()
    } else {
        XmlDocument::new(activation.context.gc_context)
    };

    let nodename = args
        .get(0)
        .map(|v| v.coerce_to_string(activation).unwrap_or_default())
        .unwrap_or_default();
    let mut xml_node = XmlNode::new_element(activation.context.gc_context, nodename, document);
    let object = XmlObject::from_xml_node(
        activation.context.gc_context,
        xml_node,
        Some(activation.context.avm1.prototypes().xml_node),
    );

    xml_node.introduce_script_object(activation.context.gc_context, object);

    Ok(object.into())
}

pub fn xml_create_text_node<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let document = if let Some(node) = this.as_xml_node() {
        node.document()
    } else {
        XmlDocument::new(activation.context.gc_context)
    };

    let text_node = args
        .get(0)
        .map(|v| v.coerce_to_string(activation).unwrap_or_default())
        .unwrap_or_default();
    let mut xml_node = XmlNode::new_text(activation.context.gc_context, text_node, document);
    let object = XmlObject::from_xml_node(
        activation.context.gc_context,
        xml_node,
        Some(activation.context.avm1.prototypes().xml_node),
    );

    xml_node.introduce_script_object(activation.context.gc_context, object);

    Ok(object.into())
}

pub fn xml_parse_xml<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut node) = this.as_xml_node() {
        let xmlstring =
            if let Some(Ok(xmlstring)) = args.get(0).map(|s| s.coerce_to_string(activation)) {
                xmlstring
            } else {
                "".into()
            };

        for child in node.children().rev() {
            let result = node.remove_child(activation.context.gc_context, child);
            if let Err(e) = result {
                avm_warn!(
                    activation,
                    "XML.parseXML: Error removing node contents: {}",
                    e
                );
                return Ok(Value::Undefined);
            }
        }

        let ignore_whitespace = this
            .get("ignoreWhite", activation)?
            .as_bool(activation.swf_version());

        let result = node.replace_with_str(
            activation.context.gc_context,
            xmlstring.borrow(),
            true,
            ignore_whitespace,
        );
        if let Err(e) = result {
            avm_warn!(activation, "XML parsing error: {}", e);
        }
    }

    Ok(Value::Undefined)
}

pub fn xml_send_and_load<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Null = url_val {
        return Ok(Value::Undefined);
    }

    let target = match args.get(1) {
        Some(&Value::Object(o)) => o,
        _ => return Ok(Value::Undefined),
    };

    if let Some(node) = this.as_xml_node() {
        let url = url_val.coerce_to_string(activation)?;
        spawn_xml_fetch(activation, this, target, url.borrow(), Some(node))?;
    }
    Ok(Value::Undefined)
}

pub fn xml_load<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Null = url_val {
        return Ok(false.into());
    }

    if let Some(_node) = this.as_xml_node() {
        let url = url_val.coerce_to_string(activation)?;
        spawn_xml_fetch(activation, this, this, url.borrow(), None)?;

        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

pub fn xml_on_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let src = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Undefined = src {
        this.call_method("onLoad".into(), &[false.into()], activation)?;
    } else {
        let src = src.coerce_to_string(activation)?;
        this.call_method("parseXML".into(), &[src.into()], activation)?;

        this.set("loaded", true.into(), activation)?;

        this.call_method("onLoad".into(), &[true.into()], activation)?;
    }

    Ok(Value::Undefined)
}

pub fn xml_doc_type_decl<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(doctype) = node.document().doctype() {
            let result = doctype.into_string(&mut |_| true);

            return Ok(AvmString::new_utf8(
                activation.context.gc_context,
                result.unwrap_or_else(|e| {
                    avm_warn!(activation, "Error occurred when serializing DOCTYPE: {}", e);
                    "".to_string()
                }),
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}

pub fn xml_xml_decl<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let result = node.document().xmldecl_string();

        if let Err(e) = result {
            avm_warn!(
                activation,
                "Could not generate XML declaration for document: {}",
                e
            );
        } else if let Ok(Some(result_str)) = result {
            return Ok(AvmString::new_utf8(activation.context.gc_context, result_str).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn xml_id_map<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .document()
            .idmap_script_object(activation.context.gc_context)
            .into());
    }

    Ok(Value::Undefined)
}

pub fn xml_status<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let status = match node.document().last_parse_error() {
            None => XML_NO_ERROR,
            Some(err) => match err.ref_error() {
                ParseError::UnexpectedEof(_) => XML_ELEMENT_MALFORMED,
                ParseError::EndEventMismatch { .. } => XML_MISMATCHED_END,
                ParseError::XmlDeclWithoutVersion(_) => XML_DECL_NOT_TERMINATED,
                ParseError::NameWithQuote(_) => XML_ELEMENT_MALFORMED,
                ParseError::NoEqAfterName(_) => XML_ELEMENT_MALFORMED,
                ParseError::UnquotedValue(_) => XML_ATTRIBUTE_NOT_TERMINATED,
                ParseError::DuplicatedAttribute(_, _) => XML_ELEMENT_MALFORMED,
                _ => XML_OUT_OF_MEMORY,
                // Not accounted for:
                // ParseError::UnexpectedToken(_)
                // ParseError::UnexpectedBang
                // ParseError::TextNotFound
                // ParseError::EscapeError(_)
            },
        };
        return Ok(status.into());
    }

    Ok(Value::Undefined)
}

fn spawn_xml_fetch<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    loader_object: Object<'gc>,
    url: WStr<'_>,
    send_object: Option<XmlNode<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    let request_options = if let Some(node) = send_object {
        // Send `node` as string
        RequestOptions::post(Some((
            node.into_string(&mut is_as2_compatible)
                .unwrap_or_default()
                .into_bytes(),
            "application/x-www-form-urlencoded".to_string(),
        )))
    } else {
        // Not sending any parameters.
        RequestOptions::get()
    };

    this.set("loaded", false.into(), activation)?;

    let fetch = activation
        .context
        .navigator
        .fetch(&url.to_utf8_lossy(), request_options);
    let target_clip = activation.target_clip_or_root()?;
    // given any defined loader object, sends the request. Will load into LoadVars if given.
    let process = if let Some(node) = loader_object.as_xml_node() {
        activation.context.load_manager.load_xml_into_node(
            activation.context.player.clone().unwrap(),
            node,
            target_clip,
            fetch,
        )
    } else {
        activation.context.load_manager.load_form_into_load_vars(
            activation.context.player.clone().unwrap(),
            loader_object,
            fetch,
        )
    };

    activation.context.navigator.spawn_future(process);

    Ok(true.into())
}

/// Construct the prototype for `XML`.
pub fn create_xml_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_proto = XmlObject::empty_node(gc_context, Some(proto));
    let object = xml_proto.as_script_object().unwrap();
    define_properties_on(XML_PROTO_DECLS, gc_context, object, fn_proto);
    xml_proto
}
