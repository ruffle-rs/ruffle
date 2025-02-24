use crate::events::{
    GamepadButton, KeyCode, KeyDescriptor, KeyLocation, LogicalKey, MouseButton, MouseWheelDelta,
    NamedKey, PhysicalKey, PlayerEvent, TextControlCode,
};
use chrono::{DateTime, TimeDelta, Utc};
use std::collections::{HashMap, HashSet};

pub enum KeyCodeMappingType {
    // TODO Make this configurable, it's not
    //   yet possible to use this mapping type.
    #[allow(dead_code)]
    Physical,
    Logical,
}

/// An event describing input in general.
///
/// It's usually a processed [`PlayerEvent`].
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyDown {
        key_code: KeyCode,
        key_char: Option<char>,
        key_location: KeyLocation,
    },
    KeyUp {
        key_code: KeyCode,
        key_char: Option<char>,
        key_location: KeyLocation,
    },
    MouseMove {
        x: f64,
        y: f64,
    },
    MouseUp {
        x: f64,
        y: f64,
        button: MouseButton,
    },
    MouseDown {
        x: f64,
        y: f64,
        button: MouseButton,
        index: usize,
    },
    MouseLeave,
    MouseWheel {
        delta: MouseWheelDelta,
    },
    TextInput {
        codepoint: char,
    },
    TextControl {
        code: TextControlCode,
    },
}

struct ClickEventData {
    x: f64,
    y: f64,
    time: DateTime<Utc>,
    index: usize,
}

impl ClickEventData {
    fn distance_squared_to(&self, x: f64, y: f64) -> f64 {
        let dx = x - self.x;
        let dy = y - self.y;
        dx * dx + dy * dy
    }
}

pub struct InputManager {
    keys_down: HashSet<KeyCode>,
    keys_toggled: HashSet<KeyCode>,
    last_key: KeyCode,
    last_char: Option<char>,
    last_click: Option<ClickEventData>,

    /// A map from gamepad buttons to key codes.
    gamepad_button_mapping: HashMap<GamepadButton, KeyCode>,

    key_code_mapping_type: KeyCodeMappingType,
}

