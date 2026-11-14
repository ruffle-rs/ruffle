//! flash.external.ExternalInterface object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::parameters::{ParametersExt, UndefinedAs};
use crate::avm1::property_decl::{DeclContext, PropertyOrder, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
use crate::avm1_stub;
use crate::external::{Callback, ExternalInterface, Value as ExternalValue};
use ruffle_common::avm_string::AvmString;
use ruffle_wstr::WStr;

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
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    // It's a custom prototype but it's empty.
    let class = context.empty_class(super_proto, PropertyOrder::PrototypeLast);
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

    let result = arg
        .replace(b'&', WStr::from_units(b"&amp;"))
        .replace(b'"', WStr::from_units(b"&quot;"))
        .replace(b'\'', WStr::from_units(b"&apos;"))
        .replace(b'<', WStr::from_units(b"&lt;"))
        .replace(b'>', WStr::from_units(b"&gt;"));
    if result.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(AvmString::new(activation.gc(), result).into())
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

    let result = arg
        .replace(WStr::from_units(b"&gt;"), WStr::from_units(b">"))
        .replace(WStr::from_units(b"&lt;"), WStr::from_units(b"<"))
        .replace(WStr::from_units(b"&apos;"), WStr::from_units(b"'"))
        .replace(WStr::from_units(b"&quot;"), WStr::from_units(b"\""))
        .replace(WStr::from_units(b"&amp;"), WStr::from_units(b"&"));
    if result.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(AvmString::new(activation.gc(), result).into())
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

    let result = arg
        .replace(b'"', WStr::from_units(b"\\\""))
        .replace(b'\n', WStr::from_units(b"\\n"))
        .replace(b'\r', WStr::from_units(b"\\r"));
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
