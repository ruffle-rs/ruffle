use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::events::KeyCode;
use gc_arena::MutationContext;
use std::convert::TryFrom;

pub fn is_down<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(key) = args
        .get(0)
        .and_then(|v| v.coerce_to_f64(activation).ok())
        .and_then(|k| KeyCode::try_from(k as u8).ok())
    {
        Ok(activation.context.ui.is_key_down(key).into())
    } else {
        Ok(false.into())
    }
}

pub fn get_ascii<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let ord = activation.context.ui.last_key_char().unwrap_or_default() as u32;
    Ok(ord.into())
}

pub fn get_code<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let code: u8 = activation.context.ui.last_key_code().into();
    Ok(code.into())
}

pub fn create_key_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let mut key = ScriptObject::object(gc_context, proto);

    broadcaster_functions.initialize(gc_context, key.into(), array_proto);

    key.define_value(
        gc_context,
        "ALT",
        18.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "BACKSPACE",
        8.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "CAPSLOCK",
        20.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "CONTROL",
        17.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "DELETEKEY",
        46.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "DOWN",
        40.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "END",
        35.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "ENTER",
        13.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "ESCAPE",
        27.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "HOME",
        36.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "INSERT",
        45.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "LEFT",
        37.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "PGDN",
        34.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "PGUP",
        33.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "RIGHT",
        39.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "SHIFT",
        16.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "SPACE",
        32.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "TAB",
        9.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );
    key.define_value(
        gc_context,
        "UP",
        38.into(),
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
    );

    key.force_set_function(
        "isDown",
        is_down,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    key.force_set_function(
        "getAscii",
        get_ascii,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    key.force_set_function(
        "getCode",
        get_code,
        gc_context,
        Attribute::DONT_DELETE | Attribute::READ_ONLY | Attribute::DONT_ENUM,
        fn_proto,
    );

    key.into()
}
