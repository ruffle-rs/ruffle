use ruffle_core::events::{KeyCode, TextControlCode};

/// Convert a web `KeyboardEvent.code` value into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
pub fn web_to_ruffle_key_code(key_code: &str) -> KeyCode {
    match key_code {
        "Backspace" => KeyCode::BACKSPACE,
        "Tab" => KeyCode::TAB,
        "Enter" => KeyCode::RETURN,
        "ShiftLeft" | "ShiftRight" => KeyCode::SHIFT,
        "ControlLeft" | "ControlRight" => KeyCode::CONTROL,
        "AltLeft" | "AltRight" => KeyCode::ALT,
        "CapsLock" => KeyCode::CAPS_LOCK,
        "Escape" => KeyCode::ESCAPE,
        "Space" => KeyCode::SPACE,
        "Digit0" => KeyCode::KEY0,
        "Digit1" => KeyCode::KEY1,
        "Digit2" => KeyCode::KEY2,
        "Digit3" => KeyCode::KEY3,
        "Digit4" => KeyCode::KEY4,
        "Digit5" => KeyCode::KEY5,
        "Digit6" => KeyCode::KEY6,
        "Digit7" => KeyCode::KEY7,
        "Digit8" => KeyCode::KEY8,
        "Digit9" => KeyCode::KEY9,
        "KeyA" => KeyCode::A,
        "KeyB" => KeyCode::B,
        "KeyC" => KeyCode::C,
        "KeyD" => KeyCode::D,
        "KeyE" => KeyCode::E,
        "KeyF" => KeyCode::F,
        "KeyG" => KeyCode::G,
        "KeyH" => KeyCode::H,
        "KeyI" => KeyCode::I,
        "KeyJ" => KeyCode::J,
        "KeyK" => KeyCode::K,
        "KeyL" => KeyCode::L,
        "KeyM" => KeyCode::M,
        "KeyN" => KeyCode::N,
        "KeyO" => KeyCode::O,
        "KeyP" => KeyCode::P,
        "KeyQ" => KeyCode::Q,
        "KeyR" => KeyCode::R,
        "KeyS" => KeyCode::S,
        "KeyT" => KeyCode::T,
        "KeyU" => KeyCode::U,
        "KeyV" => KeyCode::V,
        "KeyW" => KeyCode::W,
        "KeyX" => KeyCode::X,
        "KeyY" => KeyCode::Y,
        "KeyZ" => KeyCode::Z,
        "Semicolon" => KeyCode::SEMICOLON,
        "Equal" => KeyCode::EQUALS,
        "Comma" => KeyCode::COMMA,
        "Minus" => KeyCode::MINUS,
        "Period" => KeyCode::PERIOD,
        "Slash" => KeyCode::SLASH,
        "Backquote" => KeyCode::GRAVE,
        "BracketLeft" => KeyCode::LBRACKET,
        "Backslash" => KeyCode::BACKSLASH,
        "BracketRight" => KeyCode::RBRACKET,
        "Quote" => KeyCode::APOSTROPHE,
        "Numpad0" => KeyCode::NUMPAD0,
        "Numpad1" => KeyCode::NUMPAD1,
        "Numpad2" => KeyCode::NUMPAD2,
        "Numpad3" => KeyCode::NUMPAD3,
        "Numpad4" => KeyCode::NUMPAD4,
        "Numpad5" => KeyCode::NUMPAD5,
        "Numpad6" => KeyCode::NUMPAD6,
        "Numpad7" => KeyCode::NUMPAD7,
        "Numpad8" => KeyCode::NUMPAD8,
        "Numpad9" => KeyCode::NUMPAD9,
        "NumpadMultiply" => KeyCode::MULTIPLY,
        "NumpadAdd" => KeyCode::PLUS,
        "NumpadSubtract" => KeyCode::NUMPAD_MINUS,
        "NumpadDecimal" => KeyCode::NUMPAD_PERIOD,
        "NumpadDivide" => KeyCode::NUMPAD_SLASH,
        "NumpadEnter" => KeyCode::RETURN,
        "PageUp" => KeyCode::PG_UP,
        "PageDown" => KeyCode::PG_DOWN,
        "End" => KeyCode::END,
        "Home" => KeyCode::HOME,
        "ArrowLeft" => KeyCode::LEFT,
        "ArrowUp" => KeyCode::UP,
        "ArrowRight" => KeyCode::RIGHT,
        "ArrowDown" => KeyCode::DOWN,
        "Insert" => KeyCode::INSERT,
        "Delete" => KeyCode::DELETE,
        "Pause" => KeyCode::PAUSE,
        "NumLock" => KeyCode::NUM_LOCK,
        "ScrollLock" => KeyCode::SCROLL_LOCK,
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        "F13" => KeyCode::F13,
        "F14" => KeyCode::F14,
        "F15" => KeyCode::F15,
        "F16" => KeyCode::F16,
        "F17" => KeyCode::F17,
        "F18" => KeyCode::F18,
        "F19" => KeyCode::F19,
        "F20" => KeyCode::F20,
        "F21" => KeyCode::F21,
        "F22" => KeyCode::F22,
        "F23" => KeyCode::F23,
        "F24" => KeyCode::F24,
        _ => KeyCode::UNKNOWN,
    }
}

