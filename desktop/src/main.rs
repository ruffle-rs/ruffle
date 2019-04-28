use fluster_core::{backend::render::glium::GliumRenderBackend, Player};
use glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent};
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
    let opt = Opt::from_args();

    let ret = run_player(opt.input_path);

    if let Err(e) = ret {
        eprintln!("Fatal error:\n{}", e);
        std::process::exit(-1);
    }
}

fn run_player(input_path: PathBuf) -> Result<(), Box<std::error::Error>> {
    let swf_data = std::fs::read(input_path)?;

    let mut events_loop = EventsLoop::new();
    let window_builder = WindowBuilder::new();
    let windowed_context = ContextBuilder::new().build_windowed(window_builder, &events_loop)?;
    let renderer = GliumRenderBackend::new(windowed_context)?;
    let mut player = Player::new(Box::new(renderer), swf_data)?;

    let mut time = Instant::now();
    loop {
        // Poll UI events
        let mut request_close = false;
        events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => request_close = true,
            _ => (),
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

// impl UiBackend for GlutinBackend {
//     fn poll_events(&mut self) -> bool {
//         let mut request_close = false;
//         self.events_loop.poll_events(|event| match event {
//             Event::WindowEvent {
//                 event: WindowEvent::CloseRequested,
//                 ..
//             } => request_close = true,
//             _ => (),
//         });

//         !request_close
//     }
// }