impl InputManager {
    pub fn new(gamepad_button_mapping: HashMap<GamepadButton, KeyCode>) -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_toggled: HashSet::new(),
            last_key: KeyCode::UNKNOWN,
            last_char: None,
            last_click: None,
            gamepad_button_mapping,
            key_code_mapping_type: KeyCodeMappingType::Logical,
        }
    }

    fn add_key(&mut self, key_code: KeyCode) {
        self.last_key = key_code;
        if key_code != KeyCode::UNKNOWN {
            self.keys_down.insert(key_code);
        }
    }

    fn toggle_key(&mut self, key_code: KeyCode) {
        if key_code == KeyCode::UNKNOWN || self.keys_down.contains(&key_code) {
            return;
        }
        if self.keys_toggled.contains(&key_code) {
            self.keys_toggled.remove(&key_code);
        } else {
            self.keys_toggled.insert(key_code);
        }
    }

    fn remove_key(&mut self, key_code: KeyCode) {
        self.last_key = key_code;
        if key_code != KeyCode::UNKNOWN {
            self.keys_down.remove(&key_code);
        }
    }

    pub fn process_event(&mut self, event: PlayerEvent) -> Option<InputEvent> {
        let event = match event {
            // Optionally transform gamepad button events into key events.
            PlayerEvent::GamepadButtonDown { button } => {
                if let Some(key_code) = self.gamepad_button_mapping.get(&button) {
                    InputEvent::KeyDown {
                        key_code: *key_code,
                        key_char: None,
                        // TODO what location shoud we use here?
                        key_location: KeyLocation::Standard,
                    }
                } else {
                    // Just ignore this event.
                    return None;
                }
            }
            PlayerEvent::GamepadButtonUp { button } => {
                if let Some(key_code) = self.gamepad_button_mapping.get(&button) {
                    InputEvent::KeyUp {
                        key_code: *key_code,
                        key_char: None,
                        // TODO what location shoud we use here?
                        key_location: KeyLocation::Standard,
                    }
                } else {
                    // Just ignore this event.
                    return None;
                }
            }

            PlayerEvent::KeyDown { key } => {
                let key_code = self.map_to_key_code(key)?;
                let key_char = self.map_to_key_char(key);
                let key_location = self.map_to_key_location(key);
                InputEvent::KeyDown {
                    key_code,
                    key_char,
                    key_location,
                }
            }
            PlayerEvent::KeyUp { key } => {
                let key_code = self.map_to_key_code(key)?;
                let key_char = self.map_to_key_char(key);
                let key_location = self.map_to_key_location(key);
                InputEvent::KeyUp {
                    key_code,
                    key_char,
                    key_location,
                }
            }

            PlayerEvent::MouseMove { x, y } => InputEvent::MouseMove { x, y },
            PlayerEvent::MouseUp { x, y, button } => InputEvent::MouseUp { x, y, button },
            PlayerEvent::MouseDown {
                x,
                y,
                button,
                index,
            } => InputEvent::MouseDown {
                x,
                y,
                button,
                index: self.update_last_click(x, y, index),
            },
            PlayerEvent::MouseLeave => InputEvent::MouseLeave,
            PlayerEvent::MouseWheel { delta } => InputEvent::MouseWheel { delta },

            PlayerEvent::TextInput { codepoint } => InputEvent::TextInput { codepoint },
            PlayerEvent::TextControl { code } => InputEvent::TextControl { code },

            // The following are not input events.
            PlayerEvent::FocusGained | PlayerEvent::FocusLost => return None,
        };

        self.handle_event(&event);

        Some(event)
    }

    fn map_to_key_code(&self, descriptor: KeyDescriptor) -> Option<KeyCode> {
        match self.key_code_mapping_type {
            KeyCodeMappingType::Physical => map_to_key_code_physical(descriptor.physical_key),
            KeyCodeMappingType::Logical => {
                map_to_key_code_logical(descriptor.logical_key, descriptor.key_location)
            }
        }
    }

    fn map_to_key_char(&self, descriptor: KeyDescriptor) -> Option<char> {
        descriptor.logical_key.character()
    }

    fn map_to_key_location(&self, descriptor: KeyDescriptor) -> KeyLocation {
        match descriptor.logical_key {
            // NumLock in FP reports Standard location, not Numpad
            LogicalKey::Named(NamedKey::NumLock) => KeyLocation::Standard,
            _ => descriptor.key_location,
        }
    }

    fn handle_event(&mut self, event: &InputEvent) {
        match *event {
            InputEvent::KeyDown {
                key_code, key_char, ..
            } => {
                self.last_char = key_char;
                self.toggle_key(key_code);
                self.add_key(key_code);
            }
            InputEvent::KeyUp {
                key_code, key_char, ..
            } => {
                self.last_char = key_char;
                self.remove_key(key_code);
            }
            InputEvent::MouseDown { button, .. } => {
                self.toggle_key(button.into());
                self.add_key(button.into());
            }
            InputEvent::MouseUp { button, .. } => self.remove_key(button.into()),
            _ => {}
        }
    }

    fn update_last_click(&mut self, x: f64, y: f64, index: Option<usize>) -> usize {
        let time = Utc::now();
        let index = index.unwrap_or_else(|| {
            let Some(last_click) = self.last_click.as_ref() else {
                return 0;
            };

            // TODO Make this configurable as "double click delay" and "double click distance"
            if (time - last_click.time).abs() < TimeDelta::milliseconds(500)
                && last_click.distance_squared_to(x, y) < 4.0
            {
                last_click.index + 1
            } else {
                0
            }
        });
        self.last_click = Some(ClickEventData { x, y, time, index });
        index
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_toggled(&self, key: KeyCode) -> bool {
        self.keys_toggled.contains(&key)
    }

    pub fn last_key_code(&self) -> KeyCode {
        self.last_key
    }

    pub fn last_key_char(&self) -> Option<char> {
        self.last_char
    }

    pub fn last_click_index(&self) -> usize {
        self.last_click
            .as_ref()
            .map(|lc| lc.index)
            .unwrap_or_default()
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.is_key_down(button.into())
    }

    pub fn get_mouse_down_buttons(&self) -> HashSet<MouseButton> {
        let mut buttons = HashSet::new();
        if self.is_mouse_down(MouseButton::Left) {
            buttons.insert(MouseButton::Left);
        }
        if self.is_mouse_down(MouseButton::Middle) {
            buttons.insert(MouseButton::Middle);
        }
        if self.is_mouse_down(MouseButton::Right) {
            buttons.insert(MouseButton::Right);
        }
        buttons
    }
}

