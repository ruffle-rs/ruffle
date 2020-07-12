//! XML/XMLNode global classes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::object::script_object::ScriptObject;
use crate::avm1::object::xml_object::XMLObject;
use crate::avm1::property::Attribute::*;
use crate::avm1::{Avm1String, Object, TObject, UpdateContext, Value};
use crate::backend::navigator::RequestOptions;
use crate::xml;
use crate::xml::{XMLDocument, XMLNode};
use enumset::EnumSet;
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

/// Returns true if a particular node can or cannot be exposed to AVM1.
///
/// Our internal XML tree representation supports node types that AVM1 XML did
/// not. Those nodes are filtered from all attributes that return XML nodes to
/// act as if those nodes did not exist. For example, `prevSibling` skips
/// past incompatible nodes, etc.
fn is_as2_compatible(node: XMLNode<'_>) -> bool {
    node.is_document_root() || node.is_element() || node.is_text()
}

/// XMLNode constructor
pub fn xmlnode_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blank_document = XMLDocument::new(ac.gc_context);

    match (
        args.get(0)
            .map(|v| v.coerce_to_f64(activation, ac).map(|v| v as u32)),
        args.get(1).map(|v| v.coerce_to_string(activation, ac)),
        this.as_xml_node(),
    ) {
        (Some(Ok(1)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement = XMLNode::new_element(ac.gc_context, strval, blank_document);
            xmlelement.introduce_script_object(ac.gc_context, this);
            this_node.swap(ac.gc_context, xmlelement);
        }
        (Some(Ok(3)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement = XMLNode::new_text(ac.gc_context, strval, blank_document);
            xmlelement.introduce_script_object(ac.gc_context, this);
            this_node.swap(ac.gc_context, xmlelement);
        }
        //Invalid nodetype ID, string value missing, or not an XMLElement
        _ => {}
    };

    Ok(Value::Undefined)
}

pub fn xmlnode_append_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(mut xmlnode), Some(child_xmlnode)) = (
        this.as_xml_node(),
        args.get(0)
            .and_then(|n| n.coerce_to_object(activation, ac).as_xml_node()),
    ) {
        if let Ok(None) = child_xmlnode.parent() {
            let position = xmlnode.children_len();
            if let Err(e) = xmlnode.insert_child(ac.gc_context, position, child_xmlnode) {
                log::warn!("Couldn't insert_child inside of XMLNode.appendChild: {}", e);
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_insert_before<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(mut xmlnode), Some(child_xmlnode), Some(insertpoint_xmlnode)) = (
        this.as_xml_node(),
        args.get(0)
            .and_then(|n| n.coerce_to_object(activation, ac).as_xml_node()),
        args.get(1)
            .and_then(|n| n.coerce_to_object(activation, ac).as_xml_node()),
    ) {
        if let Ok(None) = child_xmlnode.parent() {
            if let Some(position) = xmlnode.child_position(insertpoint_xmlnode) {
                if let Err(e) = xmlnode.insert_child(ac.gc_context, position, child_xmlnode) {
                    log::warn!(
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
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), deep) = (
        this.as_xml_node(),
        args.get(0)
            .map(|v| v.as_bool(activation.current_swf_version()))
            .unwrap_or(false),
    ) {
        let mut clone_node = xmlnode.duplicate(ac.gc_context, deep);

        return Ok(Value::Object(clone_node.script_object(
            ac.gc_context,
            Some(activation.avm.prototypes.xml_node),
        )));
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_get_namespace_for_prefix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(prefix_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation, ac)),
    ) {
        if let Some(uri) = xmlnode.lookup_uri_for_namespace(&prefix_string?) {
            Ok(Avm1String::new(ac.gc_context, uri).into())
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Undefined)
    }
}

pub fn xmlnode_get_prefix_for_namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(uri_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation, ac)),
    ) {
        if let Some(prefix) = xmlnode.lookup_namespace_for_uri(&uri_string?) {
            Ok(Avm1String::new(ac.gc_context, prefix).into())
        } else {
            Ok(Value::Null)
        }
    } else {
        Ok(Value::Undefined)
    }
}

