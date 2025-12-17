use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};
use crate::string::AvmString;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "UNKNOWN" => string("UNKNOWN"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "KOREAN" => string("KOREAN"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "JAPANESE_KATAKANA_HALF" => string("JAPANESE_KATAKANA_HALF"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "JAPANESE_KATAKANA_FULL" => string("JAPANESE_KATAKANA_FULL"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "JAPANESE_HIRAGANA" => string("JAPANESE_HIRAGANA"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CHINESE" => string("CHINESE"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ALPHANUMERIC_HALF" => string("ALPHANUMERIC_HALF"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ALPHANUMERIC_FULL" => string("ALPHANUMERIC_FULL"; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getEnabled" => method(get_enabled; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setEnabled" => method(set_enabled; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getConversionMode" => method(get_conversion_mode; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setConversionMode" => method(set_conversion_mode; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "setCompositionString" => method(set_composition_string; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "doConversion" => method(do_conversion; DONT_ENUM | DONT_DELETE | READ_ONLY);
    // TODO: onIMEComposition doesn't look like it's a built-in property.
    "onIMEComposition" => method(on_ime_composition; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let ime = Object::new(context.strings, Some(context.object_proto));
    broadcaster_functions.initialize(context.strings, ime, array_proto);
    context.define_properties_on(ime, OBJECT_DECLS(context));
    ime
}

fn on_ime_composition<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn do_conversion<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(true.into())
}

fn get_conversion_mode<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(AvmString::new_ascii_static(activation.gc(), b"KOREAN").into())
}

fn get_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_composition_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_conversion_mode<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}
