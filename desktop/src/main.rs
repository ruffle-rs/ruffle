#![allow(clippy::unneeded_field_pattern)]

mod audio;
mod custom_event;
mod executor;
mod input;
mod navigator;
mod render;
mod task;

use crate::custom_event::RuffleEvent;
use crate::executor::GlutinAsyncExecutor;
use crate::render::GliumRenderBackend;
use glutin::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use ruffle_core::{
    backend::audio::{AudioBackend, NullAudioBackend},
    Player,
};
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;

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
    let swf_data = std::fs::read(&input_path)?;

    let event_loop: EventLoop<RuffleEvent> = EventLoop::with_user_event();
    let window_builder = WindowBuilder::new().with_title(format!(
        "Ruffle - {}",
        input_path.file_name().unwrap_or_default().to_string_lossy()
    ));
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4)
        .with_srgb(true)
        .with_stencil_buffer(8)
        .build_windowed(window_builder, &event_loop)?;
    let audio: Box<dyn AudioBackend> = match audio::CpalAudioBackend::new() {
        Ok(audio) => Box::new(audio),
        Err(e) => {
            log::error!("Unable to create audio device: {}", e);
            Box::new(NullAudioBackend::new())
        }
    };
    let renderer = Box::new(GliumRenderBackend::new(windowed_context)?);
    let (executor, chan) = GlutinAsyncExecutor::new(event_loop.create_proxy());
    let navigator = Box::new(navigator::ExternalNavigatorBackend::new(
        chan,
        event_loop.create_proxy(),
    )); //TODO: actually implement this backend type
    let display = renderer.display().clone();
    let input = Box::new(input::WinitInputBackend::new(display.clone()));
    let player = Player::new(renderer, audio, navigator, input, swf_data)?;

    let logical_size: LogicalSize<u32> = {
        let mut player_lock = player.lock().unwrap();
        player_lock.set_is_playing(true); // Desktop player will auto-play.

        (player_lock.movie_width(), player_lock.movie_height()).into()
    };
    let scale_factor = display.gl_window().window().scale_factor();

    // Set initial size to movie dimensions.
    display.gl_window().window().set_inner_size(logical_size);
    display
        .gl_window()
        .resize(logical_size.to_physical(scale_factor));

    let mut mouse_pos = PhysicalPosition::new(0.0, 0.0);
    let mut time = Instant::now();
    loop {
        // Poll UI events
        event_loop.run(move |event, _window_target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                glutin::event::Event::LoopDestroyed => return,
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        let mut player_lock = player.lock().unwrap();
                        player_lock.set_viewport_dimensions(size.width, size.height);
                        player_lock
                            .renderer_mut()
                            .set_viewport_dimensions(size.width, size.height);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let mut player_lock = player.lock().unwrap();
                        mouse_pos = position;
                        let event = ruffle_core::PlayerEvent::MouseMove {
                            x: position.x,
                            y: position.y,
                        };
                        player_lock.handle_event(event);
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
                    }
                    WindowEvent::CursorLeft { .. } => {
                        let mut player_lock = player.lock().unwrap();
                        player_lock.handle_event(ruffle_core::PlayerEvent::MouseLeft)
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
                        }
                    }
                    _ => (),
                },
                glutin::event::Event::UserEvent(RuffleEvent::TaskPoll) => executor
                    .lock()
                    .expect("active executor reference")
                    .poll_all(),
                _ => (),
            }

            // After polling events, sleep the event loop until the next event or the next frame.
            if *control_flow == ControlFlow::Wait {
                let new_time = Instant::now();
                let dt = new_time.duration_since(time).as_micros();
                if dt > 0 {
                    time = new_time;
                    player.lock().unwrap().tick(dt as f64 / 1000.0);
                }

                *control_flow =
                    ControlFlow::WaitUntil(new_time + player.lock().unwrap().time_til_next_frame());
            }
        });
    }
}
