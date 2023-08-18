use crate::custom_event::RuffleEvent;
use anyhow::{anyhow, Error};
use rfd::FileDialog;
use ruffle_core::events::{KeyCode, TextControlCode};
use std::path::{Path, PathBuf};
use url::Url;
use winit::dpi::PhysicalSize;
use winit::keyboard::{Key, ModifiersState};

use winit::event_loop::EventLoop;

/// Converts a `Key` and `ModifiersState` to a Ruffle `TextControlCode`.
/// Returns `None` if there is no match.
/// TODO: Handle Ctrl+Arrows and Home/End keys
pub fn winit_to_ruffle_text_control(
    key: Key,
    modifiers: ModifiersState,
) -> Option<TextControlCode> {
    let shift = modifiers.contains(ModifiersState::SHIFT);
    let ctrl_cmd = modifiers.contains(ModifiersState::CONTROL)
        || (modifiers.contains(ModifiersState::SUPER) && cfg!(target_os = "macos"));
    if ctrl_cmd {
        match key {
            Key::Character(A) => Some(TextControlCode::SelectAll),
            Key::Character(C) => Some(TextControlCode::Copy),
            Key::Character(V) => Some(TextControlCode::Paste),
            Key::Character(X) => Some(TextControlCode::Cut),
            _ => None,
        }
    } else {
        match key {
            /*Key::Backspace => Some(TextControlCode::Backspace),
            Key::Delete => Some(TextControlCode::Delete),
            Key::ArrowLeft => {
                if shift {
                    Some(TextControlCode::SelectLeft)
                } else {
                    Some(TextControlCode::MoveLeft)
                }
            }
            Key::ArrowRight => {
                if shift {
                    Some(TextControlCode::SelectRight)
                } else {
                    Some(TextControlCode::MoveRight)
                }
            }*/
            _ => None,
        }
    }
}

