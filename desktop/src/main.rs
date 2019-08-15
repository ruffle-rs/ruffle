mod audio;
mod render;

use crate::render::GliumRenderBackend;
use glutin::{
    dpi::{LogicalPosition, LogicalSize},
    ContextBuilder, ElementState, Event, EventsLoop, MouseButton, WindowBuilder, WindowEvent,
};
use ruffle_core::{backend::render::RenderBackend, Player};
use std::path::PathBuf;
use std::time::{Duration, Instant};
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
    let swf_data = std::fs::read(input_path)?;

    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new().with_title("Ruffle");
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4)
        .with_srgb(true)
        .build_windowed(window_builder, &events_loop)?;
    let audio = audio::RodioAudioBackend::new()?;
    let renderer = GliumRenderBackend::new(windowed_context)?;
    let display = renderer.display().clone();
    let mut player = Player::new(renderer, audio, swf_data)?;
    player.set_is_playing(true); // Desktop player will auto-play.

    let logical_size: LogicalSize = (player.movie_width(), player.movie_height()).into();
    let hidpi_factor = display.gl_window().get_hidpi_factor();

    display
        .gl_window()
        .resize(logical_size.to_physical(hidpi_factor));

    dbg!((player.movie_width(), player.movie_height()));

    display.gl_window().set_inner_size(logical_size);

    let mut mouse_pos = LogicalPosition::new(0.0, 0.0);
    let mut time = Instant::now();
    loop {
        // Poll UI events
        let mut request_close = false;
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Resized(logical_size) => {
                        let size = logical_size.to_physical(hidpi_factor);
                        player.renderer_mut().set_viewport_dimensions(
                            size.width.ceil() as u32,
                            size.height.ceil() as u32,
                        )
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = position;
                        player.mouse_move((position.x as f32, position.y as f32));
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Pressed,
                        ..
                    } => player.mouse_down(),
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Released,
                        ..
                    } => player.mouse_up(),
                    WindowEvent::CloseRequested => request_close = true,
                    _ => (),
                }
            }
        });

        if request_close {
            break;
        }

        let new_time = Instant::now();
        let dt = new_time.duration_since(time).as_millis();
        time = new_time;

        player.tick(dt as f64);

        std::thread::sleep(Duration::from_millis(1000 / 60));
    }
    Ok(())
}
