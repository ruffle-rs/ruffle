use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::events::KeyCode;
use gc_arena::MutationContext;
use std::convert::TryFrom;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "ALT" => int(18; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "BACKSPACE" => int(8; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CAPSLOCK" => int(20; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CONTROL" => int(17; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DELETEKEY" => int(46; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DOWN" => int(40; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "END" => int(35; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ENTER" => int(13; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ESCAPE" => int(27; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "HOME" => int(36; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "INSERT" => int(45; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LEFT" => int(37; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGDN" => int(34; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGUP" => int(33; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "RIGHT" => int(39; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SHIFT" => int(16; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SPACE" => int(32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "TAB" => int(9; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "UP" => int(38; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "isDown" => method(is_down; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getAscii" => method(get_ascii; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getCode" => method(get_code; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

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
    fn_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let key = ScriptObject::object(gc_context, proto);
    broadcaster_functions.initialize(gc_context, key.into(), array_proto);
    define_properties_on(OBJECT_DECLS, gc_context, key, fn_proto);
    key.into()
}
