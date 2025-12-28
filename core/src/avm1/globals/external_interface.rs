//! flash.external.ExternalInterface object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Object, Value};
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
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_escapeXML");
    Ok(Value::Undefined)
}

pub fn unescape_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_unescapeXML"
    );
    Ok(Value::Undefined)
}

pub fn js_quote_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_jsQuoteString"
    );
    Ok(Value::Undefined)
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
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_arrayToXML"
    );
    Ok(Value::Undefined)
}

pub fn arguments_to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_argumentsToXML"
    );
    Ok(Value::Undefined)
}

pub fn object_to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(
        activation,
        "flash.external.ExternalInterface",
        "_objectToXML"
    );
    Ok(Value::Undefined)
}

pub fn to_xml<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm1_stub!(activation, "flash.external.ExternalInterface", "_toXML");
    Ok(Value::Undefined)
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