pub fn xmlnode_has_child_nodes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _ac: &mut UpdateContext<'_, 'gc, '_>,
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
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Ok(Some(mut parent)) = node.parent() {
            if let Err(e) = parent.remove_child(ac.gc_context, node) {
                log::warn!("Error in XML.removeNode: {}", e);
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_to_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let result = node.into_string(&mut is_as2_compatible);

        return Ok(Avm1String::new(
            ac.gc_context,
            result.unwrap_or_else(|e| {
                log::warn!("XMLNode toString failed: {}", e);
                "".to_string()
            }),
        )
        .into());
    }

    Ok("".into())
}

pub fn xmlnode_local_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| Avm1String::new(ac.gc_context, n.local_name().to_string()).into())
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_node_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| Avm1String::new(ac.gc_context, n.node_name().to_string()).into())
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_node_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _ac: &mut UpdateContext<'_, 'gc, '_>,
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
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.node_value())
        .map(|n| Avm1String::new(ac.gc_context, n).into())
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_prefix<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| {
            Avm1String::new(
                ac.gc_context,
                n.prefix()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "".to_string()),
            )
            .into()
        })
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_child_nodes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let array = ScriptObject::array(ac.gc_context, Some(activation.avm.prototypes.array));
        if let Some(children) = node.children() {
            let mut compatible_nodes = 0;
            for mut child in children {
                if !is_as2_compatible(child) {
                    continue;
                }

                array.set_array_element(
                    compatible_nodes as usize,
                    child
                        .script_object(ac.gc_context, Some(activation.avm.prototypes.xml_node))
                        .into(),
                    ac.gc_context,
                );

                compatible_nodes += 1;
            }
        }

        return Ok(array.into());
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_first_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(mut children) = node.children() {
            return Ok(children
                .next()
                .map(|mut child| {
                    child
                        .script_object(ac.gc_context, Some(activation.avm.prototypes.xml_node))
                        .into()
                })
                .unwrap_or_else(|| Value::Null));
        }
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_last_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(mut children) = node.children() {
            return Ok(children
                .next_back()
                .map(|mut child| {
                    child
                        .script_object(ac.gc_context, Some(activation.avm.prototypes.xml_node))
                        .into()
                })
                .unwrap_or_else(|| Value::Null));
        }
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_parent_node<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .parent()
            .unwrap_or(None)
            .map(|mut parent| {
                parent
                    .script_object(ac.gc_context, Some(activation.avm.prototypes.xml_node))
                    .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_previous_sibling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut prev = node.prev_sibling().unwrap_or(None);
        while let Some(my_prev) = prev {
            if is_as2_compatible(my_prev) {
                break;
            }

            prev = my_prev.prev_sibling().unwrap_or(None);
        }

        return Ok(prev
            .map(|mut prev| {
                prev.script_object(ac.gc_context, Some(activation.avm.prototypes.xml_node))
                    .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_next_sibling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let mut next = node.next_sibling().unwrap_or(None);
        while let Some(my_next) = next {
            if is_as2_compatible(my_next) {
                break;
            }

            next = my_next.next_sibling().unwrap_or(None);
        }

        return Ok(next
            .map(|mut next| {
                next.script_object(ac.gc_context, Some(activation.avm.prototypes.xml_node))
                    .into()
            })
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_attributes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut node) = this.as_xml_node() {
        return Ok(node
            .attribute_script_object(ac.gc_context)
            .map(|o| o.into())
            .unwrap_or_else(|| Value::Undefined));
    }

    Ok(Value::Undefined)
}

pub fn xmlnode_namespace_uri<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(name) = node.tag_name() {
            return Ok(Avm1String::new(
                ac.gc_context,
                node.lookup_uri_for_namespace(name.prefix().unwrap_or(""))
                    .unwrap_or_else(|| "".to_string()),
            )
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
    let xmlnode_proto = XMLObject::empty_node(gc_context, Some(proto));

    xmlnode_proto.add_property(
        gc_context,
        "localName",
        Executable::Native(xmlnode_local_name),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeName",
        Executable::Native(xmlnode_node_name),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeType",
        Executable::Native(xmlnode_node_type),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeValue",
        Executable::Native(xmlnode_node_value),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "prefix",
        Executable::Native(xmlnode_prefix),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "childNodes",
        Executable::Native(xmlnode_child_nodes),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "firstChild",
        Executable::Native(xmlnode_first_child),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "lastChild",
        Executable::Native(xmlnode_last_child),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "parentNode",
        Executable::Native(xmlnode_parent_node),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "previousSibling",
        Executable::Native(xmlnode_previous_sibling),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nextSibling",
        Executable::Native(xmlnode_next_sibling),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "attributes",
        Executable::Native(xmlnode_attributes),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "namespaceURI",
        Executable::Native(xmlnode_namespace_uri),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "appendChild",
            xmlnode_append_child,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "insertBefore",
            xmlnode_insert_before,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "cloneNode",
            xmlnode_clone_node,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "getNamespaceForPrefix",
            xmlnode_get_namespace_for_prefix,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "getPrefixForNamespace",
            xmlnode_get_prefix_for_namespace,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "hasChildNodes",
            xmlnode_has_child_nodes,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "removeNode",
            xmlnode_remove_node,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "toString",
            xmlnode_to_string,
            gc_context,
            EnumSet::empty(),
            Some(fn_proto),
        );

    xmlnode_proto
}

/// XML (document) constructor
pub fn xml_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    match (
        args.get(0).map(|v| v.coerce_to_string(activation, ac)),
        this.as_xml_node(),
    ) {
        (Some(Ok(ref string)), Some(ref mut this_node)) => {
            let xmldoc = XMLDocument::new(ac.gc_context);
            let mut xmlnode = xmldoc.as_node();
            xmlnode.introduce_script_object(ac.gc_context, this);
            this_node.swap(ac.gc_context, xmlnode);

            if let Err(e) = this_node.replace_with_str(ac.gc_context, string, true) {
                log::warn!("Couldn't replace_with_str inside of XML constructor: {}", e);
            }
        }
        (None, Some(ref mut this_node)) => {
            let xmldoc = XMLDocument::new(ac.gc_context);
            let mut xmlnode = xmldoc.as_node();
            xmlnode.introduce_script_object(ac.gc_context, this);
            this_node.swap(ac.gc_context, xmlnode);
        }
        //Non-string argument or not an XML document
        _ => {}
    };

    Ok(Value::Undefined)
}

pub fn xml_create_element<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let document = if let Some(node) = this.as_xml_node() {
        node.document()
    } else {
        XMLDocument::new(ac.gc_context)
    };

    let nodename = args
        .get(0)
        .map(|v| v.coerce_to_string(activation, ac).unwrap_or_default())
        .unwrap_or_default();
    let mut xml_node = XMLNode::new_element(ac.gc_context, &nodename, document);
    let object = XMLObject::from_xml_node(
        ac.gc_context,
        xml_node,
        Some(activation.avm.prototypes().xml_node),
    );

    xml_node.introduce_script_object(ac.gc_context, object);

    Ok(object.into())
}

pub fn xml_create_text_node<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let document = if let Some(node) = this.as_xml_node() {
        node.document()
    } else {
        XMLDocument::new(ac.gc_context)
    };

    let text_node = args
        .get(0)
        .map(|v| v.coerce_to_string(activation, ac).unwrap_or_default())
        .unwrap_or_default();
    let mut xml_node = XMLNode::new_text(ac.gc_context, &text_node, document);
    let object = XMLObject::from_xml_node(
        ac.gc_context,
        xml_node,
        Some(activation.avm.prototypes().xml_node),
    );

    xml_node.introduce_script_object(ac.gc_context, object);

    Ok(object.into())
}

pub fn xml_parse_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut node) = this.as_xml_node() {
        let xmlstring =
            if let Some(Ok(xmlstring)) = args.get(0).map(|s| s.coerce_to_string(activation, ac)) {
                xmlstring
            } else {
                "".into()
            };

        if let Some(children) = node.children() {
            for child in children.rev() {
                let result = node.remove_child(ac.gc_context, child);
                if let Err(e) = result {
                    log::warn!("XML.parseXML: Error removing node contents: {}", e);
                    return Ok(Value::Undefined);
                }
            }
        }

        let result = node.replace_with_str(ac.gc_context, &xmlstring, true);
        if let Err(e) = result {
            log::warn!("XML parsing error: {}", e);
        }
    }

    Ok(Value::Undefined)
}

