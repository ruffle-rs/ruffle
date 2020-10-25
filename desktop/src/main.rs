#![allow(clippy::unneeded_field_pattern)]

mod audio;
mod custom_event;
mod executor;
mod input;
mod locale;
mod navigator;
mod storage;
mod task;

use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use clap::Clap;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use ruffle_core::{
    backend::audio::{AudioBackend, NullAudioBackend},
    Player,
};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use url::Url;

use crate::storage::DiskStorageBackend;
use ruffle_core::backend::log::NullLogBackend;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::io::Read;
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Icon, WindowBuilder};

#[derive(Clap, Debug)]
#[clap(
    name = "Ruffle",
    author,
    version = include_str!(concat!(env!("OUT_DIR"), "/version-info.txt")),
)]
struct Opt {
    /// Path to a flash movie (swf) to play
    #[clap(name = "FILE", parse(from_os_str))]
    input_path: PathBuf,

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

    let ret = run_player(opt);

    if let Err(e) = ret {
        eprintln!("Fatal error:\n{}", e);
        std::process::exit(-1);
    }
}

fn load_movie_from_path(
    movie_url: Url,
    proxy: Option<Url>,
) -> Result<SwfMovie, Box<dyn std::error::Error>> {
    if movie_url.scheme() == "file" {
        if let Ok(path) = movie_url.to_file_path() {
            return SwfMovie::from_path(path);
        }
    }
    let proxy = proxy.and_then(|url| url.as_str().parse().ok());
    let builder = HttpClient::builder()
        .proxy(proxy)
        .redirect_policy(RedirectPolicy::Follow);
    let client = builder.build()?;
    let res = client.get(movie_url.to_string())?;
    let mut buffer: Vec<u8> = Vec::new();
    res.into_body().read_to_end(&mut buffer)?;
    SwfMovie::from_data(&buffer, Some(movie_url.to_string()))
}

fn run_player(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let movie_url = if opt.input_path.exists() {
        let absolute_path = opt.input_path.canonicalize()?;
        Url::from_file_path(absolute_path).map_err(|_| "Path cannot be a URL")?
    } else {
        Url::parse(opt.input_path.to_str().unwrap_or_default())
            .map_err(|_| "Input path is not a file and could not be parsed as a URL.")?
    };
    let mut movie = load_movie_from_path(movie_url.to_owned(), opt.proxy.to_owned())?;
    let movie_size = LogicalSize::new(movie.width(), movie.height());

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
            .with_inner_size(movie_size)
            .build(&event_loop)?,
    );
    let viewport_size = movie_size.to_physical(window.scale_factor());

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
        movie_url,
        chan,
        event_loop.create_proxy(),
        opt.proxy,
    )); //TODO: actually implement this backend type
    let input = Box::new(input::WinitInputBackend::new(window.clone()));
    let storage = Box::new(DiskStorageBackend::new(
        opt.input_path.file_name().unwrap_or_default().as_ref(),
    ));
    let locale = Box::new(locale::DesktopLocaleBackend::new());
    let player = Player::new(
        renderer,
        audio,
        navigator,
        input,
        storage,
        locale,
        Box::new(NullLogBackend::new()),
    )?;
    player.lock().unwrap().set_root_movie(Arc::new(movie));
    player.lock().unwrap().set_is_playing(true); // Desktop player will auto-play.

    player
        .lock()
        .unwrap()
        .set_viewport_dimensions(viewport_size.width, viewport_size.height);

    let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
    let mut time = Instant::now();
    let mut next_frame_time = Instant::now();
    loop {
        // Poll UI events
        event_loop.run(move |event, _window_target, control_flow| {
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
                winit::event::Event::RedrawRequested(_) => player.lock().unwrap().render(),

                winit::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
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
