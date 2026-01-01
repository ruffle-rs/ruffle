//! flash.external.ExternalInterface object

use ruffle_common::avm_string::AvmString;
use ruffle_macros::istr;
use ruffle_wstr::{WStr, WString};

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::parameters::{ParametersExt, UndefinedAs};
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{NativeObject, Object, Value};
use crate::avm1_stub;
use crate::external::{Callback, ExternalInterface, Value as ExternalValue};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "_initJS" => method(init_js; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_objectID" => method(object_id; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_addCallback" => method(add_callback2; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_evalJS" => method(eval_js; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_callOut" => method(call_out; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_escapeXML" => method(escape_xml; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_unescapeXML" => method(unescape_xml; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_jsQuoteString" => method(js_quote_string; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_useSetReturnValueHack" => method(use_set_return_value_hack; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "available" => property(get_available; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "addCallback" => method(add_callback; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "call" => method(call; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "_callIn" => method(call_in; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_arrayToXML" => method(array_to_xml; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_argumentsToXML" => method(arguments_to_xml; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_objectToXML" => method(object_to_xml; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_toXML" => method(to_xml; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_objectToAS" => method(object_to_as; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_arrayToAS" => method(array_to_as; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_toAS" => method(to_as; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_argumentsToAS" => method(arguments_to_as; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_arrayToJS" => method(array_to_js; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_objectToJS" => method(object_to_js; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
    "_toJS" => method(to_js; DONT_ENUM | DONT_DELETE | READ_ONLY | VERSION_8);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    // It's a custom prototype but it's empty.
    let class = context.empty_class(super_proto);
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

pub fn get_available<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.external_interface.available().into())
}

pub fn add_callback<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !activation.context.external_interface.available() || args.len() < 3 {
        return Ok(false.into());
    }

    let name = args.get(0).unwrap().coerce_to_string(activation)?;
    let this = args.get(1).unwrap().to_owned();
    let method = args.get(2).unwrap();

    if let Value::Object(method) = method {
        activation.context.external_interface.add_callback(
            name.to_string(),
            Callback::Avm1 {
                this,
                method: *method,
            },
        );
        Ok(true.into())
    } else {
        Ok(false.into())
    }
}

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if !activation.context.external_interface.available() {
        return Ok(Value::Null);
    }

    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    let external_args = args
        .iter()
        .skip(1)
        .map(|arg| ExternalValue::from_avm1(activation, arg.to_owned()))
        .collect::<Result<Vec<ExternalValue>, Error<'gc>>>()?;

    Ok(
        ExternalInterface::call_method(activation.context, &name.to_utf8_lossy(), &external_args)
            .into_avm1(activation),
    )
}

pub fn init_js<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_initJS");
    Ok(Value::Undefined)
}

pub fn object_id<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_objectID");
    Ok(Value::Undefined)
}

pub fn add_callback2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_addCallback"
    );
    Ok(Value::Undefined)
}

pub fn eval_js<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_evalJS");
    Ok(Value::Undefined)
}

pub fn call_out<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_callOut");
    Ok(Value::Undefined)
}

pub fn escape_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(arg) = args.try_get_string(activation, 0, UndefinedAs::Some)? else {
        return Ok(Value::Null);
    };

    Ok(escape_xml_inner(activation, arg.as_wstr()))
}

fn escape_xml_inner<'gc>(activation: &mut Activation<'_, 'gc>, arg: &WStr) -> Value<'gc> {
    let result = arg
        .replace(b'&', WStr::from_units(b"&amp;"))
        .replace(b'"', WStr::from_units(b"&quot;"))
        .replace(b'\'', WStr::from_units(b"&apos;"))
        .replace(b'<', WStr::from_units(b"&lt;"))
        .replace(b'>', WStr::from_units(b"&gt;"));
    if result.is_empty() {
        Value::Null
    } else {
        AvmString::new(activation.gc(), result).into()
    }
}

pub fn unescape_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(arg) = args.try_get_string(activation, 0, UndefinedAs::Some)? else {
        return Ok(Value::Null);
    };

    Ok(unescape_xml_inner(activation, arg.as_wstr()))
}

fn unescape_xml_inner<'gc>(activation: &mut Activation<'_, 'gc>, arg: &WStr) -> Value<'gc> {
    let result = arg
        .replace(WStr::from_units(b"&gt;"), WStr::from_units(b">"))
        .replace(WStr::from_units(b"&lt;"), WStr::from_units(b"<"))
        .replace(WStr::from_units(b"&apos;"), WStr::from_units(b"'"))
        .replace(WStr::from_units(b"&quot;"), WStr::from_units(b"\""))
        .replace(WStr::from_units(b"&amp;"), WStr::from_units(b"&"));
    if result.is_empty() {
        Value::Null
    } else {
        AvmString::new(activation.gc(), result).into()
    }
}

pub fn js_quote_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let Some(arg) = args.try_get_string(activation, 0, UndefinedAs::Some)? else {
        return Ok(Value::Null);
    };

    let result = arg.replace(b'"', WStr::from_units(b"\\\""));
    if result.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(AvmString::new(activation.gc(), result).into())
    }
}

