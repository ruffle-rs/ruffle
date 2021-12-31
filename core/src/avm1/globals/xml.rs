//! XML class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::xml_node_object::XmlNodeObject;
use crate::avm1::object::xml_object::XmlObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::avm_warn;
use crate::backend::navigator::RequestOptions;
use crate::string::{AvmString, WStr};
use crate::xml::XmlNode;
use gc_arena::MutationContext;
use quick_xml::Error as ParseError;

const XML_NO_ERROR: i32 = 0;
#[allow(dead_code)]
const XML_CDATA_NOT_TERMINATED: i32 = -2;
const XML_DECL_NOT_TERMINATED: i32 = -3;
#[allow(dead_code)]
const XML_DOCTYPE_NOT_TERMINATED: i32 = -4;
#[allow(dead_code)]
const XML_COMMENT_NOT_TERMINATED: i32 = -5;
const XML_ELEMENT_MALFORMED: i32 = -6;
const XML_OUT_OF_MEMORY: i32 = -7;
const XML_ATTRIBUTE_NOT_TERMINATED: i32 = -8;
#[allow(dead_code)]
const XML_MISMATCHED_START: i32 = -9;
const XML_MISMATCHED_END: i32 = -10;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "docTypeDecl" => property(doc_type_decl; READ_ONLY);
    "ignoreWhite" => bool(false);
    "contentType" => string("application/x-www-form-urlencoded"; READ_ONLY);
    "xmlDecl" => property(xml_decl; READ_ONLY);
    "idMap" => property(id_map; READ_ONLY);
    "status" => property(status; READ_ONLY);
    "createElement" => method(create_element);
    "createTextNode" => method(create_text_node);
    "parseXML" => method(parse_xml);
    "load" => method(load);
    "sendAndLoad" => method(send_and_load);
    "onData" => method(on_data);
};

/// XML (document) constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let (Some(Ok(ref string)), Some(ref mut document)) = (
        args.get(0).map(|v| v.coerce_to_string(activation)),
        this.as_xml(),
    ) {
        let ignore_whitespace = this
            .get("ignoreWhite", activation)?
            .as_bool(activation.swf_version());

        if let Err(e) = document.replace_with_str(
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

    Ok(this.into())
}

fn create_element<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(_document) = this.as_xml() {
        let nodename = args
            .get(0)
            .map(|v| v.coerce_to_string(activation).unwrap_or_default())
            .unwrap_or_default();
        let mut xml_node = XmlNode::new_element(activation.context.gc_context, nodename);
        let object = XmlNodeObject::from_xml_node(
            activation.context.gc_context,
            xml_node,
            Some(activation.context.avm1.prototypes().xml_node),
        );

        xml_node.introduce_script_object(activation.context.gc_context, object);

        return Ok(object.into());
    }

    Ok(Value::Undefined)
}

fn create_text_node<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(_document) = this.as_xml() {
        let text_node = args
            .get(0)
            .map(|v| v.coerce_to_string(activation).unwrap_or_default())
            .unwrap_or_default();
        let mut xml_node = XmlNode::new_text(activation.context.gc_context, text_node);
        let object = XmlNodeObject::from_xml_node(
            activation.context.gc_context,
            xml_node,
            Some(activation.context.avm1.prototypes().xml_node),
        );

        xml_node.introduce_script_object(activation.context.gc_context, object);

        return Ok(object.into());
    }

    Ok(Value::Undefined)
}

fn parse_xml<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut document) = this.as_xml() {
        let xmlstring =
            if let Some(Ok(xmlstring)) = args.get(0).map(|s| s.coerce_to_string(activation)) {
                xmlstring
            } else {
                "".into()
            };

        let mut node = document.as_node();
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

        let result = document.replace_with_str(
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

fn send_and_load<'gc>(
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

    if let Some(document) = this.as_xml() {
        let url = url_val.coerce_to_string(activation)?;
        spawn_xml_fetch(activation, this, target, &url, Some(document.as_node()))?;
    }
    Ok(Value::Undefined)
}

fn load<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let url_val = args.get(0).cloned().unwrap_or(Value::Undefined);

    if let Value::Null = url_val {
        return Ok(false.into());
    }

    if let Some(_document) = this.as_xml() {
        let url = url_val.coerce_to_string(activation)?;
        spawn_xml_fetch(activation, this, this, &url, None)?;

        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

fn on_data<'gc>(
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

fn doc_type_decl<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(document) = this.as_xml() {
        if let Some(doctype) = document.doctype() {
            return Ok(doctype.into());
        }
    }

    Ok(Value::Undefined)
}

fn xml_decl<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(document) = this.as_xml() {
        let result = document.xmldecl_string();

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

fn id_map<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(mut document) = this.as_xml() {
        return Ok(document
            .idmap_script_object(activation.context.gc_context)
            .into());
    }

    Ok(Value::Undefined)
}

fn status<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(document) = this.as_xml() {
        let status = match document.last_parse_error() {
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
    url: &WStr,
    send_object: Option<XmlNode<'gc>>,
) -> Result<Value<'gc>, Error<'gc>> {
    let request_options = if let Some(node) = send_object {
        // Send `node` as string.
        RequestOptions::post(Some((
            node.into_string().unwrap_or_default().into_bytes(),
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
    let process = activation.context.load_manager.load_form_into_load_vars(
        activation.context.player.clone().unwrap(),
        loader_object,
        fetch,
    );

    activation.context.navigator.spawn_future(process);

    Ok(true.into())
}

/// Construct the prototype for `XML`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let xml_proto = XmlObject::empty(gc_context, Some(proto));
    let object = xml_proto.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    xml_proto.into()
}
