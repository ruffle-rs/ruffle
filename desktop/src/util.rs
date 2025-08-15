use anyhow::{anyhow, Error};
use gilrs::Button;
use ruffle_core::events::{
    GamepadButton, KeyDescriptor, KeyLocation, LogicalKey, NamedKey as RuffleNamedKey, PhysicalKey,
    TextControlCode,
};
use std::path::Path;
use url::Url;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, Modifiers};
use winit::keyboard::{
    Key, KeyCode as WinitKeyCode, KeyLocation as WinitKeyLocation, NamedKey,
    PhysicalKey as WinitPhysicalKey,
};
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

pub fn winit_input_to_ruffle_key_descriptor(event: &KeyEvent) -> KeyDescriptor {
    let physical_key = map_physical_key(event);
    let logical_key = map_logical_key(event);
    let key_location = map_key_location(event);
    KeyDescriptor {
        physical_key,
        logical_key,
        key_location,
    }
}

fn map_physical_key(event: &KeyEvent) -> PhysicalKey {
    match event.physical_key {
        WinitPhysicalKey::Code(key_code) => match key_code {
            WinitKeyCode::Backquote => PhysicalKey::Backquote,
            WinitKeyCode::Backslash => PhysicalKey::Backslash,
            WinitKeyCode::BracketLeft => PhysicalKey::BracketLeft,
            WinitKeyCode::BracketRight => PhysicalKey::BracketRight,
            WinitKeyCode::Comma => PhysicalKey::Comma,
            WinitKeyCode::Digit0 => PhysicalKey::Digit0,
            WinitKeyCode::Digit1 => PhysicalKey::Digit1,
            WinitKeyCode::Digit2 => PhysicalKey::Digit2,
            WinitKeyCode::Digit3 => PhysicalKey::Digit3,
            WinitKeyCode::Digit4 => PhysicalKey::Digit4,
            WinitKeyCode::Digit5 => PhysicalKey::Digit5,
            WinitKeyCode::Digit6 => PhysicalKey::Digit6,
            WinitKeyCode::Digit7 => PhysicalKey::Digit7,
            WinitKeyCode::Digit8 => PhysicalKey::Digit8,
            WinitKeyCode::Digit9 => PhysicalKey::Digit9,
            WinitKeyCode::Equal => PhysicalKey::Equal,
            WinitKeyCode::IntlBackslash => PhysicalKey::IntlBackslash,
            WinitKeyCode::IntlRo => PhysicalKey::IntlRo,
            WinitKeyCode::IntlYen => PhysicalKey::IntlYen,
            WinitKeyCode::KeyA => PhysicalKey::KeyA,
            WinitKeyCode::KeyB => PhysicalKey::KeyB,
            WinitKeyCode::KeyC => PhysicalKey::KeyC,
            WinitKeyCode::KeyD => PhysicalKey::KeyD,
            WinitKeyCode::KeyE => PhysicalKey::KeyE,
            WinitKeyCode::KeyF => PhysicalKey::KeyF,
            WinitKeyCode::KeyG => PhysicalKey::KeyG,
            WinitKeyCode::KeyH => PhysicalKey::KeyH,
            WinitKeyCode::KeyI => PhysicalKey::KeyI,
            WinitKeyCode::KeyJ => PhysicalKey::KeyJ,
            WinitKeyCode::KeyK => PhysicalKey::KeyK,
            WinitKeyCode::KeyL => PhysicalKey::KeyL,
            WinitKeyCode::KeyM => PhysicalKey::KeyM,
            WinitKeyCode::KeyN => PhysicalKey::KeyN,
            WinitKeyCode::KeyO => PhysicalKey::KeyO,
            WinitKeyCode::KeyP => PhysicalKey::KeyP,
            WinitKeyCode::KeyQ => PhysicalKey::KeyQ,
            WinitKeyCode::KeyR => PhysicalKey::KeyR,
            WinitKeyCode::KeyS => PhysicalKey::KeyS,
            WinitKeyCode::KeyT => PhysicalKey::KeyT,
            WinitKeyCode::KeyU => PhysicalKey::KeyU,
            WinitKeyCode::KeyV => PhysicalKey::KeyV,
            WinitKeyCode::KeyW => PhysicalKey::KeyW,
            WinitKeyCode::KeyX => PhysicalKey::KeyX,
            WinitKeyCode::KeyY => PhysicalKey::KeyY,
            WinitKeyCode::KeyZ => PhysicalKey::KeyZ,
            WinitKeyCode::Minus => PhysicalKey::Minus,
            WinitKeyCode::Period => PhysicalKey::Period,
            WinitKeyCode::Quote => PhysicalKey::Quote,
            WinitKeyCode::Semicolon => PhysicalKey::Semicolon,
            WinitKeyCode::Slash => PhysicalKey::Slash,
            WinitKeyCode::AltLeft => PhysicalKey::AltLeft,
            WinitKeyCode::AltRight => PhysicalKey::AltRight,
            WinitKeyCode::Backspace => PhysicalKey::Backspace,
            WinitKeyCode::CapsLock => PhysicalKey::CapsLock,
            WinitKeyCode::ContextMenu => PhysicalKey::ContextMenu,
            WinitKeyCode::ControlLeft => PhysicalKey::ControlLeft,
            WinitKeyCode::ControlRight => PhysicalKey::ControlRight,
            WinitKeyCode::Enter => PhysicalKey::Enter,
            WinitKeyCode::SuperLeft => PhysicalKey::SuperLeft,
            WinitKeyCode::SuperRight => PhysicalKey::SuperRight,
            WinitKeyCode::ShiftLeft => PhysicalKey::ShiftLeft,
            WinitKeyCode::ShiftRight => PhysicalKey::ShiftRight,
            WinitKeyCode::Space => PhysicalKey::Space,
            WinitKeyCode::Tab => PhysicalKey::Tab,
            WinitKeyCode::Convert => PhysicalKey::Unknown,
            WinitKeyCode::KanaMode => PhysicalKey::Unknown,
            WinitKeyCode::Lang1 => PhysicalKey::Unknown,
            WinitKeyCode::Lang2 => PhysicalKey::Unknown,
            WinitKeyCode::Lang3 => PhysicalKey::Unknown,
            WinitKeyCode::Lang4 => PhysicalKey::Unknown,
            WinitKeyCode::Lang5 => PhysicalKey::Unknown,
            WinitKeyCode::NonConvert => PhysicalKey::Unknown,
            WinitKeyCode::Delete => PhysicalKey::Delete,
            WinitKeyCode::End => PhysicalKey::End,
            WinitKeyCode::Help => PhysicalKey::Unknown,
            WinitKeyCode::Home => PhysicalKey::Home,
            WinitKeyCode::Insert => PhysicalKey::Insert,
            WinitKeyCode::PageDown => PhysicalKey::PageDown,
            WinitKeyCode::PageUp => PhysicalKey::PageUp,
            WinitKeyCode::ArrowDown => PhysicalKey::ArrowDown,
            WinitKeyCode::ArrowLeft => PhysicalKey::ArrowLeft,
            WinitKeyCode::ArrowRight => PhysicalKey::ArrowRight,
            WinitKeyCode::ArrowUp => PhysicalKey::ArrowUp,
            WinitKeyCode::NumLock => PhysicalKey::NumLock,
            WinitKeyCode::Numpad0 => PhysicalKey::Numpad0,
            WinitKeyCode::Numpad1 => PhysicalKey::Numpad1,
            WinitKeyCode::Numpad2 => PhysicalKey::Numpad2,
            WinitKeyCode::Numpad3 => PhysicalKey::Numpad3,
            WinitKeyCode::Numpad4 => PhysicalKey::Numpad4,
            WinitKeyCode::Numpad5 => PhysicalKey::Numpad5,
            WinitKeyCode::Numpad6 => PhysicalKey::Numpad6,
            WinitKeyCode::Numpad7 => PhysicalKey::Numpad7,
            WinitKeyCode::Numpad8 => PhysicalKey::Numpad8,
            WinitKeyCode::Numpad9 => PhysicalKey::Numpad9,
            WinitKeyCode::NumpadAdd => PhysicalKey::NumpadAdd,
            WinitKeyCode::NumpadComma => PhysicalKey::NumpadComma,
            WinitKeyCode::NumpadDecimal => PhysicalKey::NumpadDecimal,
            WinitKeyCode::NumpadDivide => PhysicalKey::NumpadDivide,
            WinitKeyCode::NumpadEnter => PhysicalKey::NumpadEnter,
            WinitKeyCode::NumpadMultiply => PhysicalKey::NumpadMultiply,
            WinitKeyCode::NumpadSubtract => PhysicalKey::NumpadSubtract,
            WinitKeyCode::Escape => PhysicalKey::Escape,
            WinitKeyCode::Fn => PhysicalKey::Fn,
            WinitKeyCode::FnLock => PhysicalKey::FnLock,
            WinitKeyCode::PrintScreen => PhysicalKey::PrintScreen,
            WinitKeyCode::ScrollLock => PhysicalKey::ScrollLock,
            WinitKeyCode::Pause => PhysicalKey::Pause,
            WinitKeyCode::F1 => PhysicalKey::F1,
            WinitKeyCode::F2 => PhysicalKey::F2,
            WinitKeyCode::F3 => PhysicalKey::F3,
            WinitKeyCode::F4 => PhysicalKey::F4,
            WinitKeyCode::F5 => PhysicalKey::F5,
            WinitKeyCode::F6 => PhysicalKey::F6,
            WinitKeyCode::F7 => PhysicalKey::F7,
            WinitKeyCode::F8 => PhysicalKey::F8,
            WinitKeyCode::F9 => PhysicalKey::F9,
            WinitKeyCode::F10 => PhysicalKey::F10,
            WinitKeyCode::F11 => PhysicalKey::F11,
            WinitKeyCode::F12 => PhysicalKey::F12,
            WinitKeyCode::F13 => PhysicalKey::F13,
            WinitKeyCode::F14 => PhysicalKey::F14,
            WinitKeyCode::F15 => PhysicalKey::F15,
            WinitKeyCode::F16 => PhysicalKey::F16,
            WinitKeyCode::F17 => PhysicalKey::F17,
            WinitKeyCode::F18 => PhysicalKey::F18,
            WinitKeyCode::F19 => PhysicalKey::F19,
            WinitKeyCode::F20 => PhysicalKey::F20,
            WinitKeyCode::F21 => PhysicalKey::F21,
            WinitKeyCode::F22 => PhysicalKey::F22,
            WinitKeyCode::F23 => PhysicalKey::F23,
            WinitKeyCode::F24 => PhysicalKey::F24,
            WinitKeyCode::F25 => PhysicalKey::F25,
            WinitKeyCode::F26 => PhysicalKey::F26,
            WinitKeyCode::F27 => PhysicalKey::F27,
            WinitKeyCode::F28 => PhysicalKey::F28,
            WinitKeyCode::F29 => PhysicalKey::F29,
            WinitKeyCode::F30 => PhysicalKey::F30,
            WinitKeyCode::F31 => PhysicalKey::F31,
            WinitKeyCode::F32 => PhysicalKey::F32,
            WinitKeyCode::F33 => PhysicalKey::F33,
            WinitKeyCode::F34 => PhysicalKey::F34,
            WinitKeyCode::F35 => PhysicalKey::F35,
            _ => PhysicalKey::Unknown,
        },
        WinitPhysicalKey::Unidentified(_) => PhysicalKey::Unknown,
    }
}