pub fn use_set_return_value_hack<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_useSetReturnValueHack"
    );
    Ok(Value::Undefined)
}

pub fn call_in<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_callIn");
    Ok(Value::Undefined)
}

pub fn array_to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg = args.get_object(activation, 0)?;
    let result = array_to_xml_inner(activation, arg)?;
    Ok(AvmString::new(activation.gc(), result).into())
}

fn array_to_xml_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    arg: Object<'gc>,
) -> Result<WString, Error<'gc>> {
    let mut result = WString::new();
    result.push_utf8_bytes(b"<array>");
    let length = arg.length(activation)?;
    for index in 0..length {
        let value = arg.get_element(activation, index);
        result.push_utf8_bytes(b"<property id=\"");
        result.push_utf8(&index.to_string());
        result.push_utf8_bytes(b"\">");
        result.push_str(&to_xml_inner(activation, value)?);
        result.push_utf8_bytes(b"</property>");
    }
    result.push_utf8_bytes(b"</array>");

    Ok(result)
}

pub fn arguments_to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg = args.get_object(activation, 0)?;
    let result = arguments_to_xml_inner(activation, arg)?;
    Ok(AvmString::new(activation.gc(), result).into())
}

fn arguments_to_xml_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    arg: Object<'gc>,
) -> Result<WString, Error<'gc>> {
    let mut result = WString::new();
    result.push_utf8_bytes(b"<arguments>");
    let length = arg.length(activation)?;
    for index in 1..length {
        let value = arg.get_element(activation, index);
        result.push_str(&to_xml_inner(activation, value)?);
    }
    result.push_utf8_bytes(b"</arguments>");

    Ok(result)
}

pub fn object_to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg = args.get_object(activation, 0)?;
    let result = object_to_xml_inner(activation, arg)?;
    Ok(AvmString::new(activation.gc(), result).into())
}

fn object_to_xml_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    arg: Object<'gc>,
) -> Result<WString, Error<'gc>> {
    let mut result = WString::new();
    result.push_utf8_bytes(b"<object>");
    for (name, value) in arg.own_properties() {
        result.push_utf8_bytes(b"<property id=\"");
        result.push_str(&name);
        result.push_utf8_bytes(b"\">");
        result.push_str(&to_xml_inner(activation, value)?);
        result.push_utf8_bytes(b"</property>");
    }
    result.push_utf8_bytes(b"</object>");

    Ok(result)
}

pub fn to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let arg = args.get_value(0);
    let result = to_xml_inner(activation, arg)?;
    Ok(AvmString::new(activation.gc(), result).into())
}

fn to_xml_inner<'gc>(
    activation: &mut Activation<'_, 'gc>,
    arg: Value<'gc>,
) -> Result<WString, Error<'gc>> {
    Ok(match arg {
        Value::Undefined => WString::from_utf8("<undefined/>"),
        Value::Null => WString::from_utf8("<null/>"),
        Value::Bool(true) => WString::from_utf8("<true/>"),
        Value::Bool(false) => WString::from_utf8("<false/>"),
        Value::Number(_) => {
            let mut result = WString::from_utf8("<number>");
            result.push_str(arg.coerce_to_string(activation)?.as_wstr());
            result.push_utf8_bytes(b"</number>");
            result
        }
        Value::String(string) => {
            let string = escape_xml_inner(activation, string.as_wstr())
                .coerce_to_string(activation)?
                .as_wstr();
            let mut result = WString::from_utf8("<string>");
            result.push_str(string);
            result.push_utf8_bytes(b"</string>");
            result
        }
        _ => {
            let Some(object) = arg.as_object(activation) else {
                // TODO What to do in this case? This is hit when the MCR fails to resolve.
                return Ok(WString::from_utf8("<null/>"));
            };

            if object.has_own_property(activation, istr!("length")) {
                // Yes, `new String("hello")` serializes to an array with 5 undefined elements.
                array_to_xml_inner(activation, object)?
            } else if let NativeObject::Function(_) = object.native() {
                WString::from_utf8("<null/>")
            } else {
                object_to_xml_inner(activation, object)?
            }
        }
    })
}

pub fn object_to_as<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_objectToAS"
    );
    Ok(Value::Undefined)
}

pub fn array_to_as<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_arrayToAS");
    Ok(Value::Undefined)
}

pub fn to_as<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_toAS");
    Ok(Value::Undefined)
}

pub fn arguments_to_as<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_argumentsToAS"
    );
    Ok(Value::Undefined)
}

pub fn array_to_js<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_arrayToJS");
    Ok(Value::Undefined)
}

pub fn object_to_js<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_objectToJS"
    );
    Ok(Value::Undefined)
}

pub fn to_js<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_toJS");
    Ok(Value::Undefined)
}
