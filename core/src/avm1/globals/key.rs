use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, ScriptObject, TObject, UpdateContext, Value};

use crate::events::KeyCode;
use gc_arena::MutationContext;
use std::convert::TryFrom;

pub fn is_down<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    if let Some(key) = args
        .get(0)
        .and_then(|v| v.coerce_to_f64(avm, context).ok())
        .and_then(|k| KeyCode::try_from(k as u8).ok())
    {
        Ok(context.input.is_key_down(key).into())
    } else {
        Ok(false.into())
    }
}

pub fn get_code<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    let code: u8 = context.input.get_last_key_code().into();
    Ok(code.into())
}

pub fn create_key_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut key = ScriptObject::object(gc_context, proto);

    key.define_value(
        gc_context,
        "ALT",
        18.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "BACKSPACE",
        8.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "CAPSLOCK",
        20.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "CONTROL",
        17.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "DELETEKEY",
        46.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "DOWN",
        40.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "END",
        35.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "ENTER",
        13.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "ESCAPE",
        27.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "HOME",
        36.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "INSERT",
        45.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "LEFT",
        37.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "PGDN",
        34.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "PGUP",
        33.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "RIGHT",
        39.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "SHIFT",
        16.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "SPACE",
        32.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "TAB",
        9.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );
    key.define_value(
        gc_context,
        "UP",
        38.into(),
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
    );

    key.force_set_function(
        "isDown",
        is_down,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    key.force_set_function(
        "getCode",
        get_code,
        gc_context,
        Attribute::DontEnum | Attribute::DontDelete | Attribute::ReadOnly,
        fn_proto,
    );

    key.into()
}
