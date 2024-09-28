//! XMLNode class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{NativeObject, Object, ScriptObject, TObject, Value};
use crate::string::{AvmString, StringContext, WStr};
use crate::xml::{XmlNode, TEXT_NODE};

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "localName" => property(local_name);
    "nodeName" => property(node_name, set_node_value);
    "nodeType" => property(node_type);
    "nodeValue" => property(node_value, set_node_value);
    "prefix" => property(prefix);
    "childNodes" => property(child_nodes);
    "firstChild" => property(first_child);
    "lastChild" => property(last_child);
    "parentNode" => property(parent_node);
    "previousSibling" => property(previous_sibling);
    "nextSibling" => property(next_sibling);
    "attributes" => property(attributes);
    "namespaceURI" => property(namespace_uri);
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut node = if let [node_type, value, ..] = args {
        let node_type = node_type.coerce_to_u8(activation)?;
        let node_value = value.coerce_to_string(activation)?;
        XmlNode::new(activation.context.gc_context, node_type, Some(node_value))
    } else {
        XmlNode::new(activation.context.gc_context, TEXT_NODE, Some("".into()))
    };
    node.introduce_script_object(activation.context.gc_context, this);
    this.set_native(activation.context.gc_context, NativeObject::XmlNode(node));

    Ok(this.into())
}

fn append_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
            xmlnode.insert_child(activation.context.gc_context, position, child_xmlnode);
            xmlnode.refresh_cached_child_nodes(activation)?;
        }
    }

    Ok(Value::Undefined)
}

fn insert_before<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
                xmlnode.insert_child(activation.context.gc_context, position, child_xmlnode);
                xmlnode.refresh_cached_child_nodes(activation)?;
            }
        }
    }

    Ok(Value::Undefined)
}

fn clone_node<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
        return Ok(clone_node.script_object(activation).into());
    }

    Ok(Value::Undefined)
}

fn get_namespace_for_prefix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(xmlnode), Some(prefix_string)) = (
        this.as_xml_node(),
        args.get(0).map(|v| v.coerce_to_string(activation)),
    ) {
        Ok(xmlnode
            .lookup_namespace_uri(&prefix_string?)
            .unwrap_or(Value::Null))
    } else {
        Ok(Value::Undefined)
    }
}

fn get_prefix_for_namespace<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(node), Some(uri)) = (this.as_xml_node(), args.get(0)) {
        let uri = uri.coerce_to_string(activation)?;
        for ancestor in node.ancestors() {
            // Iterate attributes by their definition order, so the first matching one
            // is returned.
            for (key, value) in ancestor.attributes().own_properties() {
                let value = value.coerce_to_string(activation)?;
                if value == uri {
                    if let Some(prefix) = key.strip_prefix(WStr::from_units(b"xmlns")) {
                        if let Some(prefix) = prefix.strip_prefix(b':') {
                            return Ok(AvmString::new(activation.context.gc_context, prefix).into());
                        } else {
                            return Ok("".into());
                        }
                    }
                }
            }
        }
        return Ok(Value::Null);
    }
    Ok(Value::Undefined)
}

fn has_child_nodes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut node) = this.as_xml_node() {
        let old_parent = node.parent();
        node.remove_node(activation.context.gc_context);
        if let Some(old_parent) = old_parent {
            old_parent.refresh_cached_child_nodes(activation)?;
        }
    }

    Ok(Value::Undefined)
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        let string = node.into_string(activation)?;
        return Ok(AvmString::new(activation.context.gc_context, string).into());
    }

    Ok("".into())
}

fn local_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.local_name(activation.context.gc_context))
        .map_or(Value::Null, Value::from))
}

fn node_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.node_name())
        .map_or(Value::Null, Value::from))
}

/// This functions acts as a setter for both `nodeName` and `nodeValue`.
fn set_node_value<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let [name, ..] = args {
        if name == &Value::Undefined {
            return Ok(Value::Undefined);
        }

        if let Some(node) = this.as_xml_node() {
            node.set_node_value(
                activation.context.gc_context,
                name.coerce_to_string(activation)?,
            );
        }
    }
    Ok(Value::Undefined)
}

fn node_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .map(|n| n.node_type().into())
        .unwrap_or_else(|| Value::Undefined))
}

fn node_value<'gc>(
    _activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this
        .as_xml_node()
        .and_then(|n| n.prefix(activation.context.gc_context))
        .map_or(Value::Null, Value::from))
}

fn child_nodes<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node.get_or_init_cached_child_nodes(activation)?.into());
    }

    Ok(Value::Undefined)
}

fn first_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .children()
            .next()
            .map(|mut child| child.script_object(activation).into())
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn last_child<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .children()
            .next_back()
            .map(|mut child| child.script_object(activation).into())
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn parent_node<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .parent()
            .map(|mut parent| parent.script_object(activation).into())
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn previous_sibling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .prev_sibling()
            .map(|mut prev| prev.script_object(activation).into())
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn next_sibling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node
            .next_sibling()
            .map(|mut next| next.script_object(activation).into())
            .unwrap_or_else(|| Value::Null));
    }

    Ok(Value::Undefined)
}

fn attributes<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        return Ok(node.attributes().into());
    }

    Ok(Value::Undefined)
}

fn namespace_uri<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(node) = this.as_xml_node() {
        if let Some(prefix) = node.prefix(activation.context.gc_context) {
            return Ok(node
                .lookup_namespace_uri(&prefix)
                .unwrap_or_else(|| "".into()));
        }

        return Ok(Value::Null);
    }

    Ok(Value::Undefined)
}

/// Construct the prototype for `XMLNode`.
pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_node_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, xml_node_proto, fn_proto);
    xml_node_proto.into()
}