fn map_logical_key(event: &KeyEvent) -> LogicalKey {
    // Note: it would be tempting to use event.key_without_modifiers() here, but FP
    // does not care about keys without modifiers at all, it does its own mapping,
    // so that on English UK, Shift+3 produces 16+163, not 16+51.

    match event.logical_key.as_ref() {
        Key::Named(NamedKey::Alt) => LogicalKey::Named(RuffleNamedKey::Alt),
        Key::Named(NamedKey::AltGraph) => LogicalKey::Named(RuffleNamedKey::AltGraph),
        Key::Named(NamedKey::CapsLock) => LogicalKey::Named(RuffleNamedKey::CapsLock),
        Key::Named(NamedKey::Control) => LogicalKey::Named(RuffleNamedKey::Control),
        Key::Named(NamedKey::Fn) => LogicalKey::Named(RuffleNamedKey::Fn),
        Key::Named(NamedKey::FnLock) => LogicalKey::Named(RuffleNamedKey::FnLock),
        Key::Named(NamedKey::NumLock) => LogicalKey::Named(RuffleNamedKey::NumLock),
        Key::Named(NamedKey::ScrollLock) => LogicalKey::Named(RuffleNamedKey::ScrollLock),
        Key::Named(NamedKey::Shift) => LogicalKey::Named(RuffleNamedKey::Shift),
        Key::Named(NamedKey::Symbol) => LogicalKey::Named(RuffleNamedKey::Symbol),
        Key::Named(NamedKey::SymbolLock) => LogicalKey::Named(RuffleNamedKey::SymbolLock),
        Key::Named(NamedKey::Super) => LogicalKey::Named(RuffleNamedKey::Super),
        Key::Named(NamedKey::Enter) => LogicalKey::Named(RuffleNamedKey::Enter),
        Key::Named(NamedKey::Tab) => LogicalKey::Named(RuffleNamedKey::Tab),
        Key::Named(NamedKey::Space) => LogicalKey::Character(' '),
        Key::Named(NamedKey::ArrowDown) => LogicalKey::Named(RuffleNamedKey::ArrowDown),
        Key::Named(NamedKey::ArrowLeft) => LogicalKey::Named(RuffleNamedKey::ArrowLeft),
        Key::Named(NamedKey::ArrowRight) => LogicalKey::Named(RuffleNamedKey::ArrowRight),
        Key::Named(NamedKey::ArrowUp) => LogicalKey::Named(RuffleNamedKey::ArrowUp),
        Key::Named(NamedKey::End) => LogicalKey::Named(RuffleNamedKey::End),
        Key::Named(NamedKey::Home) => LogicalKey::Named(RuffleNamedKey::Home),
        Key::Named(NamedKey::PageDown) => LogicalKey::Named(RuffleNamedKey::PageDown),
        Key::Named(NamedKey::PageUp) => LogicalKey::Named(RuffleNamedKey::PageUp),
        Key::Named(NamedKey::Backspace) => LogicalKey::Named(RuffleNamedKey::Backspace),
        Key::Named(NamedKey::Clear) => LogicalKey::Named(RuffleNamedKey::Clear),
        Key::Named(NamedKey::Copy) => LogicalKey::Named(RuffleNamedKey::Copy),
        Key::Named(NamedKey::CrSel) => LogicalKey::Named(RuffleNamedKey::CrSel),
        Key::Named(NamedKey::Cut) => LogicalKey::Named(RuffleNamedKey::Cut),
        Key::Named(NamedKey::Delete) => LogicalKey::Named(RuffleNamedKey::Delete),
        Key::Named(NamedKey::EraseEof) => LogicalKey::Named(RuffleNamedKey::EraseEof),
        Key::Named(NamedKey::ExSel) => LogicalKey::Named(RuffleNamedKey::ExSel),
        Key::Named(NamedKey::Insert) => LogicalKey::Named(RuffleNamedKey::Insert),
        Key::Named(NamedKey::Paste) => LogicalKey::Named(RuffleNamedKey::Paste),
        Key::Named(NamedKey::Redo) => LogicalKey::Named(RuffleNamedKey::Redo),
        Key::Named(NamedKey::Undo) => LogicalKey::Named(RuffleNamedKey::Undo),
        Key::Named(NamedKey::ContextMenu) => LogicalKey::Named(RuffleNamedKey::ContextMenu),
        Key::Named(NamedKey::Escape) => LogicalKey::Named(RuffleNamedKey::Escape),
        Key::Named(NamedKey::Pause) => LogicalKey::Named(RuffleNamedKey::Pause),
        Key::Named(NamedKey::Play) => LogicalKey::Named(RuffleNamedKey::Play),
        Key::Named(NamedKey::Select) => LogicalKey::Named(RuffleNamedKey::Select),
        Key::Named(NamedKey::ZoomIn) => LogicalKey::Named(RuffleNamedKey::ZoomIn),
        Key::Named(NamedKey::ZoomOut) => LogicalKey::Named(RuffleNamedKey::ZoomOut),
        Key::Named(NamedKey::PrintScreen) => LogicalKey::Named(RuffleNamedKey::PrintScreen),
        Key::Named(NamedKey::F1) => LogicalKey::Named(RuffleNamedKey::F1),
        Key::Named(NamedKey::F2) => LogicalKey::Named(RuffleNamedKey::F2),
        Key::Named(NamedKey::F3) => LogicalKey::Named(RuffleNamedKey::F3),
        Key::Named(NamedKey::F4) => LogicalKey::Named(RuffleNamedKey::F4),
        Key::Named(NamedKey::F5) => LogicalKey::Named(RuffleNamedKey::F5),
        Key::Named(NamedKey::F6) => LogicalKey::Named(RuffleNamedKey::F6),
        Key::Named(NamedKey::F7) => LogicalKey::Named(RuffleNamedKey::F7),
        Key::Named(NamedKey::F8) => LogicalKey::Named(RuffleNamedKey::F8),
        Key::Named(NamedKey::F9) => LogicalKey::Named(RuffleNamedKey::F9),
        Key::Named(NamedKey::F10) => LogicalKey::Named(RuffleNamedKey::F10),
        Key::Named(NamedKey::F11) => LogicalKey::Named(RuffleNamedKey::F11),
        Key::Named(NamedKey::F12) => LogicalKey::Named(RuffleNamedKey::F12),
        Key::Named(NamedKey::F13) => LogicalKey::Named(RuffleNamedKey::F13),
        Key::Named(NamedKey::F14) => LogicalKey::Named(RuffleNamedKey::F14),
        Key::Named(NamedKey::F15) => LogicalKey::Named(RuffleNamedKey::F15),
        Key::Named(NamedKey::F16) => LogicalKey::Named(RuffleNamedKey::F16),
        Key::Named(NamedKey::F17) => LogicalKey::Named(RuffleNamedKey::F17),
        Key::Named(NamedKey::F18) => LogicalKey::Named(RuffleNamedKey::F18),
        Key::Named(NamedKey::F19) => LogicalKey::Named(RuffleNamedKey::F19),
        Key::Named(NamedKey::F20) => LogicalKey::Named(RuffleNamedKey::F20),
        Key::Named(NamedKey::F21) => LogicalKey::Named(RuffleNamedKey::F21),
        Key::Named(NamedKey::F22) => LogicalKey::Named(RuffleNamedKey::F22),
        Key::Named(NamedKey::F23) => LogicalKey::Named(RuffleNamedKey::F23),
        Key::Named(NamedKey::F24) => LogicalKey::Named(RuffleNamedKey::F24),
        Key::Named(NamedKey::F25) => LogicalKey::Named(RuffleNamedKey::F25),
        Key::Named(NamedKey::F26) => LogicalKey::Named(RuffleNamedKey::F26),
        Key::Named(NamedKey::F27) => LogicalKey::Named(RuffleNamedKey::F27),
        Key::Named(NamedKey::F28) => LogicalKey::Named(RuffleNamedKey::F28),
        Key::Named(NamedKey::F29) => LogicalKey::Named(RuffleNamedKey::F29),
        Key::Named(NamedKey::F30) => LogicalKey::Named(RuffleNamedKey::F30),
        Key::Named(NamedKey::F31) => LogicalKey::Named(RuffleNamedKey::F31),
        Key::Named(NamedKey::F32) => LogicalKey::Named(RuffleNamedKey::F32),
        Key::Named(NamedKey::F33) => LogicalKey::Named(RuffleNamedKey::F33),
        Key::Named(NamedKey::F34) => LogicalKey::Named(RuffleNamedKey::F34),
        Key::Named(NamedKey::F35) => LogicalKey::Named(RuffleNamedKey::F35),
        Key::Character(ch) => {
            // Handle alphabetic characters
            alpha_to_logical(ch)
        }
        _ => LogicalKey::Unknown,
    }
}

fn alpha_to_logical(ch: &str) -> LogicalKey {
    // TODO What if we get multiple chars?
    if let Some(ch) = ch.chars().last() {
        LogicalKey::Character(ch)
    } else {
        LogicalKey::Unknown
    }
}

fn map_key_location(event: &KeyEvent) -> KeyLocation {
    match event.location {
        WinitKeyLocation::Standard => KeyLocation::Standard,
        WinitKeyLocation::Left => KeyLocation::Left,
        WinitKeyLocation::Right => KeyLocation::Right,
        WinitKeyLocation::Numpad => KeyLocation::Numpad,
    }
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

    tracy.plot(BIND_GROUPS, report.hub.bind_groups.num_allocated as f64);
    tracy.plot(BUFFERS, report.hub.buffers.num_allocated as f64);
    tracy.plot(TEXTURES, report.hub.textures.num_allocated as f64);
    tracy.plot(TEXTURE_VIEWS, report.hub.texture_views.num_allocated as f64);

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
