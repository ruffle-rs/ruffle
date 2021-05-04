//! XML/XMLNode global classes

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::script_object::ScriptObject;
use crate::avm1::object::xml_object::XmlObject;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, TObject, Value};
use crate::avm_warn;
use crate::backend::navigator::RequestOptions;
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

/// Returns true if a particular node can or cannot be exposed to AVM1.
///
/// Our internal XML tree representation supports node types that AVM1 XML did
/// not. Those nodes are filtered from all attributes that return XML nodes to
/// act as if those nodes did not exist. For example, `prevSibling` skips
/// past incompatible nodes, etc.
fn is_as2_compatible(node: XmlNode<'_>) -> bool {
    node.is_document_root() || node.is_element() || node.is_text()
}

/// XMLNode constructor
pub fn xmlnode_constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    println!("Creating xml({:?})", args);
    let blank_document = XmlDocument::new(activation.context.gc_context);

    match (
        args.get(0)
            .map(|v| v.coerce_to_f64(activation).map(|v| v as u32)),
        args.get(1).map(|v| v.coerce_to_string(activation)),
        this.as_xml_node(),
    ) {
        (Some(Ok(1)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement =
                XmlNode::new_element(activation.context.gc_context, strval, blank_document);
            xmlelement.introduce_script_object(activation.context.gc_context, this);
            this_node.swap(activation.context.gc_context, xmlelement);
        }
        (Some(Ok(3)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let mut xmlelement =
                XmlNode::new_text(activation.context.gc_context, strval, blank_document);
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

        return Ok(Value::Object(clone_node.script_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.xml_node),
        )));
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
        if let Some(uri) = xmlnode.lookup_uri_for_namespace(&prefix_string?) {
            Ok(AvmString::new(activation.context.gc_context, uri).into())
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
        if let Some(prefix) = xmlnode.lookup_namespace_for_uri(&uri_string?) {
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

        return Ok(AvmString::new(
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
        .map(|n| AvmString::new(activation.context.gc_context, n.local_name().to_string()).into())
        .unwrap_or_else(|| Value::Null))
}

pub fn xmlnode_node_name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.tag_name())
        .map(|n| AvmString::new(activation.context.gc_context, n.node_name().to_string()).into())
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
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.node_value())
        .map(|n| AvmString::new(activation.context.gc_context, n).into())
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
            AvmString::new(
                activation.context.gc_context,
                n.prefix()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "".to_string()),
            )
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
        let array = ScriptObject::array(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.array),
        );

        let mut compatible_nodes = 0;
        for mut child in node.children() {
            if !is_as2_compatible(child) {
                continue;
            }

            array.set_array_element(
                compatible_nodes as usize,
                child
                    .script_object(
                        activation.context.gc_context,
                        Some(activation.context.avm1.prototypes.xml_node),
                    )
                    .into(),
                activation.context.gc_context,
            );

            compatible_nodes += 1;
        }

        return Ok(array.into());
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
            if is_as2_compatible(my_next) {
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
            if is_as2_compatible(my_prev) {
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
            if is_as2_compatible(my_prev) {
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
            if is_as2_compatible(my_next) {
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
            return Ok(AvmString::new(
                activation.context.gc_context,
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
    let xmlnode_proto = XmlObject::empty_node(gc_context, Some(proto));

    xmlnode_proto.add_property(
        gc_context,
        "localName",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_local_name),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeName",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_node_name),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeType",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_node_type),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "nodeValue",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_node_value),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "prefix",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_prefix),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "childNodes",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_child_nodes),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "firstChild",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_first_child),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "lastChild",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_last_child),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "parentNode",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_parent_node),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "previousSibling",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_previous_sibling),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "nextSibling",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_next_sibling),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "attributes",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_attributes),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto.add_property(
        gc_context,
        "namespaceURI",
        FunctionObject::function(
            gc_context,
            Executable::Native(xmlnode_namespace_uri),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "appendChild",
            xmlnode_append_child,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "insertBefore",
            xmlnode_insert_before,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "cloneNode",
            xmlnode_clone_node,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "getNamespaceForPrefix",
            xmlnode_get_namespace_for_prefix,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "getPrefixForNamespace",
            xmlnode_get_prefix_for_namespace,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "hasChildNodes",
            xmlnode_has_child_nodes,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "removeNode",
            xmlnode_remove_node,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );
    xmlnode_proto
        .as_script_object()
        .unwrap()
        .force_set_function(
            "toString",
            xmlnode_to_string,
            gc_context,
            Attribute::empty(),
            Some(fn_proto),
        );

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
                string,
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
    let mut xml_node = XmlNode::new_element(activation.context.gc_context, &nodename, document);
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
    let mut xml_node = XmlNode::new_text(activation.context.gc_context, &text_node, document);
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
            &xmlstring,
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
        spawn_xml_fetch(activation, this, target, &url, Some(node))?;
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
        spawn_xml_fetch(activation, this, this, &url, None)?;

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
        this.call_method("onLoad", &[false.into()], activation)?;
    } else {
        let src = src.coerce_to_string(activation)?;
        this.call_method(
            "parseXML",
            &[AvmString::new(activation.context.gc_context, src.to_string()).into()],
            activation,
        )?;

        this.set("loaded", true.into(), activation)?;

        this.call_method("onLoad", &[true.into()], activation)?;
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

            return Ok(AvmString::new(
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
            return Ok(AvmString::new(activation.context.gc_context, result_str).into());
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

fn spawn_xml_fetch<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    loader_object: Object<'gc>,
    url: &AvmString,
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

    let fetch = activation.context.navigator.fetch(&url, request_options);
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

    xml_proto.add_property(
        gc_context,
        "docTypeDecl",
        FunctionObject::function(
            gc_context,
            Executable::Native(xml_doc_type_decl),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xml_proto.define_value(gc_context, "ignoreWhite", false.into(), Attribute::empty());
    xml_proto.define_value(
        gc_context,
        "contentType",
        "application/x-www-form-urlencoded".into(),
        Attribute::empty(),
    );
    xml_proto.add_property(
        gc_context,
        "xmlDecl",
        FunctionObject::function(
            gc_context,
            Executable::Native(xml_xml_decl),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xml_proto.add_property(
        gc_context,
        "idMap",
        FunctionObject::function(
            gc_context,
            Executable::Native(xml_id_map),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xml_proto.add_property(
        gc_context,
        "status",
        FunctionObject::function(
            gc_context,
            Executable::Native(xml_status),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::READ_ONLY,
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "createElement",
        xml_create_element,
        gc_context,
        Attribute::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "createTextNode",
        xml_create_text_node,
        gc_context,
        Attribute::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "parseXML",
        xml_parse_xml,
        gc_context,
        Attribute::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "load",
        xml_load,
        gc_context,
        Attribute::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "sendAndLoad",
        xml_send_and_load,
        gc_context,
        Attribute::empty(),
        Some(fn_proto),
    );
    xml_proto.as_script_object().unwrap().force_set_function(
        "onData",
        xml_on_data,
        gc_context,
        Attribute::empty(),
        Some(fn_proto),
    );

    xml_proto
}
