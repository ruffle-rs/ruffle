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
        return Ok(activation.context.input.is_key_down(key).into());
    }
    Ok(false.into())
}

pub fn get_ascii<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let key = activation.context.input.last_key_code();
    let is_shift_down = activation.context.input.is_key_down(KeyCode::Shift);
    let ord = match (key as u8, is_shift_down) {
        (27, _) => key as u8,                                             // Escape
        (32, _) => key as u8,                                             // Space
        (46, _) => 127,                                                   // Delete
        (48..=57, false) => key as u8,                                    // Key0..=Key9
        (48, true) => b')',                                               // Key0
        (49, true) => b'!',                                               // Key1
        (50, true) => b'@',                                               // Key2
        (51, true) => b'#',                                               // Key3
        (52, true) => b'$',                                               // Key4
        (53, true) => b'%',                                               // Key5
        (54, true) => b'^',                                               // Key6
        (55, true) => b'&',                                               // Key7
        (56, true) => b'*',                                               // Key8
        (57, true) => b'(',                                               // Key9
        (64..=90, false) => key as u8,                                    // A..Z
        (64..=90, true) => key as u8 + (b'a' - b'A'),                     // A..Z
        (96..=105, false) => key as u8 - (KeyCode::Numpad0 as u8 - b'0'), // Numpad0..=Numpad9
        (106, _) => b'*',                                                 // Multiply
        (107, _) => b'+',                                                 // Plus
        (109, _) => b'-',                                                 // NumpadMinus
        (110, _) => b'.',                                                 // NumpadPeriod
        (111, _) => b'/',                                                 // NumpadSlash
        (186, false) => b';',                                             // Semicolon
        (186, true) => b':',                                              // Semicolon
        (187, false) => b'=',                                             // Equals
        (187, true) => b'+',                                              // Equals
        (188, false) => b',',                                             // Comma
        (188, true) => b'<',                                              // Comma
        (189, false) => b'-',                                             // Minus
        (189, true) => b'_',                                              // Minus
        (190, false) => b'.',                                             // Period
        (190, true) => b'>',                                              // Period
        (191, false) => b'/',                                             // Slash
        (191, true) => b'?',                                              // Slash
        (192, false) => b'`',                                             // Grave
        (192, true) => b'~',                                              // Grave
        (219, false) => b'[',                                             // LBracket
        (219, true) => b'{',                                              // LBracket
        (220, false) => b'\\',                                            // Backslash
        (220, true) => b'|',                                              // Backslash
        (221, false) => b']',                                             // RBracket
        (221, true) => b'}',                                              // RBracket
        (222, false) => b'\'',                                            // Apostrophe
        (222, true) => b'"',                                              // Apostrophe
        _ => 0,
    };
    Ok(ord.into())
}

pub fn get_code<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let code = activation.context.input.last_key_code() as u8;
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
