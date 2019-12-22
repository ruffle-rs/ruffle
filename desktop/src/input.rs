use glium::Display;
use ruffle_core::backend::input::InputBackend;
use ruffle_core::events::KeyCode;
use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub struct WinitInputBackend {
    keys_down: HashSet<VirtualKeyCode>,
    display: Display,
    cursor_visible: bool,
}

impl WinitInputBackend {
    pub fn new(display: Display) -> Self {
        Self {
            keys_down: HashSet::new(),
            cursor_visible: true,
            display,
        }
    }

    pub fn update(&mut self, input: KeyboardInput) {
        match input.state {
            ElementState::Pressed => {
                if let Some(key) = input.virtual_keycode {
                    self.keys_down.insert(key);
                }
            }
            ElementState::Released => {
                if let Some(key) = input.virtual_keycode {
                    self.keys_down.remove(&key);
                }
            }
        }
    }
}

impl InputBackend for WinitInputBackend {
    fn is_key_down(&self, key: KeyCode) -> bool {
        match key {
            KeyCode::Unknown => false,
            KeyCode::Backspace => self.keys_down.contains(&VirtualKeyCode::Back),
            KeyCode::Return => self.keys_down.contains(&VirtualKeyCode::Return),
            KeyCode::Shift => {
                self.keys_down.contains(&VirtualKeyCode::LShift)
                    || self.keys_down.contains(&VirtualKeyCode::RShift)
            }
            KeyCode::Control => {
                self.keys_down.contains(&VirtualKeyCode::LControl)
                    || self.keys_down.contains(&VirtualKeyCode::RControl)
            }
            KeyCode::Alt => {
                self.keys_down.contains(&VirtualKeyCode::LAlt)
                    || self.keys_down.contains(&VirtualKeyCode::RAlt)
            }
            KeyCode::CapsLock => self.keys_down.contains(&VirtualKeyCode::Capital),
            KeyCode::Escape => self.keys_down.contains(&VirtualKeyCode::Escape),
            KeyCode::Space => self.keys_down.contains(&VirtualKeyCode::Space),
            KeyCode::Key0 => self.keys_down.contains(&VirtualKeyCode::Key0),
            KeyCode::Key1 => self.keys_down.contains(&VirtualKeyCode::Key1),
            KeyCode::Key2 => self.keys_down.contains(&VirtualKeyCode::Key2),
            KeyCode::Key3 => self.keys_down.contains(&VirtualKeyCode::Key3),
            KeyCode::Key4 => self.keys_down.contains(&VirtualKeyCode::Key4),
            KeyCode::Key5 => self.keys_down.contains(&VirtualKeyCode::Key5),
            KeyCode::Key6 => self.keys_down.contains(&VirtualKeyCode::Key6),
            KeyCode::Key7 => self.keys_down.contains(&VirtualKeyCode::Key7),
            KeyCode::Key8 => self.keys_down.contains(&VirtualKeyCode::Key8),
            KeyCode::Key9 => self.keys_down.contains(&VirtualKeyCode::Key9),
            KeyCode::A => self.keys_down.contains(&VirtualKeyCode::A),
            KeyCode::B => self.keys_down.contains(&VirtualKeyCode::B),
            KeyCode::C => self.keys_down.contains(&VirtualKeyCode::C),
            KeyCode::D => self.keys_down.contains(&VirtualKeyCode::D),
            KeyCode::E => self.keys_down.contains(&VirtualKeyCode::E),
            KeyCode::F => self.keys_down.contains(&VirtualKeyCode::F),
            KeyCode::G => self.keys_down.contains(&VirtualKeyCode::G),
            KeyCode::H => self.keys_down.contains(&VirtualKeyCode::H),
            KeyCode::I => self.keys_down.contains(&VirtualKeyCode::I),
            KeyCode::J => self.keys_down.contains(&VirtualKeyCode::J),
            KeyCode::K => self.keys_down.contains(&VirtualKeyCode::K),
            KeyCode::L => self.keys_down.contains(&VirtualKeyCode::L),
            KeyCode::M => self.keys_down.contains(&VirtualKeyCode::M),
            KeyCode::N => self.keys_down.contains(&VirtualKeyCode::N),
            KeyCode::O => self.keys_down.contains(&VirtualKeyCode::O),
            KeyCode::P => self.keys_down.contains(&VirtualKeyCode::P),
            KeyCode::Q => self.keys_down.contains(&VirtualKeyCode::Q),
            KeyCode::R => self.keys_down.contains(&VirtualKeyCode::R),
            KeyCode::S => self.keys_down.contains(&VirtualKeyCode::S),
            KeyCode::T => self.keys_down.contains(&VirtualKeyCode::T),
            KeyCode::U => self.keys_down.contains(&VirtualKeyCode::U),
            KeyCode::V => self.keys_down.contains(&VirtualKeyCode::V),
            KeyCode::W => self.keys_down.contains(&VirtualKeyCode::W),
            KeyCode::X => self.keys_down.contains(&VirtualKeyCode::X),
            KeyCode::Y => self.keys_down.contains(&VirtualKeyCode::Y),
            KeyCode::Z => self.keys_down.contains(&VirtualKeyCode::Z),
            KeyCode::Semicolon => self.keys_down.contains(&VirtualKeyCode::Semicolon),
            KeyCode::Equals => self.keys_down.contains(&VirtualKeyCode::Equals),
            KeyCode::Comma => self.keys_down.contains(&VirtualKeyCode::Comma),
            KeyCode::Minus => self.keys_down.contains(&VirtualKeyCode::Minus),
            KeyCode::Period => self.keys_down.contains(&VirtualKeyCode::Period),
            KeyCode::Slash => self.keys_down.contains(&VirtualKeyCode::Slash),
            KeyCode::Grave => self.keys_down.contains(&VirtualKeyCode::Grave),
            KeyCode::LBracket => self.keys_down.contains(&VirtualKeyCode::LBracket),
            KeyCode::Backslash => self.keys_down.contains(&VirtualKeyCode::Backslash),
            KeyCode::RBracket => self.keys_down.contains(&VirtualKeyCode::RBracket),
            KeyCode::Apostrophe => self.keys_down.contains(&VirtualKeyCode::Apostrophe),
            KeyCode::Numpad0 => self.keys_down.contains(&VirtualKeyCode::Numpad0),
            KeyCode::Numpad1 => self.keys_down.contains(&VirtualKeyCode::Numpad1),
            KeyCode::Numpad2 => self.keys_down.contains(&VirtualKeyCode::Numpad2),
            KeyCode::Numpad3 => self.keys_down.contains(&VirtualKeyCode::Numpad3),
            KeyCode::Numpad4 => self.keys_down.contains(&VirtualKeyCode::Numpad4),
            KeyCode::Numpad5 => self.keys_down.contains(&VirtualKeyCode::Numpad5),
            KeyCode::Numpad6 => self.keys_down.contains(&VirtualKeyCode::Numpad6),
            KeyCode::Numpad7 => self.keys_down.contains(&VirtualKeyCode::Numpad7),
            KeyCode::Numpad8 => self.keys_down.contains(&VirtualKeyCode::Numpad8),
            KeyCode::Numpad9 => self.keys_down.contains(&VirtualKeyCode::Numpad9),
            KeyCode::Multiply => self.keys_down.contains(&VirtualKeyCode::Multiply),
            KeyCode::Plus => self.keys_down.contains(&VirtualKeyCode::Add),
            KeyCode::NumpadMinus => self.keys_down.contains(&VirtualKeyCode::Subtract),
            KeyCode::NumpadPeriod => self.keys_down.contains(&VirtualKeyCode::Decimal),
            KeyCode::NumpadSlash => self.keys_down.contains(&VirtualKeyCode::Divide),
            KeyCode::PgUp => self.keys_down.contains(&VirtualKeyCode::PageUp),
            KeyCode::PgDown => self.keys_down.contains(&VirtualKeyCode::PageDown),
            KeyCode::End => self.keys_down.contains(&VirtualKeyCode::End),
            KeyCode::Home => self.keys_down.contains(&VirtualKeyCode::Home),
            KeyCode::Left => self.keys_down.contains(&VirtualKeyCode::Left),
            KeyCode::Up => self.keys_down.contains(&VirtualKeyCode::Up),
            KeyCode::Right => self.keys_down.contains(&VirtualKeyCode::Right),
            KeyCode::Down => self.keys_down.contains(&VirtualKeyCode::Down),
            KeyCode::Insert => self.keys_down.contains(&VirtualKeyCode::Insert),
            KeyCode::Delete => self.keys_down.contains(&VirtualKeyCode::Delete),
            KeyCode::Pause => self.keys_down.contains(&VirtualKeyCode::Pause),
            KeyCode::ScrollLock => self.keys_down.contains(&VirtualKeyCode::Scroll),
            KeyCode::F1 => self.keys_down.contains(&VirtualKeyCode::F1),
            KeyCode::F2 => self.keys_down.contains(&VirtualKeyCode::F2),
            KeyCode::F3 => self.keys_down.contains(&VirtualKeyCode::F3),
            KeyCode::F4 => self.keys_down.contains(&VirtualKeyCode::F4),
            KeyCode::F5 => self.keys_down.contains(&VirtualKeyCode::F5),
            KeyCode::F6 => self.keys_down.contains(&VirtualKeyCode::F6),
            KeyCode::F7 => self.keys_down.contains(&VirtualKeyCode::F7),
            KeyCode::F8 => self.keys_down.contains(&VirtualKeyCode::F8),
            KeyCode::F9 => self.keys_down.contains(&VirtualKeyCode::F9),
            KeyCode::F10 => self.keys_down.contains(&VirtualKeyCode::F10),
            KeyCode::F11 => self.keys_down.contains(&VirtualKeyCode::F11),
            KeyCode::F12 => self.keys_down.contains(&VirtualKeyCode::F12),
        }
    }

    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn hide_mouse(&mut self) {
        self.display.gl_window().window().set_cursor_visible(false);
        self.cursor_visible = false;
    }

    fn show_mouse(&mut self) {
        self.display.gl_window().window().set_cursor_visible(true);
        self.cursor_visible = true;
    }
}
