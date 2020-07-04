use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::listeners::Listeners;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::{ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::{Gc, MutationContext};
use std::convert::Into;

fn on_ime_composition<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn do_conversion<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(true.into())
}

fn get_conversion_mode<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Gc::allocate(context.gc_context, "KOREAN".to_string()).into())
}

fn get_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_composition_string<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_conversion_mode<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(false.into())
}

fn set_enabled<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
        Gc::allocate(gc_context, "ALPHANUMERIC_FULL".to_string()).into(),
        Attribute::DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "ALPHANUMERIC_HALF",
        Gc::allocate(gc_context, "ALPHANUMERIC_HALF".to_string()).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "CHINESE",
        Gc::allocate(gc_context, "CHINESE".to_string()).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "JAPANESE_HIRAGANA",
        Gc::allocate(gc_context, "JAPANESE_HIRAGANA".to_string()).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "JAPANESE_KATAKANA_FULL",
        Gc::allocate(gc_context, "JAPANESE_KATAKANA_FULL".to_string()).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "KOREAN",
        Gc::allocate(gc_context, "KOREAN".to_string()).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.define_value(
        gc_context,
        "UNKNOWN",
        Gc::allocate(gc_context, "UNKNOWN".to_string()).into(),
        DontDelete | ReadOnly | DontEnum,
    );

    ime.force_set_function(
        "onIMEComposition",
        on_ime_composition,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.force_set_function(
        "doConversion",
        do_conversion,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.force_set_function(
        "getConversionMode",
        get_conversion_mode,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.force_set_function(
        "getEnabled",
        get_enabled,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.force_set_function(
        "setCompositionString",
        set_composition_string,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.force_set_function(
        "setConversionMode",
        set_conversion_mode,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.force_set_function(
        "setEnabled",
        set_enabled,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    ime.into()
}
