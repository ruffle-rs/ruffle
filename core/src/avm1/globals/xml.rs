//! XML class

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::object::xml_object::XmlObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::avm_warn;
use crate::backend::navigator::RequestOptions;
use crate::string::WStr;
use crate::xml::{XmlNode, ELEMENT_NODE, TEXT_NODE};
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "docTypeDecl" => property(doc_type_decl; READ_ONLY);
    "ignoreWhite" => bool(false);
    "contentType" => string("application/x-www-form-urlencoded"; READ_ONLY);
    "xmlDecl" => property(xml_decl);
    "idMap" => property(id_map);
    "status" => property(status);
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

        if let Err(e) = document.replace_with_str(activation, string, ignore_whitespace) {
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
        if let Some(name) = args.get(0) {
            let name = name.coerce_to_string(activation)?;
            let mut node = XmlNode::new(activation.context.gc_context, ELEMENT_NODE, Some(name));
            return Ok(node.script_object(activation).into());
        }
    }

    Ok(Value::Undefined)
}

fn create_text_node<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(_document) = this.as_xml() {
        if let Some(text) = args.get(0) {
            let text = text.coerce_to_string(activation)?;
            let mut node = XmlNode::new(activation.context.gc_context, TEXT_NODE, Some(text));
            return Ok(node.script_object(activation).into());
        }
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

        let node = document.as_node();
        for mut child in node.children().rev() {
            child.remove_node(activation.context.gc_context);
        }

        let ignore_whitespace = this
            .get("ignoreWhite", activation)?
            .as_bool(activation.swf_version());

        let result = document.replace_with_str(activation, &xmlstring, ignore_whitespace);
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
        this.call_method(
            "onLoad".into(),
            &[false.into()],
            activation,
            ExecutionReason::FunctionCall,
        )?;
    } else {
        let src = src.coerce_to_string(activation)?;
        this.call_method(
            "parseXML".into(),
            &[src.into()],
            activation,
            ExecutionReason::FunctionCall,
        )?;

        this.set("loaded", true.into(), activation)?;

        this.call_method(
            "onLoad".into(),
            &[true.into()],
            activation,
            ExecutionReason::FunctionCall,
        )?;
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
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(document) = this.as_xml() {
        if let Some(xml_decl) = document.xml_decl() {
            return Ok(xml_decl.into());
        }
    }

    Ok(Value::Undefined)
}

fn id_map<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(document) = this.as_xml() {
        return Ok(document.id_map().into());
    }

    Ok(Value::Undefined)
}

fn status<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(document) = this.as_xml() {
        return Ok((document.status() as i8).into());
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
        let string = node.into_string(activation)?;
        RequestOptions::post(Some((
            string.to_utf8_lossy().into_owned().into_bytes(),
            "application/x-www-form-urlencoded".to_string(),
        )))
    } else {
        // Not sending any parameters.
        RequestOptions::get()
    };

    this.set("loaded", false.into(), activation)?;

    let future = activation.context.load_manager.load_form_into_load_vars(
        activation.context.player.clone(),
        loader_object,
        &url.to_utf8_lossy(),
        request_options,
    );
    activation.context.navigator.spawn_future(future);

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