/// Convert a web `KeyboardEvent.key` value into a character codepoint.
/// Return `None` if they input was not a printable character.
pub fn web_key_to_codepoint(key: &str) -> Option<char> {
    // TODO: This is a very cheesy way to tell if a `KeyboardEvent.key` is a printable character.
    // Single character strings will be an actual printable char that we can use as text input.
    // All the other special values are multiple characters (e.g. "ArrowLeft").
    // It's probably better to explicitly match on all the variants.
    let mut chars = key.chars();
    let (c1, c2) = (chars.next(), chars.next());
    if c2.is_none() {
        // Single character.
        c1
    } else {
        // Check for special characters.
        match key {
            "Backspace" => Some(8 as char),
            "Delete" => Some(127 as char),
            "Enter" => Some(13 as char),
            _ => None,
        }
    }
}

/// Convert a web `KeyboardEvent.key` value to a Ruffle `TextControlCode`,
/// given the states of the modifier keys. Return `None` if there is no match.
pub fn web_to_ruffle_text_control(
    key: &str,
    ctrl_key: bool,
    shift_key: bool,
) -> Option<TextControlCode> {
    let mut chars = key.chars();
    let (c1, c2) = (chars.next(), chars.next());
    if c2.is_none() {
        // Single character.
        if ctrl_key {
            match c1 {
                Some('a') => Some(TextControlCode::SelectAll),
                Some('c') => Some(TextControlCode::Copy),
                Some('v') => Some(TextControlCode::Paste),
                Some('x') => Some(TextControlCode::Cut),
                _ => None,
            }
        } else {
            None
        }
    } else {
        match key {
            "Enter" => Some(TextControlCode::Enter),
            "Delete" if ctrl_key => Some(TextControlCode::DeleteWord),
            "Delete" => Some(TextControlCode::Delete),
            "Backspace" if ctrl_key => Some(TextControlCode::BackspaceWord),
            "Backspace" => Some(TextControlCode::Backspace),
            "ArrowLeft" if ctrl_key && shift_key => Some(TextControlCode::SelectLeftWord),
            "ArrowLeft" if ctrl_key => Some(TextControlCode::MoveLeftWord),
            "ArrowLeft" if shift_key => Some(TextControlCode::SelectLeft),
            "ArrowLeft" => Some(TextControlCode::MoveLeft),
            "ArrowRight" if ctrl_key && shift_key => Some(TextControlCode::SelectRightWord),
            "ArrowRight" if ctrl_key => Some(TextControlCode::MoveRightWord),
            "ArrowRight" if shift_key => Some(TextControlCode::SelectRight),
            "ArrowRight" => Some(TextControlCode::MoveRight),
            "Home" if ctrl_key && shift_key => Some(TextControlCode::SelectLeftDocument),
            "Home" if ctrl_key => Some(TextControlCode::MoveLeftDocument),
            "Home" if shift_key => Some(TextControlCode::SelectLeftLine),
            "Home" => Some(TextControlCode::MoveLeftLine),
            "End" if ctrl_key && shift_key => Some(TextControlCode::SelectRightDocument),
            "End" if ctrl_key => Some(TextControlCode::MoveRightDocument),
            "End" if shift_key => Some(TextControlCode::SelectRightLine),
            "End" => Some(TextControlCode::MoveRightLine),
            _ => None,
        }
    }
}
