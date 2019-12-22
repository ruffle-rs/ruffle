//! XML/XMLNode global classes

use crate::avm1::function::Executable;
use crate::avm1::property::Attribute::*;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::script_object::ScriptObject;
use crate::avm1::xml_object::XMLObject;
use crate::avm1::{Avm1, Error, Object, TObject, UpdateContext, Value};
use crate::xml::{XMLDocument, XMLNode};
use gc_arena::MutationContext;

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
            let xmlelement = XMLNode::new_text(ac.gc_context, strval, blank_document);
            this_node.swap(ac.gc_context, xmlelement);
        }
        (Some(Ok(3)), Some(Ok(ref strval)), Some(ref mut this_node)) => {
            let xmlelement = XMLNode::new_element(ac.gc_context, strval, blank_document)?;
            this_node.swap(ac.gc_context, xmlelement);
        }
        //Invalid nodetype ID, string value missing, or not an XMLElement
        _ => {}
    };

    Ok(Value::Undefined.into())
}

/// Construct the prototype for `XMLNode`.
pub fn create_xmlnode_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    _fn_proto: Object<'gc>,
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
    // TODO: AS2 only ever supported `Element` and `Text` nodes
    xmlnode_proto.add_property(
        gc_context,
        "nodeType",
        Executable::Native(|_avm, _ac, this: Object<'gc>, _args| {
            Ok(this
                .as_xml_node()
                .map(|n| n.node_type().into())
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
                        .unwrap_or("".to_string().into())
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
                    for (i, mut child) in children.enumerate() {
                        array.set_array_element(
                            i as usize,
                            child
                                .script_object(ac.gc_context, Some(avm.prototypes.xml_node))
                                .into(),
                            ac.gc_context,
                        );
                    }

                    return Ok(array.into());
                }
            }

            Ok(Value::Undefined.into())
        }),
        None,
        ReadOnly.into(),
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
            let xmldoc = XMLDocument::from_str(ac.gc_context, string)?;
            this_node.swap(ac.gc_context, xmldoc.as_node());
        }
        (None, Some(ref mut this_node)) => {
            let xmldoc = XMLDocument::new(ac.gc_context);
            this_node.swap(ac.gc_context, xmldoc.as_node());
        }
        //Non-string argument or not an XML document
        _ => {}
    };

    Ok(Value::Undefined.into())
}

/// Construct the prototype for `XML`.
pub fn create_xml_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    _fn_proto: Object<'gc>,
) -> Object<'gc> {
    XMLObject::empty_node(gc_context, Some(proto))
}