/// Convert a winit `Key` into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
pub fn winit_to_ruffle_key_code(key_code: Key) -> KeyCode {
    match key_code {
        /*Key::Backspace => KeyCode::Backspace,
        Key::Tab => KeyCode::Tab,
        Key::Enter => KeyCode::Return,
        Key::Shift => KeyCode::Shift,
        Key::Control => KeyCode::Control,
        Key::Alt => KeyCode::Alt,
        Key::CapsLock => KeyCode::CapsLock,
        Key::Escape => KeyCode::Escape,
        Key::Space => KeyCode::Space,
        Key::Key0 => KeyCode::Key0,
        Key::Key1 => KeyCode::Key1,
        Key::Key2 => KeyCode::Key2,
        Key::Key3 => KeyCode::Key3,
        Key::Key4 => KeyCode::Key4,
        Key::Key5 => KeyCode::Key5,
        Key::Key6 => KeyCode::Key6,
        Key::Key7 => KeyCode::Key7,
        Key::Key8 => KeyCode::Key8,
        Key::Key9 => KeyCode::Key9,*/
        Key::Character(A) => KeyCode::A,
        Key::Character(B) => KeyCode::B,
        Key::Character(C) => KeyCode::C,
        Key::Character(D) => KeyCode::D,
        Key::Character(E) => KeyCode::E,
        Key::Character(F) => KeyCode::F,
        Key::Character(G) => KeyCode::G,
        Key::Character(H) => KeyCode::H,
        Key::Character(I) => KeyCode::I,
        Key::Character(J) => KeyCode::J,
        Key::Character(K) => KeyCode::K,
        Key::Character(L) => KeyCode::L,
        Key::Character(M) => KeyCode::M,
        Key::Character(N) => KeyCode::N,
        Key::Character(O) => KeyCode::O,
        Key::Character(P) => KeyCode::P,
        Key::Character(Q) => KeyCode::Q,
        Key::Character(R) => KeyCode::R,
        Key::Character(S) => KeyCode::S,
        Key::Character(T) => KeyCode::T,
        Key::Character(U) => KeyCode::U,
        Key::Character(V) => KeyCode::V,
        Key::Character(W) => KeyCode::W,
        Key::Character(X) => KeyCode::X,
        Key::Character(Y) => KeyCode::Y,
        Key::Character(Z) => KeyCode::Z, /*
        Key::Semicolon => KeyCode::Semicolon,
        Key::Equals => KeyCode::Equals,
        Key::Comma => KeyCode::Comma,
        Key::Minus => KeyCode::Minus,
        Key::Period => KeyCode::Period,
        Key::Slash => KeyCode::Slash,
        Key::Grave => KeyCode::Grave,
        Key::LBracket => KeyCode::LBracket,
        Key::Backslash => KeyCode::Backslash,
        Key::RBracket => KeyCode::RBracket,
        Key::Apostrophe => KeyCode::Apostrophe,
        Key::Numpad0 => KeyCode::Numpad0,
        Key::Numpad1 => KeyCode::Numpad1,
        Key::Numpad2 => KeyCode::Numpad2,
        Key::Numpad3 => KeyCode::Numpad3,
        Key::Numpad4 => KeyCode::Numpad4,
        Key::Numpad5 => KeyCode::Numpad5,
        Key::Numpad6 => KeyCode::Numpad6,
        Key::Numpad7 => KeyCode::Numpad7,
        Key::Numpad8 => KeyCode::Numpad8,
        Key::Numpad9 => KeyCode::Numpad9,
        Key::NumpadMultiply => KeyCode::Multiply,
        Key::NumpadAdd => KeyCode::Plus,
        Key::NumpadSubtract => KeyCode::NumpadMinus,
        Key::NumpadDecimal => KeyCode::NumpadPeriod,
        Key::NumpadDivide => KeyCode::NumpadSlash,
        Key::PageUp => KeyCode::PgUp,
        Key::PageDown => KeyCode::PgDown,
        Key::End => KeyCode::End,
        Key::Home => KeyCode::Home,
        Key::ArrowLeft => KeyCode::Left,
        Key::ArrowUp => KeyCode::Up,
        Key::ArrowRight => KeyCode::Right,
        Key::ArrowDown => KeyCode::Down,
        Key::Insert => KeyCode::Insert,
        Key::Delete => KeyCode::Delete,
        Key::Pause => KeyCode::Pause,
        Key::ScrollLock => KeyCode::ScrollLock,
        Key::F1 => KeyCode::F1,
        Key::F2 => KeyCode::F2,
        Key::F3 => KeyCode::F3,
        Key::F4 => KeyCode::F4,
        Key::F5 => KeyCode::F5,
        Key::F6 => KeyCode::F6,
        Key::F7 => KeyCode::F7,
        Key::F8 => KeyCode::F8,
        Key::F9 => KeyCode::F9,
        Key::F10 => KeyCode::F10,
        Key::F11 => KeyCode::F11,
        Key::F12 => KeyCode::F12,*/
        _ => KeyCode::Unknown,
    }
}

