use anyhow::{anyhow, Error};
use gilrs::Button;
use ruffle_core::events::{GamepadButton, KeyCode, TextControlCode};
use std::path::Path;
use url::Url;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, Modifiers};
use winit::keyboard::{Key, KeyLocation, NamedKey};
use winit::window::Window;

/// Converts a winit event to a Ruffle `TextControlCode`.
/// Returns `None` if there is no match.
pub fn winit_to_ruffle_text_control(
    event: &KeyEvent,
    modifiers: &Modifiers,
) -> Option<TextControlCode> {
    let shift = modifiers.state().shift_key();
    let ctrl_cmd = modifiers.state().control_key()
        || (modifiers.state().super_key() && cfg!(target_os = "macos"));
    match event.logical_key.as_ref() {
        Key::Named(NamedKey::Enter) => Some(TextControlCode::Enter),
        Key::Character("a") if ctrl_cmd => Some(TextControlCode::SelectAll),
        Key::Character("c") if ctrl_cmd => Some(TextControlCode::Copy),
        Key::Character("v") if ctrl_cmd => Some(TextControlCode::Paste),
        Key::Character("x") if ctrl_cmd => Some(TextControlCode::Cut),
        Key::Named(NamedKey::Backspace) if ctrl_cmd => Some(TextControlCode::BackspaceWord),
        Key::Named(NamedKey::Backspace) => Some(TextControlCode::Backspace),
        Key::Named(NamedKey::Delete) if ctrl_cmd => Some(TextControlCode::DeleteWord),
        Key::Named(NamedKey::Delete) => Some(TextControlCode::Delete),
        Key::Named(NamedKey::ArrowLeft) if ctrl_cmd && shift => {
            Some(TextControlCode::SelectLeftWord)
        }
        Key::Named(NamedKey::ArrowLeft) if ctrl_cmd => Some(TextControlCode::MoveLeftWord),
        Key::Named(NamedKey::ArrowLeft) if shift => Some(TextControlCode::SelectLeft),
        Key::Named(NamedKey::ArrowLeft) => Some(TextControlCode::MoveLeft),
        Key::Named(NamedKey::ArrowRight) if ctrl_cmd && shift => {
            Some(TextControlCode::SelectRightWord)
        }
        Key::Named(NamedKey::ArrowRight) if ctrl_cmd => Some(TextControlCode::MoveRightWord),
        Key::Named(NamedKey::ArrowRight) if shift => Some(TextControlCode::SelectRight),
        Key::Named(NamedKey::ArrowRight) => Some(TextControlCode::MoveRight),
        Key::Named(NamedKey::Home) if ctrl_cmd && shift => {
            Some(TextControlCode::SelectLeftDocument)
        }
        Key::Named(NamedKey::Home) if ctrl_cmd => Some(TextControlCode::MoveLeftDocument),
        Key::Named(NamedKey::Home) if shift => Some(TextControlCode::SelectLeftLine),
        Key::Named(NamedKey::Home) => Some(TextControlCode::MoveLeftLine),
        Key::Named(NamedKey::End) if ctrl_cmd && shift => {
            Some(TextControlCode::SelectRightDocument)
        }
        Key::Named(NamedKey::End) if ctrl_cmd => Some(TextControlCode::MoveRightDocument),
        Key::Named(NamedKey::End) if shift => Some(TextControlCode::SelectRightLine),
        Key::Named(NamedKey::End) => Some(TextControlCode::MoveRightLine),
        _ => None,
    }
}

