#![allow(clippy::unneeded_field_pattern)]

mod audio;
mod custom_event;
mod executor;
mod input;
mod locale;
mod navigator;
mod storage;
mod task;
mod ui;

use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use clap::Clap;
use isahc::{config::RedirectPolicy, prelude::*, HttpClient};
use ruffle_core::{
    backend::audio::{AudioBackend, NullAudioBackend},
    config::Letterbox,
    Player,
};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tinyfiledialogs::open_file_dialog;
use url::Url;

use crate::storage::DiskStorageBackend;
use ruffle_core::backend::log::NullLogBackend;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::io::Read;
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{
    ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, WindowBuilder};

#[derive(Clap, Debug)]
#[clap(
    name = "Ruffle",
    author,
    version = include_str!(concat!(env!("OUT_DIR"), "/version-info.txt")),
)]
struct Opt {
    /// Path to a flash movie (swf) to play
    #[clap(name = "FILE", parse(from_os_str))]
    input_path: Option<PathBuf>,

    /// A "flashvars" parameter to provide to the movie.
    /// This can be repeated multiple times, for example -Pkey=value -Pfoo=bar
    #[clap(short = 'P', number_of_values = 1)]
    parameters: Vec<String>,

    /// Type of graphics backend to use. Not all options may be supported by your current system.
    /// Default will attempt to pick the most supported graphics backend.
    #[clap(
        long,
        short,
        case_insensitive = true,
        default_value = "default",
        arg_enum
    )]
    graphics: GraphicsBackend,

    /// Power preference for the graphics device used. High power usage tends to prefer dedicated GPUs,
    /// whereas a low power usage tends prefer integrated GPUs.
    #[clap(long, short, case_insensitive = true, default_value = "high", arg_enum)]
    power: PowerPreference,

    /// Location to store a wgpu trace output
    #[clap(long, parse(from_os_str))]
    #[cfg(feature = "render_trace")]
    trace_path: Option<PathBuf>,

    /// (Optional) Proxy to use when loading movies via URL
    #[clap(long, case_insensitive = true)]
    proxy: Option<Url>,

    /// (Optional) Replace all embedded http URLs with https
    #[clap(long, case_insensitive = true, takes_value = false)]
    upgrade_to_https: bool,

    #[clap(long, case_insensitive = true, takes_value = false)]
    timedemo: bool,
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

fn main() {
    win32_hide_console();

    env_logger::init();

    let opt = Opt::parse();

    let ret = if opt.timedemo {
        run_timedemo(opt)
    } else {
        run_player(opt)
    };

    if let Err(e) = ret {
        eprintln!("Fatal error:\n{}", e);
        std::process::exit(-1);
    }
}

fn load_movie_from_path(movie_url: Url, opt: &Opt) -> Result<SwfMovie, Box<dyn std::error::Error>> {
    if movie_url.scheme() == "file" {
        if let Ok(path) = movie_url.to_file_path() {
            return SwfMovie::from_path(path);
        }
    }
    let proxy = opt.proxy.as_ref().and_then(|url| url.as_str().parse().ok());
    let builder = HttpClient::builder()
        .proxy(proxy)
        .redirect_policy(RedirectPolicy::Follow);
    let client = builder.build()?;
    let res = client.get(movie_url.to_string())?;
    let mut buffer: Vec<u8> = Vec::new();
    res.into_body().read_to_end(&mut buffer)?;

    let mut movie = SwfMovie::from_data(&buffer, Some(movie_url.to_string()))?;

    // Set query parameters.
    for parameter in &opt.parameters {
        let mut split = parameter.splitn(2, '=');
        if let (Some(key), Some(value)) = (split.next(), split.next()) {
            movie.parameters_mut().insert(key, value.to_string(), true);
        } else {
            movie
                .parameters_mut()
                .insert(&parameter, "".to_string(), true);
        }
    }

    Ok(movie)
}

