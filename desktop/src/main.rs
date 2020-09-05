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
use ruffle_core::{
    backend::audio::{AudioBackend, NullAudioBackend},
    Player,
};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::storage::DiskStorageBackend;
use ruffle_core::backend::log::NullLogBackend;
use ruffle_core::tag_utils::SwfMovie;
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Icon, WindowBuilder};

#[derive(Clap, PartialEq, Debug)]
pub enum GraphicsBackend {
    Default,
    Vulkan,
    Metal,
    Dx12,
    Dx11,
}

impl From<GraphicsBackend> for ruffle_render_wgpu::wgpu::BackendBit {
    fn from(backend: GraphicsBackend) -> Self {
        match backend {
            GraphicsBackend::Default => ruffle_render_wgpu::wgpu::BackendBit::PRIMARY,
            GraphicsBackend::Vulkan => ruffle_render_wgpu::wgpu::BackendBit::VULKAN,
            GraphicsBackend::Metal => ruffle_render_wgpu::wgpu::BackendBit::METAL,
            GraphicsBackend::Dx12 => ruffle_render_wgpu::wgpu::BackendBit::DX12,
            GraphicsBackend::Dx11 => ruffle_render_wgpu::wgpu::BackendBit::DX11,
        }
    }
}

#[derive(Clap, PartialEq, Debug)]
pub enum PowerPreference {
    Default = 0,
    Low = 1,
    High = 2,
}

impl From<PowerPreference> for ruffle_render_wgpu::wgpu::PowerPreference {
    fn from(preference: PowerPreference) -> Self {
        match preference {
            PowerPreference::Default => ruffle_render_wgpu::wgpu::PowerPreference::Default,
            PowerPreference::Low => ruffle_render_wgpu::wgpu::PowerPreference::LowPower,
            PowerPreference::High => ruffle_render_wgpu::wgpu::PowerPreference::HighPerformance,
        }
    }
}

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
    /// Default will pick the best device depending on the status of your computer (ie, wall-power
    /// may choose high, battery power may choose low)
    #[clap(
        long,
        short,
        case_insensitive = true,
        default_value = "default",
        arg_enum
    )]
    power: PowerPreference,
}

fn main() {
    win32_hide_console();

    env_logger::init();

    let opt = Opt::parse();

    let ret = run_player(opt.input_path, opt.graphics, opt.power);

    if let Err(e) = ret {
        eprintln!("Fatal error:\n{}", e);
        std::process::exit(-1);
    }
}

fn run_player(
    input_path: PathBuf,
    graphics: GraphicsBackend,
    power_preference: PowerPreference,
) -> Result<(), Box<dyn std::error::Error>> {
    let movie = SwfMovie::from_path(&input_path)?;
    let movie_size = LogicalSize::new(movie.width(), movie.height());

    let icon_bytes = include_bytes!("../assets/favicon-32.rgba");
    let icon = Icon::from_rgba(icon_bytes.to_vec(), 32, 32)?;

    let event_loop: EventLoop<RuffleEvent> = EventLoop::with_user_event();
    let window = Rc::new(
        WindowBuilder::new()
            .with_title(format!(
                "Ruffle - {}",
                input_path.file_name().unwrap_or_default().to_string_lossy()
            ))
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
        graphics.into(),
        power_preference.into(),
    )?);
    let (executor, chan) = GlutinAsyncExecutor::new(event_loop.create_proxy());
    let navigator = Box::new(navigator::ExternalNavigatorBackend::with_base_path(
        input_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("")),
        chan,
        event_loop.create_proxy(),
    )); //TODO: actually implement this backend type
    let input = Box::new(input::WinitInputBackend::new(window.clone()));
    let storage = Box::new(DiskStorageBackend::new(
        input_path.file_name().unwrap_or_default().as_ref(),
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
