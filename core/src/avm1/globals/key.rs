use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, ScriptObject, Value};
use crate::events::KeyCode;
use crate::string::StringContext;

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "ALT" => int(KeyCode::ALT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "BACKSPACE" => int(KeyCode::BACKSPACE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CAPSLOCK" => int(KeyCode::CAPS_LOCK.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CONTROL" => int(KeyCode::CONTROL.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DELETEKEY" => int(KeyCode::DELETE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DOWN" => int(KeyCode::DOWN.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "END" => int(KeyCode::END.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ENTER" => int(KeyCode::RETURN.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ESCAPE" => int(KeyCode::ESCAPE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "HOME" => int(KeyCode::HOME.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "INSERT" => int(KeyCode::INSERT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LEFT" => int(KeyCode::LEFT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGDN" => int(KeyCode::PG_DOWN.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGUP" => int(KeyCode::PG_UP.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "RIGHT" => int(KeyCode::RIGHT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SHIFT" => int(KeyCode::SHIFT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SPACE" => int(KeyCode::SPACE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "TAB" => int(KeyCode::TAB.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "UP" => int(KeyCode::UP.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "isDown" => method(is_down; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "isToggled" => method(is_toggled; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getAscii" => method(get_ascii; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getCode" => method(get_code; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn is_down<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let key = KeyCode::from_code(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)? as u32,
    );

    Ok(activation.context.input.is_key_down(key).into())
}

pub fn is_toggled<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This is not perfect: Flash Player could get the status of the Caps Lock, Num Lock,
    // and Scroll Lock keys properly. We are just toggling them if the Ruffle window is
    // in focus. This is the desired behavior for all keys, except these three.
    let key = KeyCode::from_code(
        args.get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)? as u32,
    );

    Ok(activation.context.input.is_key_toggled(key).into())
}

pub fn get_ascii<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let ord = activation.context.input.last_key_char().unwrap_or_default() as u32;
    Ok(ord.into())
}

pub fn get_code<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let code = activation.context.input.last_key_code().value();
    Ok(code.into())
}

pub fn create_key_object<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let key = ScriptObject::new(context.gc_context, Some(proto));
    broadcaster_functions.initialize(context.gc_context, key.into(), array_proto);
    define_properties_on(OBJECT_DECLS, context, key, fn_proto);
    key.into()
}
