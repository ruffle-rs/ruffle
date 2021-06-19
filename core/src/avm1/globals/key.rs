use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::events::KeyCode;
use gc_arena::MutationContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "ALT" => int(KeyCode::Alt as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "BACKSPACE" => int(KeyCode::Backspace as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CAPSLOCK" => int(KeyCode::CapsLock as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CONTROL" => int(KeyCode::Control as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DELETEKEY" => int(KeyCode::Delete as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DOWN" => int(KeyCode::Down as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "END" => int(KeyCode::End as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ENTER" => int(KeyCode::Return as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ESCAPE" => int(KeyCode::Escape as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "HOME" => int(KeyCode::Home as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "INSERT" => int(KeyCode::Insert as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LEFT" => int(KeyCode::Left as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGDN" => int(KeyCode::PgDown as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGUP" => int(KeyCode::PgUp as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "RIGHT" => int(KeyCode::Right as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SHIFT" => int(KeyCode::Shift as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SPACE" => int(KeyCode::Space as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "TAB" => int(KeyCode::Tab as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "UP" => int(KeyCode::Up as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "isDown" => method(is_down; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getAscii" => method(get_ascii; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getCode" => method(get_code; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn is_down<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(key) = KeyCode::from_u8(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)? as u8,
    ) {
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
    let code = activation.context.ui.last_key_code() as u8;
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