fn map_to_key_code_physical(key: PhysicalKey) -> Option<KeyCode> {
    Some(match key {
        PhysicalKey::Unknown => KeyCode::UNKNOWN,
        PhysicalKey::Backquote => KeyCode::BACKQUOTE,
        PhysicalKey::Digit0 => KeyCode::NUMBER_0,
        PhysicalKey::Digit1 => KeyCode::NUMBER_1,
        PhysicalKey::Digit2 => KeyCode::NUMBER_2,
        PhysicalKey::Digit3 => KeyCode::NUMBER_3,
        PhysicalKey::Digit4 => KeyCode::NUMBER_4,
        PhysicalKey::Digit5 => KeyCode::NUMBER_5,
        PhysicalKey::Digit6 => KeyCode::NUMBER_6,
        PhysicalKey::Digit7 => KeyCode::NUMBER_7,
        PhysicalKey::Digit8 => KeyCode::NUMBER_8,
        PhysicalKey::Digit9 => KeyCode::NUMBER_9,
        PhysicalKey::Minus => KeyCode::MINUS,
        PhysicalKey::Equal => KeyCode::EQUAL,
        PhysicalKey::KeyA => KeyCode::A,
        PhysicalKey::KeyB => KeyCode::B,
        PhysicalKey::KeyC => KeyCode::C,
        PhysicalKey::KeyD => KeyCode::D,
        PhysicalKey::KeyE => KeyCode::E,
        PhysicalKey::KeyF => KeyCode::F,
        PhysicalKey::KeyG => KeyCode::G,
        PhysicalKey::KeyH => KeyCode::H,
        PhysicalKey::KeyI => KeyCode::I,
        PhysicalKey::KeyJ => KeyCode::J,
        PhysicalKey::KeyK => KeyCode::K,
        PhysicalKey::KeyL => KeyCode::L,
        PhysicalKey::KeyM => KeyCode::M,
        PhysicalKey::KeyN => KeyCode::N,
        PhysicalKey::KeyO => KeyCode::O,
        PhysicalKey::KeyP => KeyCode::P,
        PhysicalKey::KeyQ => KeyCode::Q,
        PhysicalKey::KeyR => KeyCode::R,
        PhysicalKey::KeyS => KeyCode::S,
        PhysicalKey::KeyT => KeyCode::T,
        PhysicalKey::KeyU => KeyCode::U,
        PhysicalKey::KeyV => KeyCode::V,
        PhysicalKey::KeyW => KeyCode::W,
        PhysicalKey::KeyX => KeyCode::X,
        PhysicalKey::KeyY => KeyCode::Y,
        PhysicalKey::KeyZ => KeyCode::Z,
        PhysicalKey::BracketLeft => KeyCode::LEFTBRACKET,
        PhysicalKey::BracketRight => KeyCode::RIGHTBRACKET,
        PhysicalKey::Backslash => KeyCode::BACKSLASH,
        PhysicalKey::Semicolon => KeyCode::SEMICOLON,
        PhysicalKey::Quote => KeyCode::QUOTE,
        PhysicalKey::Comma => KeyCode::COMMA,
        PhysicalKey::Period => KeyCode::PERIOD,
        PhysicalKey::Slash => KeyCode::SLASH,
        PhysicalKey::Backspace => KeyCode::BACKSPACE,
        PhysicalKey::Tab => KeyCode::TAB,
        PhysicalKey::CapsLock => KeyCode::CAPS_LOCK,
        PhysicalKey::Enter => KeyCode::ENTER,
        PhysicalKey::Space => KeyCode::SPACE,
        PhysicalKey::AltLeft => KeyCode::ALT,
        PhysicalKey::AltRight => return None,
        PhysicalKey::SuperLeft | PhysicalKey::SuperRight => return None,
        PhysicalKey::ContextMenu => return None,
        PhysicalKey::ShiftLeft | PhysicalKey::ShiftRight => KeyCode::SHIFT,
        PhysicalKey::ControlRight | PhysicalKey::ControlLeft => KeyCode::CONTROL,
        PhysicalKey::Insert => KeyCode::INSERT,
        PhysicalKey::Delete => KeyCode::DELETE,
        PhysicalKey::Home => KeyCode::HOME,
        PhysicalKey::End => KeyCode::END,
        PhysicalKey::PageUp => KeyCode::PAGE_UP,
        PhysicalKey::PageDown => KeyCode::PAGE_DOWN,
        PhysicalKey::ArrowUp => KeyCode::UP,
        PhysicalKey::ArrowLeft => KeyCode::LEFT,
        PhysicalKey::ArrowDown => KeyCode::DOWN,
        PhysicalKey::ArrowRight => KeyCode::RIGHT,
        PhysicalKey::NumLock => KeyCode::NUM_LOCK,
        PhysicalKey::NumpadDivide => KeyCode::NUMPAD_DIVIDE,
        PhysicalKey::NumpadMultiply => KeyCode::NUMPAD_MULTIPLY,
        PhysicalKey::NumpadSubtract => KeyCode::NUMPAD_SUBTRACT,
        PhysicalKey::Numpad1 => KeyCode::NUMPAD_1,
        PhysicalKey::Numpad2 => KeyCode::NUMPAD_2,
        PhysicalKey::Numpad3 => KeyCode::NUMPAD_3,
        PhysicalKey::Numpad4 => KeyCode::NUMPAD_4,
        PhysicalKey::Numpad5 => KeyCode::NUMPAD_5,
        PhysicalKey::Numpad6 => KeyCode::NUMPAD_6,
        PhysicalKey::Numpad7 => KeyCode::NUMPAD_7,
        PhysicalKey::Numpad8 => KeyCode::NUMPAD_8,
        PhysicalKey::Numpad9 => KeyCode::NUMPAD_9,
        PhysicalKey::Numpad0 => KeyCode::NUMPAD_0,
        PhysicalKey::NumpadAdd => KeyCode::NUMPAD_ADD,
        PhysicalKey::NumpadEnter => KeyCode::NUMPAD_ENTER,
        PhysicalKey::NumpadDecimal => KeyCode::NUMPAD_DECIMAL,
        PhysicalKey::Escape => KeyCode::ESCAPE,
        PhysicalKey::F1 => KeyCode::F1,
        PhysicalKey::F2 => KeyCode::F2,
        PhysicalKey::F3 => KeyCode::F3,
        PhysicalKey::F4 => KeyCode::F4,
        PhysicalKey::F5 => KeyCode::F5,
        PhysicalKey::F6 => KeyCode::F6,
        PhysicalKey::F7 => KeyCode::F7,
        PhysicalKey::F8 => KeyCode::F8,
        PhysicalKey::F9 => KeyCode::F9,
        PhysicalKey::F10 => KeyCode::F10,
        PhysicalKey::F11 => KeyCode::F11,
        PhysicalKey::F12 => KeyCode::F12,
        PhysicalKey::F13 => KeyCode::F13,
        PhysicalKey::F14 => KeyCode::F14,
        PhysicalKey::F15 => KeyCode::F15,
        PhysicalKey::F16 => KeyCode::F16,
        PhysicalKey::F17 => KeyCode::F17,
        PhysicalKey::F18 => KeyCode::F18,
        PhysicalKey::F19 => KeyCode::F19,
        PhysicalKey::F20 => KeyCode::F20,
        PhysicalKey::F21 => KeyCode::F21,
        PhysicalKey::F22 => KeyCode::F22,
        PhysicalKey::F23 => KeyCode::F23,
        PhysicalKey::F24 => KeyCode::F24,
        PhysicalKey::Fn => return None,
        PhysicalKey::FnLock => return None,
        // TODO FP returns -1 for PrintScreen?
        PhysicalKey::PrintScreen => KeyCode::UNKNOWN,
        PhysicalKey::ScrollLock => KeyCode::SCROLL_LOCK,
        PhysicalKey::Pause => KeyCode::PAUSE,
        _ => return None,
    })
}

