use ruffle_core::events::{
    ImeEvent, KeyDescriptor, KeyLocation, LogicalKey, NamedKey, PhysicalKey,
    TextControlCode as RuffleTextControlCode,
};
use ruffle_core::events::{MouseButton as RuffleMouseButton, MouseWheelDelta};
use ruffle_core::{Player, PlayerEvent};
use ruffle_input_format::{
    AutomatedEvent, AutomatedKey, MouseButton as InputMouseButton,
    TextControlCode as InputTextControlCode,
};

pub fn perform_automated_event(evt: &AutomatedEvent, player: &mut Player) {
    if let AutomatedEvent::SetClipboardText { text } = evt {
        player.ui_mut().set_clipboard_content(text.to_owned());
        return;
    }

    let handled = player.handle_event(match evt {
        AutomatedEvent::MouseDown {
            pos, btn, index, ..
        } => PlayerEvent::MouseDown {
            x: pos.0,
            y: pos.1,
            button: match btn {
                InputMouseButton::Left => RuffleMouseButton::Left,
                InputMouseButton::Middle => RuffleMouseButton::Middle,
                InputMouseButton::Right => RuffleMouseButton::Right,
            },
            // None here means that the core will compute index automatically,
            // however we do not want that in tests.
            index: Some(index.unwrap_or_default()),
        },
        AutomatedEvent::MouseMove { pos } => PlayerEvent::MouseMove { x: pos.0, y: pos.1 },
        AutomatedEvent::MouseUp { pos, btn } => PlayerEvent::MouseUp {
            x: pos.0,
            y: pos.1,
            button: match btn {
                InputMouseButton::Left => RuffleMouseButton::Left,
                InputMouseButton::Middle => RuffleMouseButton::Middle,
                InputMouseButton::Right => RuffleMouseButton::Right,
            },
        },
        AutomatedEvent::MouseWheel { lines, pixels } => PlayerEvent::MouseWheel {
            delta: match (lines, pixels) {
                (Some(lines), None) => MouseWheelDelta::Lines(*lines),
                (None, Some(pixels)) => MouseWheelDelta::Pixels(*pixels),
                _ => panic!("MouseWheel: expected only one of 'lines' or 'pixels'"),
            },
        },
        AutomatedEvent::KeyDown { key } => PlayerEvent::KeyDown {
            key: automated_key_to_descriptor(*key),
        },
        AutomatedEvent::KeyUp { key } => PlayerEvent::KeyUp {
            key: automated_key_to_descriptor(*key),
        },
        AutomatedEvent::TextInput { codepoint } => PlayerEvent::TextInput {
            codepoint: *codepoint,
        },
        AutomatedEvent::TextControl { code } => PlayerEvent::TextControl {
            code: match code {
                InputTextControlCode::MoveLeft => RuffleTextControlCode::MoveLeft,
                InputTextControlCode::MoveLeftWord => RuffleTextControlCode::MoveLeftWord,
                InputTextControlCode::MoveLeftLine => RuffleTextControlCode::MoveLeftLine,
                InputTextControlCode::MoveLeftDocument => RuffleTextControlCode::MoveLeftDocument,
                InputTextControlCode::MoveRight => RuffleTextControlCode::MoveRight,
                InputTextControlCode::MoveRightWord => RuffleTextControlCode::MoveRightWord,
                InputTextControlCode::MoveRightLine => RuffleTextControlCode::MoveRightLine,
                InputTextControlCode::MoveRightDocument => RuffleTextControlCode::MoveRightDocument,
                InputTextControlCode::SelectLeft => RuffleTextControlCode::SelectLeft,
                InputTextControlCode::SelectLeftWord => RuffleTextControlCode::SelectLeftWord,
                InputTextControlCode::SelectLeftLine => RuffleTextControlCode::SelectLeftLine,
                InputTextControlCode::SelectLeftDocument => {
                    RuffleTextControlCode::SelectLeftDocument
                }
                InputTextControlCode::SelectRight => RuffleTextControlCode::SelectRight,
                InputTextControlCode::SelectRightWord => RuffleTextControlCode::SelectRightWord,
                InputTextControlCode::SelectRightLine => RuffleTextControlCode::SelectRightLine,
                InputTextControlCode::SelectRightDocument => {
                    RuffleTextControlCode::SelectRightDocument
                }
                InputTextControlCode::SelectAll => RuffleTextControlCode::SelectAll,
                InputTextControlCode::Copy => RuffleTextControlCode::Copy,
                InputTextControlCode::Paste => RuffleTextControlCode::Paste,
                InputTextControlCode::Cut => RuffleTextControlCode::Cut,
                InputTextControlCode::Backspace => RuffleTextControlCode::Backspace,
                InputTextControlCode::Enter => RuffleTextControlCode::Enter,
                InputTextControlCode::Delete => RuffleTextControlCode::Delete,
            },
        },
        AutomatedEvent::FocusGained => PlayerEvent::FocusGained,
        AutomatedEvent::FocusLost => PlayerEvent::FocusLost,
        AutomatedEvent::ImePreedit { text, cursor } => {
            PlayerEvent::Ime(ImeEvent::Preedit(text.clone(), *cursor))
        }
        AutomatedEvent::ImeCommit { text } => PlayerEvent::Ime(ImeEvent::Commit(text.clone())),
        AutomatedEvent::Wait | AutomatedEvent::SetClipboardText { .. } => unreachable!(),
    });

    #[expect(clippy::single_match)]
    match evt {
        AutomatedEvent::MouseDown {
            assert_handled: Some(assert_handled),
            ..
        } => {
            if handled != assert_handled.value {
                panic!(
                    "Event handled status assertion failed: \n\
                            \x20   expected to be handled: {}\n\
                            \x20   was handled: {}\n\
                            \x20   message: {}",
                    assert_handled.value, handled, assert_handled.message
                );
            }
        }
        _ => {}
    }
}

