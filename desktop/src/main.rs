// By default, Windows creates an additional console window for our program.
//
//
// This is silently ignored on non-windows systems.
// See https://docs.microsoft.com/en-us/cpp/build/reference/subsystem?view=msvc-160 for details.
#![windows_subsystem = "windows"]

mod audio;
mod custom_event;
mod executor;
mod navigator;
mod storage;
mod task;
mod ui;

use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use clap::Parser;
use isahc::{config::RedirectPolicy, prelude::*, HttpClient};
use rfd::FileDialog;
use ruffle_core::{
    config::Letterbox, events::KeyCode, tag_utils::SwfMovie, Player, PlayerBuilder, PlayerEvent,
    StageDisplayState,
};
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use url::Url;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size};
use winit::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, Window, WindowBuilder};

#[derive(Parser, Debug)]
#[clap(
    name = "Ruffle",
    author,
    version = include_str!(concat!(env!("OUT_DIR"), "/version-info.txt")),
)]
struct Opt {
    /// Path to a Flash movie (SWF) to play.
    #[clap(name = "FILE", parse(from_os_str))]
    input_path: Option<PathBuf>,

    /// A "flashvars" parameter to provide to the movie.
    /// This can be repeated multiple times, for example -Pkey=value -Pfoo=bar.
    #[clap(short = 'P', number_of_values = 1, multiple_occurrences = true)]
    parameters: Vec<String>,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(long, short, default_value = "default", arg_enum)]
    graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, default_value = "high", arg_enum)]
    power: PowerPreference,

    /// Width of window in pixels.
    #[clap(long, display_order = 1)]
    width: Option<f64>,

    /// Height of window in pixels.
    #[clap(long, display_order = 2)]
    height: Option<f64>,

    /// Location to store a wgpu trace output
    #[clap(long, parse(from_os_str))]
    #[cfg(feature = "render_trace")]
    trace_path: Option<PathBuf>,

    /// Proxy to use when loading movies via URL.
    #[clap(long)]
    proxy: Option<Url>,

    /// Replace all embedded HTTP URLs with HTTPS.
    #[clap(long, takes_value = false)]
    upgrade_to_https: bool,

    /// Start application in fullscreen.
    #[clap(long, takes_value = false)]
    fullscreen: bool,

    #[clap(long, takes_value = false)]
    timedemo: bool,

    #[clap(long, takes_value = false)]
    dont_warn_on_unsupported_content: bool,
}

#[cfg(feature = "render_trace")]
fn trace_path(opt: &Opt) -> Option<&Path> {
    if let Some(path) = &opt.trace_path {
        let _ = std::fs::create_dir_all(path);
        Some(path)
    } else {
        None
    }
}

#[cfg(not(feature = "render_trace"))]
fn trace_path(_opt: &Opt) -> Option<&Path> {
    None
}

fn parse_url(path: &Path) -> Result<Url, Box<dyn std::error::Error>> {
    Ok(if path.exists() {
        let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        Url::from_file_path(absolute_path)
            .map_err(|_| "Path must be absolute and cannot be a URL")?
    } else {
        Url::parse(path.to_str().unwrap_or_default())
            .ok()
            .filter(|url| url.host().is_some())
            .ok_or("Input path is not a file and could not be parsed as a URL.")?
    })
}

fn parse_parameters(opt: &Opt) -> impl '_ + Iterator<Item = (String, String)> {
    opt.parameters.iter().map(|parameter| {
        let mut split = parameter.splitn(2, '=');
        if let (Some(key), Some(value)) = (split.next(), split.next()) {
            (key.to_owned(), value.to_owned())
        } else {
            (parameter.clone(), "".to_string())
        }
    })
}

fn pick_file() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter(".swf", &["swf"])
        .set_title("Load a Flash File")
        .pick_file()
}

fn load_movie(url: &Url, opt: &Opt) -> Result<SwfMovie, Box<dyn std::error::Error>> {
    let mut movie = if url.scheme() == "file" {
        SwfMovie::from_path(url.to_file_path().unwrap(), None)?
    } else {
        let proxy = opt.proxy.as_ref().and_then(|url| url.as_str().parse().ok());
        let builder = HttpClient::builder()
            .proxy(proxy)
            .redirect_policy(RedirectPolicy::Follow);
        let client = builder.build()?;
        let response = client.get(url.to_string())?;
        let mut buffer: Vec<u8> = Vec::new();
        response.into_body().read_to_end(&mut buffer)?;

        SwfMovie::from_data(&buffer, Some(url.to_string()), None)?
    };

    movie.append_parameters(parse_parameters(opt));

    Ok(movie)
}

