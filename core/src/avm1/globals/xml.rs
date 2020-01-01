//! XML/XMLNode global classes

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::script_object::ScriptObject;
use crate::avm1::xml_object::XMLObject;
use crate::avm1::{Avm1, Error, Object, TObject, UpdateContext, Value};
use crate::xml;
use crate::xml::{XMLDocument, XMLNode};
use enumset::EnumSet;
use gc_arena::MutationContext;

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
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let blank_document = XMLDocument::new(ac.gc_context);

    match (
        args.get(0).map(|v| v.as_number(avm, ac).map(|v| v as u32)),
        args.get(1).map(|v| v.clone().coerce_to_string(avm, ac)),
        this.as_xml_node(),
    ) {
        (Some(Ok(1)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement = XMLNode::new_element(ac.gc_context, strval, blank_document)?;
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

    Ok(Value::Undefined.into())
}

pub fn xmlnode_append_child<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let (Some(mut xmlnode), Some(Ok(Some(child_xmlnode)))) = (
        this.as_xml_node(),
        args.get(0).map(|n| n.as_object().map(|n| n.as_xml_node())),
    ) {
        if let Ok(None) = child_xmlnode.parent() {
            let position = xmlnode.children_len();
            xmlnode.insert_child(ac.gc_context, position, child_xmlnode)?;
        }
    }

    Ok(Value::Undefined.into())
}

pub fn xmlnode_insert_before<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let (Some(mut xmlnode), Some(Ok(Some(child_xmlnode))), Some(Ok(Some(insertpoint_xmlnode)))) = (
        this.as_xml_node(),
        args.get(0).map(|n| n.as_object().map(|n| n.as_xml_node())),
        args.get(1).map(|n| n.as_object().map(|n| n.as_xml_node())),
    ) {
        if let Ok(None) = child_xmlnode.parent() {
            if let Some(position) = xmlnode.child_position(insertpoint_xmlnode) {
                xmlnode.insert_child(ac.gc_context, position, child_xmlnode)?;
            }
        }
    }

    Ok(Value::Undefined.into())
}

pub fn xmlnode_clone_node<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let (Some(xmlnode), deep) = (
        this.as_xml_node(),
        args.get(0)
            .map(|v| v.as_bool(avm.current_swf_version()))
            .unwrap_or(false),
    ) {
        let mut clone_node = xmlnode.duplicate(ac.gc_context, deep);

        return Ok(Value::Object(
            clone_node.script_object(ac.gc_context, Some(avm.prototypes.xml_node)),
        )
        .into());
    }

    Ok(Value::Undefined.into())
}

pub fn xmlnode_get_namespace_for_prefix<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let (Some(xmlnode), Some(prefix_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.clone().coerce_to_string(avm, ac)),
    ) {
        if let Some(uri) = xmlnode.lookup_uri_for_namespace(&prefix_string?) {
            Ok(uri.into())
        } else {
            Ok(Value::Null.into())
        }
    } else {
        Ok(Value::Undefined.into())
    }
}

pub fn xmlnode_get_prefix_for_namespace<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let (Some(xmlnode), Some(uri_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.clone().coerce_to_string(avm, ac)),
    ) {
        if let Some(prefix) = xmlnode.lookup_namespace_for_uri(&uri_string?) {
            Ok(prefix.into())
        } else {
            Ok(Value::Null.into())
        }
    } else {
        Ok(Value::Undefined.into())
    }
}

pub fn xmlnode_has_child_nodes<'gc>(
    _avm: &mut Avm1<'gc>,
    _ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(xmlnode) = this.as_xml_node() {
        Ok((xmlnode.children_len() > 0).into())
    } else {
        Ok(Value::Undefined.into())
    }
}

#[allow(unused_must_use)]
pub fn xmlnode_remove_node<'gc>(
    _avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(node) = this.as_xml_node() {
        if let Ok(Some(mut parent)) = node.parent() {
            if let Err(e) = parent.remove_child(ac.gc_context, node) {
                log::warn!("Error in XML.removeNode: {}", e);
            }
        }
    }

    Ok(Value::Undefined.into())
}

