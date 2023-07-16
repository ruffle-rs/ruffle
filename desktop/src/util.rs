use crate::custom_event::RuffleEvent;
use anyhow::{anyhow, Error};
use rfd::FileDialog;
use ruffle_core::events::{KeyCode, TextControlCode};
use std::path::{Path, PathBuf};
use url::Url;
use winit::dpi::PhysicalSize;
use winit::event::{ModifiersState, VirtualKeyCode};
use winit::event_loop::EventLoop;

/// Converts a `VirtualKeyCode` and `ModifiersState` to a Ruffle `TextControlCode`.
/// Returns `None` if there is no match.
/// TODO: Handle Ctrl+Arrows and Home/End keys
pub fn winit_to_ruffle_text_control(
    key: VirtualKeyCode,
    modifiers: ModifiersState,
) -> Option<TextControlCode> {
    let shift = modifiers.contains(ModifiersState::SHIFT);
    let ctrl_cmd = modifiers.contains(ModifiersState::CTRL)
        || (modifiers.contains(ModifiersState::LOGO) && cfg!(target_os = "macos"));
    if ctrl_cmd {
        match key {
            VirtualKeyCode::A => Some(TextControlCode::SelectAll),
            VirtualKeyCode::C => Some(TextControlCode::Copy),
            VirtualKeyCode::V => Some(TextControlCode::Paste),
            VirtualKeyCode::X => Some(TextControlCode::Cut),
            _ => None,
        }
    } else {
        match key {
            VirtualKeyCode::Back => Some(TextControlCode::Backspace),
            VirtualKeyCode::Delete => Some(TextControlCode::Delete),
            VirtualKeyCode::Left => {
                if shift {
                    Some(TextControlCode::SelectLeft)
                } else {
                    Some(TextControlCode::MoveLeft)
                }
            }
            VirtualKeyCode::Right => {
                if shift {
                    Some(TextControlCode::SelectRight)
                } else {
                    Some(TextControlCode::MoveRight)
                }
            }
            _ => None,
        }
    }
}

