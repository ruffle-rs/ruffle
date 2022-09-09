//! `flash.ui.Keyboard` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::value::Value;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::avm2::{Error, Object};
use crate::events::KeyCode;
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.ui.Keyboard`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.ui.Keyboard`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "Keyboard"),
        Some(Multiname::public("Object")),
        Method::from_builtin(instance_init, "<Keyboard instance initializer>", mc),
        Method::from_builtin(class_init, "<Keyboard class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::FINAL | ClassAttributes::SEALED);

    const STRING_CONSTANTS: &[(&str, &str)] = &[
        ("KEYNAME_UPARROW", "Up"),
        ("KEYNAME_DOWNARROW", "Down"),
        ("KEYNAME_LEFTARROW", "Left"),
        ("KEYNAME_RIGHTARROW", "Right"),
        ("KEYNAME_F1", "F1"),
        ("KEYNAME_F2", "F2"),
        ("KEYNAME_F3", "F3"),
        ("KEYNAME_F4", "F4"),
        ("KEYNAME_F5", "F5"),
        ("KEYNAME_F6", "F6"),
        ("KEYNAME_F7", "F7"),
        ("KEYNAME_F8", "F8"),
        ("KEYNAME_F9", "F9"),
        ("KEYNAME_F10", "F10"),
        ("KEYNAME_F11", "F11"),
        ("KEYNAME_F12", "F12"),
        ("KEYNAME_F13", "F13"),
        ("KEYNAME_F14", "F14"),
        ("KEYNAME_F15", "F15"),
        ("KEYNAME_F16", "F16"),
        ("KEYNAME_F17", "F17"),
        ("KEYNAME_F18", "F18"),
        ("KEYNAME_F19", "F19"),
        ("KEYNAME_F20", "F20"),
        ("KEYNAME_F21", "F21"),
        ("KEYNAME_F22", "F22"),
        ("KEYNAME_F23", "F23"),
        ("KEYNAME_F24", "F24"),
        ("KEYNAME_F25", "F25"),
        ("KEYNAME_F26", "F26"),
        ("KEYNAME_F27", "F27"),
        ("KEYNAME_F28", "F28"),
        ("KEYNAME_F29", "F29"),
        ("KEYNAME_F30", "F30"),
        ("KEYNAME_F31", "F31"),
        ("KEYNAME_F32", "F32"),
        ("KEYNAME_F33", "F33"),
        ("KEYNAME_F34", "F34"),
        ("KEYNAME_F35", "F35"),
        ("KEYNAME_INSERT", "Insert"),
        ("KEYNAME_DELETE", "Delete"),
        ("KEYNAME_HOME", "Home"),
        ("KEYNAME_BEGIN", "Begin"),
        ("KEYNAME_END", "End"),
        ("KEYNAME_PAGEUP", "PgUp"),
        ("KEYNAME_PAGEDOWN", "PgDn"),
        ("KEYNAME_PRINTSCREEN", "PrntScrn"),
        ("KEYNAME_SCROLLLOCK", "ScrlLck"),
        ("KEYNAME_PAUSE", "Pause"),
        ("KEYNAME_SYSREQ", "SysReq"),
        ("KEYNAME_BREAK", "Break"),
        ("KEYNAME_RESET", "Reset"),
        ("KEYNAME_STOP", "Stop"),
        ("KEYNAME_MENU", "Menu"),
        ("KEYNAME_USER", "User"),
        ("KEYNAME_SYSTEM", "Sys"),
        ("KEYNAME_PRINT", "Print"),
        ("KEYNAME_CLEARLINE", "ClrLn"),
        ("KEYNAME_CLEARDISPLAY", "ClrDsp"),
        ("KEYNAME_INSERTLINE", "InsLn"),
        ("KEYNAME_DELETELINE", "DelLn"),
        ("KEYNAME_INSERTCHAR", "InsChr"),
        ("KEYNAME_DELETECHAR", "DelChr"),
        ("KEYNAME_PREV", "Prev"),
        ("KEYNAME_NEXT", "Next"),
        ("KEYNAME_SELECT", "Select"),
        ("KEYNAME_EXECUTE", "Exec"),
        ("KEYNAME_UNDO", "Undo"),
        ("KEYNAME_REDO", "Redo"),
        ("KEYNAME_FIND", "Find"),
        ("KEYNAME_HELP", "Help"),
        ("KEYNAME_MODESWITCH", "ModeSw"),
        ("KEYNAME_PLAYPAUSE", "PlayPause"),
        ("STRING_UPARROW", "\u{f700}"),
        ("STRING_DOWNARROW", "\u{f701}"),
        ("STRING_LEFTARROW", "\u{f702}"),
        ("STRING_RIGHTARROW", "\u{f703}"),
        ("STRING_F1", "\u{f704}"),
        ("STRING_F2", "\u{f705}"),
        ("STRING_F3", "\u{f706}"),
        ("STRING_F4", "\u{f707}"),
        ("STRING_F5", "\u{f708}"),
        ("STRING_F6", "\u{f709}"),
        ("STRING_F7", "\u{f70a}"),
        ("STRING_F8", "\u{f70b}"),
        ("STRING_F9", "\u{f70c}"),
        ("STRING_F10", "\u{f70d}"),
        ("STRING_F11", "\u{f70e}"),
        ("STRING_F12", "\u{f70f}"),
        ("STRING_F13", "\u{f710}"),
        ("STRING_F14", "\u{f711}"),
        ("STRING_F15", "\u{f712}"),
        ("STRING_F16", "\u{f713}"),
        ("STRING_F17", "\u{f714}"),
        ("STRING_F18", "\u{f715}"),
        ("STRING_F19", "\u{f716}"),
        ("STRING_F20", "\u{f717}"),
        ("STRING_F21", "\u{f718}"),
        ("STRING_F22", "\u{f719}"),
        ("STRING_F23", "\u{f71a}"),
        ("STRING_F24", "\u{f71b}"),
        ("STRING_F25", "\u{f71c}"),
        ("STRING_F26", "\u{f71d}"),
        ("STRING_F27", "\u{f71e}"),
        ("STRING_F28", "\u{f71f}"),
        ("STRING_F29", "\u{f720}"),
        ("STRING_F30", "\u{f721}"),
        ("STRING_F31", "\u{f722}"),
        ("STRING_F32", "\u{f723}"),
        ("STRING_F33", "\u{f724}"),
        ("STRING_F34", "\u{f725}"),
        ("STRING_F35", "\u{f726}"),
        ("STRING_INSERT", "\u{f727}"),
        ("STRING_DELETE", "\u{f728}"),
        ("STRING_HOME", "\u{f729}"),
        ("STRING_BEGIN", "\u{f72a}"),
        ("STRING_END", "\u{f72b}"),
        ("STRING_PAGEUP", "\u{f72c}"),
        ("STRING_PAGEDOWN", "\u{f72d}"),
        ("STRING_PRINTSCREEN", "\u{f72e}"),
        ("STRING_SCROLLLOCK", "\u{f72f}"),
        ("STRING_PAUSE", "\u{f730}"),
        ("STRING_SYSREQ", "\u{f731}"),
        ("STRING_BREAK", "\u{f732}"),
        ("STRING_RESET", "\u{f733}"),
        ("STRING_STOP", "\u{f734}"),
        ("STRING_MENU", "\u{f735}"),
        ("STRING_USER", "\u{f736}"),
        ("STRING_SYSTEM", "\u{f737}"),
        ("STRING_PRINT", "\u{f738}"),
        ("STRING_CLEARLINE", "\u{f739}"),
        ("STRING_CLEARDISPLAY", "\u{f73a}"),
        ("STRING_INSERTLINE", "\u{f73b}"),
        ("STRING_DELETELINE", "\u{f73c}"),
        ("STRING_INSERTCHAR", "\u{f73d}"),
        ("STRING_DELETECHAR", "\u{f73e}"),
        ("STRING_PREV", "\u{f73f}"),
        ("STRING_NEXT", "\u{f740}"),
        ("STRING_SELECT", "\u{f741}"),
        ("STRING_EXECUTE", "\u{f742}"),
        ("STRING_UNDO", "\u{f743}"),
        ("STRING_REDO", "\u{f744}"),
        ("STRING_FIND", "\u{f745}"),
        ("STRING_HELP", "\u{f746}"),
        ("STRING_MODESWITCH", "\u{f747}"),
    ];
    write.define_public_constant_string_class_traits(STRING_CONSTANTS);

    // TODO: Add more of these constants to KeyCode enum.
    const UINT_CONSTANTS: &[(&str, u32)] = &[
        ("NUMBER_0", KeyCode::Key0 as u32),
        ("NUMBER_1", KeyCode::Key1 as u32),
        ("NUMBER_2", KeyCode::Key2 as u32),
        ("NUMBER_3", KeyCode::Key3 as u32),
        ("NUMBER_4", KeyCode::Key4 as u32),
        ("NUMBER_5", KeyCode::Key5 as u32),
        ("NUMBER_6", KeyCode::Key6 as u32),
        ("NUMBER_7", KeyCode::Key7 as u32),
        ("NUMBER_8", KeyCode::Key8 as u32),
        ("NUMBER_9", KeyCode::Key9 as u32),
        ("A", KeyCode::A as u32),
        ("B", KeyCode::B as u32),
        ("C", KeyCode::C as u32),
        ("D", KeyCode::D as u32),
        ("E", KeyCode::E as u32),
        ("F", KeyCode::F as u32),
        ("G", KeyCode::G as u32),
        ("H", KeyCode::H as u32),
        ("I", KeyCode::I as u32),
        ("J", KeyCode::J as u32),
        ("K", KeyCode::K as u32),
        ("L", KeyCode::L as u32),
        ("M", KeyCode::M as u32),
        ("N", KeyCode::N as u32),
        ("O", KeyCode::O as u32),
        ("P", KeyCode::P as u32),
        ("Q", KeyCode::Q as u32),
        ("R", KeyCode::R as u32),
        ("S", KeyCode::S as u32),
        ("T", KeyCode::T as u32),
        ("U", KeyCode::U as u32),
        ("V", KeyCode::V as u32),
        ("W", KeyCode::W as u32),
        ("X", KeyCode::X as u32),
        ("Y", KeyCode::Y as u32),
        ("Z", KeyCode::Z as u32),
        ("SEMICOLON", KeyCode::Semicolon as u32),
        ("EQUAL", KeyCode::Equals as u32),
        ("COMMA", KeyCode::Comma as u32),
        ("MINUS", KeyCode::Minus as u32),
        ("PERIOD", KeyCode::Period as u32),
        ("SLASH", KeyCode::Slash as u32),
        ("BACKQUOTE", KeyCode::Grave as u32),
        ("LEFTBRACKET", KeyCode::LBracket as u32),
        ("BACKSLASH", KeyCode::Backslash as u32),
        ("RIGHTBRACKET", KeyCode::RBracket as u32),
        ("QUOTE", KeyCode::Apostrophe as u32),
        ("ALTERNATE", KeyCode::Alt as u32),
        ("BACKSPACE", KeyCode::Backspace as u32),
        ("CAPS_LOCK", KeyCode::CapsLock as u32),
        ("COMMAND", KeyCode::Command as u32),
        ("CONTROL", KeyCode::Control as u32),
        ("DELETE", KeyCode::Delete as u32),
        ("DOWN", KeyCode::Down as u32),
        ("END", KeyCode::End as u32),
        ("ENTER", KeyCode::Return as u32),
        ("ESCAPE", KeyCode::Escape as u32),
        ("F1", KeyCode::F1 as u32),
        ("F2", KeyCode::F2 as u32),
        ("F3", KeyCode::F3 as u32),
        ("F4", KeyCode::F4 as u32),
        ("F5", KeyCode::F5 as u32),
        ("F6", KeyCode::F6 as u32),
        ("F7", KeyCode::F7 as u32),
        ("F8", KeyCode::F8 as u32),
        ("F9", KeyCode::F9 as u32),
        ("F10", KeyCode::F10 as u32),
        ("F11", KeyCode::F11 as u32),
        ("F12", KeyCode::F12 as u32),
        ("F13", KeyCode::F13 as u32),
        ("F14", KeyCode::F14 as u32),
        ("F15", KeyCode::F15 as u32),
        ("HOME", KeyCode::Home as u32),
        ("INSERT", KeyCode::Insert as u32),
        ("LEFT", KeyCode::Left as u32),
        ("NUMPAD", KeyCode::Numpad as u32), // Numpad meta key, not Num Lock
        ("NUMPAD_0", KeyCode::Numpad0 as u32),
        ("NUMPAD_1", KeyCode::Numpad1 as u32),
        ("NUMPAD_2", KeyCode::Numpad2 as u32),
        ("NUMPAD_3", KeyCode::Numpad3 as u32),
        ("NUMPAD_4", KeyCode::Numpad4 as u32),
        ("NUMPAD_5", KeyCode::Numpad5 as u32),
        ("NUMPAD_6", KeyCode::Numpad6 as u32),
        ("NUMPAD_7", KeyCode::Numpad7 as u32),
        ("NUMPAD_8", KeyCode::Numpad8 as u32),
        ("NUMPAD_9", KeyCode::Numpad9 as u32),
        ("NUMPAD_ADD", KeyCode::Plus as u32),
        ("NUMPAD_DECIMAL", KeyCode::NumpadPeriod as u32),
        ("NUMPAD_DIVIDE", KeyCode::NumpadSlash as u32),
        ("NUMPAD_ENTER", KeyCode::NumpadEnter as u32),
        ("NUMPAD_MULTIPLY", KeyCode::Multiply as u32),
        ("NUMPAD_SUBTRACT", KeyCode::NumpadMinus as u32),
        ("PAGE_DOWN", KeyCode::PgDown as u32),
        ("PAGE_UP", KeyCode::PgUp as u32),
        ("RIGHT", KeyCode::Right as u32),
        ("SHIFT", KeyCode::Shift as u32),
        ("SPACE", KeyCode::Space as u32),
        ("TAB", KeyCode::Tab as u32),
        ("UP", KeyCode::Up as u32),
        // Meta keys
        ("RED", 16777216),
        ("GREEN", 16777217),
        ("YELLOW", 16777218),
        ("BLUE", 16777219),
        ("CHANNEL_UP", 16777220),
        ("CHANNEL_DOWN", 16777221),
        ("RECORD", 16777222),
        ("PLAY", 16777223),
        ("PAUSE", 16777224),
        ("STOP", 16777225),
        ("FAST_FORWARD", 16777226),
        ("REWIND", 16777227),
        ("SKIP_FORWARD", 16777228),
        ("SKIP_BACKWARD", 16777229),
        ("NEXT", 16777230),
        ("PREVIOUS", 16777231),
        ("LIVE", 16777232),
        ("LAST", 16777233),
        ("MENU", 16777234),
        ("INFO", 16777235),
        ("GUIDE", 16777236),
        ("EXIT", 16777237),
        ("BACK", 16777238),
        ("AUDIO", 16777239),
        ("SUBTITLE", 16777240),
        ("DVR", 16777241),
        ("VOD", 16777242),
        ("INPUT", 16777243),
        ("SETUP", 16777244),
        ("HELP", 16777245),
        ("MASTER_SHELL", 16777246),
        ("SEARCH", 16777247),
        ("PLAY_PAUSE", 16777248),
    ];
    write.define_public_constant_uint_class_traits(UINT_CONSTANTS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("capsLock", Some(caps_lock), None),
        ("hasVirtualKeyboard", Some(has_virtual_keyboard), None),
        ("numLock", Some(num_lock), None),
        ("physicalKeyboardType", Some(physical_keyboard_type), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[("isAccessible", is_accessible)];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    class
}

fn caps_lock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Keyboard.capsLock: not yet implemented");
    Ok(false.into())
}

fn has_virtual_keyboard<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Keyboard.hasVirtualKeyboard: not yet implemented");
    Ok(false.into())
}

fn num_lock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Keyboard.numLock: not yet implemented");
    Ok(false.into())
}

fn physical_keyboard_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Keyboard.physicalKeyboardType: not yet implemented");
    Ok(AvmString::new_utf8(activation.context.gc_context, "alphanumeric").into())
}

fn is_accessible<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Keyboard.isAccessible: not yet implemented");
    Ok(true.into())
}