fn run_player(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let movie_url = match &opt.input_path {
        Some(path) => {
            if path.exists() {
                let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
                Url::from_file_path(absolute_path)
                    .map_err(|_| "Path must be absolute and cannot be a URL")?
            } else {
                Url::parse(path.to_str().unwrap_or_default())
                    .map_err(|_| "Input path is not a file and could not be parsed as a URL.")?
            }
        }
        None => {
            let result = open_file_dialog("Load a Flash File", "", Some((&["*.swf"], ".swf")));

            let selected = match result {
                Some(file_path) => PathBuf::from(file_path),
                None => return Ok(()),
            };

            let absolute_path = selected
                .canonicalize()
                .unwrap_or_else(|_| selected.to_owned());
            Url::from_file_path(absolute_path)
                .map_err(|_| "Path must be absolute and cannot be a URL")?
        }
    };

    let movie = load_movie_from_path(movie_url.to_owned(), &opt)?;
    let movie_size = LogicalSize::new(movie.width(), movie.height());

    let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
    let icon = Icon::from_rgba(icon_bytes.to_vec(), 32, 32)?;

    let event_loop: EventLoop<RuffleEvent> = EventLoop::with_user_event();
    let window_title = movie_url
        .path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or_else(|| movie_url.as_str());
    let window = Rc::new(
        WindowBuilder::new()
            .with_title(format!("Ruffle - {}", window_title))
            .with_window_icon(Some(icon))
            .with_max_inner_size(LogicalSize::new(i16::MAX, i16::MAX))
            .build(&event_loop)?,
    );
    window.set_inner_size(movie_size);
    let viewport_size = window.inner_size();

    let audio: Box<dyn AudioBackend> = match audio::CpalAudioBackend::new() {
        Ok(audio) => Box::new(audio),
        Err(e) => {
            log::error!("Unable to create audio device: {}", e);
            Box::new(NullAudioBackend::new())
        }
    };
    let renderer = Box::new(WgpuRenderBackend::for_window(
        window.as_ref(),
        (viewport_size.width, viewport_size.height),
        opt.graphics.into(),
        opt.power.into(),
        trace_path(&opt),
    )?);
    let (executor, chan) = GlutinAsyncExecutor::new(event_loop.create_proxy());
    let navigator = Box::new(navigator::ExternalNavigatorBackend::new(
        movie_url.clone(),
        chan,
        event_loop.create_proxy(),
        opt.proxy,
        opt.upgrade_to_https,
    )); //TODO: actually implement this backend type
    let input = Box::new(input::WinitInputBackend::new(window.clone()));
    let storage = Box::new(DiskStorageBackend::new());
    let user_interface = Box::new(ui::DesktopUiBackend::new(window.clone()));
    let locale = Box::new(locale::DesktopLocaleBackend::new());
    let player = Player::new(
        renderer,
        audio,
        navigator,
        input,
        storage,
        locale,
        Box::new(NullLogBackend::new()),
        user_interface,
    )?;
    {
        let mut player = player.lock().unwrap();
        player.set_root_movie(Arc::new(movie));
        player.set_is_playing(true); // Desktop player will auto-play.
        player.set_letterbox(Letterbox::On);
        player.set_viewport_dimensions(viewport_size.width, viewport_size.height);
    }

    let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
    let mut time = Instant::now();
    let mut next_frame_time = Instant::now();
    let mut minimized = false;
    let mut fullscreen_down = false;
    loop {
        // Poll UI events
        event_loop.run(move |event, _window_target, control_flow| {
            // Allow KeyboardInput.modifiers (ModifiersChanged event not functional yet).
            #[allow(deprecated)]
            match event {
                winit::event::Event::LoopDestroyed => {
                    player.lock().unwrap().flush_shared_objects();
                    return;
                }

                // Core loop
                winit::event::Event::MainEventsCleared => {
                    let new_time = Instant::now();
                    let dt = new_time.duration_since(time).as_micros();
                    if dt > 0 {
                        time = new_time;
                        let mut player_lock = player.lock().unwrap();
                        player_lock.tick(dt as f64 / 1000.0);
                        next_frame_time = new_time + player_lock.time_til_next_frame();
                        if player_lock.needs_render() {
                            window.request_redraw();
                        }
                    }
                }

                // Render
                winit::event::Event::RedrawRequested(_) => {
                    // Don't render when minimized to avoid potential swap chain errors in `wgpu`.
                    if !minimized {
                        player.lock().unwrap().render();
                    }
                }

                winit::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        // TODO: Change this when winit adds a `Window::minimzed` or `WindowEvent::Minimize`.
                        minimized = size.width == 0 && size.height == 0;

                        let mut player_lock = player.lock().unwrap();
                        player_lock.set_viewport_dimensions(size.width, size.height);
                        player_lock
                            .renderer_mut()
                            .set_viewport_dimensions(size.width, size.height);
                        window.request_redraw();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let mut player_lock = player.lock().unwrap();
                        mouse_pos = position;
                        let event = ruffle_core::PlayerEvent::MouseMove {
                            x: position.x,
                            y: position.y,
                        };
                        player_lock.handle_event(event);
                        if player_lock.needs_render() {
                            window.request_redraw();
                        }
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: pressed,
                        ..
                    } => {
                        let mut player_lock = player.lock().unwrap();
                        let event = if pressed == ElementState::Pressed {
                            ruffle_core::PlayerEvent::MouseDown {
                                x: mouse_pos.x,
                                y: mouse_pos.y,
                            }
                        } else {
                            ruffle_core::PlayerEvent::MouseUp {
                                x: mouse_pos.x,
                                y: mouse_pos.y,
                            }
                        };
                        player_lock.handle_event(event);
                        if player_lock.needs_render() {
                            window.request_redraw();
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        use ruffle_core::events::MouseWheelDelta;
                        let mut player_lock = player.lock().unwrap();
                        let delta = match delta {
                            MouseScrollDelta::LineDelta(_, dy) => MouseWheelDelta::Lines(dy.into()),
                            MouseScrollDelta::PixelDelta(pos) => MouseWheelDelta::Pixels(pos.y),
                        };
                        let event = ruffle_core::PlayerEvent::MouseWheel { delta };
                        player_lock.handle_event(event);
                        if player_lock.needs_render() {
                            window.request_redraw();
                        }
                    }
                    WindowEvent::CursorLeft { .. } => {
                        let mut player_lock = player.lock().unwrap();
                        player_lock.handle_event(ruffle_core::PlayerEvent::MouseLeft);
                        if player_lock.needs_render() {
                            window.request_redraw();
                        }
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Return),
                                modifiers, // TODO: Use WindowEvent::ModifiersChanged.
                                ..
                            },
                        ..
                    } if modifiers.alt() => {
                        if !fullscreen_down {
                            window.set_fullscreen(match window.fullscreen() {
                                None => Some(Fullscreen::Borderless(None)),
                                Some(_) => None,
                            });
                        }
                        fullscreen_down = true;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(VirtualKeyCode::Return),
                                ..
                            },
                        ..
                    } if fullscreen_down => {
                        fullscreen_down = false;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        window.set_fullscreen(None);
                    }
                    WindowEvent::KeyboardInput { .. } | WindowEvent::ReceivedCharacter(_) => {
                        let mut player_lock = player.lock().unwrap();
                        if let Some(event) = player_lock
                            .input_mut()
                            .downcast_mut::<input::WinitInputBackend>()
                            .unwrap()
                            .handle_event(event)
                        {
                            player_lock.handle_event(event);
                            if player_lock.needs_render() {
                                window.request_redraw();
                            }
                        }
                    }
                    _ => (),
                },
                winit::event::Event::UserEvent(RuffleEvent::TaskPoll) => executor
                    .lock()
                    .expect("active executor reference")
                    .poll_all(),
                _ => (),
            }

            // After polling events, sleep the event loop until the next event or the next frame.
            if *control_flow != ControlFlow::Exit {
                *control_flow = ControlFlow::WaitUntil(next_frame_time);
            }
        });
    }
}

