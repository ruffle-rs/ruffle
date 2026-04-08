use ctru::prelude::*;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, ViewportDimensions};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

mod log;
mod render;

const DIMENSIONS: ViewportDimensions = ViewportDimensions {
    width: 400,
    height: 240,
    scale_factor: 1.0,
};

static SWF_BYTES: &[u8] = include_bytes!("../swf/logo-anim.swf");

fn create_player() -> Arc<Mutex<Player>> {
    println!("creating movie");
    let movie = SwfMovie::from_data(SWF_BYTES, "file:///".to_string(), None).unwrap();
    println!("movie created");

    let renderer = log::LogRenderer::new(DIMENSIONS); //render::Citro3DRenderBackend::new();

    PlayerBuilder::new()
        .with_movie(movie)
        .with_renderer(renderer)
        .with_viewport_dimensions(400, 240, 1.0)
        .build()
}

fn main() {
    let gfx = Gfx::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let apt = Apt::new().unwrap();

    ctru::applets::error::set_panic_hook(true);

    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("press start to exit");

    let player = create_player();
    println!("creating");
    player.lock().unwrap().set_is_playing(true);

    println!("hello world");

    let mut prev_time = SystemTime::now();

    while apt.main_loop() {
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        if hid.keys_down().contains(KeyPad::A) {
            let time = SystemTime::now();
            let dt = time
                .duration_since(prev_time)
                .expect("failed to read time")
                .as_millis();
            prev_time = time;

            let mut lock = player.lock().unwrap();
            lock.tick(1000.0 / 60.0 /*dt as f64*/);
            lock.render();
        }

        gfx.wait_for_vblank();
    }
}

