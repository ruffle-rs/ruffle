use crate::events::{KeyCode, MouseButton, PlayerEvent, TextControlCode};
use chrono::{DateTime, TimeDelta, Utc};
use std::collections::HashSet;

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
    last_text_control: Option<TextControlCode>,
    last_click: Option<ClickEventData>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            keys_toggled: HashSet::new(),
            last_key: KeyCode::UNKNOWN,
            last_char: None,
            last_text_control: None,
            last_click: None,
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

    pub fn handle_event(&mut self, event: &PlayerEvent) {
        match *event {
            PlayerEvent::KeyDown { key_code, key_char } => {
                self.last_char = key_char;
                self.toggle_key(key_code);
                self.add_key(key_code);
            }
            PlayerEvent::KeyUp { key_code, key_char } => {
                self.last_char = key_char;
                self.remove_key(key_code);
                self.last_text_control = None;
            }
            PlayerEvent::TextControl { code } => {
                self.last_text_control = Some(code);
            }
            PlayerEvent::MouseDown {
                x,
                y,
                button,
                index,
            } => {
                self.toggle_key(button.into());
                self.add_key(button.into());
                self.update_last_click(x, y, index);
            }
            PlayerEvent::MouseUp { button, .. } => self.remove_key(button.into()),
            _ => {}
        }
    }

    fn update_last_click(&mut self, x: f64, y: f64, index: Option<usize>) {
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

    pub fn last_text_control(&self) -> Option<TextControlCode> {
        self.last_text_control
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

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