pub fn xml_load<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Null = url {
        return Ok(false.into());
    }

    if let Some(node) = this.as_xml_node() {
        let url = url.coerce_to_string(activation, ac)?;

        this.set("loaded", false.into(), activation, ac)?;

        let fetch = ac.navigator.fetch(&url, RequestOptions::get());
        let target_clip = activation.target_clip_or_root();
        let process = ac.load_manager.load_xml_into_node(
            ac.player.clone().unwrap(),
            node,
            target_clip,
            fetch,
        );

        ac.navigator.spawn_future(process);

        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

pub fn xml_on_data<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let src = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Undefined = src {
        this.call_method("onLoad", &[false.into()], activation, ac)?;
    } else {
        let src = src.coerce_to_string(activation, ac)?;
        this.call_method(
            "parseXML",
            &[Avm1String::new(ac.gc_context, src.to_string()).into()],
            activation,
            ac,
        )?;

        this.set("loaded", true.into(), activation, ac)?;

        this.call_method("onLoad", &[true.into()], activation, ac)?;
    }

    Ok(Value::Undefined)
}

pub fn xml_doc_type_decl<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(doctype) = node.document().doctype() {
            let result = doctype.into_string(&mut |_| true);

            return Ok(Avm1String::new(
                ac.gc_context,
                result.unwrap_or_else(|e| {
                    log::warn!("Error occured when serializing DOCTYPE: {}", e);
                    "".to_string()
                }),
            )
            .into());
        }
    }

    Ok(Value::Undefined)
}