/// Convert a winit event into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
pub fn winit_to_ruffle_key_code(event: &KeyEvent) -> Option<KeyCode> {
    // Note: it would be tempting to use event.key_without_modifiers() here, but FP
    // does not care about keys without modifiers at all, it does its own mapping,
    // so that on English UK, Shift+3 produces 16+163, not 16+51.

    let is_numpad = event.location == KeyLocation::Numpad;
    let key_code = match event.logical_key.as_ref() {
        Key::Named(NamedKey::Backspace) => KeyCode::BACKSPACE,
        Key::Named(NamedKey::Tab) => KeyCode::TAB,
        Key::Named(NamedKey::Enter) => KeyCode::RETURN,
        Key::Named(NamedKey::Shift) => KeyCode::SHIFT,
        Key::Named(NamedKey::Control) => KeyCode::CONTROL,
        Key::Named(NamedKey::Alt) => KeyCode::ALT,
        // AltGr is ignored by FP
        Key::Named(NamedKey::AltGraph) => return None,
        Key::Named(NamedKey::CapsLock) => KeyCode::CAPS_LOCK,
        Key::Named(NamedKey::Escape) => KeyCode::ESCAPE,
        Key::Named(NamedKey::Space) => KeyCode::SPACE,
        // Note: FP DOES care about modifiers for numpad keys,
        // so that Shift+Numpad7 produces 16+36, not 16+103.
        Key::Character("0") if is_numpad => KeyCode::NUMPAD0,
        Key::Character("1") if is_numpad => KeyCode::NUMPAD1,
        Key::Character("2") if is_numpad => KeyCode::NUMPAD2,
        Key::Character("3") if is_numpad => KeyCode::NUMPAD3,
        Key::Character("4") if is_numpad => KeyCode::NUMPAD4,
        Key::Character("5") if is_numpad => KeyCode::NUMPAD5,
        Key::Character("6") if is_numpad => KeyCode::NUMPAD6,
        Key::Character("7") if is_numpad => KeyCode::NUMPAD7,
        Key::Character("8") if is_numpad => KeyCode::NUMPAD8,
        Key::Character("9") if is_numpad => KeyCode::NUMPAD9,
        Key::Character("*") if is_numpad => KeyCode::MULTIPLY,
        Key::Character("+") if is_numpad => KeyCode::PLUS,
        Key::Character("-") if is_numpad => KeyCode::NUMPAD_MINUS,
        Key::Character(".") if is_numpad => KeyCode::NUMPAD_PERIOD,
        Key::Character("/") if is_numpad => KeyCode::NUMPAD_SLASH,
        Key::Character("0") | Key::Character(")") => KeyCode::KEY0,
        Key::Character("1") | Key::Character("!") => KeyCode::KEY1,
        Key::Character("2") | Key::Character("@") => KeyCode::KEY2,
        Key::Character("3") | Key::Character("#") => KeyCode::KEY3,
        Key::Character("4") | Key::Character("$") => KeyCode::KEY4,
        Key::Character("5") | Key::Character("%") => KeyCode::KEY5,
        Key::Character("6") | Key::Character("^") => KeyCode::KEY6,
        Key::Character("7") | Key::Character("&") => KeyCode::KEY7,
        Key::Character("8") | Key::Character("*") => KeyCode::KEY8,
        Key::Character("9") | Key::Character("(") => KeyCode::KEY9,
        Key::Character(";") | Key::Character(":") => KeyCode::SEMICOLON,
        Key::Character("=") | Key::Character("+") => KeyCode::EQUALS,
        Key::Character(",") | Key::Character("<") => KeyCode::COMMA,
        Key::Character("-") | Key::Character("_") => KeyCode::MINUS,
        Key::Character(".") | Key::Character(">") => KeyCode::PERIOD,
        Key::Character("/") | Key::Character("?") => KeyCode::SLASH,
        Key::Character("`") | Key::Character("~") => KeyCode::GRAVE,
        Key::Character("[") | Key::Character("{") => KeyCode::LBRACKET,
        Key::Character("\\") | Key::Character("|") => KeyCode::BACKSLASH,
        Key::Character("]") | Key::Character("}") => KeyCode::RBRACKET,
        Key::Character("'") | Key::Character("\"") => KeyCode::APOSTROPHE,
        Key::Named(NamedKey::PageUp) => KeyCode::PG_UP,
        Key::Named(NamedKey::PageDown) => KeyCode::PG_DOWN,
        Key::Named(NamedKey::End) => KeyCode::END,
        Key::Named(NamedKey::Home) => KeyCode::HOME,
        Key::Named(NamedKey::ArrowLeft) => KeyCode::LEFT,
        Key::Named(NamedKey::ArrowUp) => KeyCode::UP,
        Key::Named(NamedKey::ArrowRight) => KeyCode::RIGHT,
        Key::Named(NamedKey::ArrowDown) => KeyCode::DOWN,
        Key::Named(NamedKey::Insert) => KeyCode::INSERT,
        Key::Named(NamedKey::Delete) => KeyCode::DELETE,
        Key::Named(NamedKey::Pause) => KeyCode::PAUSE,
        Key::Named(NamedKey::NumLock) => KeyCode::NUM_LOCK,
        Key::Named(NamedKey::ScrollLock) => KeyCode::SCROLL_LOCK,
        Key::Named(NamedKey::F1) => KeyCode::F1,
        Key::Named(NamedKey::F2) => KeyCode::F2,
        Key::Named(NamedKey::F3) => KeyCode::F3,
        Key::Named(NamedKey::F4) => KeyCode::F4,
        Key::Named(NamedKey::F5) => KeyCode::F5,
        Key::Named(NamedKey::F6) => KeyCode::F6,
        Key::Named(NamedKey::F7) => KeyCode::F7,
        Key::Named(NamedKey::F8) => KeyCode::F8,
        Key::Named(NamedKey::F9) => KeyCode::F9,
        Key::Named(NamedKey::F10) => KeyCode::F10,
        Key::Named(NamedKey::F11) => KeyCode::F11,
        Key::Named(NamedKey::F12) => KeyCode::F12,
        Key::Named(NamedKey::F13) => KeyCode::F13,
        Key::Named(NamedKey::F14) => KeyCode::F14,
        Key::Named(NamedKey::F15) => KeyCode::F15,
        Key::Named(NamedKey::F16) => KeyCode::F16,
        Key::Named(NamedKey::F17) => KeyCode::F17,
        Key::Named(NamedKey::F18) => KeyCode::F18,
        Key::Named(NamedKey::F19) => KeyCode::F19,
        Key::Named(NamedKey::F20) => KeyCode::F20,
        Key::Named(NamedKey::F21) => KeyCode::F21,
        Key::Named(NamedKey::F22) => KeyCode::F22,
        Key::Named(NamedKey::F23) => KeyCode::F23,
        Key::Named(NamedKey::F24) => KeyCode::F24,
        Key::Character(char) => {
            // Handle alphabetic characters
            alpha_to_ruffle_key_code(char).unwrap_or(KeyCode::UNKNOWN)
        }
        _ => KeyCode::UNKNOWN,
    };
    Some(key_code)
}

