use ruffle_core::events::{KeyCode, TextControlCode};

/// Convert a web `KeyboardEvent.code` value into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
pub fn web_to_ruffle_key_code(key_code: &str) -> KeyCode {
    match key_code {
        "Backspace" => KeyCode::Backspace,
        "Tab" => KeyCode::Tab,
        "Enter" => KeyCode::Return,
        "ShiftLeft" | "ShiftRight" => KeyCode::Shift,
        "ControlLeft" | "ControlRight" => KeyCode::Control,
        "AltLeft" | "AltRight" => KeyCode::Alt,
        "CapsLock" => KeyCode::CapsLock,
        "Escape" => KeyCode::Escape,
        "Space" => KeyCode::Space,
        "Digit0" => KeyCode::Key0,
        "Digit1" => KeyCode::Key1,
        "Digit2" => KeyCode::Key2,
        "Digit3" => KeyCode::Key3,
        "Digit4" => KeyCode::Key4,
        "Digit5" => KeyCode::Key5,
        "Digit6" => KeyCode::Key6,
        "Digit7" => KeyCode::Key7,
        "Digit8" => KeyCode::Key8,
        "Digit9" => KeyCode::Key9,
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
        "Semicolon" => KeyCode::Semicolon,
        "Equal" => KeyCode::Equals,
        "Comma" => KeyCode::Comma,
        "Minus" => KeyCode::Minus,
        "Period" => KeyCode::Period,
        "Slash" => KeyCode::Slash,
        "Backquote" => KeyCode::Grave,
        "BracketLeft" => KeyCode::LBracket,
        "Backslash" => KeyCode::Backslash,
        "BracketRight" => KeyCode::RBracket,
        "Quote" => KeyCode::Apostrophe,
        "Numpad0" => KeyCode::Numpad0,
        "Numpad1" => KeyCode::Numpad1,
        "Numpad2" => KeyCode::Numpad2,
        "Numpad3" => KeyCode::Numpad3,
        "Numpad4" => KeyCode::Numpad4,
        "Numpad5" => KeyCode::Numpad5,
        "Numpad6" => KeyCode::Numpad6,
        "Numpad7" => KeyCode::Numpad7,
        "Numpad8" => KeyCode::Numpad8,
        "Numpad9" => KeyCode::Numpad9,
        "NumpadMultiply" => KeyCode::Multiply,
        "NumpadAdd" => KeyCode::Plus,
        "NumpadSubtract" => KeyCode::NumpadMinus,
        "NumpadDecimal" => KeyCode::NumpadPeriod,
        "NumpadDivide" => KeyCode::NumpadSlash,
        "NumpadEnter" => KeyCode::Return,
        "PageUp" => KeyCode::PgUp,
        "PageDown" => KeyCode::PgDown,
        "End" => KeyCode::End,
        "Home" => KeyCode::Home,
        "ArrowLeft" => KeyCode::Left,
        "ArrowUp" => KeyCode::Up,
        "ArrowRight" => KeyCode::Right,
        "ArrowDown" => KeyCode::Down,
        "Insert" => KeyCode::Insert,
        "Delete" => KeyCode::Delete,
        "Pause" => KeyCode::Pause,
        "NumLock" => KeyCode::NumLock,
        "ScrollLock" => KeyCode::ScrollLock,
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
        _ => KeyCode::Unknown,
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