/// Convert a winit `VirtualKeyCode` into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
pub fn winit_to_ruffle_key_code(key_code: VirtualKeyCode) -> KeyCode {
    match key_code {
        VirtualKeyCode::Back => KeyCode::Backspace,
        VirtualKeyCode::Tab => KeyCode::Tab,
        VirtualKeyCode::Return => KeyCode::Return,
        VirtualKeyCode::LShift | VirtualKeyCode::RShift => KeyCode::Shift,
        VirtualKeyCode::LControl | VirtualKeyCode::RControl => KeyCode::Control,
        VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => KeyCode::Alt,
        VirtualKeyCode::Capital => KeyCode::CapsLock,
        VirtualKeyCode::Escape => KeyCode::Escape,
        VirtualKeyCode::Space => KeyCode::Space,
        VirtualKeyCode::Key0 => KeyCode::Key0,
        VirtualKeyCode::Key1 => KeyCode::Key1,
        VirtualKeyCode::Key2 => KeyCode::Key2,
        VirtualKeyCode::Key3 => KeyCode::Key3,
        VirtualKeyCode::Key4 => KeyCode::Key4,
        VirtualKeyCode::Key5 => KeyCode::Key5,
        VirtualKeyCode::Key6 => KeyCode::Key6,
        VirtualKeyCode::Key7 => KeyCode::Key7,
        VirtualKeyCode::Key8 => KeyCode::Key8,
        VirtualKeyCode::Key9 => KeyCode::Key9,
        VirtualKeyCode::A => KeyCode::A,
        VirtualKeyCode::B => KeyCode::B,
        VirtualKeyCode::C => KeyCode::C,
        VirtualKeyCode::D => KeyCode::D,
        VirtualKeyCode::E => KeyCode::E,
        VirtualKeyCode::F => KeyCode::F,
        VirtualKeyCode::G => KeyCode::G,
        VirtualKeyCode::H => KeyCode::H,
        VirtualKeyCode::I => KeyCode::I,
        VirtualKeyCode::J => KeyCode::J,
        VirtualKeyCode::K => KeyCode::K,
        VirtualKeyCode::L => KeyCode::L,
        VirtualKeyCode::M => KeyCode::M,
        VirtualKeyCode::N => KeyCode::N,
        VirtualKeyCode::O => KeyCode::O,
        VirtualKeyCode::P => KeyCode::P,
        VirtualKeyCode::Q => KeyCode::Q,
        VirtualKeyCode::R => KeyCode::R,
        VirtualKeyCode::S => KeyCode::S,
        VirtualKeyCode::T => KeyCode::T,
        VirtualKeyCode::U => KeyCode::U,
        VirtualKeyCode::V => KeyCode::V,
        VirtualKeyCode::W => KeyCode::W,
        VirtualKeyCode::X => KeyCode::X,
        VirtualKeyCode::Y => KeyCode::Y,
        VirtualKeyCode::Z => KeyCode::Z,
        VirtualKeyCode::Semicolon => KeyCode::Semicolon,
        VirtualKeyCode::Equals => KeyCode::Equals,
        VirtualKeyCode::Comma => KeyCode::Comma,
        VirtualKeyCode::Minus => KeyCode::Minus,
        VirtualKeyCode::Period => KeyCode::Period,
        VirtualKeyCode::Slash => KeyCode::Slash,
        VirtualKeyCode::Grave => KeyCode::Grave,
        VirtualKeyCode::LBracket => KeyCode::LBracket,
        VirtualKeyCode::Backslash => KeyCode::Backslash,
        VirtualKeyCode::RBracket => KeyCode::RBracket,
        VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
        VirtualKeyCode::Numpad0 => KeyCode::Numpad0,
        VirtualKeyCode::Numpad1 => KeyCode::Numpad1,
        VirtualKeyCode::Numpad2 => KeyCode::Numpad2,
        VirtualKeyCode::Numpad3 => KeyCode::Numpad3,
        VirtualKeyCode::Numpad4 => KeyCode::Numpad4,
        VirtualKeyCode::Numpad5 => KeyCode::Numpad5,
        VirtualKeyCode::Numpad6 => KeyCode::Numpad6,
        VirtualKeyCode::Numpad7 => KeyCode::Numpad7,
        VirtualKeyCode::Numpad8 => KeyCode::Numpad8,
        VirtualKeyCode::Numpad9 => KeyCode::Numpad9,
        VirtualKeyCode::NumpadMultiply => KeyCode::Multiply,
        VirtualKeyCode::NumpadAdd => KeyCode::Plus,
        VirtualKeyCode::NumpadSubtract => KeyCode::NumpadMinus,
        VirtualKeyCode::NumpadDecimal => KeyCode::NumpadPeriod,
        VirtualKeyCode::NumpadDivide => KeyCode::NumpadSlash,
        VirtualKeyCode::PageUp => KeyCode::PgUp,
        VirtualKeyCode::PageDown => KeyCode::PgDown,
        VirtualKeyCode::End => KeyCode::End,
        VirtualKeyCode::Home => KeyCode::Home,
        VirtualKeyCode::Left => KeyCode::Left,
        VirtualKeyCode::Up => KeyCode::Up,
        VirtualKeyCode::Right => KeyCode::Right,
        VirtualKeyCode::Down => KeyCode::Down,
        VirtualKeyCode::Insert => KeyCode::Insert,
        VirtualKeyCode::Delete => KeyCode::Delete,
        VirtualKeyCode::Pause => KeyCode::Pause,
        VirtualKeyCode::Scroll => KeyCode::ScrollLock,
        VirtualKeyCode::F1 => KeyCode::F1,
        VirtualKeyCode::F2 => KeyCode::F2,
        VirtualKeyCode::F3 => KeyCode::F3,
        VirtualKeyCode::F4 => KeyCode::F4,
        VirtualKeyCode::F5 => KeyCode::F5,
        VirtualKeyCode::F6 => KeyCode::F6,
        VirtualKeyCode::F7 => KeyCode::F7,
        VirtualKeyCode::F8 => KeyCode::F8,
        VirtualKeyCode::F9 => KeyCode::F9,
        VirtualKeyCode::F10 => KeyCode::F10,
        VirtualKeyCode::F11 => KeyCode::F11,
        VirtualKeyCode::F12 => KeyCode::F12,
        _ => KeyCode::Unknown,
    }
}