#[allow(unused_must_use)]
pub fn xmlnode_to_string<'gc>(
    _avm: &mut Avm1<'gc>,
    _ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(node) = this.as_xml_node() {
        let result = node.into_string(&mut is_as2_compatible);

        return Ok(result
            .unwrap_or_else(|e| {
                log::warn!("XMLNode toString failed: {}", e);
                "".to_string()
            })
            .into());
    }

    Ok("".to_string().into())
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
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            Ok(this
                .as_xml_node()
                .and_then(|n| n.tag_name())
                .map(|n| n.local_name().to_string().into())
                .unwrap_or_else(|| Value::Null.into()))
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeName",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            Ok(this
                .as_xml_node()
                .and_then(|n| n.tag_name())
                .map(|n| n.node_name().into())
                .unwrap_or_else(|| Value::Null.into()))
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeType",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
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
                .unwrap_or_else(|| Value::Undefined.into()))
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeValue",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            Ok(this
                .as_xml_node()
                .and_then(|n| n.node_value())
                .map(|n| n.into())
                .unwrap_or_else(|| Value::Null.into()))
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "prefix",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            Ok(this
                .as_xml_node()
                .and_then(|n| n.tag_name())
                .map(|n| {
                    n.prefix()
                        .map(|n| n.to_string().into())
                        .unwrap_or_else(|| "".to_string().into())
                })
                .unwrap_or_else(|| Value::Null.into()))
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "childNodes",
        Executable::Native(|avm, ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                let array = ScriptObject::array(ac.gc_context, Some(avm.prototypes.array));
                if let Some(children) = node.children() {
                    let mut compatible_nodes = 0;
                    for mut child in children {
                        if !is_as2_compatible(child) {
                            continue;
                        }

                        array.set_array_element(
                            compatible_nodes as usize,
                            child
                                .script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                                .into(),
                            ac.gc_context,
                        );

                        compatible_nodes += 1;
                    }
                }

                return Ok(array.into());
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "firstChild",
        Executable::Native(|avm, ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                if let Some(mut children) = node.children() {
                    return Ok(children
                        .next()
                        .map(|mut child| {
                            child
                                .script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                                .into()
                        })
                        .unwrap_or_else(|| Value::Null.into()));
                }
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "lastChild",
        Executable::Native(|avm, ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                if let Some(mut children) = node.children() {
                    return Ok(children
                        .next_back()
                        .map(|mut child| {
                            child
                                .script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                                .into()
                        })
                        .unwrap_or_else(|| Value::Null.into()));
                }
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "parentNode",
        Executable::Native(|avm, ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                return Ok(node
                    .parent()
                    .unwrap_or(None)
                    .map(|mut parent| {
                        parent
                            .script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                            .into()
                    })
                    .unwrap_or_else(|| Value::Null.into()));
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "previousSibling",
        Executable::Native(|avm, ac, this: Object<'gc>, _args| {
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
                        prev.script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                            .into()
                    })
                    .unwrap_or_else(|| Value::Null.into()));
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "nextSibling",
        Executable::Native(|avm, ac, this: Object<'gc>, _args| {
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
                        next.script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                            .into()
                    })
                    .unwrap_or_else(|| Value::Null.into()));
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "attributes",
        Executable::Native(|_avm, ac, this: Object<'gc>, _args| {
            if let Some(mut node) = this.as_xml_node() {
                return Ok(node
                    .attribute_script_object(ac.gc_context)
                    .map(|o| o.into())
                    .unwrap_or_else(|| Value::Undefined.into()));
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xmlnode_proto.add_property(
        gc_context,
        "namespaceURI",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                if let Some(name) = node.tag_name() {
                    return Ok(node
                        .lookup_uri_for_namespace(name.prefix().unwrap_or(""))
                        .map(|s| s.into())
                        .unwrap_or_else(|| "".into()));
                }

                return Ok(Value::Null.into());
            }

            Ok(Value::Undefined.into())
        }),
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
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match (
        args.get(0).map(|v| v.clone().coerce_to_string(avm, ac)),
        this.as_xml_node(),
    ) {
        (Some(Ok(ref string)), Some(ref mut this_node)) => {
            let xmldoc = XMLDocument::new(ac.gc_context);
            let mut xmlnode = xmldoc.as_node();
            xmlnode.introduce_script_object(ac.gc_context, this);
            this_node.swap(ac.gc_context, xmlnode);

            this_node.replace_with_str(ac.gc_context, string)?;
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

    Ok(Value::Undefined.into())
}

#[allow(unused_must_use)]
pub fn xml_create_element<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let document = if let Some(node) = this.as_xml_node() {
        node.document()
    } else {
        XMLDocument::new(ac.gc_context)
    };

    let nodename = args
        .get(0)
        .map(|v| {
            v.clone()
                .coerce_to_string(avm, ac)
                .unwrap_or_else(|_| "".to_string())
        })
        .unwrap_or_else(|| "".to_string());
    let mut xml_node = XMLNode::new_element(ac.gc_context, &nodename, document)?;
    let object = XMLObject::from_xml_node(ac.gc_context, xml_node, Some(avm.prototypes().xml_node));

    xml_node.introduce_script_object(ac.gc_context, object);

    Ok(object.into())
}

#[allow(unused_must_use)]
pub fn xml_create_text_node<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    let document = if let Some(node) = this.as_xml_node() {
        node.document()
    } else {
        XMLDocument::new(ac.gc_context)
    };

    let text_node = args
        .get(0)
        .map(|v| {
            v.clone()
                .coerce_to_string(avm, ac)
                .unwrap_or_else(|_| "".to_string())
        })
        .unwrap_or_else(|| "".to_string());
    let mut xml_node = XMLNode::new_text(ac.gc_context, &text_node, document);
    let object = XMLObject::from_xml_node(ac.gc_context, xml_node, Some(avm.prototypes().xml_node));

    xml_node.introduce_script_object(ac.gc_context, object);

    Ok(object.into())
}

#[allow(unused_must_use)]
pub fn xml_parse_xml<'gc>(
    avm: &mut Avm1<'gc>,
    ac: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(mut node) = this.as_xml_node() {
        let xmlstring =
            if let Some(Ok(xmlstring)) = args.get(0).map(|s| s.clone().coerce_to_string(avm, ac)) {
                xmlstring
            } else {
                "".to_string()
            };

        if let Some(children) = node.children() {
            for child in children.rev() {
                node.remove_child(ac.gc_context, child);
            }
        }

        let result = node.replace_with_str(ac.gc_context, &xmlstring);
        if let Err(e) = result {
            log::warn!("XML parsing error: {}", e);
        }
    }

    Ok(Value::Undefined.into())
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
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                if let Some(doctype) = node.document().doctype() {
                    let result = doctype.into_string(&mut |_| true);

                    return Ok(result
                        .unwrap_or_else(|e| {
                            log::warn!("Error occured when serializing DOCTYPE: {}", e);
                            "".to_string()
                        })
                        .into());
                }
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
    );
    xml_proto.add_property(
        gc_context,
        "xmlDecl",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            if let Some(node) = this.as_xml_node() {
                let result = node.document().xmldecl_string();

                if let Err(e) = result {
                    log::warn!("Could not generate XML declaration for document: {}", e);
                } else if let Ok(Some(result_str)) = result {
                    return Ok(result_str.into());
                }
            }

            Ok(Value::Undefined.into())
        }),
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

    xml_proto
}