fn run_timedemo(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let movie_url = match &opt.input_path {
        Some(path) => {
            if path.exists() {
                let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
                Url::from_file_path(absolute_path)
                    .map_err(|_| "Path must be absolute and cannot be a URL")?
            } else {
                Url::parse(path.to_str().unwrap_or_default())
                    .map_err(|_| "Input path is not a file and could not be parsed as a URL.")?
            }
        }
        None => return Err("Input file necessary for timedemo".into()),
    };

    let movie = load_movie_from_path(movie_url, &opt)?;
    let movie_frames = Some(movie.header().num_frames);

    let viewport_width = 1920;
    let viewport_height = 1080;

    let renderer = Box::new(WgpuRenderBackend::for_offscreen(
        (viewport_width, viewport_height),
        opt.graphics.into(),
        opt.power.into(),
        trace_path(&opt),
    )?);
    let audio: Box<dyn AudioBackend> = Box::new(NullAudioBackend::new());
    let navigator = Box::new(ruffle_core::backend::navigator::NullNavigatorBackend::new());
    let input = Box::new(ruffle_core::backend::input::NullInputBackend::new());
    let storage = Box::new(ruffle_core::backend::storage::MemoryStorageBackend::default());
    let user_interface = Box::new(ruffle_core::backend::ui::NullUiBackend::new());
    let locale = Box::new(locale::DesktopLocaleBackend::new());
    let log = Box::new(NullLogBackend::new());
    let player = Player::new(
        renderer,
        audio,
        navigator,
        input,
        storage,
        locale,
        log,
        user_interface,
    )?;
    player.lock().unwrap().set_root_movie(Arc::new(movie));
    player.lock().unwrap().set_is_playing(true);

    player
        .lock()
        .unwrap()
        .set_viewport_dimensions(viewport_width, viewport_height);

    println!("Running {}...", opt.input_path.unwrap().to_string_lossy(),);

    let start = Instant::now();
    let mut num_frames = 0;
    const MAX_FRAMES: u32 = 5000;
    let mut player = player.lock().unwrap();
    while num_frames < MAX_FRAMES && player.current_frame() < movie_frames {
        player.run_frame();
        player.render();
        num_frames += 1;
    }
    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("Ran {} frames in {}s.", num_frames, duration.as_secs_f32());

    Ok(())
}

/// Hides the Win32 console if we were not launched from the command line.
fn win32_hide_console() {
    #[cfg(windows)]
    unsafe {
        use winapi::um::{wincon::*, winuser::*};
        // If we have a console, and we are the exclusive process using that console,
        // then we were not launched from the command-line; hide the console to act like a GUI app.
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            let mut pids = [0; 2];
            let num_pids = GetConsoleProcessList(pids.as_mut_ptr(), 2);
            let is_exclusive = num_pids <= 1;
            if is_exclusive {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}
