// By default, Windows creates an additional console window for our program.
//
//
// This is silently ignored on non-windows systems.
// See https://docs.microsoft.com/en-us/cpp/build/reference/subsystem?view=msvc-160 for details.
#![windows_subsystem = "windows"]

mod audio;
mod custom_event;
mod executor;
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
    backend::{
        audio::{AudioBackend, NullAudioBackend},
        log as log_backend,
        navigator::NullNavigatorBackend,
        storage::MemoryStorageBackend,
        ui::NullUiBackend,
        video,
    },
    config::Letterbox,
    Player,
};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tinyfiledialogs::open_file_dialog;
use url::Url;

use ruffle_core::tag_utils::SwfMovie;
use ruffle_render_wgpu::clap::{GraphicsBackend, PowerPreference};
use std::io::Read;
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{
    ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Icon, Window, WindowBuilder};

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

struct Ruffle {
    opt: Opt,
    movie: Option<Arc<SwfMovie>>,
    movie_url: Option<Url>,
    window: Option<Rc<Window>>,
    player: Option<Arc<Mutex<Player>>>,
}

impl Ruffle {
    fn new(opt: Opt) -> Self {
        Self {
            opt,
            movie: None,
            movie_url: None,
            window: None,
            player: None,
        }
    }

    fn load(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let (movie, movie_url) = load_movie_from_path(path, &self.opt)?;
        self.movie = Some(Arc::new(movie));
        self.movie_url = Some(movie_url);

        if let Some(window) = self.window.as_ref() {
            let movie_url = self.movie_url.as_ref().unwrap();
            let window_title = movie_url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or_else(|| movie_url.as_str());
            let title = format!("Ruffle - {}", window_title);
            window.set_title(&title);

            // TODO: stay maximized
            let movie = self.movie.as_ref().unwrap();
            let movie_size = LogicalSize::new(movie.width(), movie.height());
            window.set_inner_size(movie_size);

            self.ensure_player()?;
        }

        Ok(())
    }

    fn ensure_player(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.movie.is_none() {
            return Ok(());
        }

        let window = self.window.as_ref().unwrap();
        let movie = self.movie.as_ref().unwrap();

        let viewport_size = window.inner_size();
        let viewport_scale_factor = window.scale_factor();
        let renderer = Box::new(WgpuRenderBackend::for_window(
            window.as_ref(),
            (viewport_size.width, viewport_size.height),
            self.opt.graphics.into(),
            self.opt.power.into(),
            trace_path(&self.opt),
        )?);
        let audio: Box<dyn AudioBackend> = match audio::CpalAudioBackend::new() {
            Ok(audio) => Box::new(audio),
            Err(e) => {
                log::error!("Unable to create audio device: {}", e);
                Box::new(NullAudioBackend::new())
            }
        };
        /*let navigator = Some(Box::new(navigator::ExternalNavigatorBackend::new(
            self.movie_url.as_ref().unwrap().clone(),
            chan,
            event_loop.create_proxy(),
            self.opt.proxy.clone(),
            self.opt.upgrade_to_https,
        )));*/
        // TODO: actually implement this backend type
        let navigator = Box::new(NullNavigatorBackend::new());
        let storage = Box::new(storage::DiskStorageBackend::new());
        let locale = Box::new(locale::DesktopLocaleBackend::new());
        let video = Box::new(video::SoftwareVideoBackend::new());
        let log = Box::new(log_backend::NullLogBackend::new());
        let ui = Box::new(ui::DesktopUiBackend::new(window.clone()));
        let player = Player::new(renderer, audio, navigator, storage, locale, video, log, ui)?;
        player.lock().unwrap().set_root_movie(movie.to_owned());
        player.lock().unwrap().set_is_playing(true);
        player.lock().unwrap().set_letterbox(Letterbox::On);
        player.lock().unwrap().set_viewport_dimensions(
            viewport_size.width,
            viewport_size.height,
            viewport_scale_factor,
        );

        self.player = Some(player);

        Ok(())
    }

