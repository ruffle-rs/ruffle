use ruffle_core::events::{
    KeyDescriptor, KeyLocation, LogicalKey, NamedKey, PhysicalKey, TextControlCode,
};
use web_sys::KeyboardEvent;

pub fn web_input_to_ruffle_key_descriptor(event: &KeyboardEvent) -> KeyDescriptor {
    let physical_key = map_physical_key(&event.code());
    let logical_key = map_logical_key(&event.key());
    let key_location = map_key_location(event.location());
    KeyDescriptor {
        physical_key,
        logical_key,
        key_location,
    }
}

/// Convert a web `KeyboardEvent.code` value into a Ruffle `PhysicalKey`.
fn map_physical_key(key_code: &str) -> PhysicalKey {
    match key_code {
        "Backquote" => PhysicalKey::Backquote,
        "Digit0" => PhysicalKey::Digit0,
        "Digit1" => PhysicalKey::Digit1,
        "Digit2" => PhysicalKey::Digit2,
        "Digit4" => PhysicalKey::Digit4,
        "Digit3" => PhysicalKey::Digit3,
        "Digit5" => PhysicalKey::Digit5,
        "Digit6" => PhysicalKey::Digit6,
        "Digit7" => PhysicalKey::Digit7,
        "Digit8" => PhysicalKey::Digit8,
        "Digit9" => PhysicalKey::Digit9,
        "Minus" => PhysicalKey::Minus,
        "Equal" => PhysicalKey::Equal,
        "IntlYen" => PhysicalKey::IntlYen,
        "KeyQ" => PhysicalKey::KeyQ,
        "KeyW" => PhysicalKey::KeyW,
        "KeyE" => PhysicalKey::KeyE,
        "KeyR" => PhysicalKey::KeyR,
        "KeyT" => PhysicalKey::KeyT,
        "KeyY" => PhysicalKey::KeyY,
        "KeyU" => PhysicalKey::KeyU,
        "KeyI" => PhysicalKey::KeyI,
        "KeyO" => PhysicalKey::KeyO,
        "KeyP" => PhysicalKey::KeyP,
        "KeyA" => PhysicalKey::KeyA,
        "KeyS" => PhysicalKey::KeyS,
        "KeyD" => PhysicalKey::KeyD,
        "KeyF" => PhysicalKey::KeyF,
        "KeyG" => PhysicalKey::KeyG,
        "KeyH" => PhysicalKey::KeyH,
        "KeyJ" => PhysicalKey::KeyJ,
        "KeyK" => PhysicalKey::KeyK,
        "KeyL" => PhysicalKey::KeyL,
        "KeyZ" => PhysicalKey::KeyZ,
        "KeyX" => PhysicalKey::KeyX,
        "KeyC" => PhysicalKey::KeyC,
        "KeyV" => PhysicalKey::KeyV,
        "KeyB" => PhysicalKey::KeyB,
        "KeyN" => PhysicalKey::KeyN,
        "KeyM" => PhysicalKey::KeyM,
        "BracketLeft" => PhysicalKey::BracketLeft,
        "BracketRight" => PhysicalKey::BracketRight,
        "Backslash" => PhysicalKey::Backslash,
        "Semicolon" => PhysicalKey::Semicolon,
        "Quote" => PhysicalKey::Quote,
        "IntlBackslash" => PhysicalKey::IntlBackslash,
        "Comma" => PhysicalKey::Comma,
        "Period" => PhysicalKey::Period,
        "Slash" => PhysicalKey::Slash,
        "IntlRo" => PhysicalKey::IntlRo,
        "Backspace" => PhysicalKey::Backspace,
        "Tab" => PhysicalKey::Tab,
        "CapsLock" => PhysicalKey::CapsLock,
        "Enter" => PhysicalKey::Enter,
        "ShiftLeft" => PhysicalKey::ShiftLeft,
        "ShiftRight" => PhysicalKey::ShiftRight,
        "ControlLeft" => PhysicalKey::ControlLeft,
        "SuperLeft" => PhysicalKey::SuperLeft,
        "AltLeft" => PhysicalKey::AltLeft,
        "Space" => PhysicalKey::Space,
        "AltRight" => PhysicalKey::AltRight,
        "SuperRight" => PhysicalKey::SuperRight,
        "ContextMenu" => PhysicalKey::ContextMenu,
        "ControlRight" => PhysicalKey::ControlRight,
        "Insert" => PhysicalKey::Insert,
        "Delete" => PhysicalKey::Delete,
        "Home" => PhysicalKey::Home,
        "End" => PhysicalKey::End,
        "PageUp" => PhysicalKey::PageUp,
        "PageDown" => PhysicalKey::PageDown,
        "ArrowUp" => PhysicalKey::ArrowUp,
        "ArrowLeft" => PhysicalKey::ArrowLeft,
        "ArrowDown" => PhysicalKey::ArrowDown,
        "ArrowRight" => PhysicalKey::ArrowRight,
        "NumLock" => PhysicalKey::NumLock,
        "NumpadDivide" => PhysicalKey::NumpadDivide,
        "NumpadMultiply" => PhysicalKey::NumpadMultiply,
        "NumpadSubtract" => PhysicalKey::NumpadSubtract,
        "Numpad7" => PhysicalKey::Numpad7,
        "Numpad8" => PhysicalKey::Numpad8,
        "Numpad9" => PhysicalKey::Numpad9,
        "Numpad4" => PhysicalKey::Numpad4,
        "Numpad5" => PhysicalKey::Numpad5,
        "Numpad6" => PhysicalKey::Numpad6,
        "Numpad1" => PhysicalKey::Numpad1,
        "Numpad2" => PhysicalKey::Numpad2,
        "Numpad3" => PhysicalKey::Numpad3,
        "Numpad0" => PhysicalKey::Numpad0,
        "NumpadAdd" => PhysicalKey::NumpadAdd,
        "NumpadComma" => PhysicalKey::NumpadComma,
        "NumpadEnter" => PhysicalKey::NumpadEnter,
        "NumpadDecimal" => PhysicalKey::NumpadDecimal,
        "Escape" => PhysicalKey::Escape,
        "F1" => PhysicalKey::F1,
        "F2" => PhysicalKey::F2,
        "F3" => PhysicalKey::F3,
        "F4" => PhysicalKey::F4,
        "F5" => PhysicalKey::F5,
        "F6" => PhysicalKey::F6,
        "F7" => PhysicalKey::F7,
        "F8" => PhysicalKey::F8,
        "F9" => PhysicalKey::F9,
        "F10" => PhysicalKey::F10,
        "F11" => PhysicalKey::F11,
        "F12" => PhysicalKey::F12,
        "F13" => PhysicalKey::F13,
        "F14" => PhysicalKey::F14,
        "F15" => PhysicalKey::F15,
        "F16" => PhysicalKey::F16,
        "F17" => PhysicalKey::F17,
        "F18" => PhysicalKey::F18,
        "F19" => PhysicalKey::F19,
        "F20" => PhysicalKey::F20,
        "F21" => PhysicalKey::F21,
        "F22" => PhysicalKey::F22,
        "F23" => PhysicalKey::F23,
        "F24" => PhysicalKey::F24,
        "F25" => PhysicalKey::F25,
        "F26" => PhysicalKey::F26,
        "F27" => PhysicalKey::F27,
        "F28" => PhysicalKey::F28,
        "F29" => PhysicalKey::F29,
        "F30" => PhysicalKey::F30,
        "F31" => PhysicalKey::F31,
        "F32" => PhysicalKey::F32,
        "F33" => PhysicalKey::F33,
        "F34" => PhysicalKey::F34,
        "F35" => PhysicalKey::F35,
        "Fn" => PhysicalKey::Fn,
        "FnLock" => PhysicalKey::FnLock,
        "PrintScreen" => PhysicalKey::PrintScreen,
        "ScrollLock" => PhysicalKey::ScrollLock,
        "Pause" => PhysicalKey::Pause,
        _ => PhysicalKey::Unknown,
    }
}