pub fn automated_key_to_descriptor(automated_key: AutomatedKey) -> KeyDescriptor {
    let (logical_key, physical_key) = match automated_key {
        AutomatedKey::Char(ch) | AutomatedKey::Numpad(ch) => (
            LogicalKey::Character(ch),
            match ch {
                'a' | 'A' => PhysicalKey::KeyA,
                'b' | 'B' => PhysicalKey::KeyB,
                'c' | 'C' => PhysicalKey::KeyC,
                'd' | 'D' => PhysicalKey::KeyD,
                'e' | 'E' => PhysicalKey::KeyE,
                'f' | 'F' => PhysicalKey::KeyF,
                'g' | 'G' => PhysicalKey::KeyG,
                'h' | 'H' => PhysicalKey::KeyH,
                'i' | 'I' => PhysicalKey::KeyI,
                'j' | 'J' => PhysicalKey::KeyJ,
                'k' | 'K' => PhysicalKey::KeyK,
                'l' | 'L' => PhysicalKey::KeyL,
                'm' | 'M' => PhysicalKey::KeyM,
                'n' | 'N' => PhysicalKey::KeyN,
                'o' | 'O' => PhysicalKey::KeyO,
                'p' | 'P' => PhysicalKey::KeyP,
                'q' | 'Q' => PhysicalKey::KeyQ,
                'r' | 'R' => PhysicalKey::KeyR,
                's' | 'S' => PhysicalKey::KeyS,
                't' | 'T' => PhysicalKey::KeyT,
                'u' | 'U' => PhysicalKey::KeyU,
                'v' | 'V' => PhysicalKey::KeyV,
                'w' | 'W' => PhysicalKey::KeyW,
                'x' | 'X' => PhysicalKey::KeyX,
                'y' | 'Y' => PhysicalKey::KeyY,
                'z' | 'Z' => PhysicalKey::KeyZ,
                '0' => PhysicalKey::Digit0,
                '1' => PhysicalKey::Digit1,
                '2' => PhysicalKey::Digit2,
                '3' => PhysicalKey::Digit3,
                '4' => PhysicalKey::Digit4,
                '5' => PhysicalKey::Digit5,
                '6' => PhysicalKey::Digit6,
                '7' => PhysicalKey::Digit7,
                '8' => PhysicalKey::Digit8,
                '9' => PhysicalKey::Digit9,
                '!' => PhysicalKey::Digit1,
                '@' => PhysicalKey::Digit2,
                '#' => PhysicalKey::Digit3,
                '$' => PhysicalKey::Digit4,
                '%' => PhysicalKey::Digit5,
                '^' => PhysicalKey::Digit6,
                '&' => PhysicalKey::Digit7,
                '*' => PhysicalKey::Digit8,
                '(' => PhysicalKey::Digit9,
                ')' => PhysicalKey::Digit0,
                '-' | '_' => PhysicalKey::Minus,
                '=' | '+' => PhysicalKey::Equal,
                '[' | '{' => PhysicalKey::BracketLeft,
                ']' | '}' => PhysicalKey::BracketRight,
                '\\' | '|' => PhysicalKey::Backslash,
                ';' | ':' => PhysicalKey::Semicolon,
                '\'' | '"' => PhysicalKey::Quote,
                ',' | '<' => PhysicalKey::Comma,
                '.' | '>' => PhysicalKey::Period,
                '/' | '?' => PhysicalKey::Slash,
                '`' | '~' => PhysicalKey::Backquote,
                ' ' => PhysicalKey::Space,
                _ => todo!("Key {ch} is unmapped"),
            },
        ),
        AutomatedKey::ArrowDown => (
            LogicalKey::Named(NamedKey::ArrowDown),
            PhysicalKey::ArrowDown,
        ),
        AutomatedKey::ArrowLeft => (
            LogicalKey::Named(NamedKey::ArrowLeft),
            PhysicalKey::ArrowLeft,
        ),
        AutomatedKey::ArrowRight => (
            LogicalKey::Named(NamedKey::ArrowRight),
            PhysicalKey::ArrowRight,
        ),
        AutomatedKey::ArrowUp => (LogicalKey::Named(NamedKey::ArrowUp), PhysicalKey::ArrowUp),
        AutomatedKey::Backspace => (
            LogicalKey::Named(NamedKey::Backspace),
            PhysicalKey::Backspace,
        ),
        AutomatedKey::CapsLock => (LogicalKey::Named(NamedKey::CapsLock), PhysicalKey::CapsLock),
        AutomatedKey::Delete => (LogicalKey::Named(NamedKey::Delete), PhysicalKey::Delete),
        AutomatedKey::End => (LogicalKey::Named(NamedKey::End), PhysicalKey::End),
        AutomatedKey::Enter => (LogicalKey::Named(NamedKey::Enter), PhysicalKey::Enter),
        AutomatedKey::Escape => (LogicalKey::Named(NamedKey::Escape), PhysicalKey::Escape),
        AutomatedKey::F1 => (LogicalKey::Named(NamedKey::F1), PhysicalKey::F1),
        AutomatedKey::F2 => (LogicalKey::Named(NamedKey::F2), PhysicalKey::F2),
        AutomatedKey::F3 => (LogicalKey::Named(NamedKey::F3), PhysicalKey::F3),
        AutomatedKey::F4 => (LogicalKey::Named(NamedKey::F4), PhysicalKey::F4),
        AutomatedKey::F5 => (LogicalKey::Named(NamedKey::F5), PhysicalKey::F5),
        AutomatedKey::F6 => (LogicalKey::Named(NamedKey::F6), PhysicalKey::F6),
        AutomatedKey::F7 => (LogicalKey::Named(NamedKey::F7), PhysicalKey::F7),
        AutomatedKey::F8 => (LogicalKey::Named(NamedKey::F8), PhysicalKey::F8),
        AutomatedKey::F9 => (LogicalKey::Named(NamedKey::F9), PhysicalKey::F9),
        AutomatedKey::Home => (LogicalKey::Named(NamedKey::Home), PhysicalKey::Home),
        AutomatedKey::Insert => (LogicalKey::Named(NamedKey::Insert), PhysicalKey::Insert),
        AutomatedKey::LeftAlt => (LogicalKey::Named(NamedKey::Alt), PhysicalKey::AltLeft),
        AutomatedKey::LeftControl => (
            LogicalKey::Named(NamedKey::Control),
            PhysicalKey::ControlLeft,
        ),
        AutomatedKey::LeftShift => (LogicalKey::Named(NamedKey::Shift), PhysicalKey::ShiftLeft),
        AutomatedKey::NumLock => (LogicalKey::Named(NamedKey::NumLock), PhysicalKey::NumLock),
        AutomatedKey::NumpadDelete => (
            LogicalKey::Named(NamedKey::Delete),
            PhysicalKey::NumpadDecimal,
        ),
        AutomatedKey::NumpadDown => (LogicalKey::Named(NamedKey::ArrowDown), PhysicalKey::Numpad2),
        AutomatedKey::NumpadEnd => (LogicalKey::Named(NamedKey::End), PhysicalKey::Numpad1),
        AutomatedKey::NumpadHome => (LogicalKey::Named(NamedKey::Home), PhysicalKey::Numpad7),
        AutomatedKey::NumpadInsert => (LogicalKey::Named(NamedKey::Insert), PhysicalKey::Numpad0),
        AutomatedKey::NumpadLeft => (LogicalKey::Named(NamedKey::ArrowLeft), PhysicalKey::Numpad4),
        AutomatedKey::NumpadPageDown => {
            (LogicalKey::Named(NamedKey::PageDown), PhysicalKey::Numpad3)
        }
        AutomatedKey::NumpadPageUp => (LogicalKey::Named(NamedKey::PageUp), PhysicalKey::Numpad9),
        AutomatedKey::NumpadRight => (
            LogicalKey::Named(NamedKey::ArrowRight),
            PhysicalKey::Numpad6,
        ),
        AutomatedKey::NumpadUp => (LogicalKey::Named(NamedKey::ArrowUp), PhysicalKey::Numpad8),
        AutomatedKey::PageDown => (LogicalKey::Named(NamedKey::PageDown), PhysicalKey::PageDown),
        AutomatedKey::PageUp => (LogicalKey::Named(NamedKey::PageUp), PhysicalKey::PageUp),
        AutomatedKey::Pause => (LogicalKey::Named(NamedKey::Pause), PhysicalKey::Pause),
        AutomatedKey::RightControl => (
            LogicalKey::Named(NamedKey::Control),
            PhysicalKey::ControlRight,
        ),
        AutomatedKey::RightShift => (LogicalKey::Named(NamedKey::Shift), PhysicalKey::ShiftRight),
        AutomatedKey::ScrollLock => (
            LogicalKey::Named(NamedKey::ScrollLock),
            PhysicalKey::ScrollLock,
        ),
        AutomatedKey::Space => (LogicalKey::Character(' '), PhysicalKey::Space),
        AutomatedKey::Tab => (LogicalKey::Named(NamedKey::Tab), PhysicalKey::Tab),
        AutomatedKey::Unknown => (LogicalKey::Unknown, PhysicalKey::Unknown),
    };

    let key_location = match automated_key {
        AutomatedKey::Numpad(_) => KeyLocation::Numpad,
        AutomatedKey::LeftAlt => KeyLocation::Left,
        AutomatedKey::LeftControl => KeyLocation::Left,
        AutomatedKey::LeftShift => KeyLocation::Left,
        AutomatedKey::NumLock => KeyLocation::Numpad,
        AutomatedKey::NumpadDelete => KeyLocation::Numpad,
        AutomatedKey::NumpadDown => KeyLocation::Numpad,
        AutomatedKey::NumpadEnd => KeyLocation::Numpad,
        AutomatedKey::NumpadHome => KeyLocation::Numpad,
        AutomatedKey::NumpadInsert => KeyLocation::Numpad,
        AutomatedKey::NumpadLeft => KeyLocation::Numpad,
        AutomatedKey::NumpadPageDown => KeyLocation::Numpad,
        AutomatedKey::NumpadPageUp => KeyLocation::Numpad,
        AutomatedKey::NumpadRight => KeyLocation::Numpad,
        AutomatedKey::NumpadUp => KeyLocation::Numpad,
        AutomatedKey::RightControl => KeyLocation::Right,
        AutomatedKey::RightShift => KeyLocation::Right,
        _ => KeyLocation::Standard,
    };

    KeyDescriptor {
        physical_key,
        logical_key,
        key_location,
    }
}