struct App {
    opt: Opt,
    window: Rc<Window>,
    event_loop: EventLoop<RuffleEvent>,
    executor: Arc<Mutex<GlutinAsyncExecutor>>,
    player: Arc<Mutex<Player>>,
}

impl App {
    fn new(opt: Opt) -> Result<Self, Box<dyn std::error::Error>> {
        let path = match opt.input_path.as_ref() {
            Some(path) => Some(std::borrow::Cow::Borrowed(path)),
            None => pick_file().map(std::borrow::Cow::Owned),
        };
        let movie_url = if let Some(path) = path {
            Some(parse_url(&path)?)
        } else {
            shutdown(&Ok(()));
            std::process::exit(0);
        };

        let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
        let icon = Icon::from_rgba(icon_bytes.to_vec(), 32, 32)?;

        let event_loop: EventLoop<RuffleEvent> = EventLoop::with_user_event();

        let title = if let Some(movie_url) = &movie_url {
            let filename = movie_url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or_else(|| movie_url.as_str());

            format!("Ruffle - {}", filename)
        } else {
            "Ruffle".into()
        };

        let window = WindowBuilder::new()
            .with_visible(false)
            .with_title(title)
            .with_window_icon(Some(icon))
            .with_max_inner_size(LogicalSize::new(i16::MAX, i16::MAX))
            .build(&event_loop)?;

        let mut builder = PlayerBuilder::new();

        match audio::CpalAudioBackend::new() {
            Ok(audio) => builder = builder.with_audio(audio),
            Err(e) => {
                log::error!("Unable to create audio device: {}", e);
            }
        };

        let (executor, channel) = GlutinAsyncExecutor::new(event_loop.create_proxy());
        let navigator = navigator::ExternalNavigatorBackend::new(
            movie_url.as_ref().unwrap().to_owned(),
            channel,
            event_loop.create_proxy(),
            opt.proxy.clone(),
            opt.upgrade_to_https,
        );

        let viewport_size = window.inner_size();
        let renderer = WgpuRenderBackend::for_window(
            &window,
            (viewport_size.width, viewport_size.height),
            opt.graphics.into(),
            opt.power.into(),
            trace_path(&opt),
        )?;

        let window = Rc::new(window);

        builder = builder
            .with_navigator(navigator)
            .with_renderer(renderer)
            .with_storage(storage::DiskStorageBackend::new())
            .with_ui(ui::DesktopUiBackend::new(window.clone()))
            .with_software_video()
            .with_autoplay(true)
            .with_letterbox(Letterbox::On)
            .with_warn_on_unsupported_content(!opt.dont_warn_on_unsupported_content)
            .with_fullscreen(opt.fullscreen);

        let player = builder.build();

        if let Some(movie_url) = &movie_url {
            let event_loop_proxy = event_loop.create_proxy();
            let on_metadata = move |swf_header: &ruffle_core::swf::HeaderExt| {
                let _ = event_loop_proxy.send_event(RuffleEvent::OnMetadata(swf_header.clone()));
            };

            player.lock().unwrap().fetch_root_movie(
                movie_url.as_str(),
                parse_parameters(&opt).collect(),
                Box::new(on_metadata),
            );
        }

        Ok(Self {
            opt,
            window,
            event_loop,
            executor,
            player,
        })
    }