pub fn xml_xml_decl<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let result = node.document().xmldecl_string();

        if let Err(e) = result {
            log::warn!("Could not generate XML declaration for document: {}", e);
        } else if let Ok(Some(result_str)) = result {
            return Ok(Avm1String::new(ac.gc_context, result_str).into());
        }
    }

    Ok(Value::Undefined)
}

pub fn xml_id_map<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node.document().idmap_script_object(ac.gc_context).into());
    }

    Ok(Value::Undefined)
}

pub fn xml_status<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return match node.document().last_parse_error() {
            None => Ok(XML_NO_ERROR.into()),
            Some(err) => match err.ref_error() {
                ParseError::UnexpectedEof(_) => Ok(Value::Number(XML_ELEMENT_MALFORMED)),
                ParseError::EndEventMismatch { .. } => Ok(Value::Number(XML_MISMATCHED_END)),
                ParseError::XmlDeclWithoutVersion(_) => Ok(Value::Number(XML_DECL_NOT_TERMINATED)),
                ParseError::NameWithQuote(_) => Ok(Value::Number(XML_ELEMENT_MALFORMED)),
                ParseError::NoEqAfterName(_) => Ok(Value::Number(XML_ELEMENT_MALFORMED)),
                ParseError::UnquotedValue(_) => Ok(Value::Number(XML_ATTRIBUTE_NOT_TERMINATED)),
                ParseError::DuplicatedAttribute(_, _) => Ok(Value::Number(XML_ELEMENT_MALFORMED)),
                _ => Ok(Value::Number(XML_OUT_OF_MEMORY)), //Not accounted for:
                                                           //ParseError::UnexpectedToken(_)
                                                           //ParseError::UnexpectedBang
                                                           //ParseError::TextNotFound
                                                           //ParseError::EscapeError(_)
            },
        };
    }

    Ok(Value::Undefined)
}

/// Construct the prototype for `XML`.
pub fn create_xml_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_proto = XMLObject::empty_node(gc_context, Some(proto));

    xml_proto.add_property(
        gc_context,
        "docTypeDecl",
        Executable::Native(xml_doc_type_decl),
        None,
        ReadOnly.into(),
    );
    xml_proto.add_property(
        gc_context,
        "xmlDecl",
        Executable::Native(xml_xml_decl),
        None,
        ReadOnly.into(),
    );
    xml_proto.add_property(
        gc_context,
        "idMap",
        Executable::Native(xml_id_map),
        None,
        ReadOnly.into(),
    );
    xml_proto.add_property(
        gc_context,
        "status",
        Executable::Native(xml_status),
        None,
        ReadOnly.into(),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "createElement",
        xml_create_element,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "createTextNode",
        xml_create_text_node,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "parseXML",
        xml_parse_xml,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "load",
        xml_load,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "onData",
        xml_on_data,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    xml_proto
}
