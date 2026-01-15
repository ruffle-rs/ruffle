use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};
use crate::events::KeyCode;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "CAPSLOCK" => value(KeyCode::CAPS_LOCK.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "BACKSPACE" => value(KeyCode::BACKSPACE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DELETEKEY" => value(KeyCode::DELETE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "INSERT" => value(KeyCode::INSERT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ESCAPE" => value(KeyCode::ESCAPE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SHIFT" => value(KeyCode::SHIFT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "CONTROL" => value(KeyCode::CONTROL.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "TAB" => value(KeyCode::TAB.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "END" => value(KeyCode::END.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "HOME" => value(KeyCode::HOME.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGDN" => value(KeyCode::PAGE_DOWN.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PGUP" => value(KeyCode::PAGE_UP.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "RIGHT" => value(KeyCode::RIGHT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LEFT" => value(KeyCode::LEFT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "DOWN" => value(KeyCode::DOWN.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "UP" => value(KeyCode::UP.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SPACE" => value(KeyCode::SPACE.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ENTER" => value(KeyCode::ENTER.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ALT" => value(KeyCode::ALT.value() as i32; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getAscii" => method(get_ascii; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "getCode" => method(get_code; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "isDown" => method(is_down; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "isToggled" => method(is_toggled; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
    array_proto: Object<'gc>,
) -> Object<'gc> {
    let key = Object::new(context.strings, Some(context.object_proto));
    broadcaster_functions.initialize(context.strings, key, array_proto);
    context.define_properties_on(key, OBJECT_DECLS(context));
    key
}

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