fn alpha_to_ruffle_key_code(char: &str) -> Option<KeyCode> {
    if char.len() != 1 {
        return None;
    }

    let char = char.chars().next()?;

    if char.is_ascii_alphabetic() {
        // ASCII alphabetic characters are all mapped to
        // their respective KeyCodes, which happen to have
        // the same numerical value as uppercase characters.
        return Some(KeyCode::from_code(char.to_ascii_uppercase() as u32));
    }

    if !char.is_ascii() {
        // TODO Non-ASCII inputs have codes equal to their Unicode codes and yes,
        //   they overlap with other codes, so that typing '½' and '-' both produce 189.
        return None;
    }

    None
}

pub fn gilrs_button_to_gamepad_button(button: Button) -> Option<GamepadButton> {
    match button {
        Button::South => Some(GamepadButton::South),
        Button::East => Some(GamepadButton::East),
        Button::North => Some(GamepadButton::North),
        Button::West => Some(GamepadButton::West),
        Button::LeftTrigger => Some(GamepadButton::LeftTrigger),
        Button::LeftTrigger2 => Some(GamepadButton::LeftTrigger2),
        Button::RightTrigger => Some(GamepadButton::RightTrigger),
        Button::RightTrigger2 => Some(GamepadButton::RightTrigger2),
        Button::Select => Some(GamepadButton::Select),
        Button::Start => Some(GamepadButton::Start),
        Button::DPadUp => Some(GamepadButton::DPadUp),
        Button::DPadDown => Some(GamepadButton::DPadDown),
        Button::DPadLeft => Some(GamepadButton::DPadLeft),
        Button::DPadRight => Some(GamepadButton::DPadRight),
        // GilRs has some more buttons that are seemingly not supported anywhere
        // like C or Z.
        _ => None,
    }
}

pub fn get_screen_size(window: &Window) -> PhysicalSize<u32> {
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;

    for monitor in window.available_monitors() {
        let size = monitor.size();
        let position = monitor.position();
        min_x = min_x.min(position.x);
        min_y = min_y.min(position.y);
        max_x = max_x.max(position.x + size.width as i32);
        max_y = max_y.max(position.y + size.height as i32);
    }

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width <= 32 || height <= 32 {
        return (i16::MAX as u32, i16::MAX as u32).into();
    }

    (width, height).into()
}

pub fn parse_url(path: &Path) -> Result<Url, Error> {
    if path.exists() {
        let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        Url::from_file_path(absolute_path)
            .map_err(|_| anyhow!("Path must be absolute and cannot be a URL"))
    } else {
        Url::parse(path.to_str().unwrap_or_default())
            .ok()
            .filter(|url| url.host().is_some() || url.scheme() == "file")
            .ok_or_else(|| anyhow!("Input path is not a file and could not be parsed as a URL."))
    }
}

#[cfg(not(feature = "tracy"))]
pub fn plot_stats_in_tracy(_instance: &wgpu::Instance) {}

#[cfg(feature = "tracy")]
pub fn plot_stats_in_tracy(instance: &wgpu::Instance) {
    use tracing_tracy::client::*;
    const BIND_GROUPS: PlotName = plot_name!("Bind Groups");
    const BUFFERS: PlotName = plot_name!("Buffers");
    const TEXTURES: PlotName = plot_name!("Textures");
    const TEXTURE_VIEWS: PlotName = plot_name!("Texture Views");

    let tracy = Client::running().expect("tracy client must be running");
    let report = instance
        .generate_report()
        .expect("reports should be available on desktop");

    #[allow(unused_mut)]
    let mut backend = None;
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        backend = backend.or(report.vulkan).or(report.gl);
    }
    #[cfg(windows)]
    {
        backend = backend.or(report.dx12);
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        backend = backend.or(report.metal);
    }

    if let Some(stats) = backend {
        tracy.plot(BIND_GROUPS, stats.bind_groups.num_allocated as f64);
        tracy.plot(BUFFERS, stats.buffers.num_allocated as f64);
        tracy.plot(TEXTURES, stats.textures.num_allocated as f64);
        tracy.plot(TEXTURE_VIEWS, stats.texture_views.num_allocated as f64);
    }

    tracy.frame_mark();
}

pub fn open_url(url: &Url) {
    // TODO: This opens local files in the browser while flash opens them
    // in the default program for the respective filetype.
    // This especially includes mailto links. Ruffle opens the browser which opens
    // the preferred program while flash opens the preferred program directly.
    match webbrowser::open(url.as_str()) {
        Ok(_output) => {}
        Err(e) => tracing::error!("Could not open URL {}: {}", url.as_str(), e),
    };
}
