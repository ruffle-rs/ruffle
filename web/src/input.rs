use crate::utils::JsResult;
use ruffle_core::backend::input::InputBackend;
use ruffle_core::events::KeyCode;
use std::collections::HashSet;
use web_sys::HtmlCanvasElement;

/// An implementation of `InputBackend` utilizing `web_sys` bindings to input
/// APIs
pub struct WebInputBackend {
    keys_down: HashSet<String>,
    canvas: HtmlCanvasElement,
    cursor_visible: bool,
    last_key: KeyCode,
}

impl WebInputBackend {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        Self {
            keys_down: HashSet::new(),
            canvas: canvas.clone(),
            cursor_visible: true,
            last_key: KeyCode::Unknown,
        }
    }

    /// Register a key press for a given code string.
    pub fn keydown(&mut self, code: String) {
        self.last_key = web_to_ruffle_key_code(&code).unwrap_or_else(|| KeyCode::Unknown);
        self.keys_down.insert(code);
    }

    /// Register a key release for a given code string.
    pub fn keyup(&mut self, code: String) {
        self.keys_down.remove(&code);
    }
}

impl InputBackend for WebInputBackend {
    fn is_key_down(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::Unknown => false,
            KeyCode::Backspace => self.keys_down.contains("Backspace"),
            KeyCode::Return => self.keys_down.contains("Return"),
            KeyCode::Shift => {
                self.keys_down.contains("ShiftLeft") || self.keys_down.contains("ShiftRight")
            }
            KeyCode::Control => {
                self.keys_down.contains("ControlLeft") || self.keys_down.contains("ControlRight")
            }
            KeyCode::Alt => {
                self.keys_down.contains("AltLeft") || self.keys_down.contains("AltRight")
            }
            KeyCode::CapsLock => self.keys_down.contains("CapsLock"),
            KeyCode::Escape => self.keys_down.contains("Escape"),
            KeyCode::Space => self.keys_down.contains("Space"),
            KeyCode::Key0 => self.keys_down.contains("Digit0"),
            KeyCode::Key1 => self.keys_down.contains("Digit1"),
            KeyCode::Key2 => self.keys_down.contains("Digit2"),
            KeyCode::Key3 => self.keys_down.contains("Digit3"),
            KeyCode::Key4 => self.keys_down.contains("Digit4"),
            KeyCode::Key5 => self.keys_down.contains("Digit5"),
            KeyCode::Key6 => self.keys_down.contains("Digit6"),
            KeyCode::Key7 => self.keys_down.contains("Digit7"),
            KeyCode::Key8 => self.keys_down.contains("Digit8"),
            KeyCode::Key9 => self.keys_down.contains("Digit9"),
            KeyCode::A => self.keys_down.contains("KeyA"),
            KeyCode::B => self.keys_down.contains("KeyB"),
            KeyCode::C => self.keys_down.contains("KeyC"),
            KeyCode::D => self.keys_down.contains("KeyD"),
            KeyCode::E => self.keys_down.contains("KeyE"),
            KeyCode::F => self.keys_down.contains("KeyF"),
            KeyCode::G => self.keys_down.contains("KeyG"),
            KeyCode::H => self.keys_down.contains("KeyH"),
            KeyCode::I => self.keys_down.contains("KeyI"),
            KeyCode::J => self.keys_down.contains("KeyJ"),
            KeyCode::K => self.keys_down.contains("KeyK"),
            KeyCode::L => self.keys_down.contains("KeyL"),
            KeyCode::M => self.keys_down.contains("KeyM"),
            KeyCode::N => self.keys_down.contains("KeyN"),
            KeyCode::O => self.keys_down.contains("KeyO"),
            KeyCode::P => self.keys_down.contains("KeyP"),
            KeyCode::Q => self.keys_down.contains("KeyQ"),
            KeyCode::R => self.keys_down.contains("KeyR"),
            KeyCode::S => self.keys_down.contains("KeyS"),
            KeyCode::T => self.keys_down.contains("KeyT"),
            KeyCode::U => self.keys_down.contains("KeyU"),
            KeyCode::V => self.keys_down.contains("KeyV"),
            KeyCode::W => self.keys_down.contains("KeyW"),
            KeyCode::X => self.keys_down.contains("KeyX"),
            KeyCode::Y => self.keys_down.contains("KeyY"),
            KeyCode::Z => self.keys_down.contains("KeyZ"),
            KeyCode::Semicolon => self.keys_down.contains("Semicolon"),
            KeyCode::Equals => self.keys_down.contains("Equal"),
            KeyCode::Comma => self.keys_down.contains("Comma"),
            KeyCode::Minus => self.keys_down.contains("Minus"),
            KeyCode::Period => self.keys_down.contains("Period"),
            KeyCode::Slash => self.keys_down.contains("Slash"),
            KeyCode::Grave => self.keys_down.contains("Backquote"),
            KeyCode::LBracket => self.keys_down.contains("BracketLeft"),
            KeyCode::Backslash => self.keys_down.contains("Backslash"),
            KeyCode::RBracket => self.keys_down.contains("BracketRight"),
            KeyCode::Apostrophe => self.keys_down.contains("Quote"),
            KeyCode::Numpad0 => self.keys_down.contains("Numpad0"),
            KeyCode::Numpad1 => self.keys_down.contains("Numpad1"),
            KeyCode::Numpad2 => self.keys_down.contains("Numpad2"),
            KeyCode::Numpad3 => self.keys_down.contains("Numpad3"),
            KeyCode::Numpad4 => self.keys_down.contains("Numpad4"),
            KeyCode::Numpad5 => self.keys_down.contains("Numpad5"),
            KeyCode::Numpad6 => self.keys_down.contains("Numpad6"),
            KeyCode::Numpad7 => self.keys_down.contains("Numpad7"),
            KeyCode::Numpad8 => self.keys_down.contains("Numpad8"),
            KeyCode::Numpad9 => self.keys_down.contains("Numpad9"),
            KeyCode::Multiply => self.keys_down.contains("NumpadMultiply"),
            KeyCode::Plus => self.keys_down.contains("NumpadAdd"),
            KeyCode::NumpadMinus => self.keys_down.contains("NumpadSubtract"),
            KeyCode::NumpadPeriod => self.keys_down.contains("NumpadDecimal"),
            KeyCode::NumpadSlash => self.keys_down.contains("NumpadDivide"),
            KeyCode::PgUp => self.keys_down.contains("PageUp"),
            KeyCode::PgDown => self.keys_down.contains("PageDown"),
            KeyCode::End => self.keys_down.contains("End"),
            KeyCode::Home => self.keys_down.contains("Home"),
            KeyCode::Left => self.keys_down.contains("ArrowLeft"),
            KeyCode::Up => self.keys_down.contains("ArrowUp"),
            KeyCode::Right => self.keys_down.contains("ArrowRight"),
            KeyCode::Down => self.keys_down.contains("ArrowDown"),
            KeyCode::Insert => self.keys_down.contains("Insert"),
            KeyCode::Delete => self.keys_down.contains("Delete"),
            KeyCode::Pause => self.keys_down.contains("Pause"),
            KeyCode::ScrollLock => self.keys_down.contains("ScrollLock"),
            KeyCode::F1 => self.keys_down.contains("F1"),
            KeyCode::F2 => self.keys_down.contains("F2"),
            KeyCode::F3 => self.keys_down.contains("F3"),
            KeyCode::F4 => self.keys_down.contains("F4"),
            KeyCode::F5 => self.keys_down.contains("F5"),
            KeyCode::F6 => self.keys_down.contains("F6"),
            KeyCode::F7 => self.keys_down.contains("F7"),
            KeyCode::F8 => self.keys_down.contains("F8"),
            KeyCode::F9 => self.keys_down.contains("F9"),
            KeyCode::F10 => self.keys_down.contains("F10"),
            KeyCode::F11 => self.keys_down.contains("F11"),
            KeyCode::F12 => self.keys_down.contains("F12"),
        }
    }

    fn get_last_key_code(&self) -> KeyCode {
        self.last_key
    }

    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn hide_mouse(&mut self) {
        self.canvas
            .style()
            .set_property("cursor", "none")
            .warn_on_error();
        self.cursor_visible = false;
    }

    fn show_mouse(&mut self) {
        self.canvas
            .style()
            .set_property("cursor", "auto")
            .warn_on_error();
        self.cursor_visible = true;
    }
}

/// Converts a Web `KeyboardEvent.code` value into a Ruffle `KeyCode`.
/// Returns `None` if there is no matching Flash key key.
pub fn web_to_ruffle_key_code(key_code: &str) -> Option<KeyCode> {
    let out = match key_code {
        "Backspace" => KeyCode::Backspace,
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
        _ => return None,
    };
    Some(out)
}

/// Converts a Web `KeyboardEvent.key` value into a character codepoint.
/// Returns `None` if they input was not a printable character.
pub fn web_key_to_codepoint(key: &str) -> Option<char> {
    // TODO: This is a very cheesy way to tell if a KeyboardEvent.key is a printable character.
    // Single character strings will be an actual printable char that we can use as text input.
    // All the other special values are multiple characters (e.g. "ArrowLeft").
    // It's probably better to explicitly match on all the variants.
    if key.len() == 1 {
        key.chars().next()
    } else {
        None
    }
}