    fn run(self) -> ! {
        let mut loaded = false;
        let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
        let mut time = Instant::now();
        let mut next_frame_time = Instant::now();
        let mut minimized = false;
        let mut fullscreen_down = false;

        // Poll UI events.
        self.event_loop
            .run(move |event, _window_target, control_flow| {
                // Handle fullscreen keyboard shortcuts: Alt+Return, Escape.
                if let winit::event::Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } = &event
                {
                    // Allow KeyboardInput.modifiers (ModifiersChanged event not functional yet).
                    #[allow(deprecated)]
                    match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Return),
                            modifiers,
                            ..
                        } if modifiers.alt() => {
                            if !fullscreen_down {
                                self.player.lock().unwrap().update(|uc| {
                                    uc.stage.toggle_display_state(uc);
                                });
                            }
                            fullscreen_down = true;
                            return;
                        }
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(VirtualKeyCode::Return),
                            ..
                        } if fullscreen_down => {
                            fullscreen_down = false;
                        }
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => self.player.lock().unwrap().update(|uc| {
                            uc.stage.set_display_state(uc, StageDisplayState::Normal);
                        }),
                        _ => (),
                    }
                }

                match event {
                    winit::event::Event::LoopDestroyed => {
                        self.player.lock().unwrap().flush_shared_objects();
                        shutdown(&Ok(()));
                        return;
                    }

                    // Core loop
                    winit::event::Event::MainEventsCleared if loaded => {
                        let new_time = Instant::now();
                        let dt = new_time.duration_since(time).as_micros();
                        if dt > 0 {
                            time = new_time;
                            let mut player_lock = self.player.lock().unwrap();
                            player_lock.tick(dt as f64 / 1000.0);
                            next_frame_time = new_time + player_lock.time_til_next_frame();
                            if player_lock.needs_render() {
                                self.window.request_redraw();
                            }
                        }
                    }

                    // Render
                    winit::event::Event::RedrawRequested(_) => {
                        // Don't render when minimized to avoid potential swap chain errors in `wgpu`.
                        if !minimized {
                            self.player.lock().unwrap().render();
                        }
                    }

                    winit::event::Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                            return;
                        }
                        WindowEvent::Resized(size) => {
                            // TODO: Change this when winit adds a `Window::minimzed` or `WindowEvent::Minimize`.
                            minimized = size.width == 0 && size.height == 0;

                            let viewport_scale_factor = self.window.scale_factor();
                            let mut player_lock = self.player.lock().unwrap();
                            player_lock.set_viewport_dimensions(
                                size.width,
                                size.height,
                                viewport_scale_factor,
                            );
                            player_lock
                                .renderer_mut()
                                .set_viewport_dimensions(size.width, size.height);
                            self.window.request_redraw();
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let mut player_lock = self.player.lock().unwrap();
                            mouse_pos = position;
                            let event = PlayerEvent::MouseMove {
                                x: position.x,
                                y: position.y,
                            };
                            player_lock.handle_event(event);
                            if player_lock.needs_render() {
                                self.window.request_redraw();
                            }
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            use ruffle_core::events::MouseButton as RuffleMouseButton;
                            let mut player_lock = self.player.lock().unwrap();
                            let x = mouse_pos.x;
                            let y = mouse_pos.y;
                            let button = match button {
                                MouseButton::Left => RuffleMouseButton::Left,
                                MouseButton::Right => RuffleMouseButton::Right,
                                MouseButton::Middle => RuffleMouseButton::Middle,
                                MouseButton::Other(_) => RuffleMouseButton::Unknown,
                            };
                            let event = match state {
                                ElementState::Pressed => PlayerEvent::MouseDown { x, y, button },
                                ElementState::Released => PlayerEvent::MouseUp { x, y, button },
                            };
                            player_lock.handle_event(event);
                            if player_lock.needs_render() {
                                self.window.request_redraw();
                            }
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            use ruffle_core::events::MouseWheelDelta;
                            let mut player_lock = self.player.lock().unwrap();
                            let delta = match delta {
                                MouseScrollDelta::LineDelta(_, dy) => {
                                    MouseWheelDelta::Lines(dy.into())
                                }
                                MouseScrollDelta::PixelDelta(pos) => MouseWheelDelta::Pixels(pos.y),
                            };
                            let event = PlayerEvent::MouseWheel { delta };
                            player_lock.handle_event(event);
                            if player_lock.needs_render() {
                                self.window.request_redraw();
                            }
                        }
                        WindowEvent::CursorLeft { .. } => {
                            let mut player_lock = self.player.lock().unwrap();
                            player_lock.handle_event(PlayerEvent::MouseLeave);
                            if player_lock.needs_render() {
                                self.window.request_redraw();
                            }
                        }
                        // Allow KeyboardInput.modifiers (ModifiersChanged event not functional yet).
                        #[allow(deprecated)]
                        WindowEvent::KeyboardInput { input, .. } => {
                            let mut player_lock = self.player.lock().unwrap();
                            if let Some(key) = input.virtual_keycode {
                                let key_code = winit_to_ruffle_key_code(key);
                                let key_char = winit_key_to_char(
                                    key,
                                    input.modifiers.contains(ModifiersState::SHIFT),
                                );
                                let event = match input.state {
                                    ElementState::Pressed => {
                                        PlayerEvent::KeyDown { key_code, key_char }
                                    }
                                    ElementState::Released => {
                                        PlayerEvent::KeyUp { key_code, key_char }
                                    }
                                };
                                player_lock.handle_event(event);
                                if player_lock.needs_render() {
                                    self.window.request_redraw();
                                }
                            }
                        }
                        WindowEvent::ReceivedCharacter(codepoint) => {
                            let mut player_lock = self.player.lock().unwrap();
                            let event = PlayerEvent::TextInput { codepoint };
                            player_lock.handle_event(event);
                            if player_lock.needs_render() {
                                self.window.request_redraw();
                            }
                        }
                        _ => (),
                    },
                    winit::event::Event::UserEvent(RuffleEvent::TaskPoll) => self
                        .executor
                        .lock()
                        .expect("active executor reference")
                        .poll_all(),
                    winit::event::Event::UserEvent(RuffleEvent::OnMetadata(swf_header)) => {
                        // TODO: Re-use `SwfMovie::width` and `SwfMovie::height`.
                        let movie_width = (swf_header.stage_size().x_max
                            - swf_header.stage_size().x_min)
                            .to_pixels();
                        let movie_height = (swf_header.stage_size().y_max
                            - swf_header.stage_size().y_min)
                            .to_pixels();

                        let window_size: Size = match (self.opt.width, self.opt.height) {
                            (None, None) => LogicalSize::new(movie_width, movie_height).into(),
                            (Some(width), None) => {
                                let scale = width / movie_width;
                                let height = movie_height * scale;
                                PhysicalSize::new(width.max(1.0), height.max(1.0)).into()
                            }
                            (None, Some(height)) => {
                                let scale = height / movie_height;
                                let width = movie_width * scale;
                                PhysicalSize::new(width.max(1.0), height.max(1.0)).into()
                            }
                            (Some(width), Some(height)) => {
                                PhysicalSize::new(width.max(1.0), height.max(1.0)).into()
                            }
                        };
                        self.window.set_inner_size(window_size);
                        self.window.set_fullscreen(if self.opt.fullscreen {
                            Some(Fullscreen::Borderless(None))
                        } else {
                            None
                        });
                        self.window.set_visible(true);

                        let viewport_size = self.window.inner_size();
                        let viewport_scale_factor = self.window.scale_factor();
                        let mut player_lock = self.player.lock().unwrap();
                        player_lock.set_viewport_dimensions(
                            viewport_size.width,
                            viewport_size.height,
                            viewport_scale_factor,
                        );
                        player_lock
                            .renderer_mut()
                            .set_viewport_dimensions(viewport_size.width, viewport_size.height);

                        loaded = true;
                    }
                    _ => (),
                }

                // After polling events, sleep the event loop until the next event or the next frame.
                *control_flow = if loaded {
                    ControlFlow::WaitUntil(next_frame_time)
                } else {
                    ControlFlow::Wait
                };
            });
    }
}