fn map_to_key_code_logical(key: LogicalKey, location: KeyLocation) -> Option<KeyCode> {
    let is_numpad = matches!(location, KeyLocation::Numpad);
    Some(match key {
        LogicalKey::Named(NamedKey::Backspace) => KeyCode::BACKSPACE,
        LogicalKey::Named(NamedKey::Tab) => KeyCode::TAB,
        LogicalKey::Named(NamedKey::Enter) => KeyCode::ENTER,
        LogicalKey::Named(NamedKey::Shift) => KeyCode::SHIFT,
        LogicalKey::Named(NamedKey::Control) => KeyCode::CONTROL,
        LogicalKey::Named(NamedKey::Alt) => KeyCode::ALT,
        LogicalKey::Named(NamedKey::AltGraph) => return None,
        LogicalKey::Named(NamedKey::ContextMenu) => return None,
        LogicalKey::Named(NamedKey::CapsLock) => KeyCode::CAPS_LOCK,
        LogicalKey::Named(NamedKey::Escape) => KeyCode::ESCAPE,
        LogicalKey::Character(' ') => KeyCode::SPACE,
        LogicalKey::Character('0') if is_numpad => KeyCode::NUMPAD_0,
        LogicalKey::Character('1') if is_numpad => KeyCode::NUMPAD_1,
        LogicalKey::Character('2') if is_numpad => KeyCode::NUMPAD_2,
        LogicalKey::Character('3') if is_numpad => KeyCode::NUMPAD_3,
        LogicalKey::Character('4') if is_numpad => KeyCode::NUMPAD_4,
        LogicalKey::Character('5') if is_numpad => KeyCode::NUMPAD_5,
        LogicalKey::Character('6') if is_numpad => KeyCode::NUMPAD_6,
        LogicalKey::Character('7') if is_numpad => KeyCode::NUMPAD_7,
        LogicalKey::Character('8') if is_numpad => KeyCode::NUMPAD_8,
        LogicalKey::Character('9') if is_numpad => KeyCode::NUMPAD_9,
        LogicalKey::Character('*') if is_numpad => KeyCode::NUMPAD_MULTIPLY,
        LogicalKey::Character('+') if is_numpad => KeyCode::NUMPAD_ADD,
        LogicalKey::Character('-') if is_numpad => KeyCode::NUMPAD_SUBTRACT,
        LogicalKey::Character('.' | ',') if is_numpad => KeyCode::NUMPAD_DECIMAL,
        LogicalKey::Character('/') if is_numpad => KeyCode::NUMPAD_DIVIDE,
        LogicalKey::Character('0' | ')') => KeyCode::NUMBER_0,
        LogicalKey::Character('1' | '!') => KeyCode::NUMBER_1,
        LogicalKey::Character('2' | '@') => KeyCode::NUMBER_2,
        LogicalKey::Character('3' | '#') => KeyCode::NUMBER_3,
        LogicalKey::Character('4' | '$') => KeyCode::NUMBER_4,
        LogicalKey::Character('5' | '%') => KeyCode::NUMBER_5,
        LogicalKey::Character('6' | '^') => KeyCode::NUMBER_6,
        LogicalKey::Character('7' | '&') => KeyCode::NUMBER_7,
        LogicalKey::Character('8' | '*') => KeyCode::NUMBER_8,
        LogicalKey::Character('9' | '(') => KeyCode::NUMBER_9,
        LogicalKey::Character(';' | ':') => KeyCode::SEMICOLON,
        LogicalKey::Character('=' | '+') => KeyCode::EQUAL,
        LogicalKey::Character(',' | '<') => KeyCode::COMMA,
        LogicalKey::Character('-' | '_') => KeyCode::MINUS,
        LogicalKey::Character('.' | '>') => KeyCode::PERIOD,
        LogicalKey::Character('/' | '?') => KeyCode::SLASH,
        LogicalKey::Character('`' | '~') => KeyCode::BACKQUOTE,
        LogicalKey::Character('[' | '{') => KeyCode::LEFTBRACKET,
        LogicalKey::Character('\\' | '|') => KeyCode::BACKSLASH,
        LogicalKey::Character(']' | '}') => KeyCode::RIGHTBRACKET,
        LogicalKey::Character('\'' | '"') => KeyCode::QUOTE,
        LogicalKey::Named(NamedKey::PageUp) => KeyCode::PAGE_UP,
        LogicalKey::Named(NamedKey::PageDown) => KeyCode::PAGE_DOWN,
        LogicalKey::Named(NamedKey::End) => KeyCode::END,
        LogicalKey::Named(NamedKey::Home) => KeyCode::HOME,
        LogicalKey::Named(NamedKey::ArrowLeft) => KeyCode::LEFT,
        LogicalKey::Named(NamedKey::ArrowUp) => KeyCode::UP,
        LogicalKey::Named(NamedKey::ArrowRight) => KeyCode::RIGHT,
        LogicalKey::Named(NamedKey::ArrowDown) => KeyCode::DOWN,
        LogicalKey::Named(NamedKey::Insert) => KeyCode::INSERT,
        LogicalKey::Named(NamedKey::Delete) => KeyCode::DELETE,
        LogicalKey::Named(NamedKey::Pause) => KeyCode::PAUSE,
        LogicalKey::Named(NamedKey::NumLock) => KeyCode::NUM_LOCK,
        LogicalKey::Named(NamedKey::ScrollLock) => KeyCode::SCROLL_LOCK,
        LogicalKey::Named(NamedKey::F1) => KeyCode::F1,
        LogicalKey::Named(NamedKey::F2) => KeyCode::F2,
        LogicalKey::Named(NamedKey::F3) => KeyCode::F3,
        LogicalKey::Named(NamedKey::F4) => KeyCode::F4,
        LogicalKey::Named(NamedKey::F5) => KeyCode::F5,
        LogicalKey::Named(NamedKey::F6) => KeyCode::F6,
        LogicalKey::Named(NamedKey::F7) => KeyCode::F7,
        LogicalKey::Named(NamedKey::F8) => KeyCode::F8,
        LogicalKey::Named(NamedKey::F9) => KeyCode::F9,
        LogicalKey::Named(NamedKey::F10) => KeyCode::F10,
        LogicalKey::Named(NamedKey::F11) => KeyCode::F11,
        LogicalKey::Named(NamedKey::F12) => KeyCode::F12,
        LogicalKey::Named(NamedKey::F13) => KeyCode::F13,
        LogicalKey::Named(NamedKey::F14) => KeyCode::F14,
        LogicalKey::Named(NamedKey::F15) => KeyCode::F15,
        LogicalKey::Named(NamedKey::F16) => KeyCode::F16,
        LogicalKey::Named(NamedKey::F17) => KeyCode::F17,
        LogicalKey::Named(NamedKey::F18) => KeyCode::F18,
        LogicalKey::Named(NamedKey::F19) => KeyCode::F19,
        LogicalKey::Named(NamedKey::F20) => KeyCode::F20,
        LogicalKey::Named(NamedKey::F21) => KeyCode::F21,
        LogicalKey::Named(NamedKey::F22) => KeyCode::F22,
        LogicalKey::Named(NamedKey::F23) => KeyCode::F23,
        LogicalKey::Named(NamedKey::F24) => KeyCode::F24,
        LogicalKey::Character(char) => {
            // Handle alphabetic characters
            map_character_to_key_code(char).unwrap_or(KeyCode::UNKNOWN)
        }
        _ => return None,
    })
}

fn map_character_to_key_code(char: char) -> Option<KeyCode> {
    if char.is_ascii_alphabetic() {
        // ASCII alphabetic characters are all mapped to
        // their respective KeyCodes, which happen to have
        // the same numerical value as uppercase characters.
        return Some(KeyCode::from_code(char.to_ascii_uppercase() as u32));
    }

    if !char.is_ascii() {
        // Non-ASCII inputs have codes equal to their Unicode codes and yes,
        // they overlap with other codes, so that typing '½' and '-' both produce 189.
        return Some(KeyCode::from_code(char as u32));
    }

    None
}