    fn run(&'static mut self) -> Result<(), Box<dyn std::error::Error>> {
        let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
        let icon = Icon::from_rgba(icon_bytes.to_vec(), 32, 32)?;

        let event_loop: EventLoop<RuffleEvent> = EventLoop::with_user_event();
        let window = Rc::new(if let Some(movie) = self.movie.as_ref() {
            let window_title = self
                .movie_url
                .as_ref()
                .unwrap()
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or_else(|| self.movie_url.as_ref().unwrap().as_str());

            let movie_size = LogicalSize::new(movie.width(), movie.height());

            WindowBuilder::new()
                .with_window_icon(Some(icon))
                .with_title(format!("Ruffle - {}", window_title))
                .with_inner_size(movie_size)
                .with_max_inner_size(LogicalSize::new(i16::MAX, i16::MAX))
                .build(&event_loop)?
        } else {
            WindowBuilder::new()
                .with_window_icon(Some(icon))
                .with_title("Ruffle")
                .with_max_inner_size(LogicalSize::new(i16::MAX, i16::MAX))
                .build(&event_loop)?
        });

        self.window = Some(window);
        // let (executor, chan) = GlutinAsyncExecutor::new(event_loop.create_proxy());

        self.ensure_player()?;

        let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
        let mut time = Instant::now();
        let mut next_frame_time = Instant::now();
        let mut minimized = false;
        let mut fullscreen_down = false;
        loop {
            // Poll UI events
            event_loop.run(move |event, _window_target, control_flow| {
                if self.player.is_none() {
                    *control_flow = ControlFlow::Wait;
                    match event {
                        winit::event::Event::WindowEvent { event, .. } => match event {
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::DroppedFile(path) => {
                                // TODO: handle errors
                                self.load(&path).unwrap();
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                    return;
                }

                let player = self.player.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();
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

                            let viewport_scale_factor = window.scale_factor();
                            let mut player_lock = player.lock().unwrap();
                            player_lock.set_viewport_dimensions(
                                size.width,
                                size.height,
                                viewport_scale_factor,
                            );
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
                                MouseScrollDelta::LineDelta(_, dy) => {
                                    MouseWheelDelta::Lines(dy.into())
                                }
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
                                .ui_mut()
                                .downcast_mut::<ui::DesktopUiBackend>()
                                .unwrap()
                                .handle_event(event)
                            {
                                player_lock.handle_event(event);
                                if player_lock.needs_render() {
                                    window.request_redraw();
                                }
                            }
                        }
                        WindowEvent::DroppedFile(path) => {
                            // TODO: handle errors
                            self.load(&path).unwrap();
                        }
                        _ => (),
                    },
                    // TODO
                    /*winit::event::Event::UserEvent(RuffleEvent::TaskPoll) => executor
                    .lock()
                    .expect("active executor reference")
                    .poll_all(),*/
                    _ => (),
                }

                // After polling events, sleep the event loop until the next event or the next frame.
                if *control_flow != ControlFlow::Exit {
                    *control_flow = ControlFlow::WaitUntil(next_frame_time);
                }
            });
        }
    }
}

// TODO: Return just `SwfMovie` by making it hold `Url`.
fn load_movie_from_path(
    path: &PathBuf,
    opt: &Opt,
) -> Result<(SwfMovie, Url), Box<dyn std::error::Error>> {
    let movie_url = if path.exists() {
        let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
        Url::from_file_path(absolute_path)
            .map_err(|_| "Path must be absolute and cannot be a URL")?
    } else {
        Url::parse(path.to_str().unwrap_or_default())
            .map_err(|_| "Input path is not a file and could not be parsed as a URL.")?
    };

    if movie_url.scheme() == "file" {
        if let Ok(path) = movie_url.to_file_path() {
            return Ok((SwfMovie::from_path(path, None)?, movie_url));
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

    let mut movie = SwfMovie::from_data(&buffer, Some(movie_url.to_string()), None)?;

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

    Ok((movie, movie_url))
}

fn run_timedemo(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let path = opt
        .input_path
        .as_ref()
        .ok_or("Input file necessary for timedemo")?;
    let (movie, _) = load_movie_from_path(&path, &opt)?;
    let movie_frames = Some(movie.header().num_frames);

    let viewport_width = 1920;
    let viewport_height = 1080;
    let viewport_scale_factor = 1.0;

    let renderer = Box::new(WgpuRenderBackend::for_offscreen(
        (viewport_width, viewport_height),
        opt.graphics.into(),
        opt.power.into(),
        trace_path(&opt),
    )?);
    let audio: Box<dyn AudioBackend> = Box::new(NullAudioBackend::new());
    let navigator = Box::new(NullNavigatorBackend::new());
    let storage = Box::new(MemoryStorageBackend::default());
    let locale = Box::new(locale::DesktopLocaleBackend::new());
    let video = Box::new(video::NullVideoBackend::new());
    let log = Box::new(log_backend::NullLogBackend::new());
    let ui = Box::new(ui::NullUiBackend::new());
    let player = Player::new(renderer, audio, navigator, storage, locale, video, log, ui)?;
    player.lock().unwrap().set_root_movie(Arc::new(movie));
    player.lock().unwrap().set_is_playing(true);

    player.lock().unwrap().set_viewport_dimensions(
        viewport_width,
        viewport_height,
        viewport_scale_factor,
    );

    println!("Running {}...", path.to_string_lossy(),);

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

fn run_player(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let ruffle = Box::leak(Box::new(Ruffle::new(opt)));
    if let Some(path) = &ruffle.opt.input_path.to_owned() {
        ruffle.load(&path)?;
    }
    ruffle.run()
}

fn main() {
    // When linked with the windows subsystem windows won't automatically attach
    // to the console of the parent process, so we do it explicitly. This fails
    // silently if the parent has no console.
    #[cfg(windows)]
    unsafe {
        use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

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

    // Without explicitly detaching the console cmd won't redraw it's prompt.
    #[cfg(windows)]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
}