/// Return a character for the given key code and shift state.
pub fn winit_key_to_char(key_code: Key, is_shift_down: bool) -> Option<char> {
    // We need to know the character that a keypress outputs for both key down and key up events,
    // but the winit keyboard API does not provide a way to do this (winit/#753).
    // CharacterReceived events are insufficent because they only fire on key down, not on key up.
    // This is a half-measure to map from keyboard keys back to a character, but does will not work fully
    // for international layouts.
    Some(match (key_code, is_shift_down) {
        /*(Key::Space, _) => ' ',
                (Key::Key0, _) => '0',
                (Key::Key1, _) => '1',
                (Key::Key2, _) => '2',
                (Key::Key3, _) => '3',
                (Key::Key4, _) => '4',
                (Key::Key5, _) => '5',
                (Key::Key6, _) => '6',
                (Key::Key7, _) => '7',
                (Key::Key8, _) => '8',
                (Key::Key9, _) => '9',
                (Key::A, false) => 'a',
                (Key::A, true) => 'A',
                (Key::B, false) => 'b',
                (Key::B, true) => 'B',
                (Key::C, false) => 'c',
                (Key::C, true) => 'C',
                (Key::D, false) => 'd',
                (Key::D, true) => 'D',
                (Key::E, false) => 'e',
                (Key::E, true) => 'E',
                (Key::F, false) => 'f',
                (Key::F, true) => 'F',
                (Key::G, false) => 'g',
                (Key::G, true) => 'G',
                (Key::H, false) => 'h',
                (Key::H, true) => 'H',
                (Key::I, false) => 'i',
                (Key::I, true) => 'I',
                (Key::J, false) => 'j',
                (Key::J, true) => 'J',
                (Key::K, false) => 'k',
                (Key::K, true) => 'K',
                (Key::L, false) => 'l',
                (Key::L, true) => 'L',
                (Key::M, false) => 'm',
                (Key::M, true) => 'M',
                (Key::N, false) => 'n',
                (Key::N, true) => 'N',
                (Key::O, false) => 'o',
                (Key::O, true) => 'O',
                (Key::P, false) => 'p',
                (Key::P, true) => 'P',
                (Key::Q, false) => 'q',
                (Key::Q, true) => 'Q',
                (Key::R, false) => 'r',
                (Key::R, true) => 'R',
                (Key::S, false) => 's',
                (Key::S, true) => 'S',
                (Key::T, false) => 't',
                (Key::T, true) => 'T',
                (Key::U, false) => 'u',
                (Key::U, true) => 'U',
                (Key::V, false) => 'v',
                (Key::V, true) => 'V',
                (Key::W, false) => 'w',
                (Key::W, true) => 'W',
                (Key::X, false) => 'x',
                (Key::X, true) => 'X',
                (Key::Y, false) => 'y',
                (Key::Y, true) => 'Y',
                (Key::Z, false) => 'z',
                (Key::Z, true) => 'Z',

                (Key::Semicolon, false) => ';',
                (Key::Semicolon, true) => ':',
                (Key::Equals, false) => '=',
                (Key::Equals, true) => '+',
                (Key::Comma, false) => ',',
                (Key::Comma, true) => '<',
                (Key::Minus, false) => '-',
                (Key::Minus, true) => '_',
                (Key::Period, false) => '.',
                (Key::Period, true) => '>',
                (Key::Slash, false) => '/',
                (Key::Slash, true) => '?',
                (Key::Grave, false) => '`',
                (Key::Grave, true) => '~',
                (Key::LBracket, false) => '[',
                (Key::LBracket, true) => '{',
                (Key::Backslash, false) => '\\',
                (Key::Backslash, true) => '|',
                (Key::RBracket, false) => ']',
                (Key::RBracket, true) => '}',
                (Key::Apostrophe, false) => '\'',
                (Key::Apostrophe, true) => '"',
                (Key::NumpadMultiply, _) => '*',
                (Key::NumpadAdd, _) => '+',
                (Key::NumpadSubtract, _) => '-',
                (Key::NumpadDecimal, _) => '.',
                (Key::NumpadDivide, _) => '/',

                (Key::Numpad0, false) => '0',
                (Key::Numpad1, false) => '1',
                (Key::Numpad2, false) => '2',
                (Key::Numpad3, false) => '3',
                (Key::Numpad4, false) => '4',
                (Key::Numpad5, false) => '5',
                (Key::Numpad6, false) => '6',
                (Key::Numpad7, false) => '7',
                (Key::Numpad8, false) => '8',
                (Key::Numpad9, false) => '9',
                (Key::NumpadEnter, _) => '\r',

                (Key::Tab, _) => '\t',
                (Key::Return, _) => '\r',
                (Key::Back, _) => '\u{0008}',
        */
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
