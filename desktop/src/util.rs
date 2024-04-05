use crate::custom_event::RuffleEvent;
use anyhow::{anyhow, Error};
use gilrs::Button;
use rfd::FileDialog;
use ruffle_core::events::{GamepadButton, KeyCode, TextControlCode};
use std::path::{Path, PathBuf};
use url::Url;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, Modifiers};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, KeyLocation, NamedKey};

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
pub fn winit_to_ruffle_key_code(event: &KeyEvent) -> KeyCode {
    match event.logical_key.as_ref() {
        Key::Named(NamedKey::Backspace) => KeyCode::Backspace,
        Key::Named(NamedKey::Tab) => KeyCode::Tab,
        Key::Named(NamedKey::Enter) => KeyCode::Return,
        Key::Named(NamedKey::Shift) => KeyCode::Shift,
        Key::Named(NamedKey::Control) => KeyCode::Control,
        Key::Named(NamedKey::Alt) => KeyCode::Alt,
        Key::Named(NamedKey::CapsLock) => KeyCode::CapsLock,
        Key::Named(NamedKey::Escape) => KeyCode::Escape,
        Key::Named(NamedKey::Space) => KeyCode::Space,
        Key::Character("0") if event.location == KeyLocation::Numpad => KeyCode::Numpad0,
        Key::Character("1") if event.location == KeyLocation::Numpad => KeyCode::Numpad1,
        Key::Character("2") if event.location == KeyLocation::Numpad => KeyCode::Numpad2,
        Key::Character("3") if event.location == KeyLocation::Numpad => KeyCode::Numpad3,
        Key::Character("4") if event.location == KeyLocation::Numpad => KeyCode::Numpad4,
        Key::Character("5") if event.location == KeyLocation::Numpad => KeyCode::Numpad5,
        Key::Character("6") if event.location == KeyLocation::Numpad => KeyCode::Numpad6,
        Key::Character("7") if event.location == KeyLocation::Numpad => KeyCode::Numpad7,
        Key::Character("8") if event.location == KeyLocation::Numpad => KeyCode::Numpad8,
        Key::Character("9") if event.location == KeyLocation::Numpad => KeyCode::Numpad9,
        Key::Character("0") => KeyCode::Key0,
        Key::Character("1") => KeyCode::Key1,
        Key::Character("2") => KeyCode::Key2,
        Key::Character("3") => KeyCode::Key3,
        Key::Character("4") => KeyCode::Key4,
        Key::Character("5") => KeyCode::Key5,
        Key::Character("6") => KeyCode::Key6,
        Key::Character("7") => KeyCode::Key7,
        Key::Character("8") => KeyCode::Key8,
        Key::Character("9") => KeyCode::Key9,
        Key::Character("a") => KeyCode::A,
        Key::Character("b") => KeyCode::B,
        Key::Character("c") => KeyCode::C,
        Key::Character("d") => KeyCode::D,
        Key::Character("e") => KeyCode::E,
        Key::Character("f") => KeyCode::F,
        Key::Character("g") => KeyCode::G,
        Key::Character("h") => KeyCode::H,
        Key::Character("i") => KeyCode::I,
        Key::Character("j") => KeyCode::J,
        Key::Character("k") => KeyCode::K,
        Key::Character("l") => KeyCode::L,
        Key::Character("m") => KeyCode::M,
        Key::Character("n") => KeyCode::N,
        Key::Character("o") => KeyCode::O,
        Key::Character("p") => KeyCode::P,
        Key::Character("q") => KeyCode::Q,
        Key::Character("r") => KeyCode::R,
        Key::Character("s") => KeyCode::S,
        Key::Character("t") => KeyCode::T,
        Key::Character("u") => KeyCode::U,
        Key::Character("v") => KeyCode::V,
        Key::Character("w") => KeyCode::W,
        Key::Character("x") => KeyCode::X,
        Key::Character("y") => KeyCode::Y,
        Key::Character("z") => KeyCode::Z,
        Key::Character(";") => KeyCode::Semicolon,
        Key::Character("=") => KeyCode::Equals,
        Key::Character(",") => KeyCode::Comma,
        Key::Character("-") if event.location == KeyLocation::Numpad => KeyCode::NumpadMinus,
        Key::Character("-") => KeyCode::Minus,
        Key::Character(".") if event.location == KeyLocation::Numpad => KeyCode::NumpadPeriod,
        Key::Character(".") => KeyCode::Period,
        Key::Character("/") if event.location == KeyLocation::Numpad => KeyCode::NumpadSlash,
        Key::Character("/") => KeyCode::Slash,
        Key::Character("`") => KeyCode::Grave,
        Key::Character("[") => KeyCode::LBracket,
        Key::Character("\\") => KeyCode::Backslash,
        Key::Character("]") => KeyCode::RBracket,
        Key::Character("'") => KeyCode::Apostrophe,
        Key::Character("*") => KeyCode::Multiply,
        Key::Character("+") => KeyCode::Plus,
        Key::Named(NamedKey::PageUp) => KeyCode::PgUp,
        Key::Named(NamedKey::PageDown) => KeyCode::PgDown,
        Key::Named(NamedKey::End) => KeyCode::End,
        Key::Named(NamedKey::Home) => KeyCode::Home,
        Key::Named(NamedKey::ArrowLeft) => KeyCode::Left,
        Key::Named(NamedKey::ArrowUp) => KeyCode::Up,
        Key::Named(NamedKey::ArrowRight) => KeyCode::Right,
        Key::Named(NamedKey::ArrowDown) => KeyCode::Down,
        Key::Named(NamedKey::Insert) => KeyCode::Insert,
        Key::Named(NamedKey::Delete) => KeyCode::Delete,
        Key::Named(NamedKey::Pause) => KeyCode::Pause,
        Key::Named(NamedKey::NumLock) => KeyCode::NumLock,
        Key::Named(NamedKey::ScrollLock) => KeyCode::ScrollLock,
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
        _ => KeyCode::Unknown,
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

pub fn get_screen_size(event_loop: &EventLoop<RuffleEvent>) -> PhysicalSize<u32> {
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;

    for monitor in event_loop.available_monitors() {
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

fn actually_pick_file(dir: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = FileDialog::new()
        .add_filter("Flash Files", &["swf", "spl", "ruf"])
        .add_filter("All Files", &["*"])
        .set_title("Load a Flash File");

    if let Some(dir) = dir {
        dialog = dialog.set_directory(dir);
    }

    dialog.pick_file()
}

// [NA] Horrible hacky workaround for https://github.com/rust-windowing/winit/issues/2291
// We only need the workaround from within UI code, not when executing custom events
// The workaround causes Ruffle to show as "not responding" on windows, so we don't use it if we don't need to
#[cfg(windows)]
pub fn pick_file(in_ui: bool, path: Option<PathBuf>) -> Option<PathBuf> {
    if in_ui {
        std::thread::spawn(move || actually_pick_file(path))
            .join()
            .ok()
            .flatten()
    } else {
        actually_pick_file(path)
    }
}

#[cfg(not(windows))]
pub fn pick_file(_in_ui: bool, path: Option<PathBuf>) -> Option<PathBuf> {
    actually_pick_file(path)
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