/// Convert a web `KeyboardEvent.key` value into a Ruffle `LogicalKey`.
fn map_logical_key(key: &str) -> LogicalKey {
    // TODO: This is a very cheesy way to tell if a `KeyboardEvent.key` is a printable character.
    // Single character strings will be an actual printable char that we can use as text input.
    // All the other special values are multiple characters (e.g. "ArrowLeft").
    // It's probably better to explicitly match on all the variants.
    let mut chars = key.chars();
    let (c1, c2) = (chars.next(), chars.next());
    if c1.is_none() {
        LogicalKey::Unknown
    } else if let (Some(ch), None) = (c1, c2) {
        // Single character.
        LogicalKey::Character(ch)
    } else {
        // Check for special characters.
        match key {
            "Unidentified" => LogicalKey::Unknown,
            "Alt" => LogicalKey::Named(NamedKey::Alt),
            "AltGraph" => LogicalKey::Named(NamedKey::AltGraph),
            "CapsLock" => LogicalKey::Named(NamedKey::CapsLock),
            "Control" => LogicalKey::Named(NamedKey::Control),
            "Fn" => LogicalKey::Named(NamedKey::Fn),
            "FnLock" => LogicalKey::Named(NamedKey::FnLock),
            "Hyper" => LogicalKey::Unknown,
            "Meta" => LogicalKey::Unknown,
            "NumLock" => LogicalKey::Named(NamedKey::NumLock),
            "ScrollLock" => LogicalKey::Named(NamedKey::ScrollLock),
            "Shift" => LogicalKey::Named(NamedKey::Shift),
            "Super" => LogicalKey::Named(NamedKey::Super),
            "Symbol" => LogicalKey::Named(NamedKey::Symbol),
            "SymbolLock" => LogicalKey::Named(NamedKey::SymbolLock),
            "Enter" => LogicalKey::Named(NamedKey::Enter),
            "Tab" => LogicalKey::Named(NamedKey::Tab),
            "ArrowDown" => LogicalKey::Named(NamedKey::ArrowDown),
            "ArrowLeft" => LogicalKey::Named(NamedKey::ArrowLeft),
            "ArrowRight" => LogicalKey::Named(NamedKey::ArrowRight),
            "ArrowUp" => LogicalKey::Named(NamedKey::ArrowUp),
            "End" => LogicalKey::Named(NamedKey::End),
            "Home" => LogicalKey::Named(NamedKey::Home),
            "PageDown" => LogicalKey::Named(NamedKey::PageDown),
            "PageUp" => LogicalKey::Named(NamedKey::PageUp),
            "Backspace" => LogicalKey::Named(NamedKey::Backspace),
            "Clear" => LogicalKey::Named(NamedKey::Clear),
            "Copy" => LogicalKey::Named(NamedKey::Copy),
            "CrSel" => LogicalKey::Named(NamedKey::CrSel),
            "Cut" => LogicalKey::Named(NamedKey::Cut),
            "Delete" => LogicalKey::Named(NamedKey::Delete),
            "EraseEof" => LogicalKey::Named(NamedKey::EraseEof),
            "ExSel" => LogicalKey::Named(NamedKey::ExSel),
            "Insert" => LogicalKey::Named(NamedKey::Insert),
            "Paste" => LogicalKey::Named(NamedKey::Paste),
            "Redo" => LogicalKey::Named(NamedKey::Redo),
            "Undo" => LogicalKey::Named(NamedKey::Undo),
            "Accept" => LogicalKey::Unknown,
            "Again" => LogicalKey::Unknown,
            "Attn" => LogicalKey::Unknown,
            "Cancel" => LogicalKey::Unknown,
            "ContextMenu" => LogicalKey::Named(NamedKey::ContextMenu),
            "Escape" => LogicalKey::Named(NamedKey::Escape),
            "Execute" => LogicalKey::Unknown,
            "Find" => LogicalKey::Unknown,
            "Finish" => LogicalKey::Unknown,
            "Help" => LogicalKey::Unknown,
            "Pause" => LogicalKey::Named(NamedKey::Pause),
            "Play" => LogicalKey::Named(NamedKey::Play),
            "Props" => LogicalKey::Unknown,
            "Select" => LogicalKey::Named(NamedKey::Select),
            "ZoomIn" => LogicalKey::Named(NamedKey::ZoomIn),
            "ZoomOut" => LogicalKey::Named(NamedKey::ZoomOut),
            "F1" => LogicalKey::Named(NamedKey::F1),
            "F2" => LogicalKey::Named(NamedKey::F2),
            "F3" => LogicalKey::Named(NamedKey::F3),
            "F4" => LogicalKey::Named(NamedKey::F4),
            "F5" => LogicalKey::Named(NamedKey::F5),
            "F6" => LogicalKey::Named(NamedKey::F6),
            "F7" => LogicalKey::Named(NamedKey::F7),
            "F8" => LogicalKey::Named(NamedKey::F8),
            "F9" => LogicalKey::Named(NamedKey::F9),
            "F10" => LogicalKey::Named(NamedKey::F10),
            "F11" => LogicalKey::Named(NamedKey::F11),
            "F12" => LogicalKey::Named(NamedKey::F12),
            "F13" => LogicalKey::Named(NamedKey::F13),
            "F14" => LogicalKey::Named(NamedKey::F14),
            "F15" => LogicalKey::Named(NamedKey::F15),
            "F16" => LogicalKey::Named(NamedKey::F16),
            "F17" => LogicalKey::Named(NamedKey::F17),
            "F18" => LogicalKey::Named(NamedKey::F18),
            "F19" => LogicalKey::Named(NamedKey::F19),
            "F20" => LogicalKey::Named(NamedKey::F20),
            "Decimal" => LogicalKey::Character('.'),
            "Multiply" => LogicalKey::Character('*'),
            "Add" => LogicalKey::Character('+'),
            "Divide" => LogicalKey::Character('/'),
            "Subtract" => LogicalKey::Character('-'),
            _ => LogicalKey::Unknown,
        }
    }
}

/// Convert a web `KeyboardEvent.location` value into a Ruffle `KeyLocation`.
fn map_key_location(location: u32) -> KeyLocation {
    match location {
        1 => KeyLocation::Left,
        2 => KeyLocation::Right,
        3 => KeyLocation::Numpad,
        _ => KeyLocation::Standard,
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