/// Convert a winit `VirtualKeyCode` into a Ruffle `KeyCode`.
/// Return `KeyCode::Unknown` if there is no matching Flash key code.
fn winit_to_ruffle_key_code(key_code: VirtualKeyCode) -> KeyCode {
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
fn winit_key_to_char(key_code: VirtualKeyCode, is_shift_down: bool) -> Option<char> {
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

        _ => return None,
    })
}

fn run_timedemo(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let path = opt
        .input_path
        .as_ref()
        .ok_or("Input file necessary for timedemo")?;
    let movie_url = parse_url(path)?;
    let movie = load_movie(&movie_url, &opt)?;
    let movie_frames = Some(movie.num_frames());

    let viewport_width = 1920;
    let viewport_height = 1080;
    let viewport_scale_factor = 1.0;

    let renderer = WgpuRenderBackend::for_offscreen(
        (viewport_width, viewport_height),
        opt.graphics.into(),
        opt.power.into(),
        trace_path(&opt),
    )?;
    let player = PlayerBuilder::new()
        .with_renderer(renderer)
        .with_software_video()
        .with_movie(movie)
        .with_viewport_dimensions(viewport_width, viewport_height, viewport_scale_factor)
        .with_autoplay(true)
        .build();

    let mut player_lock = player.lock().unwrap();

    println!("Running {}...", path.to_string_lossy());

    let start = Instant::now();
    let mut num_frames = 0;
    const MAX_FRAMES: u32 = 5000;
    while num_frames < MAX_FRAMES && player_lock.current_frame() < movie_frames {
        player_lock.run_frame();
        player_lock.render();
        num_frames += 1;
    }
    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("Ran {} frames in {}s.", num_frames, duration.as_secs_f32());

    Ok(())
}

fn init() {
    // When linked with the windows subsystem windows won't automatically attach
    // to the console of the parent process, so we do it explicitly. This fails
    // silently if the parent has no console.
    #[cfg(windows)]
    unsafe {
        use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    env_logger::init();
}

fn shutdown(result: &Result<(), Box<dyn std::error::Error>>) {
    if let Err(e) = result {
        eprintln!("Fatal error:\n{}", e);
    }

    // Without explicitly detaching the console cmd won't redraw it's prompt.
    #[cfg(windows)]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init();
    let opt = Opt::parse();
    let result = if opt.timedemo {
        run_timedemo(opt)
    } else {
        App::new(opt).map(|app| app.run())
    };
    shutdown(&result);
    result
}
