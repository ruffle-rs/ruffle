use crate::avm1::object::Object;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::{ScriptObject, TObject, Avm1, Value, Error};
use gc_arena::MutationContext;
use std::convert::Into;
use crate::context::UpdateContext;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::property::Attribute;
use crate::avm1::listeners::Listeners;

fn on_ime_composition<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(false.into())
}

fn do_conversion<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(true.into())
}

fn get_conversion_mode<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok("KOREAN".into())
}

fn get_enabled<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(false.into())
}

fn set_composition_string<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(false.into())
}

fn set_conversion_mode<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(false.into())
}

fn set_enabled<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(false.into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
    listener: &Listeners<'gc>,
) -> Object<'gc> {
    let mut ime = ScriptObject::object(gc_context, proto);

    register_listener!(gc_context, ime, listener, fn_proto, ime);

    ime.define_value(
        gc_context,
        "ALPHANUMERIC_FULL",
        "ALPHANUMERIC_FULL".into(),
        Attribute::DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "ALPHANUMERIC_HALF",
        "ALPHANUMERIC_HALF".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "CHINESE",
        "CHINESE".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "JAPANESE_HIRAGANA",
        "JAPANESE_HIRAGANA".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "JAPANESE_KATAKANA_FULL",
        "JAPANESE_KATAKANA_FULL".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "KOREAN",
        "KOREAN".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "UNKNOWN",
        "UNKNOWN".into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.force_set_function(
        "onIMEComposition",
        on_ime_composition,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );

    ime.force_set_function(
        "doConversion",
        do_conversion,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );


    ime.force_set_function(
        "getConversionMode",
        get_conversion_mode,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );

    ime.force_set_function(
        "getEnabled",
        get_enabled,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );

    ime.force_set_function(
        "setCompositionString",
        set_composition_string,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );

    ime.force_set_function(
        "setConversionMode",
        set_conversion_mode,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );

    ime.force_set_function(
        "setEnabled",
        set_enabled,
        gc_context,
        DontDelete | DontEnum,
        fn_proto
    );

    ime.into()
}
