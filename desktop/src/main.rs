#![allow(clippy::unneeded_field_pattern)]

mod audio;
mod custom_event;
mod executor;
mod input;
mod navigator;
mod task;

use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use ruffle_core::{
    backend::audio::{AudioBackend, NullAudioBackend},
    Player,
};
use ruffle_render_wgpu::WgpuRenderBackend;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

use ruffle_core::tag_utils::SwfMovie;
use std::rc::Rc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(name = "FILE", parse(from_os_str))]
    input_path: PathBuf,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let ret = run_player(opt.input_path);

    if let Err(e) = ret {
        eprintln!("Fatal error:\n{}", e);
        std::process::exit(-1);
    }
}

fn run_player(input_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let movie = SwfMovie::from_path(&input_path)?;

    let event_loop: EventLoop<RuffleEvent> = EventLoop::with_user_event();
    let window = Rc::new(
        WindowBuilder::new()
            .with_title(format!(
                "Ruffle - {}",
                input_path.file_name().unwrap_or_default().to_string_lossy()
            ))
            .with_inner_size(LogicalSize::new(movie.width(), movie.height()))
            .build(&event_loop)?,
    );

    let audio: Box<dyn AudioBackend> = match audio::CpalAudioBackend::new() {
        Ok(audio) => Box::new(audio),
        Err(e) => {
            log::error!("Unable to create audio device: {}", e);
            Box::new(NullAudioBackend::new())
        }
    };
    let renderer = Box::new(WgpuRenderBackend::for_window(
        window.as_ref(),
        (movie.width(), movie.height()),
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
    let player = Player::new(renderer, audio, navigator, input, movie)?;
    player.lock().unwrap().set_is_playing(true); // Desktop player will auto-play.

    let size = window.inner_size();
    player
        .lock()
        .unwrap()
        .set_viewport_dimensions(size.width, size.height);

    let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
    let mut time = Instant::now();
    let mut next_frame_time = Instant::now();
    loop {
        // Poll UI events
        event_loop.run(move |event, _window_target, control_flow| {
            match event {
                winit::event::Event::LoopDestroyed => return,

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
