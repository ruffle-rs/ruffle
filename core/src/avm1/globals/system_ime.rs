use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::{ScriptObject, TObject, Value};
use gc_arena::MutationContext;
use std::convert::Into;

fn on_ime_composition<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn do_conversion<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(true.into())
}

fn get_conversion_mode<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok("KOREAN".into())
}

fn get_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_composition_string<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_conversion_mode<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let mut ime = ScriptObject::object(gc_context, proto);

    broadcaster_functions.initialize(gc_context, ime.into(), array_proto);

    ime.define_value(
        gc_context,
        "ALPHANUMERIC_FULL",
        "ALPHANUMERIC_FULL".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.define_value(
        gc_context,
        "ALPHANUMERIC_HALF",
        "ALPHANUMERIC_HALF".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.define_value(
        gc_context,
        "CHINESE",
        "CHINESE".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.define_value(
        gc_context,
        "JAPANESE_HIRAGANA",
        "JAPANESE_HIRAGANA".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.define_value(
        gc_context,
        "JAPANESE_KATAKANA_FULL",
        "JAPANESE_KATAKANA_FULL".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.define_value(
        gc_context,
        "KOREAN",
        "KOREAN".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.define_value(
        gc_context,
        "UNKNOWN",
        "UNKNOWN".into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    ime.force_set_function(
        "onIMEComposition",
        on_ime_composition,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.force_set_function(
        "doConversion",
        do_conversion,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.force_set_function(
        "getConversionMode",
        get_conversion_mode,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.force_set_function(
        "getEnabled",
        get_enabled,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.force_set_function(
        "setCompositionString",
        set_composition_string,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.force_set_function(
        "setConversionMode",
        set_conversion_mode,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.force_set_function(
        "setEnabled",
        set_enabled,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    ime.into()
}