/// Return a character for the given key code and shift state.
pub fn winit_key_to_char(key_code: VirtualKeyCode, is_shift_down: bool) -> Option<char> {
    // We need to know the character that a keypress outputs for both key down and key up events,
    // but the winit keyboard API does not provide a way to do this (winit/#753).
    // CharacterReceived events are insufficent because they only fire on key down, not on key up.
    // This is a half-measure to map from keyboard keys back to a character, but does will not work fully
    // for international layouts.
    Some(match (key_code, is_shift_down) {
        (VirtualKeyCode::Space, _) => ' ',
        (VirtualKeyCode::Key0, _) => '0',
        (VirtualKeyCode::Key1, _) => '1',
        (VirtualKeyCode::Key2, _) => '2',
        (VirtualKeyCode::Key3, _) => '3',
        (VirtualKeyCode::Key4, _) => '4',
        (VirtualKeyCode::Key5, _) => '5',
        (VirtualKeyCode::Key6, _) => '6',
        (VirtualKeyCode::Key7, _) => '7',
        (VirtualKeyCode::Key8, _) => '8',
        (VirtualKeyCode::Key9, _) => '9',
        (VirtualKeyCode::A, false) => 'a',
        (VirtualKeyCode::A, true) => 'A',
        (VirtualKeyCode::B, false) => 'b',
        (VirtualKeyCode::B, true) => 'B',
        (VirtualKeyCode::C, false) => 'c',
        (VirtualKeyCode::C, true) => 'C',
        (VirtualKeyCode::D, false) => 'd',
        (VirtualKeyCode::D, true) => 'D',
        (VirtualKeyCode::E, false) => 'e',
        (VirtualKeyCode::E, true) => 'E',
        (VirtualKeyCode::F, false) => 'f',
        (VirtualKeyCode::F, true) => 'F',
        (VirtualKeyCode::G, false) => 'g',
        (VirtualKeyCode::G, true) => 'G',
        (VirtualKeyCode::H, false) => 'h',
        (VirtualKeyCode::H, true) => 'H',
        (VirtualKeyCode::I, false) => 'i',
        (VirtualKeyCode::I, true) => 'I',
        (VirtualKeyCode::J, false) => 'j',
        (VirtualKeyCode::J, true) => 'J',
        (VirtualKeyCode::K, false) => 'k',
        (VirtualKeyCode::K, true) => 'K',
        (VirtualKeyCode::L, false) => 'l',
        (VirtualKeyCode::L, true) => 'L',
        (VirtualKeyCode::M, false) => 'm',
        (VirtualKeyCode::M, true) => 'M',
        (VirtualKeyCode::N, false) => 'n',
        (VirtualKeyCode::N, true) => 'N',
        (VirtualKeyCode::O, false) => 'o',
        (VirtualKeyCode::O, true) => 'O',
        (VirtualKeyCode::P, false) => 'p',
        (VirtualKeyCode::P, true) => 'P',
        (VirtualKeyCode::Q, false) => 'q',
        (VirtualKeyCode::Q, true) => 'Q',
        (VirtualKeyCode::R, false) => 'r',
        (VirtualKeyCode::R, true) => 'R',
        (VirtualKeyCode::S, false) => 's',
        (VirtualKeyCode::S, true) => 'S',
        (VirtualKeyCode::T, false) => 't',
        (VirtualKeyCode::T, true) => 'T',
        (VirtualKeyCode::U, false) => 'u',
        (VirtualKeyCode::U, true) => 'U',
        (VirtualKeyCode::V, false) => 'v',
        (VirtualKeyCode::V, true) => 'V',
        (VirtualKeyCode::W, false) => 'w',
        (VirtualKeyCode::W, true) => 'W',
        (VirtualKeyCode::X, false) => 'x',
        (VirtualKeyCode::X, true) => 'X',
        (VirtualKeyCode::Y, false) => 'y',
        (VirtualKeyCode::Y, true) => 'Y',
        (VirtualKeyCode::Z, false) => 'z',
        (VirtualKeyCode::Z, true) => 'Z',

        (VirtualKeyCode::Semicolon, false) => ';',
        (VirtualKeyCode::Semicolon, true) => ':',
        (VirtualKeyCode::Equals, false) => '=',
        (VirtualKeyCode::Equals, true) => '+',
        (VirtualKeyCode::Comma, false) => ',',
        (VirtualKeyCode::Comma, true) => '<',
        (VirtualKeyCode::Minus, false) => '-',
        (VirtualKeyCode::Minus, true) => '_',
        (VirtualKeyCode::Period, false) => '.',
        (VirtualKeyCode::Period, true) => '>',
        (VirtualKeyCode::Slash, false) => '/',
        (VirtualKeyCode::Slash, true) => '?',
        (VirtualKeyCode::Grave, false) => '`',
        (VirtualKeyCode::Grave, true) => '~',
        (VirtualKeyCode::LBracket, false) => '[',
        (VirtualKeyCode::LBracket, true) => '{',
        (VirtualKeyCode::Backslash, false) => '\\',
        (VirtualKeyCode::Backslash, true) => '|',
        (VirtualKeyCode::RBracket, false) => ']',
        (VirtualKeyCode::RBracket, true) => '}',
        (VirtualKeyCode::Apostrophe, false) => '\'',
        (VirtualKeyCode::Apostrophe, true) => '"',
        (VirtualKeyCode::NumpadMultiply, _) => '*',
        (VirtualKeyCode::NumpadAdd, _) => '+',
        (VirtualKeyCode::NumpadSubtract, _) => '-',
        (VirtualKeyCode::NumpadDecimal, _) => '.',
        (VirtualKeyCode::NumpadDivide, _) => '/',

        (VirtualKeyCode::Numpad0, false) => '0',
        (VirtualKeyCode::Numpad1, false) => '1',
        (VirtualKeyCode::Numpad2, false) => '2',
        (VirtualKeyCode::Numpad3, false) => '3',
        (VirtualKeyCode::Numpad4, false) => '4',
        (VirtualKeyCode::Numpad5, false) => '5',
        (VirtualKeyCode::Numpad6, false) => '6',
        (VirtualKeyCode::Numpad7, false) => '7',
        (VirtualKeyCode::Numpad8, false) => '8',
        (VirtualKeyCode::Numpad9, false) => '9',
        (VirtualKeyCode::NumpadEnter, _) => '\r',

        (VirtualKeyCode::Tab, _) => '\t',
        (VirtualKeyCode::Return, _) => '\r',
        (VirtualKeyCode::Back, _) => '\u{0008}',

        _ => return None,
    })
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
        .add_filter("Flash Files", &["swf", "spl"])
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
    let report = instance.generate_report();

    #[allow(unused_mut)]
    let mut backend = None;
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        backend = backend.or(report.vulkan).or(report.gl);
    }
    #[cfg(windows)]
    {
        backend = backend.or(report.dx12).or(report.dx11);
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        backend = backend.or(report.metal);
    }

    if let Some(stats) = backend {
        tracy.plot(BIND_GROUPS, stats.bind_groups.num_occupied as f64);
        tracy.plot(BUFFERS, stats.buffers.num_occupied as f64);
        tracy.plot(TEXTURES, stats.textures.num_occupied as f64);
        tracy.plot(TEXTURE_VIEWS, stats.texture_views.num_occupied as f64);
    }

    tracy.frame_mark();
}
